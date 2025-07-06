mod chat;
pub mod client;
pub mod models;

use color_eyre::Result;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::service::{
    client::{OpenAIClient, OpenAIClientImpl},
    models::{ServiceReq, ServiceResp},
};

pub struct Service<C: OpenAIClient> {
    open_ai_client: C,
    req_rx: UnboundedReceiver<ServiceReq>,
    resp_tx: UnboundedSender<ServiceResp>,
}

impl<C: OpenAIClient> Service<C> {
    pub fn new(
        client: C,
        req_rx: UnboundedReceiver<ServiceReq>,
        resp_tx: UnboundedSender<ServiceResp>,
    ) -> Self {
        Self {
            open_ai_client: client,
            req_rx,
            resp_tx,
        }
    }

    async fn run(mut self) -> Result<()> {
        while let Some(req) = self.req_rx.recv().await {
            let ServiceReq::ChatMessage(msg) = req;
            self.fetch_response(msg).await?;
        }
        Ok(())
    }
}

pub async fn run_service_loop_with_openai(
    req_rx: UnboundedReceiver<ServiceReq>,
    resp_tx: UnboundedSender<ServiceResp>,
) -> Result<()> {
    let client = OpenAIClientImpl::new();
    let service = Service::new(client, req_rx, resp_tx);
    service.run().await
}
