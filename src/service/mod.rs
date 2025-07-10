mod chat;
pub mod client;
pub mod models;

use color_eyre::Result;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    Config,
    service::{
        client::{OpenAIClient, OpenAIClientImpl},
        models::{ServiceReq, ServiceResp},
    },
};

pub struct ServiceBuilder {
    cfg: Config,
    req_rx: UnboundedReceiver<ServiceReq>,
    resp_tx: UnboundedSender<ServiceResp>,
}

impl ServiceBuilder {
    pub fn new(
        cfg: Config,
        req_rx: UnboundedReceiver<ServiceReq>,
        resp_tx: UnboundedSender<ServiceResp>,
    ) -> Self {
        Self {
            cfg,
            req_rx,
            resp_tx,
        }
    }

    pub fn build(self) -> Service {
        let client: Box<dyn OpenAIClient> = {
            #[cfg(debug_assertions)]
            {
                use crate::service::client::mock::MockOpenAIClient;

                match self.cfg.default_llm {
                    models::LlmProvider::Mock { latency } => Box::new(MockOpenAIClient {}),
                    models::LlmProvider::OpenAI { model, web_search } => {
                        Box::new(OpenAIClientImpl::new())
                    }
                }
            }

            #[cfg(not(debug_assertions))]
            {
                Box::new(OpenAIClientImpl::new())
            }
        };
        Service::new(client, self.req_rx, self.resp_tx)
    }
}

pub struct Service {
    open_ai_client: Box<dyn OpenAIClient>,
    req_rx: UnboundedReceiver<ServiceReq>,
    resp_tx: UnboundedSender<ServiceResp>,

    previous_response_id: Option<String>,
}

impl Service {
    pub fn new(
        client: Box<dyn OpenAIClient>,
        req_rx: UnboundedReceiver<ServiceReq>,
        resp_tx: UnboundedSender<ServiceResp>,
    ) -> Self {
        Self {
            open_ai_client: client,
            req_rx,
            resp_tx,
            previous_response_id: None,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        while let Some(req) = self.req_rx.recv().await {
            let ServiceReq::ChatMessage(msg) = req;
            self.fetch_response(msg).await?;
        }
        Ok(())
    }
}
