use crate::{chat::ChatSession, service::database::Job};
use async_trait::async_trait;
use color_eyre::{Result, eyre::eyre};
use prost::Message;
use rusqlite::{Connection, OptionalExtension as _};
use std::sync::mpsc::Sender;
use tokio::sync::oneshot;

#[async_trait]
pub trait ChatSessionStore: Send + Sync {
    async fn get_chat_sessions(&self) -> Result<Vec<ChatSession>>;
    async fn get_chat_session(&self, session_id: &str) -> Result<Option<ChatSession>>;
    async fn create_chat_session(&self, chat_session: ChatSession) -> Result<ChatSession>;
    async fn update_chat_session(&self, chat_session: ChatSession) -> Result<ChatSession>;
    async fn delete_chat_session(&self, session_id: &str) -> Result<()>;
}

pub struct ChatSessionStoreImpl {
    job_tx: Sender<Job>,
}

impl ChatSessionStoreImpl {
    pub fn new(job_tx: Sender<Job>) -> Self {
        Self { job_tx }
    }
}

#[async_trait]
impl ChatSessionStore for ChatSessionStoreImpl {
    async fn get_chat_sessions(&self) -> Result<Vec<ChatSession>> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let job = Box::new(move |conn: &mut Connection| {
            let result = Self::get_chat_sessions_internal(conn);
            let _ = resp_tx.send(result);
        });

        self.job_tx
            .send(job)
            .map_err(|e| eyre!("failed to send job to DB thread: {}", e))?;
        resp_rx.await?
    }

    async fn get_chat_session(&self, session_id: &str) -> Result<Option<ChatSession>> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let session_id = session_id.to_string();

        let job = Box::new(move |conn: &mut Connection| {
            let result = Self::get_chat_session_internal(conn, session_id);
            let _ = resp_tx.send(result);
        });

        self.job_tx
            .send(job)
            .map_err(|e| eyre!("failed to send job to DB thread: {}", e))?;
        resp_rx.await?
    }

    async fn create_chat_session(&self, chat_session: ChatSession) -> Result<ChatSession> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let job = Box::new(move |conn: &mut Connection| {
            let result = Self::create_chat_session_internal(conn, chat_session);
            let _ = resp_tx.send(result);
        });

        self.job_tx
            .send(job)
            .map_err(|e| eyre!("failed to send job to DB thread: {}", e))?;
        resp_rx.await?
    }

    async fn update_chat_session(&self, chat_session: ChatSession) -> Result<ChatSession> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let job = Box::new(move |conn: &mut Connection| {
            let result = Self::update_chat_session_internal(conn, chat_session);
            let _ = resp_tx.send(result);
        });

        self.job_tx
            .send(job)
            .map_err(|e| eyre!("failed to send job to DB thread: {}", e))?;
        resp_rx.await?
    }

    async fn delete_chat_session(&self, session_id: &str) -> Result<()> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let session_id = session_id.to_string();
        let job = Box::new(move |conn: &mut Connection| {
            let result = Self::delete_chat_session_internal(conn, session_id);
            let _ = resp_tx.send(result);
        });

        self.job_tx
            .send(job)
            .map_err(|e| eyre!("failed to send job to DB thread: {}", e))?;
        resp_rx.await?
    }
}

impl ChatSessionStoreImpl {
    fn get_chat_sessions_internal(conn: &mut Connection) -> Result<Vec<ChatSession>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT data, created_at, updated_at
            FROM chat_sessions
            ORDER BY created_at ASC
            "#,
        )?;
        let rows = stmt.query_map([], ChatSession::from_row)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn get_chat_session_internal(
        conn: &mut Connection,
        session_id: String,
    ) -> Result<Option<ChatSession>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT data, created_at, updated_at
            FROM chat_sessions
            WHERE id = ?
            "#,
        )?;
        let chat_session = stmt
            .query_row([session_id], ChatSession::from_row)
            .optional()?;
        Ok(chat_session)
    }

    fn create_chat_session_internal(
        conn: &mut Connection,
        chat_session: ChatSession,
    ) -> Result<ChatSession> {
        let mut buf = Vec::new();
        chat_session.encode(&mut buf)?;
        let mut stmt = conn.prepare(
            r#"
            INSERT INTO chat_sessions (id, data, updated_at)
            VALUES (?1, ?2, strftime('%s', 'now'))
            RETURNING id, data, created_at, updated_at
            "#,
        )?;
        let session = stmt.query_row((&chat_session.id, &buf), ChatSession::from_row)?;
        Ok(session)
    }

    fn update_chat_session_internal(
        conn: &mut Connection,
        chat_session: ChatSession,
    ) -> Result<ChatSession> {
        let mut buf = Vec::new();
        chat_session.encode(&mut buf)?;
        let mut stmt = conn.prepare(
            r#"
            UPDATE chat_sessions
            SET data = ?1, updated_at = strftime('%s', 'now')
            WHERE id = ?2
            RETURNING id, data, created_at, updated_at
            "#,
        )?;
        let session = stmt.query_row((&buf, &chat_session.id), ChatSession::from_row)?;
        Ok(session)
    }

    fn delete_chat_session_internal(conn: &mut Connection, session_id: String) -> Result<()> {
        let mut stmt = conn.prepare(
            r#"
            DELETE FROM chat_sessions
            WHERE id = ?1
            "#,
        )?;
        stmt.execute((session_id,))?;
        Ok(())
    }
}

impl ChatSession {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<ChatSession> {
        let data: Vec<u8> = row.get("data")?;
        let created_at: i64 = row.get("created_at")?;
        let updated_at: i64 = row.get("updated_at")?;
        let mut chat_session =
            ChatSession::decode(&*data).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?;
        chat_session.created_at = Some(prost_types::Timestamp {
            seconds: created_at,
            nanos: 0,
        });
        chat_session.updated_at = Some(prost_types::Timestamp {
            seconds: updated_at,
            nanos: 0,
        });
        Ok(chat_session)
    }
}
