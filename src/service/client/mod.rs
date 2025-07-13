pub mod api;
pub mod mock;

use async_trait::async_trait;
use color_eyre::eyre::{Result, WrapErr, bail, eyre};
use reqwest::header::AUTHORIZATION;
use serde::{Serialize, de::DeserializeOwned};

use crate::models::LlmSettings;
use api::{ContentItem, InputItem, OutputItem, ResponsesReq, ResponsesResp, Role};

#[async_trait]
pub trait LlmClient {
    async fn responses(&self, llm_req: LlmReq) -> Result<LlmResp>;
}

pub struct LlmReq {
    pub msg: String,
    pub previous_response_id: Option<String>,
    pub settings: LlmSettings,
}

pub struct LlmResp {
    pub msg: String,
    pub id: String,
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
            input: vec![InputItem {
                role: Role::User,
                content: llm_req.msg,
            }],
            previous_response_id: llm_req.previous_response_id.clone(),
            ..ResponsesReq::default()
        };
        let resp = self
            .post::<ResponsesReq, ResponsesResp>("v1/responses", &req)
            .await?;

        tracing::debug!("req model {}", req.model.display_name());
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
        // TODO: fetch kay at service bootstrap and error out if doesn't exist
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
            bail!("request failed: HTTP {status} with body:\n{body}");
        }

        let result: T = serde_json::from_str(&body)
            .wrap_err_with(|| format!("could not deserialize response body:\n{}", body))?;

        Ok(result)
    }
}
