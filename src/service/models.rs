use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ResponsesReq {
    pub model: String,
    pub instructions: String,
    pub input: String, // TODO: this can be an array of input items
}

#[derive(Deserialize, Debug)]
pub struct ResponsesResp {
    pub output: Vec<Output>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Output {
    #[serde(rename = "message")]
    Message {
        status: String,
        role: String,
        content: Vec<Content>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "output_text")]
    OutputText {
        text: String,
        annotations: Vec<serde_json::Value>,
    },
}
