use serde::{Deserialize, Serialize};

use crate::models::{ChatEventPayload, Role, ToolEventPayload};

// TODO: handle error response and timeout
#[derive(Serialize, Default)]
pub struct ResponsesReq {
    pub model: OpenAIModel,
    /// A system (or developer) message inserted into model's context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    pub input: Vec<InputItem>,
    pub tools: Vec<Tool>,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputItem {
    Message {
        role: Role,
        content: String,
    },
    WebSearchCall {
        action: serde_json::Value,
        id: String,
        status: String,
    },
    FunctionCall {
        name: String,
        call_id: String,
        arguments: String,
    },
    FunctionCallOutput {
        name: String,
        call_id: String,
        output: String,
    },
}

impl From<&ChatEventPayload> for InputItem {
    fn from(value: &ChatEventPayload) -> Self {
        match value {
            ChatEventPayload::Message(payload) => InputItem::Message {
                role: payload.role.clone(),
                content: payload.msg.clone(),
            },
            ChatEventPayload::ToolEvent(payload) => match payload {
                ToolEventPayload::WebSearchCall { action, id, status } => {
                    InputItem::WebSearchCall {
                        action: action.clone(),
                        id: id.clone(),
                        status: status.clone(),
                    }
                }
            },
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Tool {
    WebSearchPreview,
    Function {
        name: String,
        description: String,
        strict: bool,
        parameters: serde_json::Value,
    },
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub enum OpenAIModel {
    #[default]
    #[serde(rename = "gpt-4o")]
    Gpt4o,
    #[serde(rename = "gpt-4o-mini")]
    Gpt4oMini,
    #[serde(rename = "o4-mini")]
    O4Mini,
    #[serde(rename = "o3")]
    O3,
    #[serde(rename = "o3-mini")]
    O3Mini,
}

impl OpenAIModel {
    pub fn display_name(&self) -> &'static str {
        match self {
            OpenAIModel::Gpt4o => "4o",
            OpenAIModel::Gpt4oMini => "4o-mini",
            OpenAIModel::O4Mini => "o4-mini",
            OpenAIModel::O3 => "o3",
            OpenAIModel::O3Mini => "o3-mini",
        }
    }
}

pub const OPENAI_MODELS: &[OpenAIModel] = &[
    OpenAIModel::Gpt4o,
    OpenAIModel::Gpt4oMini,
    OpenAIModel::O4Mini,
    OpenAIModel::O3,
    OpenAIModel::O3Mini,
];

#[derive(Deserialize, Debug)]
pub struct ResponsesResp {
    pub id: String,
    pub output: Vec<OutputItem>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputItem {
    Message {
        role: Role,
        content: Vec<ContentItem>,
    },
    WebSearchCall {
        action: serde_json::Value,
        id: String,
        status: String,
    },
    FunctionCall {
        name: String,
        call_id: String,
        arguments: String,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentItem {
    OutputText {
        text: String,
        annotations: Vec<Annotation>,
    },
    Refusal {
        refusal: String,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Annotation {
    UrlCitation {
        /// The index of the last character of the URL citation in the message.
        end_index: u64,
        /// The index of the first character of the URL citation in the message.
        start_index: u64,
        title: String,
        url: String,
    },
}
