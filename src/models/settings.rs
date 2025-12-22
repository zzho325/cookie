use crate::llm::*;

impl LlmSettings {
    /// Returns provider display name.
    pub fn provider_name(&self) -> &'static str {
        match self.default_model {
            Some(llm_settings::DefaultModel::OpenaiModel(_)) => "openAI",
            Some(llm_settings::DefaultModel::GeminiModel(_)) => "gemini",
            None => "Unspecified",
        }
    }

    /// Returns the model display name.
    pub fn model_name(&self) -> &'static str {
        match self.default_model {
            Some(llm_settings::DefaultModel::OpenaiModel(m)) => OpenAiModel::try_from(m)
                .ok()
                .map(|model| model.display_name())
                .unwrap_or("Unknown"),
            Some(llm_settings::DefaultModel::GeminiModel(m)) => GeminiModel::try_from(m)
                .ok()
                .map(|model| model.display_name())
                .unwrap_or("Unknown"),
            None => "Unspecified",
        }
    }
}
