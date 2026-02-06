//! promptctl - A personal CLI tool for managing coding prompts across projects.

mod agents;
mod cli;
mod clipboard;
mod config;
mod indexer;
mod presets;
mod prompt_builder;
mod prompts;
mod roles;

use agents::Agent;
use clap::Parser;
use cli::{Cli, Commands, HooksAction, OutputFormat, PresetAction};
use colored::Colorize;
use config::{Config, PromptMode};
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
            agent,
            no_guardrails,
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
            agent,
            no_guardrails,
        }),
        Commands::Emit {
            agent,
            role,
            language,
            path,
            size,
            sections,
            smart,
            preset,
            global,
            force,
            dry_run,
            quiet,
            no_guardrails,
        } => cmd_emit(EmitOptions {
            agent,
            role,
            language,
            path,
            size,
            sections,
            smart,
            preset,
            global,
            force,
            dry_run,
            quiet,
            no_guardrails,
        }),
        Commands::Hooks { action } => cmd_hooks(action),
        Commands::Agents => cmd_agents(),
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
    agent: Option<String>,
    no_guardrails: bool,
}

/// Options for the emit command
struct EmitOptions {
    agent: String,
    role: String,
    language: Option<String>,
    path: Option<String>,
    size: String,
    sections: Option<Vec<String>>,
    smart: bool,
    preset: Option<String>,
    global: bool,
    force: bool,
    dry_run: bool,
    quiet: bool,
    no_guardrails: bool,
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

fn cmd_scan(path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let scan_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    println!("{} Scanning {}...", "→".blue().bold(), scan_path.display());
    println!();

    let index = ProjectIndex::scan(&scan_path);

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

    if !index.frameworks.is_empty() {
        println!("{}", "Frameworks/Libraries:".bold());
        for fw in &index.frameworks {
            println!("  {} {} ({:?})", "•".yellow(), fw.name, fw.category);
        }
        println!();
    }

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

fn cmd_generate(opts: GenerateOptions) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;

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

    // Resolve agent: CLI flag > preset > config default > None
    let agent = opts
        .agent
        .as_deref()
        .or(preset.agent.as_deref())
        .or(config.as_ref().and_then(|c| c.default_agent.as_deref()))
        .and_then(Agent::from_str);

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

    let lang = if let Some(ref l) = opts.language {
        l.clone()
    } else if let Some(ref l) = preset.language {
        l.clone()
    } else if let Some(primary) = index.primary_language() {
        primary.name.clone()
    } else {
        return Err("Could not detect project language. Use --language to specify.".into());
    };

    let size_str = if opts.preset.is_some() && opts.size == "compact" {
        preset.size
    } else {
        PromptSize::from_str(&opts.size).unwrap_or(PromptSize::Compact)
    };

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
    let lang_prompt = if let Some(structured) = prompts::get_structured_prompt(&lang) {
        let builder = PromptBuilder::new()
            .size(size_str)
            .smart(smart);

        let builder = if let Some(sec) = sections {
            builder.sections(sec)
        } else {
            builder
        };

        let built = format!("# {} Development Guidelines\n\n{}",
            lang.to_uppercase(),
            builder.build(&structured, Some(&index)));

        // Apply custom merge (prepend/append) if configured
        apply_custom_merge(&lang, config.as_ref(), &built)
    } else {
        get_prompt(&lang, config.as_ref())?
    };

    full_prompt.push_str(&lang_prompt);

    // Add hallucination prevention guardrails
    if !opts.no_guardrails {
        full_prompt.push_str("\n\n");
        full_prompt.push_str(&agents::hallucination_guardrails(&lang));
    }

    // Apply agent formatting
    let full_prompt = if let Some(ref ag) = agent {
        ag.format_prompt(&full_prompt, &lang)
    } else {
        full_prompt
    };

    // Estimate tokens
    let token_estimate = full_prompt.len() / 4;

    // Strip markdown if plain format
    let output = match opts.format {
        OutputFormat::Markdown => full_prompt.clone(),
        OutputFormat::Plain => strip_markdown(&full_prompt),
    };

    if opts.copy {
        clipboard::copy_to_clipboard(&output)?;
        let agent_info = agent
            .as_ref()
            .map(|a| format!(" ({})", a.display_name()))
            .unwrap_or_default();
        println!(
            "{} Copied {} prompt with {} role to clipboard!{}",
            "✓".green().bold(),
            lang.cyan(),
            role.name().yellow(),
            agent_info.dimmed()
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

/// Emit instructions to an agent's convention file
fn cmd_emit(opts: EmitOptions) -> Result<(), Box<dyn std::error::Error>> {
    let agent = Agent::from_str(&opts.agent).ok_or_else(|| {
        format!(
            "unknown agent: '{}'. Use 'promptctl agents' to see supported agents.",
            opts.agent
        )
    })?;

    if agent == Agent::Raw {
        return Err("cannot emit to 'raw' agent — use 'generate' instead.".into());
    }

    // Generate the prompt content using the same logic as cmd_generate
    let config = Config::load()?;

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

    let role_name = if opts.preset.is_some() && opts.role == "developer" {
        preset.role.as_deref().unwrap_or(&opts.role).to_string()
    } else {
        opts.role.clone()
    };

    let role = Role::from_str(&role_name).ok_or_else(|| {
        format!("unknown role: '{role_name}'. Use 'promptctl roles' to see available roles.")
    })?;

    let scan_path = opts
        .path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let index = ProjectIndex::scan(&scan_path);

    let lang = if let Some(ref l) = opts.language {
        l.clone()
    } else if let Some(ref l) = preset.language {
        l.clone()
    } else if let Some(primary) = index.primary_language() {
        primary.name.clone()
    } else {
        return Err("Could not detect project language. Use --language to specify.".into());
    };

    let size_str = if opts.preset.is_some() && opts.size == "compact" {
        preset.size
    } else {
        PromptSize::from_str(&opts.size).unwrap_or(PromptSize::Compact)
    };

    let sections: Option<HashSet<Section>> = if let Some(ref sec_list) = opts.sections {
        Some(sec_list.iter().filter_map(|s| Section::from_str(s)).collect())
    } else {
        preset.parsed_sections()
    };

    let smart = opts.smart || preset.smart;

    // Build prompt content
    let mut content = String::new();
    content.push_str(role.prompt_prefix());

    let context = index.to_context_string();
    if !context.is_empty() {
        content.push_str("## Project Context\n\n");
        content.push_str(&context);
        content.push_str("\n\n");
    }

    let lang_prompt = if let Some(structured) = prompts::get_structured_prompt(&lang) {
        let builder = PromptBuilder::new().size(size_str).smart(smart);
        let builder = if let Some(sec) = sections {
            builder.sections(sec)
        } else {
            builder
        };
        let built = format!(
            "# {} Development Guidelines\n\n{}",
            lang.to_uppercase(),
            builder.build(&structured, Some(&index))
        );

        // Apply custom merge (prepend/append) if configured
        apply_custom_merge(&lang, config.as_ref(), &built)
    } else {
        get_prompt(&lang, config.as_ref())?
    };
    content.push_str(&lang_prompt);

    // Add hallucination prevention guardrails
    if !opts.no_guardrails {
        content.push_str("\n\n");
        content.push_str(&agents::hallucination_guardrails(&lang));
    }

    // Apply agent-specific formatting
    let formatted = agent.format_prompt(&content, &lang);

    if opts.dry_run {
        if !opts.quiet {
            let path = agent
                .resolve_path(&scan_path, opts.global)
                .unwrap_or_default();
            println!(
                "{} Dry run — would write to: {}",
                "→".blue().bold(),
                path.display()
            );
            println!();
        }
        println!("{formatted}");
        return Ok(());
    }

    let path = agent.emit(&formatted, &scan_path, opts.global, opts.force)?;

    if !opts.quiet {
        let token_estimate = formatted.len() / 4;
        println!(
            "{} Wrote {} instructions to {}",
            "✓".green().bold(),
            agent.display_name().cyan(),
            path.display()
        );
        println!("{}", format!("~{} tokens", token_estimate).dimmed());
    }

    Ok(())
}

/// Install, remove, or list agent-native hooks
fn cmd_hooks(action: HooksAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        HooksAction::Install {
            agent,
            path,
            force,
            dry_run,
        } => {
            let ag = Agent::from_str(&agent).ok_or_else(|| {
                format!(
                    "unknown agent: '{}'. Use 'promptctl agents' to see supported agents.",
                    agent
                )
            })?;

            if !agents::supports_hooks(ag) {
                return Err(format!(
                    "agent '{}' has no native hook system. Use 'promptctl emit {}' instead.",
                    agent, agent
                )
                .into());
            }

            let scan_path = path
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

            // Detect languages for cursor/copilot per-lang hooks
            let index = ProjectIndex::scan(&scan_path);
            let languages: Vec<String> = index
                .languages
                .keys()
                .map(|l| l.to_lowercase())
                .collect();

            if dry_run {
                println!(
                    "{} Dry run — would install {} hooks in {}",
                    "→".blue().bold(),
                    ag.display_name().cyan(),
                    scan_path.display()
                );
                println!();
                match ag {
                    Agent::Claude => {
                        println!("  {} .claude/hooks/promptctl-session-start.sh", "•".green());
                        println!("  {} .claude/hooks/promptctl-pre-write.sh", "•".green());
                        println!("  {} .claude/settings.json (merged)", "•".green());
                    }
                    Agent::Cursor => {
                        for lang in &languages {
                            if agents::supports_hooks(Agent::Cursor) {
                                println!(
                                    "  {} .cursor/rules/promptctl-{lang}.mdc",
                                    "•".green()
                                );
                            }
                        }
                    }
                    Agent::Copilot => {
                        for lang in &languages {
                            println!(
                                "  {} .github/instructions/promptctl-{lang}.instructions.md",
                                "•".green()
                            );
                        }
                    }
                    _ => {}
                }
                return Ok(());
            }

            let files = agents::install_agent_hooks(&scan_path, ag, &languages, force)?;

            println!(
                "{} Installed {} agent hooks:",
                "✓".green().bold(),
                ag.display_name().cyan()
            );
            for f in &files {
                println!(
                    "  {} {} {}",
                    "•".green(),
                    f.path.display(),
                    format!("— {}", f.description).dimmed()
                );
            }

            println!();
            match ag {
                Agent::Claude => {
                    println!(
                        "{}",
                        "Claude Code will load promptctl guidelines on session start".dimmed()
                    );
                    println!(
                        "{}",
                        "and validate language context before file writes.".dimmed()
                    );
                }
                Agent::Cursor => {
                    println!(
                        "{}",
                        "Cursor will apply per-language rules when editing matching files."
                            .dimmed()
                    );
                }
                Agent::Copilot => {
                    println!(
                        "{}",
                        "Copilot will apply path-specific instructions when editing matching files."
                            .dimmed()
                    );
                }
                _ => {}
            }
        }

        HooksAction::Remove { agent, path } => {
            let ag = Agent::from_str(&agent).ok_or_else(|| {
                format!("unknown agent: '{}'", agent)
            })?;

            let scan_path = path
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

            let removed = agents::remove_agent_hooks(&scan_path, ag)?;

            if removed.is_empty() {
                println!("  {}", "No promptctl hooks found for this agent.".dimmed());
            } else {
                println!(
                    "{} Removed {} hook files:",
                    "✓".green().bold(),
                    ag.display_name().cyan()
                );
                for p in &removed {
                    println!("  {} {}", "✗".red(), p.display());
                }
            }
        }

        HooksAction::List { path } => {
            let scan_path = path
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

            let hooks = agents::list_agent_hooks(&scan_path);

            if hooks.is_empty() {
                println!("{}", "No agent hooks installed.".dimmed());
                println!();
                println!(
                    "{}",
                    "Use 'promptctl hooks install <agent>' to install hooks.".dimmed()
                );
                println!(
                    "{}",
                    "Supported: claude, cursor, copilot".dimmed()
                );
            } else {
                println!("{}", "Installed agent hooks:".bold());
                println!();
                for (agent, files) in &hooks {
                    println!(
                        "  {} {} ({} file{})",
                        "✓".green(),
                        agent.display_name().cyan().bold(),
                        files.len(),
                        if files.len() == 1 { "" } else { "s" }
                    );
                    for f in files {
                        println!("    {}", f.display().to_string().dimmed());
                    }
                }
            }
        }
    }

    Ok(())
}

/// List supported agents
fn cmd_agents() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Supported AI agents:".bold());
    println!();

    for agent in Agent::all() {
        println!(
            "  {} {} {}",
            "•".green(),
            agent.name().cyan().bold(),
            format!("- {}", agent.description()).dimmed()
        );
    }

    println!();
    println!("{}", "Usage:".bold());
    println!(
        "  {}",
        "promptctl generate --agent copilot    # Format for Copilot".dimmed()
    );
    println!(
        "  {}",
        "promptctl emit claude                  # Write CLAUDE.md".dimmed()
    );
    println!(
        "  {}",
        "promptctl emit copilot --global        # Write ~/.github/copilot-instructions.md"
            .dimmed()
    );
    println!(
        "  {}",
        "promptctl hooks install claude         # Install Claude session/write hooks".dimmed()
    );

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

/// Apply custom prepend/append merge from config around a built-in prompt.
/// If no custom config exists or mode is Replace, returns the built-in unchanged.
fn apply_custom_merge(language: &str, config: Option<&Config>, builtin: &str) -> String {
    let lang_lower = language.to_lowercase();
    let Some(cfg) = config else {
        return builtin.to_string();
    };
    let Some(custom) = cfg.get_prompt(&lang_lower) else {
        return builtin.to_string();
    };

    match custom.mode {
        PromptMode::Replace => {
            // In structured prompt path, Replace means use custom entirely
            custom.content.clone()
        }
        PromptMode::Prepend => {
            let pre = custom.prepend.as_deref().unwrap_or(&custom.content);
            format!("{pre}\n\n{builtin}")
        }
        PromptMode::Append => {
            let post = custom.append.as_deref().unwrap_or(&custom.content);
            format!("{builtin}\n\n{post}")
        }
        PromptMode::Merge => {
            let pre = custom.prepend.as_deref().unwrap_or("");
            let post = custom.append.as_deref().unwrap_or("");
            let mut result = String::new();
            if !pre.is_empty() {
                result.push_str(pre);
                result.push_str("\n\n");
            }
            result.push_str(builtin);
            if !post.is_empty() {
                result.push_str("\n\n");
                result.push_str(post);
            }
            result
        }
    }
}

/// Get a prompt by language, checking custom config first, then built-ins.
/// Supports merge modes: replace (default), prepend, append, merge.
fn get_prompt(
    language: &str,
    config: Option<&Config>,
) -> Result<String, Box<dyn std::error::Error>> {
    let lang_lower = language.to_lowercase();
    let builtin = prompts::get_builtin_prompt(&lang_lower);

    // If we have config, try to resolve with merge logic
    if let Some(cfg) = config {
        if let Some(resolved) = cfg.resolve_prompt(&lang_lower, builtin) {
            return Ok(resolved);
        }
    }

    // Fall back to built-in prompts
    builtin
        .map(|s| s.to_string())
        .ok_or_else(|| {
            format!(
                "unknown language: '{language}'. Use 'promptctl list' to see available prompts."
            )
            .into()
        })
}
