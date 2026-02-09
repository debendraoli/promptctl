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

/// How a custom prompt interacts with the built-in prompt for the same language
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PromptMode {
    #[default]
    Replace,
    Prepend,
    Append,
    Merge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPrompt {
    pub name: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub mode: PromptMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepend: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub append: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub prompts: HashMap<String, CustomPrompt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_agent: Option<String>,
}

impl Config {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load() -> Result<Option<Self>, ConfigError> {
        if let Some(path) = Self::find_config_file() {
            let content = fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

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

        if let Some(home) = dirs::home_dir() {
            let home_config = home.join(CONFIG_FILENAME);
            if home_config.exists() {
                return Some(home_config);
            }
        }

        None
    }

    pub fn init(dir: &Path, force: bool) -> Result<PathBuf, ConfigError> {
        let config_path = dir.join(CONFIG_FILENAME);

        if config_path.exists() && !force {
            return Err(ConfigError::AlreadyExists(config_path));
        }

        let default_config = Self::default_config_content();
        fs::write(&config_path, default_config)?;

        Ok(config_path)
    }

    fn default_config_content() -> String {
        r#"# promptctl configuration file
# Customize prompts for your project.
# Re-run 'promptctl init <agent>' after editing.

# Optional: set a default AI agent for this project
# default_agent = "copilot"  # copilot, claude, cursor, codex, aider

# ── Prompt modes ──
# mode = "replace"  → fully replace the built-in prompt (default)
# mode = "append"   → add your content AFTER the built-in prompt
# mode = "prepend"  → add your content BEFORE the built-in prompt
# mode = "merge"    → use both prepend and append around the built-in

# Example: extend built-in Rust prompt with project-specific rules
# [prompts.rust]
# name = "Rust"
# mode = "append"
# append = """
# ## Project-Specific Rules
# - Use workspace dependencies from root Cargo.toml
# - All public APIs must have doc comments
# """
"#
        .to_string()
    }

    pub fn get_prompt(&self, language: &str) -> Option<&CustomPrompt> {
        self.prompts.get(language)
    }

    /// Resolve a prompt for a language, applying merge logic.
    /// Returns the final prompt string, merging custom + built-in as needed.
    pub fn resolve_prompt(&self, language: &str, builtin: Option<&str>) -> Option<String> {
        let custom = self.prompts.get(language)?;

        match custom.mode {
            PromptMode::Replace => {
                // Full replacement — use content (backward compat)
                Some(custom.content.clone())
            }
            PromptMode::Prepend => {
                let pre = custom.prepend.as_deref().unwrap_or(&custom.content);
                let base = builtin.unwrap_or("");
                Some(format!("{pre}\n\n{base}"))
            }
            PromptMode::Append => {
                let base = builtin.unwrap_or("");
                let post = custom.append.as_deref().unwrap_or(&custom.content);
                Some(format!("{base}\n\n{post}"))
            }
            PromptMode::Merge => {
                let pre = custom.prepend.as_deref().unwrap_or("");
                let base = builtin.unwrap_or("");
                let post = custom.append.as_deref().unwrap_or("");
                let mut result = String::new();
                if !pre.is_empty() {
                    result.push_str(pre);
                    result.push_str("\n\n");
                }
                result.push_str(base);
                if !post.is_empty() {
                    result.push_str("\n\n");
                    result.push_str(post);
                }
                Some(result)
            }
        }
    }

    pub fn custom_languages(&self) -> Vec<&str> {
        self.prompts.keys().map(String::as_str).collect()
    }
}
