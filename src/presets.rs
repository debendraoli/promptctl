//! Preset management for saving and loading prompt configurations.

use crate::prompt_builder::{PromptSize, Section};
use crate::roles::Role;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

const PRESETS_FILENAME: &str = ".promptctl-presets.toml";

#[derive(Error, Debug)]
pub enum PresetError {
    #[error("failed to read presets file: {0}")]
    Read(#[from] std::io::Error),
    #[error("failed to parse presets file: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("failed to serialize presets: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("preset '{0}' not found")]
    NotFound(String),
    #[error("preset '{0}' already exists (use --force to overwrite)")]
    AlreadyExists(String),
}

/// A saved preset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    /// Optional description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Role to use
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Language (or auto-detect)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Prompt size tier
    #[serde(default)]
    pub size: PromptSize,
    /// Specific sections to include (overrides size)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<String>,
    /// Enable smart filtering
    #[serde(default)]
    pub smart: bool,
}

impl Preset {
    pub fn new() -> Self {
        Self {
            description: None,
            role: None,
            language: None,
            size: PromptSize::Compact,
            sections: Vec::new(),
            smart: false,
        }
    }

    pub fn with_role(mut self, role: &str) -> Self {
        self.role = Some(role.to_string());
        self
    }

    pub fn with_size(mut self, size: PromptSize) -> Self {
        self.size = size;
        self
    }

    pub fn with_sections(mut self, sections: Vec<String>) -> Self {
        self.sections = sections;
        self
    }

    pub fn with_smart(mut self, smart: bool) -> Self {
        self.smart = smart;
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    /// Parse sections from preset
    pub fn parsed_sections(&self) -> Option<HashSet<Section>> {
        if self.sections.is_empty() {
            return None;
        }

        let sections: HashSet<Section> = self
            .sections
            .iter()
            .filter_map(|s| Section::from_str(s))
            .collect();

        if sections.is_empty() {
            None
        } else {
            Some(sections)
        }
    }

    /// Parse role from preset
    #[allow(dead_code)]
    pub fn parsed_role(&self) -> Option<Role> {
        self.role.as_ref().and_then(|r| Role::from_str(r))
    }
}

impl Default for Preset {
    fn default() -> Self {
        Self::new()
    }
}

/// Collection of presets
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Presets {
    #[serde(default)]
    pub presets: HashMap<String, Preset>,
}

impl Presets {
    /// Load presets from disk
    pub fn load() -> Result<Self, PresetError> {
        if let Some(path) = Self::find_presets_file() {
            let content = fs::read_to_string(&path)?;
            let presets: Presets = toml::from_str(&content)?;
            Ok(presets)
        } else {
            Ok(Self::default())
        }
    }

    /// Find presets file (check current dir, then home)
    fn find_presets_file() -> Option<PathBuf> {
        // Check current directory
        let cwd = std::env::current_dir().ok()?;
        let local = cwd.join(PRESETS_FILENAME);
        if local.exists() {
            return Some(local);
        }

        // Check home directory
        if let Some(home) = dirs::home_dir() {
            let home_presets = home.join(PRESETS_FILENAME);
            if home_presets.exists() {
                return Some(home_presets);
            }
        }

        None
    }

    /// Get presets file path (creates in home if not exists)
    fn presets_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(PRESETS_FILENAME)
    }

    /// Save presets to disk
    pub fn save(&self) -> Result<PathBuf, PresetError> {
        let path = Self::presets_path();
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(path)
    }

    /// Get a preset by name
    pub fn get(&self, name: &str) -> Option<&Preset> {
        self.presets.get(name)
    }

    /// Add or update a preset
    pub fn set(&mut self, name: String, preset: Preset, force: bool) -> Result<(), PresetError> {
        if self.presets.contains_key(&name) && !force {
            return Err(PresetError::AlreadyExists(name));
        }
        self.presets.insert(name, preset);
        Ok(())
    }

    /// Remove a preset
    pub fn remove(&mut self, name: &str) -> Result<Preset, PresetError> {
        self.presets
            .remove(name)
            .ok_or_else(|| PresetError::NotFound(name.to_string()))
    }

    /// List all preset names
    #[allow(dead_code)]
    pub fn list(&self) -> Vec<&str> {
        self.presets.keys().map(String::as_str).collect()
    }
}

/// Built-in presets that are always available
pub fn builtin_presets() -> HashMap<String, Preset> {
    let mut presets = HashMap::new();

    presets.insert(
        "quick".to_string(),
        Preset::new()
            .with_size(PromptSize::Minimal)
            .with_description("Quick fixes - minimal context"),
    );

    presets.insert(
        "review".to_string(),
        Preset::new()
            .with_role("reviewer")
            .with_size(PromptSize::Compact)
            .with_sections(vec![
                "error-handling".to_string(),
                "types".to_string(),
                "testing".to_string(),
                "style".to_string(),
            ])
            .with_description("Code review focused"),
    );

    presets.insert(
        "security".to_string(),
        Preset::new()
            .with_role("security")
            .with_size(PromptSize::Compact)
            .with_sections(vec![
                "error-handling".to_string(),
                "types".to_string(),
                "memory".to_string(),
            ])
            .with_description("Security audit"),
    );

    presets.insert(
        "learn".to_string(),
        Preset::new()
            .with_role("mentor")
            .with_size(PromptSize::Full)
            .with_description("Learning mode - full explanations"),
    );

    presets.insert(
        "perf".to_string(),
        Preset::new()
            .with_role("performance")
            .with_sections(vec![
                "memory".to_string(),
                "concurrency".to_string(),
                "async".to_string(),
            ])
            .with_description("Performance optimization"),
    );

    presets.insert(
        "daily".to_string(),
        Preset::new()
            .with_size(PromptSize::Compact)
            .with_smart(true)
            .with_description("Daily development with smart filtering"),
    );

    presets
}
