pub mod configs;

use serde::Deserialize;

use crate::service::client::api::OpenAIModel;

#[derive(Debug)]
pub enum ServiceReq {
    ChatMessage(String),
    UpdateSettings(LlmSettings),
}

pub enum ServiceResp {
    ChatMessage(String),
    Refusal(String),
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
