pub mod api;
pub mod mock;

use std::sync::Arc;

use async_trait::async_trait;
use color_eyre::eyre::{Result, WrapErr, bail, eyre};
use reqwest::header::AUTHORIZATION;
use serde::{Serialize, de::DeserializeOwned};

use crate::models::LlmSettings;
use api::{ContentItem, InputItem, OpenAIRole, OutputItem, ResponsesReq, ResponsesResp};

#[async_trait]
pub trait LlmClient {
    async fn responses(&self, llm_req: LlmReq) -> Result<LlmResp>;
}

#[derive(Debug, Clone)]
pub struct LlmReq {
    pub msg: String,
    pub previous_response_id: Option<String>,
    pub settings: LlmSettings,
    pub instructions: Option<String>,
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
    async fn responses(&self, llm_req: LlmReq) -> Result<LlmResp> {
        tracing::debug!("router handle request {:?}", llm_req);
        match llm_req.settings {
            LlmSettings::OpenAI { .. } => return self.open_ai.responses(llm_req).await,
            LlmSettings::Mock { .. } => {
                #[cfg(debug_assertions)]
                {
                    self.mock.responses(llm_req).await
                }
                #[cfg(not(debug_assertions))]
                panic!("using mock llm provider with non debug build")
            }
        }
    }
}

pub struct OpenAIClientImpl {
    client: reqwest::Client,
}

#[async_trait]
impl LlmClient for OpenAIClientImpl {
    async fn responses(&self, llm_req: LlmReq) -> Result<LlmResp> {
        let (model, web_search_) = match llm_req.settings {
            LlmSettings::OpenAI { model, web_search } => (model, web_search),
            _ => return Err(eyre!("Client and settings do not match")),
        };

        let req = ResponsesReq {
            model,
            instructions: llm_req.instructions,
            input: vec![InputItem {
                role: OpenAIRole::User,
                content: llm_req.msg,
            }],
            previous_response_id: llm_req.previous_response_id.clone(),
            ..ResponsesReq::default()
        };
        let resp = self
            .post::<ResponsesReq, ResponsesResp>("v1/responses", &req)
            .await?;

        tracing::debug!("resp {resp:?}");
        // TODO: assert role and handle refusal
        let mut texts = Vec::new();
        for output in &resp.output {
            let OutputItem::Message { content, .. } = output;
            for item in content {
                if let ContentItem::OutputText { text } = item {
                    texts.push(text.clone());
                }
            }
        }
        Ok(LlmResp {
            msg: texts.join(""),
            id: resp.id,
        })
    }
}

impl OpenAIClientImpl {
    const OPENAI_HOST: &str = "https://api.openai.com";

    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    async fn post<U: Serialize, T: DeserializeOwned>(
        &self,
        resource: &str,
        payload: &U,
    ) -> Result<T> {
        // TODO: fetch key at service bootstrap and error out if doesn't exist
        let api_key = std::env::var("OPENAI_API_KEY")
            .wrap_err("set the OPENAI_API_KEY environment variable")?;

        let resp = self
            .client
            .post(format!("{}/{}", Self::OPENAI_HOST, resource))
            .json(payload)
            .header(AUTHORIZATION, format!("Bearer {}", api_key))
            .send()
            .await
            .wrap_err("failed to send request")?;

        Self::handle_resp(resp).await
    }

    async fn handle_resp<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
        let status = resp.status();
        let body = resp.text().await.wrap_err("failed to read response body")?;
        if !status.is_success() {
            // TODO: return error
            bail!("request failed: HTTP {status} with body:\n{body}");
        }

        let result: T = serde_json::from_str(&body)
            .wrap_err_with(|| format!("could not deserialize response body:\n{}", body))?;

        tracing::debug!(body);
        Ok(result)
    }
}
