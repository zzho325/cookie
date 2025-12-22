use color_eyre::{
    Result,
    eyre::{Context, eyre},
};
use serde::Deserialize;

use crate::{
    llm::*,
    models::LlmSettings,
    service::llms::{gemini, open_ai},
};

#[derive(Deserialize)]
pub struct OpenAIConfig {
    pub model: open_ai::api::Model,
    pub web_search: bool,
}

#[derive(Deserialize)]
pub struct GeminiConfig {
    pub model: gemini::api::Model,
}

/// Boot time static configs.
#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub open_ai: Option<OpenAIConfig>,
    #[serde(default)]
    pub gemini: Option<GeminiConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: "gemini".to_string(),
            open_ai: None,
            gemini: Some(GeminiConfig {
                model: gemini::api::Model::default(),
            }),
        }
    }
}

impl Config {
    /// Loads the configuration from the default location (using $XDG_CONFIG_HOME if exists or the
    /// platform’s standard config directory). If the config file doesn’t exist, returns the
    /// built-in default configuration.
    pub fn load() -> Result<Self> {
        const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
        const COOKIE_CONFIG_PATH: &str = "cookie/config.toml";

        let config_dir = std::env::var(XDG_CONFIG_HOME)
            .map(std::path::PathBuf::from)
            .or_else(|_| dirs::config_dir().ok_or_else(|| eyre!("failed to get config dir")))?;
        let config_path = config_dir.join(COOKIE_CONFIG_PATH);

        if !config_path.exists() {
            tracing::info!(
                "{} does not exist, using default config",
                config_path.display()
            );
            return Ok(Self::default());
        }

        let cfg_str = std::fs::read_to_string(config_path.clone())
            .wrap_err_with(|| format!("failed to read file: {}", config_path.display()))?;
        let cfg = toml::from_str(&cfg_str).wrap_err_with(|| "failed to parth config")?;
        Ok(cfg)
    }

    pub fn derive_llm_settings(&self) -> LlmSettings {
        match self.provider.as_str() {
            "openai" => {
                let (model, open_ai_settings) = if let Some(open_ai_config) = &self.open_ai {
                    let model: OpenAiModel = (&open_ai_config.model).into();
                    (
                        model,
                        Some(OpenAiSettings {
                            web_search: open_ai_config.web_search,
                        }),
                    )
                } else {
                    (
                        OpenAiModel::Gpt4o,
                        Some(OpenAiSettings { web_search: true }),
                    )
                };
                LlmSettings {
                    default_model: Some(llm_settings::DefaultModel::OpenaiModel(model as i32)),
                    open_ai: open_ai_settings,
                    gemini: None,
                }
            }
            _ => {
                let model = if let Some(gemini_config) = &self.gemini {
                    let m: GeminiModel = (&gemini_config.model).into();
                    m
                } else {
                    GeminiModel::GeminiModel3FlashPreview
                };
                LlmSettings {
                    default_model: Some(llm_settings::DefaultModel::GeminiModel(model as i32)),
                    open_ai: None,
                    gemini: Some(GeminiSettings {}),
                }
            }
        }
    }
}
