use color_eyre::{
    Result,
    eyre::{Context, eyre},
};
use serde::Deserialize;

use crate::{llm::*, models::LlmSettings, service::llms::open_ai::api::Model};

#[derive(Deserialize)]
pub struct OpenAIConfig {
    pub model: Model,
    pub web_search: bool,
}

/// Boot time static configs.
#[derive(Deserialize)]
pub struct Config {
    pub open_ai: OpenAIConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            open_ai: OpenAIConfig {
                model: Model::default(),
                web_search: true,
            },
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
        let model: OpenAiModel = (&self.open_ai.model).into();
        LlmSettings {
            provider: Some(llm_settings::Provider::OpenAi(OpenAiSettings {
                model: model as i32,
                web_search: self.open_ai.web_search,
            })),
        }
    }
}
