use async_trait::async_trait;
use color_eyre::eyre::{Result, WrapErr, bail};
use reqwest::header::AUTHORIZATION;
use serde::{Serialize, de::DeserializeOwned};

use crate::service::models::{ResponsesReq, ResponsesResp};

#[async_trait]
pub trait OpenAIClient {
    async fn responses(&self, req: ResponsesReq) -> Result<ResponsesResp>;
}

pub struct OpenAIClientImpl {
    client: reqwest::Client,
}

#[async_trait]
impl OpenAIClient for OpenAIClientImpl {
    async fn responses(&self, req: ResponsesReq) -> Result<ResponsesResp> {
        let resp = self
            .post::<ResponsesReq, ResponsesResp>("v1/responses", &req)
            .await?;
        Ok(resp)
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
        let body = resp.text().await.wrap_err("Failed to read response body")?;

        if !status.is_success() {
            bail!("request failed: HTTP {status} with body:\n{body}");
        }

        let result: T = serde_json::from_str(&body)
            .wrap_err_with(|| format!("Could not deserialize response body:\n{}", body))?;

        Ok(result)
    }
}
