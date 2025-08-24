pub mod api;

use async_trait::async_trait;
use color_eyre::eyre::Result;
use futures_util::{
    StreamExt,
    stream::{self, BoxStream},
};

use crate::{
    chat::{self, *},
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
        // tracing::debug!(model=?req.model, input=?req.input);
        let resp = self.responses(req).await?;

        let mut chat_events: Vec<chat_event::Payload> = Vec::new();
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
                    chat_events.push(chat_event::Payload::Message(Message {
                        role: chat::Role::from(role) as i32,
                        msg,
                    }));
                }
                OutputItem::WebSearchCall { action, id, status } => {
                    chat_events.push(chat_event::Payload::ToolEvent(ToolEvent {
                        event: Some(tool_event::Event::WebSearchCall(
                            tool_event::WebSearchCall {
                                id: id.clone(),
                                status: status.clone(),
                                action_json: action.to_string(),
                            },
                        )),
                    }));
                }
                OutputItem::Unimplement => {
                    tracing::debug!("unimplemented type")
                }
            }
        }
        // tracing::debug!("resp {chat_events:?}");
        Ok(LlmResp {
            output: chat_events,
        })
    }

    async fn stream(&self, llm_req: LlmReq) -> Result<BoxStream<'static, chat_event::Payload>> {
        let req = ResponsesReq::build(llm_req)?.with_streaming();
        tracing::debug!(model=?req.model, input=?req.input);
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
                        vec![chat_event::Payload::MessageDelta(MessageDelta {
                            delta: d.delta,
                        })]
                    }
                    ResponsesStream::OutputTextDone(d) => {
                        vec![chat_event::Payload::Message(Message {
                            role: Role::Assistant as i32,
                            msg: d.text,
                        })]
                    }
                    // handle web search call here since streaming does not contain action payload
                    ResponsesStream::Completed { response } => response
                        .output
                        .into_iter()
                        .filter_map(|output| {
                            if let OutputItem::WebSearchCall { action, id, status } = output {
                                Some(chat_event::Payload::ToolEvent(ToolEvent {
                                    event: Some(tool_event::Event::WebSearchCall(
                                        tool_event::WebSearchCall {
                                            id,
                                            status,
                                            action_json: action.to_string(),
                                        },
                                    )),
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
