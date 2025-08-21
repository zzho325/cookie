use async_trait::async_trait;
use color_eyre::Result;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use crate::{chat::ChatSession, service::database::Job};

#[async_trait]
pub trait ChatSessionStore: Send + Sync {
    async fn get_chat_sessions(&self) -> Result<Vec<ChatSession>>;
    async fn get_chat_session(&self, session_id: &str) -> Result<Option<ChatSession>>;
    async fn create_chat_session(&self, chat_session: ChatSession) -> Result<ChatSession>;
    async fn update_chat_session(&self, chat_session: ChatSession) -> Result<ChatSession>;
}

pub struct ChatSessionStoreImpl {
    /// Db job sender.
    job_tx: Arc<UnboundedSender<Job>>,
}

impl ChatSessionStoreImpl {
    pub fn new(job_tx: Arc<UnboundedSender<Job>>) -> Self {
        Self { job_tx }
    }
}

#[async_trait]
impl ChatSessionStore for ChatSessionStoreImpl {
    async fn get_chat_sessions(&self) -> Result<Vec<ChatSession>> {
        todo!()
    }

    async fn get_chat_session(&self, session_id: &str) -> Result<Option<ChatSession>> {
        todo!()
    }

    async fn create_chat_session(&self, chat_session: ChatSession) -> Result<ChatSession> {
        todo!()
    }

    async fn update_chat_session(&self, chat_session: ChatSession) -> Result<ChatSession> {
        todo!()
    }
}
