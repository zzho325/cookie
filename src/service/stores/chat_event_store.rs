use async_trait::async_trait;
use color_eyre::Result;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use crate::{chat::ChatEvent, service::database::Job};

#[async_trait]
pub trait ChatEventStore: Send + Sync {
    async fn get_chat_events_for_session(&self, session_id: &str) -> Result<Vec<ChatEvent>>;
    async fn create_chat_event(&self, chat_event: ChatEvent) -> Result<ChatEvent>;
}

pub struct ChatEventStoreImpl {
    /// Db job sender.
    job_tx: Arc<UnboundedSender<Job>>,
}

impl ChatEventStoreImpl {
    pub fn new(job_tx: Arc<UnboundedSender<Job>>) -> Self {
        Self { job_tx }
    }
}

#[async_trait]
impl ChatEventStore for ChatEventStoreImpl {
    async fn get_chat_events_for_session(&self, session_id: &str) -> Result<Vec<ChatEvent>> {
        todo!()
    }

    /// Persists chat event to database and update session store for updated settings and time.
    async fn create_chat_event(&self, chat_event: ChatEvent) -> Result<ChatEvent> {
        todo!()
    }
}
