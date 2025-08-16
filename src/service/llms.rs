pub mod mock;
pub mod open_ai;

use async_trait::async_trait;
use color_eyre::eyre::{Context, Result};
use futures_util::stream::BoxStream;
use std::sync::Arc;

use crate::{
    models::{ChatEventPayload, settings::LlmSettings},
    service::llms::open_ai::OpenAIClientImpl,
};

#[async_trait]
pub trait LlmClient {
    async fn request(&self, llm_req: LlmReq) -> Result<LlmResp>;
    async fn stream(&self, llm_req: LlmReq) -> Result<BoxStream<'static, ChatEventPayload>>;
}

#[derive(Debug, Clone)]
pub struct LlmReq {
    pub events: Vec<ChatEventPayload>,
    pub settings: LlmSettings,
    pub instructions: Option<String>,
}

pub struct LlmResp {
    pub output: Vec<ChatEventPayload>,
}

#[derive(Clone)]
pub struct LlmClientRouter {
    open_ai: Arc<OpenAIClientImpl>,
    #[cfg(debug_assertions)]
    mock: Arc<mock::MockLlmClientImpl>,
}

impl LlmClientRouter {
    pub fn build() -> Result<Self> {
        let client = reqwest::Client::new();
        let open_ai_key = std::env::var("OPENAI_API_KEY")
            .wrap_err("set the OPENAI_API_KEY environment variable")?;

        Ok(Self {
            open_ai: Arc::new(OpenAIClientImpl::new(client, open_ai_key)),
            #[cfg(debug_assertions)]
            mock: Arc::new(mock::MockLlmClientImpl {}),
        })
    }
}

#[async_trait]
impl LlmClient for LlmClientRouter {
    async fn request(&self, llm_req: LlmReq) -> Result<LlmResp> {
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

    async fn stream(&self, llm_req: LlmReq) -> Result<BoxStream<'static, ChatEventPayload>> {
        match llm_req.settings {
            LlmSettings::OpenAI { .. } => return self.open_ai.stream(llm_req).await,
            LlmSettings::Mock { .. } => {
                todo!()
            }
        }
    }
}
