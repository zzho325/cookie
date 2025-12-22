pub mod configs;
pub mod constants;
pub mod settings;

use crate::{chat::*, llm::*};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub enum ModelOption {
    OpenAI(crate::service::llms::open_ai::api::Model),
    Gemini(crate::service::llms::gemini::api::Model),
}

impl ModelOption {
    pub fn display_name(&self) -> String {
        match self {
            ModelOption::OpenAI(m) => {
                let proto_model: OpenAiModel = m.into();
                format!("openai: {}", proto_model.display_name())
            }
            ModelOption::Gemini(m) => {
                let proto_model: GeminiModel = m.into();
                format!("gemini: {}", proto_model.display_name())
            }
        }
    }

    pub fn all_models() -> Vec<ModelOption> {
        use strum::IntoEnumIterator;

        let mut models = Vec::new();

        // Iterate over all OpenAI models
        for m in OpenAiModel::iter() {
            if m != OpenAiModel::Unspecified {
                models.push(ModelOption::OpenAI(m.into()));
            }
        }

        // Iterate over all Gemini models
        for m in GeminiModel::iter() {
            if m != GeminiModel::Unspecified {
                models.push(ModelOption::Gemini(m.into()));
            }
        }

        models
    }

    pub fn to_proto_model(&self) -> llm_settings::DefaultModel {
        match self {
            ModelOption::OpenAI(m) => {
                let proto_model: OpenAiModel = m.into();
                llm_settings::DefaultModel::OpenaiModel(proto_model as i32)
            }
            ModelOption::Gemini(m) => {
                let proto_model: GeminiModel = m.into();
                llm_settings::DefaultModel::GeminiModel(proto_model as i32)
            }
        }
    }
}

#[derive(Debug)]
pub enum ServiceReq {
    /// Sends user message.
    ChatMessage(ChatEvent),
    /// Fetches session by session_id.
    GetSession(String),
    /// Deletes session by session_id.
    DeleteSession(String),
}

pub enum ServiceResp {
    ChatEvent(ChatEvent),
    Sessions(Vec<ChatSession>),
    /// Summary for one session to update title async.
    SessionSummary(ChatSession),
    /// Fetch full session data when navigating to new session.
    Session(ChatSession),
    Error(String),
}

impl OpenAiModel {
    pub fn display_name(&self) -> &'static str {
        match self {
            OpenAiModel::Unspecified => "default",
            OpenAiModel::Gpt4o => "4o",
            OpenAiModel::Gpt4oMini => "4o-mini",
            OpenAiModel::O4Mini => "o4-mini",
            OpenAiModel::O3 => "o3",
            OpenAiModel::O3Mini => "o3-mini",
        }
    }
}

impl GeminiModel {
    pub fn display_name(&self) -> &'static str {
        match self {
            GeminiModel::Unspecified => "default",
            GeminiModel::GeminiModel3ProPreview => "3-pro",
            GeminiModel::GeminiModel3FlashPreview => "3-flash",
        }
    }
}

impl ChatEvent {
    pub fn new(
        session_id: String,
        llm_settings: Option<LlmSettings>,
        payload: chat_event::Payload,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            session_id,
            llm_settings,
            created_at: None,
            payload: Some(payload),
        }
    }

    #[cfg(test)]
    pub fn with_created_at(mut self, created_at: prost_types::Timestamp) -> Self {
        self.created_at = Some(created_at);
        self
    }
}

impl ChatSession {
    pub fn new(id: String, llm_settings: Option<LlmSettings>) -> Self {
        Self {
            id,
            llm_settings,
            ..Default::default()
        }
    }
}
