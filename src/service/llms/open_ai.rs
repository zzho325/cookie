pub mod api;

use async_trait::async_trait;
use color_eyre::eyre::{Result, WrapErr, eyre};

use crate::{
    models::LlmSettings,
    service::{
        llms::{LlmClient, LlmReq, LlmResp},
        utils,
    },
};
use api::{ContentItem, InputItem, OutputItem, ResponsesReq, ResponsesResp, Role};

pub struct OpenAIClientImpl {
    client: reqwest::Client,
}

#[async_trait]
impl LlmClient for OpenAIClientImpl {
    async fn request(&self, llm_req: LlmReq) -> Result<LlmResp> {
        let (model, web_search_) = match llm_req.settings {
            LlmSettings::OpenAI { model, web_search } => (model, web_search),
            _ => return Err(eyre!("Client and settings do not match")),
        };

        let req = ResponsesReq {
            model,
            instructions: llm_req.instructions,
            input: vec![InputItem::Message {
                role: Role::User,
                content: llm_req.msg,
            }],
            previous_response_id: llm_req.previous_response_id.clone(),
            tools: vec![],
        };

        let resp = self.responses(req).await?;

        // TODO: assert role and handle refusal
        let mut texts = Vec::new();
        for output in &resp.output {
            match output {
                OutputItem::Message { content, .. } => {
                    for item in content {
                        if let ContentItem::OutputText { text } = item {
                            texts.push(text.clone());
                        }
                    }
                }
                OutputItem::FunctionCall {
                    name,
                    call_id,
                    arguments,
                } => {
                    texts.push(format!("function call {name}, {call_id}, {arguments:?}"));
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

    async fn responses(&self, req: ResponsesReq) -> Result<ResponsesResp> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .wrap_err("set the OPENAI_API_KEY environment variable")?;

        let resp = utils::post::<ResponsesReq, ResponsesResp>(
            &self.client,
            format!("{}/v1/responses", Self::OPENAI_HOST),
            api_key,
            &req,
        )
        .await?;

        tracing::debug!("openai resp {resp:?}");
        Ok(resp)
    }
}
