use serde::{Deserialize, Serialize};

#[derive(Serialize, Default)]
pub struct ResponsesReq {
    pub model: String,
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
pub enum Role {
    User,
    Assistant,
}

#[derive(Serialize)]
pub struct InputItem {
    pub role: Role,
    pub content: String,
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
        role: Role,
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
