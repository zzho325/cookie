use crate::llm::*;

impl LlmSettings {
    /// Returns provider display name.
    pub fn provider_name(&self) -> &'static str {
        match self.provider {
            Some(llm_settings::Provider::OpenAi(_)) => "openAI",
            None => "Unspecified",
        }
    }

    /// Returns the model display name.
    pub fn model_name(&self) -> &'static str {
        match self.provider {
            Some(llm_settings::Provider::OpenAi(settings)) => settings.model().display_name(),
            None => "Unspecified",
        }
    }
}
