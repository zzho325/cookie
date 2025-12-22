pub mod api;

use async_trait::async_trait;
use color_eyre::eyre::{Context, Result, eyre};
use futures_util::{StreamExt, stream::BoxStream};

use crate::{
    chat::{self, *},
    llm::*,
    service::{
        llms::{LlmClient, LlmReq, LlmResp},
        utils,
    },
};
use api::{GenerateContentReq, GenerateContentResp};

pub struct GeminiClientImpl {
    client: reqwest::Client,
    api_key: String,
}

#[async_trait]
impl LlmClient for GeminiClientImpl {
    async fn request(&self, llm_req: LlmReq) -> Result<LlmResp> {
        let model = self.extract_model(&llm_req)?;
        let req = GenerateContentReq::build(llm_req)?;
        let resp = self.generate_content(&model, req).await?;

        let mut chat_events: Vec<chat_event::Payload> = Vec::new();

        for candidate in &resp.candidates {
            for part in &candidate.content.parts {
                match part {
                    api::Part::Text { text } => {
                        chat_events.push(chat_event::Payload::Message(Message {
                            role: chat::Role::Assistant as i32,
                            msg: text.clone(),
                        }));
                    }
                }
            }
        }

        Ok(LlmResp {
            output: chat_events,
        })
    }

    async fn stream(&self, llm_req: LlmReq) -> Result<BoxStream<'static, chat_event::Payload>> {
        let model = self.extract_model(&llm_req)?;
        let req = GenerateContentReq::build(llm_req)?;
        let mut stream = self.stream_generate_content(&model, req).await?;

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut buffer = String::new();

            while let Some(result) = stream.next().await {
                match result {
                    Ok(resp) => {
                        for candidate in resp.candidates {
                            for part in candidate.content.parts {
                                match part {
                                    api::Part::Text { text } => {
                                        if !text.is_empty() {
                                            buffer.push_str(&text);
                                            let _ = tx.send(chat_event::Payload::MessageDelta(
                                                MessageDelta { delta: text },
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("stream error: {:?}", e);
                    }
                }
            }

            // Send final complete message when stream ends
            if !buffer.is_empty() {
                let _ = tx.send(chat_event::Payload::Message(Message {
                    role: chat::Role::Assistant as i32,
                    msg: buffer,
                }));
            }
        });

        Ok(Box::pin(
            tokio_stream::wrappers::UnboundedReceiverStream::new(rx),
        ))
    }
}

impl GeminiClientImpl {
    const GEMINI_HOST: &str = "https://generativelanguage.googleapis.com";

    pub fn new(client: reqwest::Client, api_key: String) -> Self {
        Self { client, api_key }
    }

    fn extract_model(&self, llm_req: &LlmReq) -> Result<GeminiModel> {
        match &llm_req.settings.default_model {
            Some(llm_settings::DefaultModel::GeminiModel(m)) => {
                GeminiModel::try_from(*m).wrap_err("Invalid Gemini model")
            }
            _ => Err(eyre!("Client and settings do not match")),
        }
    }

    async fn generate_content(
        &self,
        model: &GeminiModel,
        req: GenerateContentReq,
    ) -> Result<GenerateContentResp> {
        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            Self::GEMINI_HOST,
            model.as_str(),
            self.api_key
        );

        let resp =
            utils::post::<GenerateContentReq, GenerateContentResp>(&self.client, url, None, &req)
                .await?;
        Ok(resp)
    }

    async fn stream_generate_content(
        &self,
        model: &GeminiModel,
        req: GenerateContentReq,
    ) -> Result<BoxStream<'static, Result<GenerateContentResp>>> {
        let url = format!(
            "{}/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
            Self::GEMINI_HOST,
            model.as_str(),
            self.api_key
        );

        let stream = utils::post_stream::<GenerateContentReq, GenerateContentResp>(
            &self.client,
            url,
            None,
            &req,
        )
        .await?;
        Ok(stream)
    }
}
