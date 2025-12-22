use crate::{
    chat::{self, *},
    llm::*,
    service::llms::LlmReq,
};
use color_eyre::eyre::{Result, eyre};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Default)]
pub struct GenerateContentReq {
    pub contents: Vec<Content>,
}

impl GenerateContentReq {
    pub fn build(llm_req: LlmReq) -> Result<Self> {
        let _model = match llm_req.settings.default_model {
            Some(llm_settings::DefaultModel::GeminiModel(m)) => m,
            _ => return Err(eyre!("Client and settings do not match")),
        };

        let mut contents = vec![];

        // Add system message if present
        if let Some(instructions) = llm_req.instructions {
            contents.push(Content {
                role: "user".to_string(),
                parts: vec![Part::Text { text: instructions }],
            });
        }

        // Convert chat events to Gemini format
        for event in &llm_req.events {
            if let Some(content) = Content::from_payload(event) {
                contents.push(content);
            }
        }

        Ok(GenerateContentReq { contents })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub role: String,
    pub parts: Vec<Part>,
}

impl Content {
    fn from_payload(payload: &chat_event::Payload) -> Option<Self> {
        match payload {
            chat_event::Payload::Message(message) => {
                let role = match message.role() {
                    chat::Role::User => "user",
                    chat::Role::Assistant => "model",
                    _ => "user",
                };
                Some(Content {
                    role: role.to_string(),
                    parts: vec![Part::Text {
                        text: message.msg.clone(),
                    }],
                })
            }
            chat_event::Payload::MessageDelta(_) => None,
            chat_event::Payload::ToolEvent(_) => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Part {
    Text { text: String },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenerateContentResp {
    pub candidates: Vec<Candidate>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub content: Content,
    #[allow(dead_code)]
    pub finish_reason: Option<String>,
}
