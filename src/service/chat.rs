use std::{sync::Arc, time::SystemTime};

use color_eyre::{Result, eyre::eyre};
use futures_util::StreamExt;
use tokio::sync::{RwLock, mpsc};

use crate::{
    chat::*,
    llm::*,
    models::{ServiceResp, constants::NEW_SESSION_TITLE},
    service::{
        Service,
        llms::{LlmClient, LlmClientRouter, LlmReq},
    },
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub type SharedSession = Arc<RwLock<ChatSession>>;

impl ChatSession {
    fn new(id: String, settings: Option<LlmSettings>) -> Self {
        // TODO: save to stores and populte timestamps
        Self {
            id,
            events: Vec::new(),
            title: NEW_SESSION_TITLE.to_string(),
            llm_settings: settings,
            created_at: Some(prost_types::Timestamp::from(SystemTime::now())),
            updated_at: Some(prost_types::Timestamp::from(SystemTime::now())),
        }
    }

    /// Persists user message and updates llm settings.
    pub fn persist_user_message(&mut self, user_message: ChatEvent) {
        self.llm_settings = user_message.llm_settings;
        self.events.push(user_message);
        // update db and populate timestamps
        self.updated_at = Some(prost_types::Timestamp::from(SystemTime::now()));
    }

    /// Saves chat event payload to session and returns chat message if exists.
    pub fn persist_chat_event(&mut self, event: ChatEvent) {
        self.events.push(event);
        self.updated_at = Some(prost_types::Timestamp::from(SystemTime::now()));
    }
}

impl Service {
    // ----------------------------------------------------------------
    // Internal session management, maybe should move out
    // ----------------------------------------------------------------

    /// Gets shared session.
    fn shared_session(&mut self, session_id: &String) -> Result<&mut SharedSession> {
        self.sessions
            .get_mut(session_id)
            .ok_or_else(|| eyre!("session {} not found", session_id))
    }

    /// Gets session chat sender.
    fn session_chat_tx(&mut self, session_id: &String) -> Result<&mut UnboundedSender<ChatEvent>> {
        self.sessions_chat_tx
            .get_mut(session_id)
            .ok_or_else(|| eyre!("session {} not found", session_id))
    }

    // ----------------------------------------------------------------
    // TUI request handling
    // ----------------------------------------------------------------

    /// Creates new session and its chat sender, sends update sessions to tui; spawns session
    /// worker and sends and first message.
    pub async fn handle_new_session(
        &mut self,
        user_message: ChatEvent,
    ) -> Result<tokio::task::JoinHandle<Result<()>>> {
        let session_id = user_message.session_id.clone();

        // create session
        let llm_settings = user_message.llm_settings;
        let session = SharedSession::new(RwLock::new(ChatSession::new(
            session_id.clone(),
            llm_settings,
        )));
        self.sessions.insert(session_id.clone(), session.clone());
        self.send_sessions().await?;

        // create session channel and send initial message
        let (chat_tx, chat_rx) = mpsc::unbounded_channel::<ChatEvent>();
        chat_tx.send(user_message.clone())?;
        self.sessions_chat_tx.insert(session_id, chat_tx);

        // spawn chat
        let llm_router = self.llm_router.clone();
        let resp_tx = self.resp_tx.clone();
        let handle = tokio::spawn(Self::chat(
            chat_rx,
            session.clone(),
            llm_router.clone(),
            resp_tx.clone(),
        ));

        tokio::spawn(Self::try_update_session_title(
            user_message,
            session,
            llm_router,
            resp_tx,
        ));

        // ge
        Ok(handle)
    }

    /// Finds session chat sender for session of `user_message` and dispatch message. Send error
    /// message to tui if session chat sender not found.
    pub fn handle_user_message(&mut self, user_message: ChatEvent) -> Result<()> {
        let session_id = &(user_message.session_id);
        match self.session_chat_tx(session_id) {
            Ok(chat_tx) => {
                chat_tx.send(user_message)?;
            }
            Err(e) => {
                self.resp_tx.send(ServiceResp::Error(e.to_string()))?;
            }
        };

        Ok(())
    }

    /// Sends `session` of `session_id` to tui. Send error message to tui if session not found.
    pub async fn handle_get_session(&mut self, session_id: &String) -> Result<()> {
        match self.shared_session(session_id) {
            Ok(shared_session) => {
                let session = {
                    let guard = shared_session.read().await;
                    (*guard).clone()
                };
                self.resp_tx.send(ServiceResp::Session(session))?;
            }
            Err(e) => {
                self.resp_tx.send(ServiceResp::Error(e.to_string()))?;
            }
        };
        Ok(())
    }

    /// Sends sessions in memory to tui.
    async fn send_sessions(&mut self) -> Result<()> {
        tracing::debug!("sending sessions");
        let mut summaries: Vec<ChatSession> = Vec::new();
        for session in self.sessions.values() {
            let session_summary = {
                let guard = session.read().await;
                ChatSession {
                    id: guard.id.clone(),
                    events: vec![],
                    title: guard.title.clone(),
                    llm_settings: None,
                    created_at: None,
                    updated_at: guard.updated_at,
                }
            };
            summaries.push(session_summary);
        }
        self.resp_tx.send(ServiceResp::Sessions(summaries))?;
        Ok(())
    }

    /// Polls chat messages from `chat_rx` and for `session`, gets chat response from `llm_router`,
    /// saves to memory and send to tui.
    pub async fn chat(
        mut chat_rx: UnboundedReceiver<ChatEvent>,
        session: SharedSession,
        llm_router: LlmClientRouter,
        resp_tx: UnboundedSender<ServiceResp>,
    ) -> Result<()> {
        // TODO: close worker after inactivity
        while let Some(user_message) = chat_rx.recv().await {
            // persist user message
            {
                let mut guard = session.write().await;
                (*guard).persist_user_message(user_message.clone());
            }

            // ----------------------------------------------------------------
            // Prepare llm request.
            // ----------------------------------------------------------------
            // load history events
            let events: Vec<chat_event::Payload> = {
                let guard = session.read().await;
                guard
                    .events
                    .iter()
                    .filter_map(|event| event.payload.clone())
                    .collect()
            };
            let llm_settings = user_message.llm_settings;
            let llm_req = LlmReq {
                events,
                instructions: None,
                settings: llm_settings.unwrap_or_default(),
            };

            // ----------------------------------------------------------------
            // Stream request and handle response.
            // ----------------------------------------------------------------
            let session_id = {
                let guard = session.read().await;
                guard.id.clone()
            };
            let mut stream = llm_router.stream(llm_req).await?;
            while let Some(payload) = stream.next().await {
                // send to tui
                let chat_event = ChatEvent::new(session_id.clone(), llm_settings, payload.clone());
                resp_tx.send(ServiceResp::ChatEvent(chat_event.clone()))?;

                // persist non delta event
                match payload {
                    chat_event::Payload::Message(_) | chat_event::Payload::ToolEvent(_) => {
                        let mut guard = session.write().await;
                        guard.persist_chat_event(chat_event);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    // Attempts to generate title with LLM and send to tui.
    pub async fn try_update_session_title(
        user_message: ChatEvent,
        session: SharedSession,
        llm_router: LlmClientRouter,
        resp_tx: UnboundedSender<ServiceResp>,
    ) {
        let title =
            match Self::generate_session_title(user_message, session.clone(), llm_router).await {
                Ok(title) => title,
                Err(e) => {
                    tracing::error!("failed to generate title with LLM {e}");
                    return;
                }
            };

        let session_summary: ChatSession = {
            let mut guard = session.write().await;
            guard.title = title;
            guard.updated_at = Some(prost_types::Timestamp::from(SystemTime::now()));
            (*guard).clone()
        };
        if let Err(e) = resp_tx.send(ServiceResp::SessionSummary(session_summary)) {
            tracing::error!("failed to send updated session: {}", e);
        }
    }

    /// Requests LLM to generate session title.
    pub async fn generate_session_title(
        user_message: ChatEvent,
        session: SharedSession,
        llm_router: LlmClientRouter,
    ) -> Result<String> {
        let prompt = "You are an AI assistant that generates a concise title for a chat session \
        based on the user's message. Generate a title that captures the essence of the \
        conversation, ideally in 3ish words. Only reply with the title, without any additional \
        punctuation, text or explanation. This is very important."
            .to_string();
        tracing::debug!("{prompt}");

        // load settings
        let settings = {
            let guard = session.read().await;
            guard.llm_settings
        };

        let payload = match user_message.payload {
            Some(p) => p,
            _ => return Err(eyre!("payload is not user message")),
        };
        let llm_req = LlmReq {
            events: vec![payload],
            instructions: Some(prompt),
            settings: settings.unwrap_or_default(),
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
