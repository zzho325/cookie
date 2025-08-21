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
        chat::SharedSession,
        database::{get_db_conn, spawn_db_thread},
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

        let (db_thread, job_tx) = spawn_db_thread(conn);
        let job_tx = Arc::new(job_tx);
        let chat_event_store = ChatEventStoreImpl::new(job_tx.clone());
        let chat_session_store = ChatSessionStoreImpl::new(job_tx.clone());

        Some(Service::new(
            self.req_rx,
            self.resp_tx,
            db_thread,
            Box::new(chat_event_store),
            Box::new(chat_session_store),
            router,
        ))
    }
}

struct DBThreadJoiner(Option<std::thread::JoinHandle<()>>);

impl Drop for DBThreadJoiner {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.join().expect("DB thread panicked");
        }
    }
}

pub struct Service {
    req_rx: UnboundedReceiver<ServiceReq>,
    resp_tx: UnboundedSender<ServiceResp>,

    /// DB thread.
    db_thread_handle: Option<std::thread::JoinHandle<()>>,
    chat_event_store: Box<dyn ChatEventStore>,
    chat_session_store: Box<dyn ChatSessionStore>,

    sessions: HashMap<String, SharedSession>,
    llm_router: LlmClientRouter,
    sessions_chat_tx: HashMap<String, UnboundedSender<ChatEvent>>,
}

impl Service {
    pub fn new(
        req_rx: UnboundedReceiver<ServiceReq>,
        resp_tx: UnboundedSender<ServiceResp>,
        db_thread: std::thread::JoinHandle<()>,
        chat_event_store: Box<dyn ChatEventStore>,
        chat_session_store: Box<dyn ChatSessionStore>,
        llm_router: LlmClientRouter,
    ) -> Self {
        Self {
            req_rx,
            resp_tx,
            db_thread_handle: Some(db_thread),
            chat_event_store,
            chat_session_store,
            llm_router,
            sessions: HashMap::new(),
            sessions_chat_tx: HashMap::new(),
        }
    }

    pub async fn run(mut self) -> Result<()> {
        // wait for db thread before returning
        let _ = DBThreadJoiner(self.db_thread_handle.take());

        let mut chat_handles = FuturesUnordered::<JoinHandle<Result<()>>>::new();
        loop {
            tokio::select! {
                maybe_req = self.req_rx.recv() => {
                    match maybe_req {
                        None => break,
                        Some(ServiceReq::ChatMessage ( user_message )) => {
                            if self.sessions.contains_key(&user_message.session_id) {
                                self.handle_user_message(user_message)?;
                            } else {
                                let chat_handle = self.handle_new_session(user_message).await?;
                                chat_handles.push(chat_handle);
                            };
                        }
                        Some(ServiceReq::GetSession(session_id)) => {
                           self.handle_get_session(&session_id).await?
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

        Ok(())
    }
}
