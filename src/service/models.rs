#[derive(Debug)]
pub enum ServiceReq {
    ChatMessage(String),
}

pub enum ServiceResp {
    ChatMessage(String),
    Refusal(String),
}

#[derive(Clone)]
pub enum LlmProvider {
    OpenAI { model: String, web_search: bool },
    Mock { latency: std::time::Duration },
}

impl Default for LlmProvider {
    fn default() -> Self {
        LlmProvider::OpenAI {
            model: "gpt-4o-mini".into(),
            web_search: false,
        }
    }
}

impl LlmProvider {
    /// Returns provider display name.
    pub fn provider_name(&self) -> &'static str {
        match self {
            LlmProvider::OpenAI { .. } => "openAI",
            LlmProvider::Mock { .. } => "mock",
        }
    }

    /// Returns the model display name.
    pub fn model_name(&self) -> &str {
        match self {
            LlmProvider::OpenAI { model, .. } => model.as_str(),
            LlmProvider::Mock { .. } => "—",
        }
    }
}
