use std::sync::Arc;

use color_eyre::Result;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use crate::{
    models::{ChatMessage, LlmSettings, Role, ServiceResp, Session, SessionSummary},
    service::{
        Service,
        client::{LlmClient, LlmClientRouter, LlmReq},
    },
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct SessionWorker {
    pub chat_tx: UnboundedSender<ChatMessage>,
    pub settings_tx: UnboundedSender<LlmSettings>,
}

impl SessionWorker {
    pub fn send_message(&mut self, chat_message: ChatMessage) -> Result<()> {
        self.chat_tx.send(chat_message)?;
        Ok(())
    }

    pub fn update_settings(&mut self, settings: LlmSettings) -> Result<()> {
        self.settings_tx.send(settings.clone())?;
        Ok(())
    }
}

pub type SharedSession = Arc<RwLock<Session>>;

impl Session {
    fn new(id: Uuid, settings: LlmSettings) -> Self {
        Self {
            id,
            chat_messages: Vec::new(),
            previous_response_id: None,
            settings,
            summary: "".to_string(),
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

        self.chat_messages.push(user_message);
        self.chat_messages.push(assistant_message.clone());
        assistant_message
    }
}

impl Service {
    pub fn new_session(
        &mut self,
        settings: LlmSettings,
        user_message: ChatMessage,
    ) -> Result<tokio::task::JoinHandle<Result<()>>> {
        // create session
        let session_id = user_message.session_id;
        let session = SharedSession::new(RwLock::new(Session::new(session_id, settings.clone())));
        self.sessions.insert(session_id, session.clone());

        // create session channel and send initial message
        let (chat_tx, chat_rx) = mpsc::unbounded_channel::<ChatMessage>();
        chat_tx.send(user_message)?;
        self.session_workers.insert(session_id, chat_tx);

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

    fn get_session(&mut self, session_id: &Uuid) -> Result<Option<&mut SharedSession>> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            Ok(Some(session))
        } else {
            self.resp_tx.send(ServiceResp::Error(format!(
                "chat session {} not found",
                session_id
            )))?;
            Ok(None)
        }
    }

    fn get_session_worker(
        &mut self,
        session_id: &Uuid,
    ) -> Result<Option<&mut UnboundedSender<ChatMessage>>> {
        if let Some(session_worker) = self.session_workers.get_mut(session_id) {
            Ok(Some(session_worker))
        } else {
            self.resp_tx.send(ServiceResp::Error(format!(
                "chat session worker {} not found",
                session_id
            )))?;
            Ok(None)
        }
    }

    pub fn handle_user_message(&mut self, user_message: ChatMessage) -> Result<()> {
        if let Some(chat_tx) = self.get_session_worker(&(user_message.session_id))? {
            chat_tx.send(user_message)?;
        }
        Ok(())
    }

    pub async fn handle_update_settings(
        &mut self,
        session_id: &Uuid,
        settings: LlmSettings,
    ) -> Result<()> {
        if let Some(session) = self.get_session(session_id)? {
            let mut guard = session.write().await;
            guard.settings = settings;
        }
        Ok(())
    }

    async fn send_sessions(&mut self) -> Result<()> {
        let mut summaries: Vec<SessionSummary> = Vec::new();
        for session in self.sessions.values() {
            let session_summary;
            {
                let guard = session.read().await;
                session_summary = SessionSummary {
                    id: guard.id,
                    summary: guard.summary.clone(),
                }
            }
            summaries.push(session_summary);
        }
        Ok(())
    }

    pub async fn chat(
        mut chat_rx: UnboundedReceiver<ChatMessage>,
        session: SharedSession,
        llm_router: LlmClientRouter,
        resp_tx: UnboundedSender<ServiceResp>,
    ) -> Result<()> {
        // TODO: close worker after inactivity
        while let Some(user_message) = chat_rx.recv().await {
            // load settings and the last ID
            let settings;
            let previous_response_id;
            {
                let guard = session.read().await;
                settings = guard.settings.clone();
                previous_response_id = guard.previous_response_id.clone();
            }
            let llm_req = LlmReq {
                msg: user_message.msg.clone(),
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
