use ratatui::widgets::ListState;

use crate::{
    llm::*,
    models::{GEMINI_MODELS, OPENAI_MODELS},
};

#[derive(Clone, Copy)]
pub enum ModelOption {
    OpenAI(OpenAiModel),
    Gemini(GeminiModel),
}

impl ModelOption {
    pub fn display_name(&self) -> String {
        match self {
            ModelOption::OpenAI(m) => format!("openai: {}", m.display_name()),
            ModelOption::Gemini(m) => format!("gemini: {}", m.display_name()),
        }
    }

    pub fn all_models() -> Vec<ModelOption> {
        let mut models = Vec::new();
        for m in OPENAI_MODELS {
            models.push(ModelOption::OpenAI(*m));
        }
        for m in GEMINI_MODELS {
            models.push(ModelOption::Gemini(*m));
        }
        models
    }
}

pub struct SettingManager {
    llm_settings: LlmSettings,
    list_state: ListState,
}

impl SettingManager {
    pub fn new(llm_settings: LlmSettings) -> Self {
        let mut list_state = ListState::default();
        let all_models = ModelOption::all_models();

        match &llm_settings.default_model {
            Some(llm_settings::DefaultModel::OpenaiModel(m)) => {
                if let Some(idx) = all_models.iter().position(
                    |opt| matches!(opt, ModelOption::OpenAI(model) if *model as i32 == *m),
                ) {
                    list_state.select(Some(idx));
                } else {
                    tracing::error!("unexpected current OpenAI model {:?}", llm_settings)
                }
            }
            Some(llm_settings::DefaultModel::GeminiModel(m)) => {
                if let Some(idx) = all_models.iter().position(
                    |opt| matches!(opt, ModelOption::Gemini(model) if *model as i32 == *m),
                ) {
                    list_state.select(Some(idx));
                } else {
                    tracing::error!("unexpected current Gemini model {:?}", llm_settings)
                }
            }
            _ => {
                tracing::error!("no model specified {:?}", llm_settings)
            }
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

    pub fn select_next(&mut self) {
        let Some(i) = self.list_state.selected() else {
            return;
        };

        let all_models = ModelOption::all_models();
        if i + 1 >= all_models.len() {
            return;
        }

        self.list_state.select_next();
        self.update_model_from_index(i + 1, &all_models);
    }

    pub fn select_previous(&mut self) {
        let Some(i) = self.list_state.selected() else {
            return;
        };

        if i == 0 {
            return;
        }

        let all_models = ModelOption::all_models();
        self.list_state.select_previous();
        self.update_model_from_index(i - 1, &all_models);
    }

    fn update_model_from_index(&mut self, idx: usize, all_models: &[ModelOption]) {
        if let Some(model_option) = all_models.get(idx) {
            match model_option {
                ModelOption::OpenAI(m) => {
                    self.llm_settings.default_model =
                        Some(llm_settings::DefaultModel::OpenaiModel(*m as i32));
                    if self.llm_settings.open_ai.is_none() {
                        self.llm_settings.open_ai = Some(OpenAiSettings { web_search: true });
                    }
                }
                ModelOption::Gemini(m) => {
                    self.llm_settings.default_model =
                        Some(llm_settings::DefaultModel::GeminiModel(*m as i32));
                    if self.llm_settings.gemini.is_none() {
                        self.llm_settings.gemini = Some(GeminiSettings {});
                    }
                }
            }
        }
    }
}
