use ratatui::widgets::ListState;

use crate::{llm::*, models::OPENAI_MODELS};

pub struct SettingManager {
    llm_settings: LlmSettings,
    list_state: ListState,
}

impl SettingManager {
    pub fn new(llm_settings: LlmSettings) -> Self {
        let mut list_state = ListState::default();
        if let Some(idx) = OPENAI_MODELS.iter().position(|m| {
            if let Some(llm_settings::Provider::OpenAi(settings)) = &llm_settings.provider {
                settings.model == *m as i32
            } else {
                false
            }
        }) {
            list_state.select(Some(idx));
        } else {
            tracing::error!("unexpected current model {:?}", llm_settings)
        }

        Self {
            llm_settings,
            list_state,
        }
    }

    pub fn llm_settings(&self) -> LlmSettings {
        self.llm_settings
    }

    pub fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    // TODO: refactor this and next when implementing other providers
    pub fn select_next(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if i + 1 < OPENAI_MODELS.len() {
                if let Some(llm_settings::Provider::OpenAi(settings)) =
                    &mut self.llm_settings.provider
                {
                    self.list_state.select_next();
                    settings.model = OPENAI_MODELS[i + 1] as i32;
                } else {
                    tracing::error!(
                        "current provider does not match {:?}",
                        self.llm_settings.provider_name()
                    )
                }
            }
        }
    }

    pub fn select_previous(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if i > 0 {
                if let Some(llm_settings::Provider::OpenAi(settings)) =
                    &mut self.llm_settings.provider
                {
                    self.list_state.select_previous();
                    settings.model = OPENAI_MODELS[i - 1] as i32;
                } else {
                    tracing::error!(
                        "current provider does not match {:?}",
                        self.llm_settings.provider_name()
                    )
                }
            }
        }
    }
}
