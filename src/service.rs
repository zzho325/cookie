mod chat;
mod chat_session_worker;
mod database;
pub mod llms;
mod stores;
mod utils;

use color_eyre::{Result, eyre::eyre};
use futures_util::StreamExt;
use futures_util::stream::FuturesUnordered;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

use crate::{
    chat::ChatEvent,
    models::{ServiceReq, ServiceResp},
    service::{
        database::{DBWorker, get_db_conn, spawn_db_thread},
        llms::LlmClientRouter,
        stores::{
            chat_event_store::{ChatEventStore, ChatEventStoreImpl},
            chat_session_store::{ChatSessionStore, ChatSessionStoreImpl},
        },
    },
};

pub struct ServiceBuilder {
    req_rx: UnboundedReceiver<ServiceReq>,
    resp_tx: UnboundedSender<ServiceResp>,
}

impl ServiceBuilder {
    pub fn new(
        req_rx: UnboundedReceiver<ServiceReq>,
        resp_tx: UnboundedSender<ServiceResp>,
    ) -> Self {
        Self { req_rx, resp_tx }
    }

    pub fn build(self) -> Option<Service> {
        // Make db connection and build llm router. Skip builder service and send an error to tui
        // on failure.
        let conn = match get_db_conn() {
            Ok(conn) => conn,
            Err(e) => {
                let message = ServiceResp::Error(e.to_string());
                self.resp_tx.send(message);
                return None;
            }
        };

        let router = match LlmClientRouter::build() {
            Ok(router) => router,
            Err(e) => {
                // If we failed to build llm router, send an error to tui and skip innitialization.
                let message = ServiceResp::Error(e.to_string());
                self.resp_tx.send(message);
                return None;
            }
        };

        // Spawn db thread and create stores.
        let db_worker = spawn_db_thread(conn);
        let chat_event_store = ChatEventStoreImpl::new(db_worker.sender());
        let chat_session_store = ChatSessionStoreImpl::new(db_worker.sender());

        Some(Service::new(
            self.req_rx,
            self.resp_tx,
            Arc::new(chat_event_store),
            Arc::new(chat_session_store),
            db_worker,
            router,
        ))
    }
}

pub struct Service {
    req_rx: UnboundedReceiver<ServiceReq>,
    resp_tx: UnboundedSender<ServiceResp>,

    /// DB thread.
    chat_event_store: Arc<dyn ChatEventStore>,
    chat_session_store: Arc<dyn ChatSessionStore>,
    _db_worker: DBWorker,

    llm_router: LlmClientRouter,
    sessions_chat_tx: HashMap<String, UnboundedSender<ChatEvent>>,
}

impl Service {
    pub fn new(
        req_rx: UnboundedReceiver<ServiceReq>,
        resp_tx: UnboundedSender<ServiceResp>,
        chat_event_store: Arc<dyn ChatEventStore>,
        chat_session_store: Arc<dyn ChatSessionStore>,
        db_worker: DBWorker,
        llm_router: LlmClientRouter,
    ) -> Self {
        Self {
            req_rx,
            resp_tx,
            chat_event_store,
            chat_session_store,
            _db_worker: db_worker,
            llm_router,
            sessions_chat_tx: HashMap::new(),
        }
    }

    pub async fn run(mut self) -> Result<()> {
        // initialize tui with stored sessions
        self.send_sessions().await?;

        let mut chat_handles = FuturesUnordered::<JoinHandle<Result<()>>>::new();

        loop {
            tokio::select! {
                maybe_req = self.req_rx.recv() => {
                    match maybe_req {
                        None => break,
                        Some(ServiceReq::ChatMessage ( user_message )) => {
                            if !self.sessions_chat_tx.contains_key(&user_message.session_id) {
                                let chat_handle = self.spawn_session(user_message.clone()).await?;
                                chat_handles.push(chat_handle);
                            }
                            self.handle_user_message(user_message)?;
                        }
                        Some(ServiceReq::GetSession(session_id)) => {
                           self.handle_get_session(&session_id).await?
                        }
                        Some(ServiceReq::DeleteSession(session_id)) => {
                            self.handle_delete_session(&session_id).await?
                        }
                    }
                }
                Some(res) = chat_handles.next(), if !chat_handles.is_empty() => {
                    match res {
                        Ok(Ok(())) => {},
                        Ok(Err(e)) => {
                            return Err(e.wrap_err("chat failed"));
                        }
                        Err(join_err) => {
                            return Err(eyre!("chat panicked: {:?}", join_err))
                        }
                    }
                }
            }
        }

        // stop chat workers to drop db job senders they hold
        for (_, tx) in self.sessions_chat_tx.drain() {
            drop(tx);
        }
        Ok(())
    }
}
