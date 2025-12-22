pub mod configs;
pub mod constants;
pub mod settings;

use crate::{chat::*, llm::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// Serde implementations for proto enums to match API names
impl Serialize for OpenAiModel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let name = match self {
            OpenAiModel::Unspecified => "gpt-4o",
            OpenAiModel::Gpt4o => "gpt-4o",
            OpenAiModel::Gpt4oMini => "gpt-4o-mini",
            OpenAiModel::O4Mini => "o4-mini",
            OpenAiModel::O3 => "o3-deep-research",
            OpenAiModel::O3Mini => "o3-mini",
        };
        serializer.serialize_str(name)
    }
}

impl<'de> Deserialize<'de> for OpenAiModel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "gpt-4o" => Ok(OpenAiModel::Gpt4o),
            "gpt-4o-mini" => Ok(OpenAiModel::Gpt4oMini),
            "o4-mini" => Ok(OpenAiModel::O4Mini),
            "o3-deep-research" => Ok(OpenAiModel::O3),
            "o3-mini" => Ok(OpenAiModel::O3Mini),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &[
                    "gpt-4o",
                    "gpt-4o-mini",
                    "o4-mini",
                    "o3-deep-research",
                    "o3-mini",
                ],
            )),
        }
    }
}

impl Serialize for GeminiModel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let name = match self {
            GeminiModel::Unspecified => "gemini-3-flash-preview",
            GeminiModel::GeminiModel3ProPreview => "gemini-3-pro-preview",
            GeminiModel::GeminiModel3FlashPreview => "gemini-3-flash-preview",
        };
        serializer.serialize_str(name)
    }
}

impl<'de> Deserialize<'de> for GeminiModel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "gemini-3-pro-preview" => Ok(GeminiModel::GeminiModel3ProPreview),
            "gemini-3-flash-preview" => Ok(GeminiModel::GeminiModel3FlashPreview),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &["gemini-3-pro-preview", "gemini-3-flash-preview"],
            )),
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub enum ModelOption {
    OpenAI(OpenAiModel),
    Gemini(GeminiModel),
}

impl ModelOption {
    pub fn display_name(&self) -> String {
        match self {
            ModelOption::OpenAI(m) => format!("openai: {}", m.display_name()),
            ModelOption::Gemini(m) => format!("gemini: {}", m.display_name()),
        }
    }

    pub fn all_models() -> Vec<ModelOption> {
        use strum::IntoEnumIterator;

        let mut models = Vec::new();

        // Iterate over all OpenAI models
        for m in OpenAiModel::iter() {
            if m != OpenAiModel::Unspecified {
                models.push(ModelOption::OpenAI(m));
            }
        }

        // Iterate over all Gemini models
        for m in GeminiModel::iter() {
            if m != GeminiModel::Unspecified {
                models.push(ModelOption::Gemini(m));
            }
        }

        models
    }

    pub fn to_proto_model(&self) -> llm_settings::DefaultModel {
        match self {
            ModelOption::OpenAI(m) => llm_settings::DefaultModel::OpenaiModel(*m as i32),
            ModelOption::Gemini(m) => llm_settings::DefaultModel::GeminiModel(*m as i32),
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

    pub fn as_str(&self) -> &'static str {
        match self {
            GeminiModel::Unspecified => "gemini-3-flash-preview",
            GeminiModel::GeminiModel3ProPreview => "gemini-3-pro-preview",
            GeminiModel::GeminiModel3FlashPreview => "gemini-3-flash-preview",
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
