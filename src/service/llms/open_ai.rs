pub mod api;

use async_trait::async_trait;
use color_eyre::eyre::{Result, WrapErr, eyre};

use crate::{
    models::{ChatEventPayload, MessagePayload, ToolEventPayload, settings::LlmSettings},
    service::{
        llms::{LlmClient, LlmReq, LlmResp, open_ai::api::Tool},
        utils,
    },
};
use api::{ContentItem, InputItem, OutputItem, ResponsesReq, ResponsesResp};

pub struct OpenAIClientImpl {
    client: reqwest::Client,
}

#[async_trait]
impl LlmClient for OpenAIClientImpl {
    async fn request(&self, llm_req: LlmReq) -> Result<LlmResp> {
        let (model, web_search) = match llm_req.settings.clone() {
            LlmSettings::OpenAI { model, web_search } => (model, web_search),
            _ => return Err(eyre!("Client and settings do not match")),
        };

        let mut tools = vec![];
        if web_search {
            tools.push(Tool::WebSearchPreview);
        }

        let req = ResponsesReq {
            model,
            instructions: llm_req.instructions,
            input: llm_req.input.iter().map(InputItem::from).collect(),
            tools,
        };
        tracing::debug!("requesting {} {:?}", req.model.display_name(), req.input);
        let resp = self.responses(req).await?;

        let mut chat_events: Vec<ChatEventPayload> = Vec::new();
        for output in &resp.output {
            match output {
                OutputItem::Message { content, role } => {
                    let mut msg = "".to_string();
                    for item in content {
                        match item {
                            ContentItem::OutputText { text, annotations } => {
                                msg.push_str(text);
                                tracing::debug!("{:?}", annotations);
                            }
                            ContentItem::Refusal { refusal } => {
                                todo!()
                            }
                        }
                    }
                    chat_events.push(
                        MessagePayload {
                            role: role.to_owned(),
                            msg,
                        }
                        .into(),
                    );
                }
                OutputItem::WebSearchCall { action, id, status } => {
                    chat_events.push(
                        ToolEventPayload::WebSearchCall {
                            action: action.clone(),
                            id: id.clone(),
                            status: status.clone(),
                        }
                        .into(),
                    );
                }
                OutputItem::FunctionCall { .. } => {
                    todo!()
                }
                OutputItem::Unknown => {
                    tracing::debug!("unimplemented type")
                }
            }
        }
        tracing::debug!("resp {chat_events:?}");
        Ok(LlmResp {
            output: chat_events,
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
        Ok(resp)
    }
}
