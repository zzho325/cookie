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
    pub tools: Vec<Tool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputItem {
    Message {
        role: Role,
        content: String,
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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputItem {
    Message {
        role: Role,
        content: Vec<ContentItem>,
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
    #[serde(rename = "output_text")]
    OutputText {
        text: String,
        // TODO: should we handle annotations?
        // annotations: Vec<serde_json::Value>,
    },
    #[serde(rename = "refusal")]
    Refusal { refusal: String },
}
