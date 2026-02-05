//! CLI argument parsing using clap.

use clap::{Parser, Subcommand, ValueEnum};

/// A personal CLI tool for managing coding prompts across projects
#[derive(Parser, Debug)]
#[command(name = "promptctl")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all available language prompts
    List,

    /// Show the prompt for a specific language
    Show {
        /// Language name (e.g., rust, go)
        language: String,

        /// Role to apply (e.g., developer, reviewer, security)
        #[arg(short, long)]
        role: Option<String>,
    },

    /// Copy the prompt for a specific language to clipboard
    Copy {
        /// Language name (e.g., rust, go)
        language: String,

        /// Role to apply (e.g., developer, reviewer, security)
        #[arg(short, long)]
        role: Option<String>,
    },

    /// Initialize a .promptctl.toml configuration file in the current directory
    Init {
        /// Force overwrite existing config
        #[arg(short, long)]
        force: bool,
    },

    /// List all available roles
    Roles,

    /// Scan the current project and show detected technologies
    Scan {
        /// Path to scan (defaults to current directory)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Generate a context-aware prompt for the current project
    Generate {
        /// Role to use (defaults to developer)
        #[arg(short, long, default_value = "developer")]
        role: String,

        /// Language to use (auto-detected if not specified)
        #[arg(short, long)]
        language: Option<String>,

        /// Copy to clipboard instead of printing
        #[arg(short, long)]
        copy: bool,

        /// Path to scan (defaults to current directory)
        #[arg(short, long)]
        path: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "markdown")]
        format: OutputFormat,

        /// Prompt size: minimal (~500 tokens), compact (~1500), full (~3000)
        #[arg(long, default_value = "compact")]
        size: String,

        /// Sections to include (comma-separated: error-handling,testing,async)
        #[arg(long, value_delimiter = ',')]
        sections: Option<Vec<String>>,

        /// Enable smart filtering based on project analysis
        #[arg(long)]
        smart: bool,

        /// Use a saved preset
        #[arg(long)]
        preset: Option<String>,
    },

    /// Manage saved presets
    Preset {
        #[command(subcommand)]
        action: PresetAction,
    },

    /// List available sections
    Sections,
}

#[derive(Subcommand, Debug)]
pub enum PresetAction {
    /// List all presets
    List,

    /// Show a preset's configuration
    Show { name: String },

    /// Save current options as a preset
    Save {
        /// Preset name
        name: String,

        /// Description
        #[arg(short, long)]
        description: Option<String>,

        /// Role
        #[arg(short, long)]
        role: Option<String>,

        /// Size (minimal, compact, full)
        #[arg(long)]
        size: Option<String>,

        /// Sections (comma-separated)
        #[arg(long, value_delimiter = ',')]
        sections: Option<Vec<String>>,

        /// Enable smart filtering
        #[arg(long)]
        smart: bool,

        /// Force overwrite existing
        #[arg(short, long)]
        force: bool,
    },

    /// Delete a preset
    Delete { name: String },
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum OutputFormat {
    #[default]
    Markdown,
    Plain,
}
