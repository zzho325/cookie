mod chat;
pub mod client;

use color_eyre::Result;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    models::{LlmSettings, ServiceReq, ServiceResp},
    service::client::{LlmClient, OpenAIClientImpl},
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
        let client: Box<dyn LlmClient> = {
            match self.llm_settings {
                LlmSettings::OpenAI { .. } => Box::new(OpenAIClientImpl::new()),
                LlmSettings::Mock { .. } => {
                    #[cfg(debug_assertions)]
                    {
                        Box::new(crate::service::client::mock::MockOpenAIClient {})
                    }
                    #[cfg(not(debug_assertions))]
                    panic!("using mock llm provider with non debug build")
                }
            }
        };
        Service::new(self.req_rx, self.resp_tx, client, self.llm_settings)
    }
}

pub struct Service {
    req_rx: UnboundedReceiver<ServiceReq>,
    resp_tx: UnboundedSender<ServiceResp>,

    llm_client: Box<dyn LlmClient>,
    llm_settings: LlmSettings,
    previous_response_id: Option<String>,
}

impl Service {
    pub fn new(
        req_rx: UnboundedReceiver<ServiceReq>,
        resp_tx: UnboundedSender<ServiceResp>,
        client: Box<dyn LlmClient>,
        llm_settings: LlmSettings,
    ) -> Self {
        Self {
            req_rx,
            resp_tx,
            llm_client: client,
            llm_settings,
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
