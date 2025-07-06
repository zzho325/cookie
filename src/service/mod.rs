mod chat;
pub mod client;
pub mod models;

use color_eyre::Result;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::service::{
    client::{OpenAIClient, OpenAIClientImpl},
    models::{ServiceReq, ServiceResp},
};

pub struct ServiceBuilder {
    /// Default to open_ai
    llm_provider: String,
    req_rx: UnboundedReceiver<ServiceReq>,
    resp_tx: UnboundedSender<ServiceResp>,
}

impl ServiceBuilder {
    pub fn new(
        req_rx: UnboundedReceiver<ServiceReq>,
        resp_tx: UnboundedSender<ServiceResp>,
    ) -> Self {
        Self {
            llm_provider: "open_ai".to_string(),
            req_rx,
            resp_tx,
        }
    }

    pub fn with_llm_provider(mut self, provider: &str) -> Self {
        self.llm_provider = provider.to_string();
        self
    }

    pub fn build(self) -> Service {
        let client: Box<dyn OpenAIClient> = {
            #[cfg(debug_assertions)]
            {
                if self.llm_provider == "mock" {
                    use crate::service::client::mock::MockOpenAIClient;

                    Box::new(MockOpenAIClient {})
                } else {
                    Box::new(OpenAIClientImpl::new())
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
