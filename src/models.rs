pub mod configs;
pub mod constants;
pub mod settings;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::settings::LlmSettings;

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
pub struct ChatEventBase {
    pub id: uuid::Uuid,
    pub session_id: uuid::Uuid,
    pub llm_settings: LlmSettings,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ChatEventBase {
    pub fn new(session_id: uuid::Uuid, llm_settings: LlmSettings) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            session_id,
            llm_settings,
            created_at: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatEvent {
    base: ChatEventBase,
    payload: ChatEventPayload,
}

impl ChatEvent {
    pub fn new(
        session_id: uuid::Uuid,
        llm_settings: LlmSettings,
        payload: ChatEventPayload,
    ) -> Self {
        Self {
            base: ChatEventBase::new(session_id, llm_settings),
            payload,
        }
    }

    pub fn payload(&self) -> &ChatEventPayload {
        &self.payload
    }

    /// Converts into a `ChatMessage` if this event is a message.
    pub fn maybe_into_chat_message(self) -> Option<ChatMessage> {
        if let ChatEventPayload::Message(payload) = self.payload {
            return Some(ChatMessage {
                base: self.base,
                payload,
            });
        }
        None
    }
}

impl From<ChatMessage> for ChatEvent {
    fn from(value: ChatMessage) -> Self {
        Self {
            base: value.base,
            payload: value.payload.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChatEventPayload {
    Message(MessagePayload),
    ToolEvent(ToolEventPayload),
}

#[derive(Debug, Clone)]
pub struct MessagePayload {
    pub role: Role,
    pub msg: String,
}

impl From<MessagePayload> for ChatEventPayload {
    fn from(value: MessagePayload) -> Self {
        Self::Message(value)
    }
}

impl From<ToolEventPayload> for ChatEventPayload {
    fn from(value: ToolEventPayload) -> Self {
        Self::ToolEvent(value)
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    base: ChatEventBase,
    payload: MessagePayload,
}

impl ChatMessage {
    pub fn new(session_id: uuid::Uuid, llm_settings: LlmSettings, role: Role, msg: String) -> Self {
        Self {
            base: ChatEventBase::new(session_id, llm_settings),
            payload: MessagePayload { role, msg },
        }
    }

    pub fn session_id(&self) -> uuid::Uuid {
        self.base.session_id
    }

    pub fn llm_settings(&self) -> &LlmSettings {
        &self.base.llm_settings
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.base.created_at
    }

    pub fn payload(&self) -> &MessagePayload {
        &self.payload
    }
}

#[derive(Debug, Clone)]
pub enum ToolEventPayload {
    WebSearchCall {
        action: serde_json::Value,
        id: String,
        status: String,
    },
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
