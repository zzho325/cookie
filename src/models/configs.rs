use color_eyre::{
    Result,
    eyre::{Context, eyre},
};
use serde::Deserialize;

use crate::{
    llm::*,
    models::{LlmSettings, ModelOption},
};

#[derive(Deserialize)]
pub struct OpenAIConfig {
    #[serde(default)]
    pub web_search: bool,
}

#[derive(Deserialize)]
pub struct GeminiConfig {}

/// Boot time static configs.
#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    pub default_model: Option<ModelOption>,
    #[serde(default)]
    pub open_ai: Option<OpenAIConfig>,
    #[serde(default)]
    #[allow(dead_code)]
    pub gemini: Option<GeminiConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_model: None,
            open_ai: None,
            gemini: Some(GeminiConfig {}),
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
        let default_model = self.default_model.as_ref().map(|m| m.to_proto_model());

        // Always populate provider settings
        let open_ai_settings = if let Some(open_ai_config) = &self.open_ai {
            Some(OpenAiSettings {
                web_search: open_ai_config.web_search,
            })
        } else {
            Some(OpenAiSettings { web_search: false })
        };

        let gemini_settings = Some(GeminiSettings {});

        LlmSettings {
            default_model,
            open_ai: open_ai_settings,
            gemini: gemini_settings,
        }
    }
}
