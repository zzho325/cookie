use color_eyre::Result;
use std::sync::Arc;

use crate::{
    models::{LlmSettings, ServiceResp},
    service::{
        Service,
        client::{LlmClient, LlmClientRouter, LlmReq},
    },
};
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
    watch,
};

impl Service {
    pub async fn chat(
        mut chat_rx: UnboundedReceiver<String>,
        llm_router: LlmClientRouter,
        settings_rx: watch::Receiver<LlmSettings>,
        previous_response_id: Arc<Mutex<Option<String>>>,
        resp_tx: UnboundedSender<ServiceResp>,
    ) -> Result<()> {
        while let Some(msg) = chat_rx.recv().await {
            let settings = settings_rx.borrow().clone();
            tracing::debug!("using setting {settings:?}");
            let mut llm_req = LlmReq {
                msg,
                previous_response_id: None,
                settings,
            };
            // load the last ID
            {
                let guard = previous_response_id.lock().await;
                llm_req.previous_response_id = guard.clone();
            }

            match llm_router.responses(llm_req).await {
                Ok(resp) => {
                    tracing::debug!("sending message {:?}", resp.msg);
                    let _ = resp_tx.send(ServiceResp::ChatMessage(resp.msg));
                    let mut guard = previous_response_id.lock().await;
                    *guard = Some(resp.id);
                }
                Err(_) => {
                    // TODO: send error as reponse
                }
            }
        }
        Ok(())
    }
}
