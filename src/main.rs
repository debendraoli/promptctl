//! promptctl - A personal CLI tool for managing coding prompts across projects.

mod cli;
mod clipboard;
mod config;
mod indexer;
mod presets;
mod prompt_builder;
mod prompts;
mod roles;

use clap::Parser;
use cli::{Cli, Commands, OutputFormat, PresetAction};
use colored::Colorize;
use config::Config;
use indexer::ProjectIndex;
use presets::{builtin_presets, Preset, Presets};
use prompt_builder::{PromptBuilder, PromptSize, Section};
use roles::Role;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{} {e}", "error:".red().bold());
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;

    match cli.command {
        Commands::List => cmd_list(config.as_ref()),
        Commands::Show { language, role } => cmd_show(&language, role.as_deref(), config.as_ref()),
        Commands::Copy { language, role } => cmd_copy(&language, role.as_deref(), config.as_ref()),
        Commands::Init { force } => cmd_init(force),
        Commands::Roles => cmd_roles(),
        Commands::Scan { path } => cmd_scan(path),
        Commands::Generate {
            role,
            language,
            copy,
            path,
            format,
            size,
            sections,
            smart,
            preset,
        } => cmd_generate(GenerateOptions {
            role,
            language,
            copy,
            path,
            format,
            size,
            sections,
            smart,
            preset,
        }),
        Commands::Preset { action } => cmd_preset(action),
        Commands::Sections => cmd_sections(),
    }
}

/// Options for the generate command
struct GenerateOptions {
    role: String,
    language: Option<String>,
    copy: bool,
    path: Option<String>,
    format: OutputFormat,
    size: String,
    sections: Option<Vec<String>>,
    smart: bool,
    preset: Option<String>,
}

/// List all available prompts
fn cmd_list(config: Option<&Config>) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Available prompts:".bold());
    println!();

    // Built-in prompts
    println!("  {}", "Built-in:".cyan().bold());
    for lang in prompts::available_languages() {
        println!("    {} {lang}", "•".green());
    }

    // Custom prompts from config
    if let Some(cfg) = config {
        let custom = cfg.custom_languages();
        if !custom.is_empty() {
            println!();
            println!("  {}", "Custom:".cyan().bold());
            for lang in custom {
                if let Some(prompt) = cfg.get_prompt(lang) {
                    let desc = prompt
                        .description
                        .as_deref()
                        .map(|d| format!(" - {d}"))
                        .unwrap_or_default();
                    println!("    {} {}{}", "•".yellow(), lang, desc.dimmed());
                }
            }
        }
    }

    println!();
    println!(
        "{}",
        "Use 'promptctl show <language>' to view a prompt.".dimmed()
    );
    println!(
        "{}",
        "Use 'promptctl roles' to see available roles.".dimmed()
    );

    Ok(())
}

/// Show a specific prompt
fn cmd_show(
    language: &str,
    role: Option<&str>,
    config: Option<&Config>,
) -> Result<(), Box<dyn std::error::Error>> {
    let prompt = get_prompt(language, config)?;

    if let Some(role_name) = role {
        let role = Role::from_str(role_name).ok_or_else(|| {
            format!("unknown role: '{role_name}'. Use 'promptctl roles' to see available roles.")
        })?;
        println!("{}", role.prompt_prefix());
    }

    println!("{prompt}");
    Ok(())
}

/// Copy a prompt to clipboard
fn cmd_copy(
    language: &str,
    role: Option<&str>,
    config: Option<&Config>,
) -> Result<(), Box<dyn std::error::Error>> {
    let prompt = get_prompt(language, config)?;

    let full_prompt = if let Some(role_name) = role {
        let role = Role::from_str(role_name).ok_or_else(|| {
            format!("unknown role: '{role_name}'. Use 'promptctl roles' to see available roles.")
        })?;
        format!("{}\n{}", role.prompt_prefix(), prompt)
    } else {
        prompt.to_string()
    };

    clipboard::copy_to_clipboard(&full_prompt)?;
    println!(
        "{} Copied {} prompt to clipboard!",
        "✓".green().bold(),
        language.cyan()
    );
    Ok(())
}

/// Initialize a config file
fn cmd_init(force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let path = Config::init(&cwd, force)?;
    println!(
        "{} Created config file: {}",
        "✓".green().bold(),
        path.display()
    );
    println!();
    println!(
        "{}",
        "Edit this file to add custom prompts for your project.".dimmed()
    );
    Ok(())
}

/// List all available roles
fn cmd_roles() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Available roles:".bold());
    println!();

    for role in Role::all() {
        println!(
            "  {} {}",
            role.name().cyan().bold(),
            format!("- {}", role.description()).dimmed()
        );
    }

    println!();
    println!(
        "{}",
        "Use 'promptctl show <language> --role <role>' to apply a role.".dimmed()
    );
    println!(
        "{}",
        "Use 'promptctl generate --role <role>' for project-aware prompts.".dimmed()
    );

    Ok(())
}

