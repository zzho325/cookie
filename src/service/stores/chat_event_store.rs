use async_trait::async_trait;
use color_eyre::{Result, eyre::eyre};
use prost::Message;
use rusqlite::Connection;
use std::sync::{Arc, mpsc::Sender};
use tokio::sync::oneshot;

use crate::{chat::ChatEvent, service::database::Job};

#[async_trait]
pub trait ChatEventStore: Send + Sync {
    async fn get_chat_events_for_session(&self, session_id: &str) -> Result<Vec<ChatEvent>>;
    /// Persists chat event to database and update session store for updated time.
    async fn create_chat_event(&self, chat_event: ChatEvent) -> Result<ChatEvent>;
}

pub struct ChatEventStoreImpl {
    /// Db job sender.
    job_tx: Arc<Sender<Job>>,
}

impl ChatEventStoreImpl {
    pub fn new(job_tx: Arc<Sender<Job>>) -> Self {
        Self { job_tx }
    }
}

#[async_trait]
impl ChatEventStore for ChatEventStoreImpl {
    async fn get_chat_events_for_session(&self, session_id: &str) -> Result<Vec<ChatEvent>> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let session_id = session_id.to_string();

        let job = Box::new(move |conn: &mut Connection| {
            let result = Self::get_chat_events_for_session_internal(conn, session_id);
            let _ = resp_tx.send(result);
        });

        self.job_tx
            .send(job)
            .map_err(|e| eyre!("failed to send job to DB thread: {}", e))?;
        resp_rx.await?
    }

    async fn create_chat_event(&self, chat_event: ChatEvent) -> Result<ChatEvent> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let job = Box::new(move |conn: &mut Connection| {
            let result = Self::create_chat_event_internal(conn, chat_event);
            let _ = resp_tx.send(result);
        });

        self.job_tx
            .send(job)
            .map_err(|e| eyre!("failed to send job to DB thread: {}", e))?;
        resp_rx.await?
    }
}

impl ChatEventStoreImpl {
    fn get_chat_events_for_session_internal(
        conn: &mut Connection,
        session_id: String,
    ) -> Result<Vec<ChatEvent>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, data, created_at
             FROM chat_events
             WHERE session_id = ?
             ORDER BY created_at ASC
             "#,
        )?;

        let rows = stmt.query_map([session_id], ChatEvent::from_row)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn create_chat_event_internal(
        conn: &mut Connection,
        chat_event: ChatEvent,
    ) -> Result<ChatEvent> {
        let mut buf = Vec::new();
        chat_event.encode(&mut buf)?;

        let mut stmt = conn.prepare(
            r#"
        INSERT INTO chat_events (id, session_id, data)
        VALUES (?1, ?2, ?3)
        RETURNING id, session_id, data, created_at
        "#,
        )?;

        let returned_event = stmt.query_row(
            (&chat_event.id, &chat_event.session_id, &buf),
            ChatEvent::from_row,
        )?;

        // update the session's updated_at to now
        conn.execute(
            r#"
        UPDATE chat_sessions
        SET updated_at = strftime('%s', 'now')
        WHERE id = ?1
        "#,
            [&chat_event.session_id],
        )?;

        Ok(returned_event)
    }
}

impl ChatEvent {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<ChatEvent> {
        let data: Vec<u8> = row.get("data")?;
        let created_at: i64 = row.get("created_at")?;
        let mut chat_event =
            ChatEvent::decode(&*data).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?;
        chat_event.created_at = Some(prost_types::Timestamp {
            seconds: created_at,
            nanos: 0,
        });
        Ok(chat_event)
    }
}
