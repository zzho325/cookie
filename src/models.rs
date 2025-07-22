pub mod configs;
pub mod constants;

use serde::Deserialize;
use uuid::Uuid;

use crate::service::client::api::OpenAIModel;

#[derive(Debug)]
pub enum ServiceReq {
    ChatMessage(ChatMessage),
    NewSession {
        settings: LlmSettings,
        user_message: ChatMessage,
    },
    GetSession(uuid::Uuid),
    UpdateSettings {
        session_id: Uuid,
        settings: LlmSettings,
    },
}

pub enum ServiceResp {
    ChatMessage(ChatMessage),
    Sessions(Vec<SessionSummary>),
    Session(Session),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum Role {
    User,
    Assistant(LlmSettings),
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: uuid::Uuid,
    pub session_id: uuid::Uuid,
    pub role: Role,
    pub msg: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ChatMessage {
    pub fn new(session_id: uuid::Uuid, role: Role, msg: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            session_id,
            role,
            msg,
            created_at: chrono::Utc::now(),
        }
    }
}

#[derive(Clone)]
pub struct Session {
    pub id: Uuid,
    pub chat_messages: Vec<ChatMessage>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    // FIXME: we shouldn't use previous id to support other providers
    pub previous_response_id: Option<String>,
    pub settings: LlmSettings,
    pub title: String,
}

#[derive(Debug)]
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