/// Scan the current project
fn cmd_scan(path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let scan_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    println!("{} Scanning {}...", "→".blue().bold(), scan_path.display());
    println!();

    let index = ProjectIndex::scan(&scan_path);

    // Languages
    if !index.languages.is_empty() {
        println!("{}", "Languages detected:".bold());
        let mut langs: Vec<_> = index.languages.values().collect();
        langs.sort_by(|a, b| b.file_count.cmp(&a.file_count));

        for lang in langs {
            let version = lang
                .version
                .as_ref()
                .map(|v| format!(" ({})", v))
                .unwrap_or_default();
            println!(
                "  {} {}{} - {} files",
                "•".green(),
                lang.name.cyan(),
                version.dimmed(),
                lang.file_count
            );
        }
        println!();
    }

    // Frameworks
    if !index.frameworks.is_empty() {
        println!("{}", "Frameworks/Libraries:".bold());
        for fw in &index.frameworks {
            println!("  {} {} ({:?})", "•".yellow(), fw.name, fw.category);
        }
        println!();
    }

    // Structure
    println!("{}", "Project structure:".bold());
    let structure = &index.structure;
    if structure.has_src {
        println!("  {} Source directory", "✓".green());
    }
    if structure.has_tests {
        println!("  {} Test directory", "✓".green());
    }
    if structure.has_docs {
        println!("  {} Documentation", "✓".green());
    }
    if structure.has_ci {
        println!("  {} CI/CD configuration", "✓".green());
    }
    println!();

    // Suggest primary language
    if let Some(primary) = index.primary_language() {
        println!(
            "{} Primary language: {}",
            "→".blue().bold(),
            primary.name.cyan().bold()
        );
        println!(
            "{}",
            format!("Use 'promptctl generate' to create a tailored prompt.").dimmed()
        );
    }

    Ok(())
}

/// Generate a context-aware prompt
fn cmd_generate(opts: GenerateOptions) -> Result<(), Box<dyn std::error::Error>> {
    // Load preset if specified
    let preset = if let Some(ref preset_name) = opts.preset {
        let presets = Presets::load()?;
        let builtin = builtin_presets();

        presets
            .get(preset_name)
            .cloned()
            .or_else(|| builtin.get(preset_name).cloned())
            .ok_or_else(|| format!("preset '{preset_name}' not found"))?
    } else {
        Preset::default()
    };

    // Merge preset with CLI options (CLI takes precedence)
    let role_name = if opts.preset.is_some() && opts.role == "developer" {
        preset.role.as_deref().unwrap_or(&opts.role)
    } else {
        &opts.role
    };

    let role = Role::from_str(role_name).ok_or_else(|| {
        format!("unknown role: '{role_name}'. Use 'promptctl roles' to see available roles.")
    })?;

    let scan_path = opts
        .path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let index = ProjectIndex::scan(&scan_path);

    // Determine language
    let lang = if let Some(ref l) = opts.language {
        l.clone()
    } else if let Some(ref l) = preset.language {
        l.clone()
    } else if let Some(primary) = index.primary_language() {
        primary.name.clone()
    } else {
        return Err("Could not detect project language. Use --language to specify.".into());
    };

    // Determine size
    let size_str = if opts.preset.is_some() && opts.size == "compact" {
        preset.size
    } else {
        PromptSize::from_str(&opts.size).unwrap_or(PromptSize::Compact)
    };

    // Determine sections
    let sections: Option<HashSet<Section>> = if let Some(ref sec_list) = opts.sections {
        Some(sec_list.iter().filter_map(|s| Section::from_str(s)).collect())
    } else {
        preset.parsed_sections()
    };

    // Determine smart filtering
    let smart = opts.smart || preset.smart;

    // Build the prompt
    let mut full_prompt = String::new();

    // Add role prefix
    full_prompt.push_str(role.prompt_prefix());

    // Add project context
    let context = index.to_context_string();
    if !context.is_empty() {
        full_prompt.push_str("## Project Context\n\n");
        full_prompt.push_str(&context);
        full_prompt.push_str("\n\n");
    }

    // Get structured prompt if available, otherwise fall back to raw
    let config = Config::load()?;
    let lang_prompt = if let Some(structured) = prompts::get_structured_prompt(&lang) {
        let builder = PromptBuilder::new()
            .size(size_str)
            .smart(smart);

        let builder = if let Some(sec) = sections {
            builder.sections(sec)
        } else {
            builder
        };

        format!("# {} Development Guidelines\n\n{}",
            lang.to_uppercase(),
            builder.build(&structured, Some(&index)))
    } else {
        get_prompt(&lang, config.as_ref())?
    };

    full_prompt.push_str(&lang_prompt);

    // Estimate tokens
    let token_estimate = full_prompt.len() / 4;

    // Strip markdown if plain format
    let output = match opts.format {
        OutputFormat::Markdown => full_prompt.clone(),
        OutputFormat::Plain => strip_markdown(&full_prompt),
    };

    if opts.copy {
        clipboard::copy_to_clipboard(&output)?;
        println!(
            "{} Copied {} prompt with {} role to clipboard!",
            "✓".green().bold(),
            lang.cyan(),
            role.name().yellow()
        );
        println!();
        println!("{}", format!("~{} tokens", token_estimate).dimmed());
        println!("{}", "Project context included:".dimmed());
        for line in context.lines() {
            println!("  {}", line.dimmed());
        }
    } else {
        println!("{output}");
    }

    Ok(())
}

