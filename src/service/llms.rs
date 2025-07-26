pub mod mock;
pub mod open_ai;

use async_trait::async_trait;
use color_eyre::eyre::Result;
use std::sync::Arc;

use crate::{
    models::{ChatEvent, LlmSettings},
    service::llms::open_ai::OpenAIClientImpl,
};

#[async_trait]
pub trait LlmClient {
    async fn request(&self, llm_req: LlmReq) -> Result<LlmResp>;
}

#[derive(Debug, Clone)]
pub struct LlmReq {
    pub input: Vec<ChatEvent>,
    pub settings: LlmSettings,
    pub instructions: Option<String>,
}

pub enum Tool {
    WebSearch,
    Function,
}

pub struct LlmResp {
    pub msg: String,
    pub id: String,
}

#[derive(Clone)]
pub struct LlmClientRouter {
    open_ai: Arc<OpenAIClientImpl>,
    #[cfg(debug_assertions)]
    mock: Arc<mock::MockLlmClientImpl>,
}

impl LlmClientRouter {
    pub fn new() -> Self {
        Self {
            open_ai: Arc::new(OpenAIClientImpl::new()),
            #[cfg(debug_assertions)]
            mock: Arc::new(mock::MockLlmClientImpl {}),
        }
    }
}

#[async_trait]
impl LlmClient for LlmClientRouter {
    async fn request(&self, llm_req: LlmReq) -> Result<LlmResp> {
        tracing::debug!("router handle request {:?}", llm_req);
        match llm_req.settings {
            LlmSettings::OpenAI { .. } => return self.open_ai.request(llm_req).await,
            LlmSettings::Mock { .. } => {
                #[cfg(debug_assertions)]
                {
                    self.mock.request(llm_req).await
                }
                #[cfg(not(debug_assertions))]
                panic!("using mock llm provider with non debug build")
            }
        }
    }
}
