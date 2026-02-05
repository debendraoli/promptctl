//! Prompt builder for creating optimized, context-aware prompts.

use crate::indexer::ProjectIndex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Available prompt sections
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Section {
    /// Language version and edition info
    Version,
    /// Code style and idioms
    Style,
    /// Error handling patterns
    ErrorHandling,
    /// Type system usage
    Types,
    /// Memory and performance
    Memory,
    /// Concurrency patterns
    Concurrency,
    /// Async programming
    Async,
    /// Testing strategies
    Testing,
    /// Project structure
    Structure,
    /// Dependencies management
    Dependencies,
    /// Documentation practices
    Documentation,
    /// Common patterns and examples
    Patterns,
    /// Tooling (linting, formatting)
    Tooling,
    /// Security best practices
    Security,
}

impl Section {
    pub const fn all() -> &'static [Section] {
        &[
            Section::Version,
            Section::Style,
            Section::ErrorHandling,
            Section::Types,
            Section::Memory,
            Section::Concurrency,
            Section::Async,
            Section::Testing,
            Section::Structure,
            Section::Dependencies,
            Section::Documentation,
            Section::Patterns,
            Section::Tooling,
            Section::Security,
        ]
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Section::Version => "version",
            Section::Style => "style",
            Section::ErrorHandling => "error-handling",
            Section::Types => "types",
            Section::Memory => "memory",
            Section::Concurrency => "concurrency",
            Section::Async => "async",
            Section::Testing => "testing",
            Section::Structure => "structure",
            Section::Dependencies => "dependencies",
            Section::Documentation => "documentation",
            Section::Patterns => "patterns",
            Section::Tooling => "tooling",
            Section::Security => "security",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "version" => Some(Section::Version),
            "style" => Some(Section::Style),
            "error-handling" | "errors" | "error" => Some(Section::ErrorHandling),
            "types" | "type" => Some(Section::Types),
            "memory" | "performance" | "perf" => Some(Section::Memory),
            "concurrency" | "concurrent" | "sync" => Some(Section::Concurrency),
            "async" | "asynchronous" => Some(Section::Async),
            "testing" | "tests" | "test" => Some(Section::Testing),
            "structure" | "project" => Some(Section::Structure),
            "dependencies" | "deps" => Some(Section::Dependencies),
            "documentation" | "docs" | "doc" => Some(Section::Documentation),
            "patterns" | "pattern" | "examples" => Some(Section::Patterns),
            "tooling" | "tools" | "lint" | "format" => Some(Section::Tooling),
            "security" | "sec" | "audit" => Some(Section::Security),
            _ => None,
        }
    }

    /// Sections included in minimal size
    pub fn minimal_set() -> HashSet<Section> {
        [Section::Version, Section::Style, Section::ErrorHandling]
            .into_iter()
            .collect()
    }

    /// Sections included in compact size
    pub fn compact_set() -> HashSet<Section> {
        [
            Section::Version,
            Section::Style,
            Section::ErrorHandling,
            Section::Types,
            Section::Testing,
            Section::Tooling,
        ]
        .into_iter()
        .collect()
    }

    /// All sections for full size
    pub fn full_set() -> HashSet<Section> {
        Section::all().iter().copied().collect()
    }
}

/// Prompt size tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PromptSize {
    /// ~500 tokens - essential rules only
    Minimal,
    /// ~1500 tokens - balanced guidelines
    #[default]
    Compact,
    /// ~3000 tokens - comprehensive
    Full,
}

impl PromptSize {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "minimal" | "min" | "tiny" | "small" => Some(PromptSize::Minimal),
            "compact" | "medium" | "default" => Some(PromptSize::Compact),
            "full" | "large" | "complete" | "all" => Some(PromptSize::Full),
            _ => None,
        }
    }

    pub fn sections(&self) -> HashSet<Section> {
        match self {
            PromptSize::Minimal => Section::minimal_set(),
            PromptSize::Compact => Section::compact_set(),
            PromptSize::Full => Section::full_set(),
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            PromptSize::Minimal => "minimal",
            PromptSize::Compact => "compact",
            PromptSize::Full => "full",
        }
    }
}

/// A structured prompt with sections
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StructuredPrompt {
    pub language: String,
    pub sections: Vec<PromptSection>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PromptSection {
    pub section: Section,
    pub title: String,
    pub content: String,
    /// Relevance indicators for smart filtering
    pub relevance_keywords: Vec<&'static str>,
}

impl StructuredPrompt {
    /// Filter sections by size tier
    pub fn filter_by_size(&self, size: PromptSize) -> String {
        let allowed = size.sections();
        self.build_with_sections(&allowed)
    }

    /// Filter by specific sections
    pub fn filter_by_sections(&self, sections: &HashSet<Section>) -> String {
        self.build_with_sections(sections)
    }

    /// Smart filter based on project index
    pub fn filter_smart(&self, index: &ProjectIndex, base_size: PromptSize) -> String {
        let mut sections = base_size.sections();

        // Add async section if async frameworks detected
        let has_async = index.frameworks.iter().any(|f| {
            matches!(
                f.name.to_lowercase().as_str(),
                "tokio" | "async-std" | "actix" | "axum"
            )
        });
        if has_async {
            sections.insert(Section::Async);
            sections.insert(Section::Concurrency);
        }

        // Add testing if tests directory exists
        if index.structure.has_tests {
            sections.insert(Section::Testing);
        }

        // Add documentation if docs exist
        if index.structure.has_docs {
            sections.insert(Section::Documentation);
        }

        // Add structure/deps for larger projects
        if index
            .languages
            .values()
            .map(|l| l.file_count)
            .sum::<usize>()
            > 20
        {
            sections.insert(Section::Structure);
            sections.insert(Section::Dependencies);
        }

        self.build_with_sections(&sections)
    }

    fn build_with_sections(&self, allowed: &HashSet<Section>) -> String {
        let mut output = String::new();

        for section in &self.sections {
            if allowed.contains(&section.section) {
                if !output.is_empty() {
                    output.push_str("\n\n");
                }
                output.push_str(&format!("## {}\n\n", section.title));
                output.push_str(&section.content);
            }
        }

        output
    }

    /// Get estimated token count
    #[allow(dead_code)]
    pub fn estimate_tokens(&self, sections: &HashSet<Section>) -> usize {
        let text = self.build_with_sections(sections);
        // Rough estimate: ~4 chars per token
        text.len() / 4
    }
}

/// Builder for creating prompts with various options
#[derive(Debug, Clone, Default)]
pub struct PromptBuilder {
    pub size: PromptSize,
    pub sections: Option<HashSet<Section>>,
    pub smart_filter: bool,
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn size(mut self, size: PromptSize) -> Self {
        self.size = size;
        self
    }

    pub fn sections(mut self, sections: HashSet<Section>) -> Self {
        self.sections = Some(sections);
        self
    }

    pub fn smart(mut self, enabled: bool) -> Self {
        self.smart_filter = enabled;
        self
    }

    pub fn build(&self, prompt: &StructuredPrompt, index: Option<&ProjectIndex>) -> String {
        // If specific sections provided, use those
        if let Some(ref sections) = self.sections {
            return prompt.filter_by_sections(sections);
        }

        // If smart filtering enabled and we have an index
        if self.smart_filter {
            if let Some(idx) = index {
                return prompt.filter_smart(idx, self.size);
            }
        }

        // Fall back to size-based filtering
        prompt.filter_by_size(self.size)
    }
}
