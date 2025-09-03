use color_eyre::{Result, eyre::eyre};
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use crate::{
    chat::*,
    llm::*,
    models::ServiceResp,
    service::{
        Service,
        chat_session_worker::ChatSessionWorker,
        llms::{LlmClient, LlmClientRouter, LlmReq},
        stores::chat_session_store::ChatSessionStore,
    },
};

impl Service {
    /// Gets chat session from stores if exists or create one if it not exists. Spawns a chat session
    /// worker job and returns the handle. Spawns a job to generate title for new session as well.
    pub async fn spawn_session(
        &mut self,
        user_message: ChatEvent,
    ) -> Result<tokio::task::JoinHandle<Result<()>>> {
        let session_id = user_message.session_id.clone();
        // get session from database or create one
        let chat_session = match self
            .chat_session_store
            .get_chat_session(&session_id)
            .await?
        {
            Some(mut chat_session) => {
                let chat_events = self
                    .chat_event_store
                    .get_chat_events_for_session(&session_id)
                    .await?;
                chat_session.events = chat_events;
                chat_session
            }
            None => {
                // create session
                let llm_settings = user_message.llm_settings;
                let mut chat_session = ChatSession::new(session_id.clone(), llm_settings);
                tracing::debug!("creating session {chat_session:?}");
                chat_session = self
                    .chat_session_store
                    .create_chat_session(chat_session)
                    .await?;

                // publich sessions to tui
                self.send_sessions().await?;

                // generate title async
                let chat_session_store = self.chat_session_store.clone();
                let llm_router = self.llm_router.clone();
                let resp_tx = self.resp_tx.clone();
                tokio::spawn(Self::try_update_session_title(
                    chat_session_store,
                    user_message,
                    chat_session.clone(),
                    llm_router,
                    resp_tx,
                ));

                chat_session
            }
        };

        // create channel and spawn session worker
        let (chat_tx, chat_rx) = unbounded_channel::<ChatEvent>();
        self.sessions_chat_tx
            .insert(session_id.to_string(), chat_tx);

        let llm_router = self.llm_router.clone();
        let resp_tx = self.resp_tx.clone();
        let chat_event_store = self.chat_event_store.clone();
        let chat_session_store = self.chat_session_store.clone();
        let worker = ChatSessionWorker::new(
            chat_rx,
            chat_session,
            llm_router,
            resp_tx,
            chat_event_store,
            chat_session_store,
        );
        // spawn chat
        let handle = tokio::spawn(worker.run());
        Ok(handle)
    }

    /// Finds session chat sender for session of `user_message` and dispatch message. Send error
    /// message to tui if session chat sender not found.
    pub fn handle_user_message(&mut self, user_message: ChatEvent) -> Result<()> {
        let session_id = &user_message.session_id;
        match self.sessions_chat_tx.get_mut(session_id) {
            Some(chat_tx) => {
                chat_tx.send(user_message)?;
            }
            None => {
                self.resp_tx.send(ServiceResp::Error(format!(
                    "session {session_id} not found"
                )))?;
            }
        };

        Ok(())
    }

    /// Sends `session` of `session_id` to tui. Send error message to tui if session not found.
    pub async fn handle_get_session(&mut self, session_id: &str) -> Result<()> {
        match self.chat_session_store.get_chat_session(session_id).await {
            Ok(Some(mut chat_session)) => {
                let chat_events = self
                    .chat_event_store
                    .get_chat_events_for_session(session_id)
                    .await?;
                chat_session.events = chat_events;
                self.resp_tx.send(ServiceResp::Session(chat_session))?;
            }
            Ok(None) => {
                self.resp_tx.send(ServiceResp::Error(format!(
                    "session {session_id} not found"
                )))?;
            }
            Err(e) => {
                self.resp_tx.send(ServiceResp::Error(e.to_string()))?;
            }
        };
        Ok(())
    }

    /// Delete `session` of `session_id` and sends updated sessions to tui.
    pub async fn handle_delete_session(&mut self, session_id: &str) -> Result<()> {
        match self
            .chat_session_store
            .delete_chat_session(session_id)
            .await
        {
            Ok(_) => self.send_sessions().await?,
            Err(e) => {
                self.resp_tx.send(ServiceResp::Error(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Sends sessions in stores to tui.
    pub async fn send_sessions(&mut self) -> Result<()> {
        tracing::debug!("sending sessions");
        match self.chat_session_store.get_chat_sessions().await {
            Ok(chat_sessions) => {
                self.resp_tx.send(ServiceResp::Sessions(chat_sessions))?;
            }
            Err(e) => {
                self.resp_tx.send(ServiceResp::Error(e.to_string()))?;
            }
        };
        Ok(())
    }

    /// Attempts to generate title with LLM and send to tui.
    pub async fn try_update_session_title(
        chat_session_store: Arc<dyn ChatSessionStore>,
        user_message: ChatEvent,
        mut chat_session: ChatSession,
        llm_router: LlmClientRouter,
        resp_tx: UnboundedSender<ServiceResp>,
    ) {
        chat_session.title = match Self::generate_session_title(
            user_message,
            chat_session.llm_settings.unwrap_or_default(),
            llm_router,
        )
        .await
        {
            Ok(title) => title,
            Err(e) => {
                tracing::error!("failed to generate title with LLM {e}");
                return;
            }
        };

        match chat_session_store.update_chat_session(chat_session).await {
            Ok(chat_session) => {
                if let Err(e) = resp_tx.send(ServiceResp::SessionSummary(chat_session)) {
                    tracing::error!("failed to send updated session: {}", e);
                }
            }
            Err(e) => tracing::error!("failed to persist session title: {}", e),
        }
    }

    /// Requests LLM to generate session title.
    pub async fn generate_session_title(
        user_message: ChatEvent,
        llm_settings: LlmSettings,
        llm_router: LlmClientRouter,
    ) -> Result<String> {
        let prompt = "You are an AI assistant that generates a concise title for a chat session \
        based on the user's message. Generate a title that captures the essence of the \
        conversation, ideally in 3ish words. Only reply with the title, without any additional \
        punctuation, text or explanation. This is very important."
            .to_string();
        tracing::debug!("{prompt}");

        let payload = match user_message.payload {
            Some(p) => p,
            _ => return Err(eyre!("payload is not user message")),
        };
        let llm_req = LlmReq {
            events: vec![payload],
            instructions: Some(prompt),
            settings: llm_settings,
        };
        let resp = llm_router.request(llm_req).await?;

        let maybe_title = resp.output.iter().find_map(|payload| {
            if let chat_event::Payload::Message(m) = payload {
                Some(m.msg.clone())
            } else {
                None
            }
        });
        maybe_title.ok_or(eyre!("Llm response has no title"))
    }
}
