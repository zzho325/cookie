pub mod configs;
pub mod constants;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::service::llms::open_ai::api::OpenAIModel;

#[derive(Debug)]
pub enum ServiceReq {
    ChatMessage(ChatMessage),
    GetSession(uuid::Uuid),
}

pub enum ServiceResp {
    ChatMessage(ChatMessage),
    Sessions(Vec<SessionSummary>),
    /// Summary for one session to update title async.
    SessionSummary(SessionSummary),
    /// Fetch full session data when navigating to new session.
    Session(Session),
    Error(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub enum ChatEvent {
    ChatMessage(ChatMessage),
}

impl From<ChatMessage> for ChatEvent {
    fn from(value: ChatMessage) -> Self {
        Self::ChatMessage(value)
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: uuid::Uuid,
    pub session_id: uuid::Uuid,
    pub llm_settings: LlmSettings,
    pub role: Role,
    pub msg: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ChatMessage {
    pub fn new(session_id: uuid::Uuid, role: Role, llm_settings: LlmSettings, msg: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            session_id,
            llm_settings,
            role,
            msg,
            created_at: chrono::Utc::now(),
        }
    }
}

#[derive(Clone)]
pub struct Session {
    pub id: Uuid,
    pub chat_events: Vec<ChatEvent>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub llm_settings: LlmSettings,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: uuid::Uuid,
    pub title: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Session> for SessionSummary {
    fn from(session: Session) -> Self {
        SessionSummary {
            id: session.id,
            updated_at: session.updated_at,
            title: session.title,
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub enum LlmSettings {
    OpenAI {
        model: OpenAIModel,
        web_search: bool,
    },
    Mock {
        latency: std::time::Duration,
    },
}

impl Default for LlmSettings {
    fn default() -> Self {
        LlmSettings::OpenAI {
            model: OpenAIModel::default(),
            web_search: false,
        }
    }
}

impl LlmSettings {
    /// Returns provider display name.
    pub fn provider_name(&self) -> &'static str {
        match self {
            LlmSettings::OpenAI { .. } => "openAI",
            LlmSettings::Mock { .. } => "mock",
        }
    }

    /// Returns the model display name.
    pub fn model_name(&self) -> &str {
        match self {
            LlmSettings::OpenAI { model, .. } => model.display_name(),
            LlmSettings::Mock { .. } => "—",
        }
    }
}
