pub mod gemini;
pub mod open_ai;

use async_trait::async_trait;
use color_eyre::eyre::{Context, Result, eyre};
use futures_util::stream::BoxStream;
use std::sync::Arc;

use crate::{
    chat::*,
    llm::*,
    service::llms::{gemini::GeminiClientImpl, open_ai::OpenAIClientImpl},
};

#[async_trait]
pub trait LlmClient {
    async fn request(&self, llm_req: LlmReq) -> Result<LlmResp>;
    async fn stream(&self, llm_req: LlmReq) -> Result<BoxStream<'static, chat_event::Payload>>;
}

#[derive(Debug, Clone)]
pub struct LlmReq {
    pub events: Vec<chat_event::Payload>,
    pub settings: LlmSettings,
    pub instructions: Option<String>,
}

pub struct LlmResp {
    pub output: Vec<chat_event::Payload>,
}

#[derive(Clone)]
pub struct LlmClientRouter {
    open_ai: Arc<OpenAIClientImpl>,
    gemini: Arc<GeminiClientImpl>,
}

impl LlmClientRouter {
    pub fn build() -> Result<Self> {
        let client = reqwest::Client::new();
        let open_ai_key = std::env::var("OPENAI_API_KEY")
            .wrap_err("set the OPENAI_API_KEY environment variable")?;
        let gemini_key = std::env::var("GEMINI_API_KEY")
            .wrap_err("set the GEMINI_API_KEY environment variable")?;

        Ok(Self {
            open_ai: Arc::new(OpenAIClientImpl::new(client.clone(), open_ai_key)),
            gemini: Arc::new(GeminiClientImpl::new(client, gemini_key)),
        })
    }
}

#[async_trait]
impl LlmClient for LlmClientRouter {
    async fn request(&self, llm_req: LlmReq) -> Result<LlmResp> {
        match llm_req.settings.default_model {
            Some(llm_settings::DefaultModel::OpenaiModel(_)) => {
                return self.open_ai.request(llm_req).await;
            }
            Some(llm_settings::DefaultModel::GeminiModel(_)) => {
                return self.gemini.request(llm_req).await;
            }
            _ => Err(eyre!("Llm settings does not specify model")),
        }
    }

    async fn stream(&self, llm_req: LlmReq) -> Result<BoxStream<'static, chat_event::Payload>> {
        match llm_req.settings.default_model {
            Some(llm_settings::DefaultModel::OpenaiModel(_)) => {
                return self.open_ai.stream(llm_req).await;
            }
            Some(llm_settings::DefaultModel::GeminiModel(_)) => {
                return self.gemini.stream(llm_req).await;
            }
            _ => Err(eyre!("Llm settings does not specify model")),
        }
    }
}
