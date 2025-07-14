mod chat;
pub mod client;

use std::sync::Arc;

use color_eyre::{Result, eyre::eyre};
use tokio::sync::{
    Mutex,
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    watch,
};

use crate::{
    models::{LlmSettings, ServiceReq, ServiceResp},
    service::client::LlmClientRouter,
};

pub struct ServiceBuilder {
    llm_settings: LlmSettings,
    req_rx: UnboundedReceiver<ServiceReq>,
    resp_tx: UnboundedSender<ServiceResp>,
}

impl ServiceBuilder {
    pub fn new(
        llm_settings: LlmSettings,
        req_rx: UnboundedReceiver<ServiceReq>,
        resp_tx: UnboundedSender<ServiceResp>,
    ) -> Self {
        Self {
            llm_settings,
            req_rx,
            resp_tx,
        }
    }

    pub fn build(self) -> Service {
        // TODO: fetch env variable here
        let router = LlmClientRouter::new();
        Service::new(self.req_rx, self.resp_tx, router, self.llm_settings)
    }
}

pub struct Service {
    req_rx: UnboundedReceiver<ServiceReq>,
    resp_tx: UnboundedSender<ServiceResp>,

    llm_router: LlmClientRouter,
    llm_settings: LlmSettings,
}

impl Service {
    pub fn new(
        req_rx: UnboundedReceiver<ServiceReq>,
        resp_tx: UnboundedSender<ServiceResp>,
        llm_router: LlmClientRouter,
        llm_settings: LlmSettings,
    ) -> Self {
        Self {
            req_rx,
            resp_tx,
            llm_router,
            llm_settings,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        let (chat_tx, chat_rx) = mpsc::unbounded_channel::<String>();
        let (settings_tx, settings_rx) = watch::channel(self.llm_settings.clone());
        // FIXME: we shouldn't use previous id to support other providers.
        let previous_response_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        let mut chat_handle = tokio::spawn(Self::chat(
            chat_rx,
            self.llm_router,
            settings_rx,
            previous_response_id,
            self.resp_tx,
        ));

        loop {
            tokio::select! {
                maybe_req = self.req_rx.recv() => {
                    match maybe_req {
                        None => break,
                        Some(ServiceReq::ChatMessage(msg)) => {
                             chat_tx.send(msg)?;
                        }
                        Some(ServiceReq::UpdateSettings(settings)) => {
                            tracing::debug!("receive update {settings:?}");
                        settings_tx.send(settings).unwrap();
                            // self.llm_settings = settings;
                        }
                    }
                }
                res = &mut chat_handle => {
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
