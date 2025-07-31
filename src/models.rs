pub mod configs;
pub mod constants;
pub mod settings;

use chrono::{DateTime, Utc};
use color_eyre::eyre::{self, eyre};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::settings::LlmSettings;

#[derive(Debug)]
pub enum ServiceReq {
    ChatMessage(ChatMessage),
    GetSession(uuid::Uuid),
}

pub enum ServiceResp {
    ChatEvent(ChatEvent),
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
pub struct ChatEvent {
    id: uuid::Uuid,
    session_id: uuid::Uuid,
    llm_settings: LlmSettings,
    created_at: chrono::DateTime<chrono::Utc>,
    payload: ChatEventPayload,
}

impl ChatEvent {
    pub fn new(
        session_id: uuid::Uuid,
        llm_settings: LlmSettings,
        payload: ChatEventPayload,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            session_id,
            llm_settings,
            created_at: chrono::Utc::now(),
            payload,
        }
    }

    pub fn session_id(&self) -> uuid::Uuid {
        self.session_id
    }

    pub fn payload(&self) -> &ChatEventPayload {
        &self.payload
    }
}

impl From<ChatMessage> for ChatEvent {
    fn from(value: ChatMessage) -> Self {
        Self {
            id: value.id,
            session_id: value.session_id,
            llm_settings: value.llm_settings,
            created_at: value.created_at,
            payload: value.payload.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChatEventPayload {
    Message(Message),
    MessageDelta(MessageDelta),
    ToolEvent(ToolEvent),
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub msg: String,
}

impl From<Message> for ChatEventPayload {
    fn from(value: Message) -> Self {
        Self::Message(value)
    }
}

#[derive(Debug, Clone)]
pub struct MessageDelta {
    pub delta: String,
}

impl From<MessageDelta> for ChatEventPayload {
    fn from(value: MessageDelta) -> Self {
        Self::MessageDelta(value)
    }
}

#[derive(Debug, Clone)]
pub enum ToolEvent {
    WebSearchCall {
        action: serde_json::Value,
        id: String,
        status: String,
    },
}

impl From<ToolEvent> for ChatEventPayload {
    fn from(value: ToolEvent) -> Self {
        Self::ToolEvent(value)
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    id: uuid::Uuid,
    session_id: uuid::Uuid,
    llm_settings: LlmSettings,
    created_at: chrono::DateTime<chrono::Utc>,
    payload: Message,
}

impl ChatMessage {
    pub fn new(session_id: uuid::Uuid, llm_settings: LlmSettings, role: Role, msg: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            session_id,
            llm_settings,
            created_at: chrono::Utc::now(),
            payload: Message { role, msg },
        }
    }

    #[cfg(test)]
    pub fn with_created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = created_at;
        self
    }

    pub fn session_id(&self) -> uuid::Uuid {
        self.session_id
    }

    pub fn llm_settings(&self) -> &LlmSettings {
        &self.llm_settings
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn payload(&self) -> &Message {
        &self.payload
    }

    pub fn msg_mut(&mut self) -> &mut String {
        &mut self.payload.msg
    }
}

impl TryFrom<ChatEvent> for ChatMessage {
    type Error = eyre::Report;
    fn try_from(value: ChatEvent) -> Result<Self, Self::Error> {
        match value.payload {
            ChatEventPayload::Message(message) => {
                return Ok(Self {
                    id: value.id,
                    session_id: value.session_id,
                    llm_settings: value.llm_settings,
                    created_at: value.created_at,
                    payload: message,
                });
            }
            ChatEventPayload::MessageDelta(message_delta) => {
                return Ok(Self {
                    id: value.id,
                    session_id: value.session_id,
                    llm_settings: value.llm_settings,
                    created_at: value.created_at,
                    payload: Message {
                        role: Role::Assistant,
                        msg: message_delta.delta,
                    },
                });
            }
            _ => return Err(eyre!("Event is not message")),
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
