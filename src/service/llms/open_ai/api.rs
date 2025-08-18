use crate::{
    chat::{self, *},
    llm::*,
    service::llms::LlmReq,
};
use color_eyre::eyre::{Result, eyre};
use serde::{Deserialize, Serialize};

// TODO: handle error response and timeout
#[derive(Serialize, Default)]
pub struct ResponsesReq {
    pub model: Model,
    /// A system (or developer) message inserted into model's context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    pub input: Vec<InputItem>,
    pub stream: bool,
    pub tools: Vec<Tool>,
}

impl ResponsesReq {
    pub fn with_streaming(mut self) -> Self {
        self.stream = true;
        self
    }

    pub fn build(llm_req: LlmReq) -> Result<Self> {
        let (model, web_search) = match llm_req.settings.provider {
            Some(llm_settings::Provider::OpenAi(open_ai_settings)) => {
                (open_ai_settings.model(), open_ai_settings.web_search)
            }
            _ => return Err(eyre!("Client and settings do not match")),
        };

        let mut tools = vec![];
        if web_search {
            tools.push(Tool::WebSearchPreview);
        }
        Ok(ResponsesReq {
            model: model.into(),
            instructions: llm_req.instructions,
            input: llm_req
                .events
                .iter()
                .filter_map(Option::<InputItem>::from)
                .collect(),
            stream: false,
            tools,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

impl From<chat::Role> for Role {
    fn from(value: chat::Role) -> Self {
        match value {
            chat::Role::Unspecified => Role::User,
            chat::Role::User => Role::User,
            chat::Role::Assistant => Role::User,
        }
    }
}

impl From<&Role> for chat::Role {
    fn from(value: &Role) -> Self {
        match value {
            Role::User => chat::Role::User,
            Role::Assistant => chat::Role::User,
        }
    }
}
#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputItem {
    Message {
        role: Role,
        content: String,
    },
    WebSearchCall {
        id: String,
        status: String,
        action: serde_json::Value,
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

impl From<&chat_event::Payload> for Option<InputItem> {
    fn from(value: &chat_event::Payload) -> Self {
        match value {
            chat_event::Payload::Message(message) => Some(InputItem::Message {
                role: message.role().into(),
                content: message.msg.clone(),
            }),
            chat_event::Payload::MessageDelta(_) => None,
            chat_event::Payload::ToolEvent(tool_event) => match &tool_event.event {
                Some(tool_event::Event::WebSearchCall(wsc)) => Some(InputItem::WebSearchCall {
                    id: wsc.id.clone(),
                    status: wsc.status.clone(),
                    action: serde_json::from_str(&wsc.action_json)
                        .unwrap_or_else(|_| serde_json::Value::Null),
                }),
                _ => None,
            },
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Tool {
    WebSearchPreview,
    // Function {
    //     name: String,
    //     description: String,
    //     strict: bool,
    //     parameters: serde_json::Value,
    // },
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub enum Model {
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

impl From<OpenAiModel> for Model {
    fn from(value: OpenAiModel) -> Self {
        match value {
            OpenAiModel::Unspecified => Model::default(),
            OpenAiModel::Gpt4o => Model::Gpt4o,
            OpenAiModel::Gpt4oMini => Model::Gpt4oMini,
            OpenAiModel::O4Mini => Model::O4Mini,
            OpenAiModel::O3 => Model::O3,
            OpenAiModel::O3Mini => Model::O3Mini,
        }
    }
}

impl From<&Model> for OpenAiModel {
    fn from(value: &Model) -> Self {
        match value {
            Model::Gpt4o => OpenAiModel::Gpt4o,
            Model::Gpt4oMini => OpenAiModel::Gpt4oMini,
            Model::O4Mini => OpenAiModel::O4Mini,
            Model::O3 => OpenAiModel::O3,
            Model::O3Mini => OpenAiModel::O3Mini,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Responses {
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
        id: String,
        status: String,
        action: serde_json::Value,
    },
    // FunctionCall {
    //     name: String,
    //     call_id: String,
    //     arguments: String,
    // },
    #[serde(other)]
    Unimplement,
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

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ResponsesStream {
    #[serde(rename = "response.created")]
    Created { response: Responses },
    #[serde(rename = "response.in_progress")]
    InProgress { response: Responses },
    #[serde(rename = "response.completed")]
    Completed { response: Responses },
    #[serde(rename = "response.failed")]
    Failed { response: Responses },
    #[serde(rename = "response.incomplete")]
    Incomplete { response: Responses },
    #[serde(rename = "response.output_text.delta")]
    OutputTextDelta(OutputTextDelta),
    #[serde(rename = "response.output_text.done")]
    OutputTextDone(OutputTextDone),
    #[serde(other)]
    Unimplement,
}

#[derive(Deserialize, Debug)]
pub struct StreamCommon {
    pub sequence_number: u64,
    pub item_id: String,
    pub output_index: u64,
    pub content_index: u64,
    pub logprobs: Vec<()>,
}

#[derive(Deserialize, Debug)]
pub struct OutputTextDelta {
    #[serde(flatten)]
    pub common: StreamCommon,
    pub delta: String,
}

#[derive(Deserialize, Debug)]
pub struct OutputTextDone {
    #[serde(flatten)]
    pub common: StreamCommon,
    pub text: String,
}
