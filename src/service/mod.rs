pub mod client;
mod models;

use color_eyre::Result;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::debug;

use crate::service::{
    client::{OpenAIClient, OpenAIClientImpl},
    models::{Content, Output, ResponsesReq},
};

async fn run_service_loop<C: OpenAIClient>(
    mut req_rx: UnboundedReceiver<String>,
    resp_tx: UnboundedSender<String>,
    client: C,
) -> Result<()> {
    while let Some(req) = req_rx.recv().await {
        let resp = client
            .responses(ResponsesReq {
                model: "gpt-4o".into(),
                instructions: "test".into(),
                input: req.clone(),
            })
            .await?;

        // resp ResponsesResp { output: [Message { status: "completed", role: "assistant", content: [OutputText { text: "Hello! How can I assist you today?", annotations: [] }] }] }

        debug!("req {req}");
        debug!("resp {resp:?}");
        let mut texts = Vec::new();
        for output in &resp.output {
            let Output::Message { content, .. } = output;
            for item in content {
                let Content::OutputText { text, .. } = item;
                texts.push(text.clone());
            }
        }
        resp_tx.send(format!("{texts:#?}"))?
    }
    Ok(())
}

pub async fn run_service_loop_with_openai(
    req_rx: UnboundedReceiver<String>,
    resp_tx: UnboundedSender<String>,
) -> Result<()> {
    let client = OpenAIClientImpl::new();
    run_service_loop::<OpenAIClientImpl>(req_rx, resp_tx, client).await
}
