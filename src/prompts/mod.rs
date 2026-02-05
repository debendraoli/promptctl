//! Built-in prompt templates for various programming languages.

mod go;
mod leo;
mod rust;

use crate::prompt_builder::StructuredPrompt;
use std::collections::HashMap;

pub use go::GO_PROMPT;
pub use leo::LEO_PROMPT;
pub use rust::RUST_PROMPT;

/// Returns a map of all built-in prompts (raw strings for backward compatibility)
pub fn builtin_prompts() -> HashMap<&'static str, &'static str> {
    let mut prompts = HashMap::new();
    prompts.insert("rust", RUST_PROMPT);
    prompts.insert("go", GO_PROMPT);
    prompts.insert("leo", LEO_PROMPT);
    prompts
}

/// Get a built-in prompt by language name (raw string)
pub fn get_builtin_prompt(language: &str) -> Option<&'static str> {
    builtin_prompts()
        .get(language.to_lowercase().as_str())
        .copied()
}

/// Get a structured prompt for a language
pub fn get_structured_prompt(language: &str) -> Option<StructuredPrompt> {
    match language.to_lowercase().as_str() {
        "rust" => Some(rust::structured_prompt()),
        "go" => Some(go::structured_prompt()),
        "leo" => Some(leo::structured_prompt()),
        _ => None,
    }
}

/// List all available built-in language names
pub fn available_languages() -> Vec<&'static str> {
    vec!["rust", "go", "leo"]
}
