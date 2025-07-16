use serde::{Deserialize, Serialize};

// TODO: handle error response and timeout
#[derive(Serialize, Default)]
pub struct ResponsesReq {
    pub model: OpenAIModel,
    /// A system (or developer) message inserted into model's context.
    /// Not carried over to next response when using previous_response_id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    pub input: Vec<InputItem>,
    /// ID of the previous response to the model. Use for multi-turn conversations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum OpenAIRole {
    User,
    Assistant,
}

#[derive(Serialize)]
pub struct InputItem {
    pub role: OpenAIRole,
    pub content: String,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct ResponsesResp {
    pub id: String,
    pub output: Vec<OutputItem>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum OutputItem {
    #[serde(rename = "message")]
    Message {
        role: OpenAIRole,
        content: Vec<ContentItem>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ContentItem {
    #[serde(rename = "output_text")]
    OutputText {
        text: String,
        // TODO: should we handle annotations?
        // annotations: Vec<serde_json::Value>,
    },
    #[serde(rename = "refusal")]
    Refusal { refusal: String },
}
