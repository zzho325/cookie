use crate::{
    models::{ChatEventPayload, Role, ToolEvent, settings::LlmSettings},
    service::llms::LlmReq,
};
use color_eyre::eyre::{Result, eyre};
use serde::{Deserialize, Serialize};

// TODO: handle error response and timeout
#[derive(Serialize, Default)]
pub struct ResponsesReq {
    pub model: OpenAIModel,
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
        let (model, web_search) = match llm_req.settings.clone() {
            LlmSettings::OpenAI { model, web_search } => (model, web_search),
            _ => return Err(eyre!("Client and settings do not match")),
        };

        let mut tools = vec![];
        if web_search {
            tools.push(Tool::WebSearchPreview);
        }
        Ok(ResponsesReq {
            model,
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

impl From<&ChatEventPayload> for Option<InputItem> {
    fn from(value: &ChatEventPayload) -> Self {
        match value {
            ChatEventPayload::Message(p) => Some(InputItem::Message {
                role: p.role.clone(),
                content: p.msg.clone(),
            }),
            ChatEventPayload::ToolEvent(te) => Some(match te {
                ToolEvent::WebSearchCall { action, id, status } => InputItem::WebSearchCall {
                    action: action.clone(),
                    id: id.clone(),
                    status: status.clone(),
                },
            }),
            ChatEventPayload::MessageDelta(_) => None,
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
        action: serde_json::Value,
        id: String,
        status: String,
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
