use std::sync::Arc;

use color_eyre::{Result, eyre::eyre};
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use crate::{
    models::{
        ChatMessage, LlmSettings, Role, ServiceResp, Session, SessionSummary,
        constants::NEW_SESSION_TITLE,
    },
    service::{
        Service,
        llms::{LlmClient, LlmClientRouter, LlmReq},
    },
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub type SharedSession = Arc<RwLock<Session>>;

impl Session {
    fn new(id: Uuid, settings: LlmSettings) -> Self {
        Self {
            id,
            chat_events: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            llm_settings: settings,
            title: NEW_SESSION_TITLE.to_string(),
        }
    }

    /// Saves user message and LLM response to session. Update llm settings with user_message
    /// settings.
    pub fn persist_messages(
        &mut self,
        user_message: ChatMessage,
        assistant_msg: String,
    ) -> ChatMessage {
        let assistant_message = ChatMessage::new(
            user_message.session_id,
            Role::Assistant,
            user_message.llm_settings.clone(),
            assistant_msg,
        );

        self.chat_events.push(user_message.clone().into());
        self.chat_events.push(assistant_message.clone().into());
        self.llm_settings = user_message.llm_settings;
        self.updated_at = chrono::Utc::now();

        assistant_message
    }
}

impl Service {
    // ----------------------------------------------------------------
    // Internal session management, maybe should move out
    // ----------------------------------------------------------------

    /// Gets shared session.
    fn shared_session(&mut self, session_id: &Uuid) -> Result<&mut SharedSession> {
        self.sessions
            .get_mut(session_id)
            .ok_or_else(|| eyre!("session {} not found", session_id))
    }

    /// Gets session chat sender.
    fn session_chat_tx(&mut self, session_id: &Uuid) -> Result<&mut UnboundedSender<ChatMessage>> {
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
        user_message: ChatMessage,
    ) -> Result<tokio::task::JoinHandle<Result<()>>> {
        // create session
        let session_id = user_message.session_id;
        let llm_settings = user_message.llm_settings.clone();
        let session = SharedSession::new(RwLock::new(Session::new(session_id, llm_settings)));
        self.sessions.insert(session_id, session.clone());
        self.send_sessions().await?;

        // create session channel and send initial message
        let (chat_tx, chat_rx) = mpsc::unbounded_channel::<ChatMessage>();
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
    pub fn handle_user_message(&mut self, user_message: ChatMessage) -> Result<()> {
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
    pub async fn handle_get_session(&mut self, session_id: &Uuid) -> Result<()> {
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
        let mut summaries: Vec<SessionSummary> = Vec::new();
        for session in self.sessions.values() {
            let session_summary = {
                let guard = session.read().await;
                SessionSummary {
                    id: guard.id,
                    title: guard.title.clone(),
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
        mut chat_rx: UnboundedReceiver<ChatMessage>,
        session: SharedSession,
        llm_router: LlmClientRouter,
        resp_tx: UnboundedSender<ServiceResp>,
    ) -> Result<()> {
        // TODO: close worker after inactivity
        while let Some(user_message) = chat_rx.recv().await {
            // load history events
            let mut chat_events = {
                let guard = session.read().await;
                guard.chat_events.clone()
            };
            // append user message
            chat_events.push(user_message.clone().into());
            let llm_settings = user_message.llm_settings.clone();
            let llm_req = LlmReq {
                input: chat_events,
                instructions: None,
                settings: llm_settings,
            };
            match llm_router.request(llm_req).await {
                Ok(resp) => {
                    // send response and update session
                    tracing::debug!("sending message {:?}", resp.msg);
                    let mut guard = session.write().await;
                    let assistant_message = (*guard).persist_messages(user_message, resp.msg);
                    resp_tx.send(ServiceResp::ChatMessage(assistant_message))?;
                }
                Err(_) => {
                    // TODO: send error as response
                }
            }
        }
        Ok(())
    }

    // Attempts to generate title with LLM and send to tui.
    pub async fn try_update_session_title(
        user_message: ChatMessage,
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

        let session_summary: SessionSummary = {
            let mut guard = session.write().await;
            guard.title = title;
            guard.updated_at = chrono::Utc::now();
            (*guard).clone().into()
        };
        if let Err(e) = resp_tx.send(ServiceResp::SessionSummary(session_summary)) {
            tracing::error!("failed to send updated session: {}", e);
        }
    }

    /// Requests LLM to generate session title.
    pub async fn generate_session_title(
        user_message: ChatMessage,
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
            guard.llm_settings.clone()
        };
        let llm_req = LlmReq {
            input: vec![user_message.clone().into()],
            instructions: Some(prompt),
            settings,
        };
        let resp = llm_router.request(llm_req).await?;
        Ok(resp.msg)
    }
}
