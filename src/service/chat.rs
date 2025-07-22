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
        client::{LlmClient, LlmClientRouter, LlmReq},
    },
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub type SharedSession = Arc<RwLock<Session>>;

impl Session {
    fn new(id: Uuid, settings: LlmSettings) -> Self {
        Self {
            id,
            chat_messages: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            previous_response_id: None,
            settings,
            title: NEW_SESSION_TITLE.to_string(),
        }
    }

    pub fn persist_messages(
        &mut self,
        user_message: ChatMessage,
        assistant_msg: String,
        settings: LlmSettings,
    ) -> ChatMessage {
        let assistant_message = ChatMessage::new(
            user_message.session_id,
            Role::Assistant(settings),
            assistant_msg,
        );

        self.chat_messages.push(user_message.clone());
        self.chat_messages.push(assistant_message.clone());
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
        settings: LlmSettings,
        user_message: ChatMessage,
    ) -> Result<tokio::task::JoinHandle<Result<()>>> {
        // create session
        let session_id = user_message.session_id;
        let session = SharedSession::new(RwLock::new(Session::new(session_id, settings.clone())));
        self.sessions.insert(session_id, session.clone());
        self.send_sessions().await?;

        // create session channel and send initial message
        let (chat_tx, chat_rx) = mpsc::unbounded_channel::<ChatMessage>();
        chat_tx.send(user_message)?;
        self.sessions_chat_tx.insert(session_id, chat_tx);

        // spawn chat
        let llm_router = self.llm_router.clone();
        let resp_tx = self.resp_tx.clone();
        Ok(tokio::spawn(Self::chat(
            chat_rx,
            session.clone(),
            llm_router,
            resp_tx,
        )))
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

    /// Updates session settings. Send error message to tui if session not found.
    pub async fn handle_update_settings(
        &mut self,
        session_id: &Uuid,
        settings: LlmSettings,
    ) -> Result<()> {
        match self.shared_session(session_id) {
            Ok(shared_session) => {
                let mut guard = shared_session.write().await;
                guard.settings = settings;
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
            // load settings and the last ID
            let (settings, previous_response_id) = {
                let guard = session.read().await;
                (guard.settings.clone(), guard.previous_response_id.clone())
            };
            let llm_req = LlmReq {
                msg: user_message.msg.clone(),
                instructions: None,
                previous_response_id,
                settings: settings.clone(),
            };
            match llm_router.responses(llm_req).await {
                Ok(resp) => {
                    // send response and update session
                    tracing::debug!("sending message {:?}", resp.msg);
                    let mut guard = session.write().await;
                    guard.previous_response_id = Some(resp.id);
                    let assistant_message =
                        (*guard).persist_messages(user_message, resp.msg, settings);
                    resp_tx.send(ServiceResp::ChatMessage(assistant_message))?;
                }
                Err(_) => {
                    // TODO: send error as response
                }
            }
        }
        Ok(())
    }
}
