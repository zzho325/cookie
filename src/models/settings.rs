use serde::Deserialize;

use crate::service::llms::open_ai::api::OpenAIModel;

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
    pub fn model_name(&self) -> &'static str {
        match self {
            LlmSettings::OpenAI { model, .. } => model.display_name(),
            LlmSettings::Mock { .. } => "—",
        }
    }
}