/// Handle preset commands
fn cmd_preset(action: PresetAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        PresetAction::List => {
            let presets = Presets::load()?;
            let builtin = builtin_presets();

            println!("{}", "Built-in presets:".bold());
            for (name, preset) in &builtin {
                let desc = preset.description.as_deref().unwrap_or("");
                println!("  {} {} {}", "•".green(), name.cyan(), desc.dimmed());
            }

            if !presets.presets.is_empty() {
                println!();
                println!("{}", "Custom presets:".bold());
                for (name, preset) in &presets.presets {
                    let desc = preset.description.as_deref().unwrap_or("");
                    println!("  {} {} {}", "•".yellow(), name.cyan(), desc.dimmed());
                }
            }

            println!();
            println!("{}", "Use 'promptctl generate --preset <name>' to use a preset.".dimmed());
        }

        PresetAction::Show { name } => {
            let presets = Presets::load()?;
            let builtin = builtin_presets();

            let preset = presets
                .get(&name)
                .or_else(|| builtin.get(&name))
                .ok_or_else(|| format!("preset '{name}' not found"))?;

            println!("{} {}", "Preset:".bold(), name.cyan());
            if let Some(ref desc) = preset.description {
                println!("{} {}", "Description:".bold(), desc);
            }
            if let Some(ref role) = preset.role {
                println!("{} {}", "Role:".bold(), role);
            }
            println!("{} {}", "Size:".bold(), preset.size.name());
            if !preset.sections.is_empty() {
                println!("{} {}", "Sections:".bold(), preset.sections.join(", "));
            }
            println!("{} {}", "Smart:".bold(), preset.smart);
        }

        PresetAction::Save {
            name,
            description,
            role,
            size,
            sections,
            smart,
            force,
        } => {
            let mut presets = Presets::load()?;

            let mut preset = Preset::new();
            if let Some(desc) = description {
                preset = preset.with_description(&desc);
            }
            if let Some(r) = role {
                preset = preset.with_role(&r);
            }
            if let Some(s) = size {
                preset = preset.with_size(PromptSize::from_str(&s).unwrap_or(PromptSize::Compact));
            }
            if let Some(sec) = sections {
                preset = preset.with_sections(sec);
            }
            preset = preset.with_smart(smart);

            presets.set(name.clone(), preset, force)?;
            let path = presets.save()?;

            println!(
                "{} Saved preset '{}' to {}",
                "✓".green().bold(),
                name.cyan(),
                path.display()
            );
        }

        PresetAction::Delete { name } => {
            let mut presets = Presets::load()?;
            presets.remove(&name)?;
            presets.save()?;
            println!("{} Deleted preset '{}'", "✓".green().bold(), name.cyan());
        }
    }

    Ok(())
}

/// List available sections
fn cmd_sections() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Available sections:".bold());
    println!();

    for section in Section::all() {
        println!("  {} {}", "•".green(), section.name().cyan());
    }

    println!();
    println!("{}", "Size tiers include:".bold());
    println!("  {} {}", "minimal".yellow(), "- version, style, error-handling".dimmed());
    println!("  {} {}", "compact".yellow(), "- minimal + types, testing, tooling".dimmed());
    println!("  {} {}", "full".yellow(), "- all sections".dimmed());

    println!();
    println!("{}", "Use 'promptctl generate --sections error-handling,testing' to select specific sections.".dimmed());

    Ok(())
}

/// Strip basic markdown formatting for plain text output
fn strip_markdown(text: &str) -> String {
    text.lines()
        .map(|line| {
            let trimmed = line.trim_start_matches('#').trim();
            if trimmed.is_empty() && line.starts_with('#') {
                String::new()
            } else {
                trimmed.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Get a prompt by language, checking custom config first, then built-ins
fn get_prompt(
    language: &str,
    config: Option<&Config>,
) -> Result<String, Box<dyn std::error::Error>> {
    let lang_lower = language.to_lowercase();

    // Check custom prompts first
    if let Some(cfg) = config {
        if let Some(custom) = cfg.get_prompt(&lang_lower) {
            return Ok(custom.content.clone());
        }
    }

    // Fall back to built-in prompts
    prompts::get_builtin_prompt(&lang_lower)
        .map(|s| s.to_string())
        .ok_or_else(|| {
            format!(
                "unknown language: '{language}'. Use 'promptctl list' to see available prompts."
            )
            .into()
        })
}
