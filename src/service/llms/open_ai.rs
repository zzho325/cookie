pub mod api;

use async_trait::async_trait;
use color_eyre::eyre::{Result, WrapErr};
use futures_util::{
    StreamExt, TryStreamExt,
    stream::{self, BoxStream},
};

use crate::{
    models::{ChatEventPayload, Message, MessageDelta, Role, ToolEvent},
    service::{
        llms::{LlmClient, LlmReq, LlmResp, open_ai::api::ResponsesStream},
        utils,
    },
};
use api::{ContentItem, OutputItem, Responses, ResponsesReq};

pub struct OpenAIClientImpl {
    client: reqwest::Client,
    api_key: String,
}

#[async_trait]
impl LlmClient for OpenAIClientImpl {
    async fn request(&self, llm_req: LlmReq) -> Result<LlmResp> {
        let req = ResponsesReq::build(llm_req)?;
        tracing::debug!(model=req.model.display_name(), input=?req.input);
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
                            ContentItem::Refusal { .. } => {
                                todo!()
                            }
                        }
                    }
                    chat_events.push(
                        Message {
                            role: role.to_owned(),
                            msg,
                        }
                        .into(),
                    );
                }
                OutputItem::WebSearchCall { action, id, status } => {
                    chat_events.push(
                        ToolEvent::WebSearchCall {
                            action: action.clone(),
                            id: id.clone(),
                            status: status.clone(),
                        }
                        .into(),
                    );
                }
                OutputItem::Unimplement => {
                    tracing::debug!("unimplemented type")
                }
            }
        }
        tracing::debug!("resp {chat_events:?}");
        Ok(LlmResp {
            output: chat_events,
        })
    }

    async fn stream(&self, llm_req: LlmReq) -> Result<BoxStream<'static, ChatEventPayload>> {
        let req = ResponsesReq::build(llm_req)?.with_streaming();
        tracing::debug!(model=req.model.display_name(), input=?req.input);
        let stream = self.stream_responses(req).await?;
        let event_stream = stream
            .filter_map(|res| async move {
                match res {
                    Ok(resp) => Some(resp),
                    Err(e) => {
                        tracing::error!("stream error: {:?}", e);
                        None
                    }
                }
            })
            .flat_map(|resp| {
                let payloads = match resp {
                    ResponsesStream::OutputTextDelta(d) => {
                        vec![ChatEventPayload::MessageDelta(MessageDelta {
                            delta: d.delta,
                        })]
                    }
                    ResponsesStream::OutputTextDone(d) => {
                        vec![ChatEventPayload::Message(Message {
                            role: Role::Assistant,
                            msg: d.text,
                        })]
                    }
                    // handle web search call here since streaming does not contain action payload
                    ResponsesStream::Completed { response } => response
                        .output
                        .into_iter()
                        .filter_map(|output| {
                            if let OutputItem::WebSearchCall { action, id, status } = output {
                                Some(ChatEventPayload::ToolEvent(ToolEvent::WebSearchCall {
                                    action,
                                    id,
                                    status,
                                }))
                            } else {
                                None
                            }
                        })
                        .collect(),
                    _ => Vec::new(),
                };
                stream::iter(payloads)
            })
            .boxed();

        Ok(event_stream)
    }
}

impl OpenAIClientImpl {
    const OPENAI_HOST: &str = "https://api.openai.com";

    pub fn new(client: reqwest::Client, api_key: String) -> Self {
        Self { client, api_key }
    }

    async fn responses(&self, req: ResponsesReq) -> Result<Responses> {
        let resp = utils::post::<ResponsesReq, Responses>(
            &self.client,
            format!("{}/v1/responses", Self::OPENAI_HOST),
            self.api_key.clone(),
            &req,
        )
        .await?;
        Ok(resp)
    }

    async fn stream_responses(
        &self,
        req: ResponsesReq,
    ) -> Result<BoxStream<'static, Result<ResponsesStream>>> {
        let stream = utils::post_stream::<ResponsesReq, ResponsesStream>(
            &self.client,
            format!("{}/v1/responses", Self::OPENAI_HOST),
            self.api_key.clone(),
            &req,
        )
        .await?;
        Ok(stream)
    }
}
