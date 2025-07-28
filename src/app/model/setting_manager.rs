use crate::models::settings::LlmSettings;

pub struct SettingManager {
    pub llm_settings: LlmSettings,
}

const LLM_PROVIDERS: &[&str] = &["OpenAI", "MOCK"];
const OPENAI_MODELS: &[&str] = &["gpt-4o", "gpt-4o-mini", "o4-mini", "o3", "o3-mini"];

impl SettingManager {
    pub fn new(llm_settings: LlmSettings) -> Self {
        Self { llm_settings }
    }
}
