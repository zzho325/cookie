pub mod configs;
pub mod constants;
pub mod settings;

use crate::{chat::*, llm::*};

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

pub const OPENAI_MODELS: &[OpenAiModel] = &[
    OpenAiModel::Gpt4o,
    OpenAiModel::Gpt4oMini,
    OpenAiModel::O4Mini,
    OpenAiModel::O3,
    OpenAiModel::O3Mini,
];

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
