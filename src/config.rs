//! Configuration management for promptctl.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const CONFIG_FILENAME: &str = ".promptctl.toml";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Read(#[from] std::io::Error),
    #[error("failed to parse config file: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("config file already exists at {0}")]
    AlreadyExists(PathBuf),
}

/// Custom prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPrompt {
    /// Display name for the prompt
    pub name: String,
    /// The prompt content
    pub content: String,
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Custom prompts defined by the user
    #[serde(default)]
    pub prompts: HashMap<String, CustomPrompt>,
}

impl Config {
    /// Create a new empty configuration
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from the current directory or parent directories
    pub fn load() -> Result<Option<Self>, ConfigError> {
        if let Some(path) = Self::find_config_file() {
            let content = fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    /// Find the config file by traversing up the directory tree
    fn find_config_file() -> Option<PathBuf> {
        let mut current = std::env::current_dir().ok()?;

        loop {
            let config_path = current.join(CONFIG_FILENAME);
            if config_path.exists() {
                return Some(config_path);
            }

            if !current.pop() {
                break;
            }
        }

        // Also check home directory
        if let Some(home) = dirs::home_dir() {
            let home_config = home.join(CONFIG_FILENAME);
            if home_config.exists() {
                return Some(home_config);
            }
        }

        None
    }

    /// Initialize a new config file in the specified directory
    pub fn init(dir: &Path, force: bool) -> Result<PathBuf, ConfigError> {
        let config_path = dir.join(CONFIG_FILENAME);

        if config_path.exists() && !force {
            return Err(ConfigError::AlreadyExists(config_path));
        }

        let default_config = Self::default_config_content();
        fs::write(&config_path, default_config)?;

        Ok(config_path)
    }

    /// Generate default config file content with examples
    fn default_config_content() -> String {
        r#"# promptctl configuration file
# Add custom prompts below

# Example custom prompt:
# [prompts.python]
# name = "Python"
# description = "Python development guidelines"
# content = """
# # Python Development Guidelines
# - Use Python 3.12+
# - Follow PEP 8 style guide
# - Use type hints
# """

# [prompts.typescript]
# name = "TypeScript"
# description = "TypeScript best practices"
# content = """
# # TypeScript Guidelines
# - Use strict mode
# - Prefer interfaces over types for object shapes
# - Use const assertions where applicable
# """
"#
        .to_string()
    }

    /// Get a custom prompt by language key
    pub fn get_prompt(&self, language: &str) -> Option<&CustomPrompt> {
        self.prompts.get(language)
    }

    /// List all custom prompt keys
    pub fn custom_languages(&self) -> Vec<&str> {
        self.prompts.keys().map(String::as_str).collect()
    }
}
