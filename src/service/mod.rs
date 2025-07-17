pub mod chat;
pub mod client;

use color_eyre::{Result, eyre::eyre};
use futures_util::StreamExt;
use futures_util::stream::FuturesUnordered;
use std::collections::HashMap;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use uuid::Uuid;

use crate::{
    models::{ChatMessage, ServiceReq, ServiceResp},
    service::{chat::SharedSession, client::LlmClientRouter},
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

    pub fn build(self) -> Service {
        // todo: fetch env variable here
        let router = LlmClientRouter::new();
        Service::new(self.req_rx, self.resp_tx, router)
    }
}

pub struct Service {
    req_rx: UnboundedReceiver<ServiceReq>,
    resp_tx: UnboundedSender<ServiceResp>,

    llm_router: LlmClientRouter,
    sessions: HashMap<Uuid, SharedSession>,
    session_workers: HashMap<Uuid, UnboundedSender<ChatMessage>>,
}

impl Service {
    pub fn new(
        req_rx: UnboundedReceiver<ServiceReq>,
        resp_tx: UnboundedSender<ServiceResp>,
        llm_router: LlmClientRouter,
    ) -> Self {
        Self {
            req_rx,
            resp_tx,
            llm_router,
            sessions: HashMap::new(),
            session_workers: HashMap::new(),
        }
    }

    pub async fn run(mut self) -> Result<()> {
        let mut chat_handles = FuturesUnordered::<JoinHandle<Result<()>>>::new();
        loop {
            tokio::select! {
                maybe_req = self.req_rx.recv() => {
                    match maybe_req {
                        None => break,
                        Some(ServiceReq::NewSession { settings, user_message })=>{
                            let chat_handle = self.new_session(settings, user_message).await?;
                            chat_handles.push(chat_handle);
                        }
                        Some(ServiceReq::ChatMessage (chat_message)) => {
                            self.handle_user_message(chat_message)?;
                        }
                        Some(ServiceReq::UpdateSettings{session_id, settings}) => {
                            tracing::debug!("receive update {settings:?}");
                            self.handle_update_settings(&session_id, settings).await?;
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
