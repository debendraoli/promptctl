//! promptctl - A CLI tool for managing AI coding agent instructions across projects.

mod agents;
mod cli;
mod clipboard;
mod config;
mod indexer;
mod prompt_builder;
mod prompts;
mod roles;

use agents::Agent;
use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;
use config::{Config, PromptMode};
use indexer::ProjectIndex;
use prompt_builder::{PromptBuilder, PromptSize};
use roles::Role;
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
    match cli.command {
        Commands::Init {
            agent,
            role,
            path,
            force,
            dry_run,
            global,
        } => cmd_init(&agent, &role, path.as_deref(), force, dry_run, global),
        Commands::Show { language, role } => cmd_show(&language, role.as_deref()),
        Commands::List => cmd_list(),
        Commands::Clean { agent, path } => cmd_clean(&agent, path.as_deref()),
    }
}

// ── init: scan + emit + hooks ────────────────────────────────────────────────

fn cmd_init(
    agent_name: &str,
    role_name: &str,
    path: Option<&str>,
    force: bool,
    dry_run: bool,
    global: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let agent = Agent::from_str(agent_name).ok_or_else(|| {
        format!(
            "unknown agent: '{agent_name}'. Supported: {}",
            Agent::all()
                .iter()
                .map(Agent::name)
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    if agent == Agent::Raw {
        return Err("cannot init for 'raw' agent — pick a real agent.".into());
    }

    let role = Role::from_str(role_name).ok_or_else(|| {
        format!(
            "unknown role: '{role_name}'. Available: {}",
            Role::all()
                .iter()
                .map(Role::name)
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    let scan_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    // Step 1: Scan
    let index = ProjectIndex::scan(&scan_path);
    let config = Config::load()?;

    let languages: Vec<String> = index.languages.keys().map(|l| l.to_lowercase()).collect();

    if !dry_run {
        println!(
            "{} Scanning {}...",
            "→".blue().bold(),
            scan_path.display()
        );
        if !languages.is_empty() {
            let lang_display: Vec<String> = {
                let mut langs: Vec<_> = index.languages.values().collect();
                langs.sort_by(|a, b| b.file_count.cmp(&a.file_count));
                langs
                    .iter()
                    .map(|l| {
                        if let Some(ref v) = l.version {
                            format!("{} {v}", l.name)
                        } else {
                            l.name.clone()
                        }
                    })
                    .collect()
            };
            println!(
                "  {} Detected: {}",
                "✓".green(),
                lang_display.join(", ").cyan()
            );
        }
        if !index.frameworks.is_empty() {
            let fw: Vec<_> = index.frameworks.iter().map(|f| f.name.as_str()).collect();
            println!("  {} Frameworks: {}", "✓".green(), fw.join(", ").dimmed());
        }
        println!();
    }

    // Determine primary language for the main instruction file
    let primary_lang = index
        .primary_language()
        .map(|l| l.name.clone())
        .unwrap_or_default();

    // Step 2: Build prompt content for the main agent instruction file
    let content = build_agent_prompt(&primary_lang, &role, &index, config.as_ref())?;
    let formatted = agent.format_prompt(&content, &primary_lang);

    if dry_run {
        let instr_path = agent
            .resolve_path(&scan_path, global)
            .unwrap_or_default();
        println!(
            "{} Dry run — would write to:",
            "→".blue().bold()
        );
        println!("  {} {}", "•".green(), instr_path.display());

        // Show hook files that would be created
        if agents::supports_hooks(agent) {
            let hook_files = preview_hook_files(agent, &languages, &scan_path);
            for f in &hook_files {
                println!("  {} {}", "•".green(), f);
            }
        }

        println!();
        println!("{formatted}");
        return Ok(());
    }

    // Write the main instruction file
    let instr_path = agent.emit(&formatted, &scan_path, global, force)?;
    let token_estimate = formatted.len() / 4;
    println!(
        "{} Wrote {} instructions to {}",
        "✓".green().bold(),
        agent.display_name().cyan(),
        instr_path.display()
    );
    println!("{}", format!("  ~{token_estimate} tokens").dimmed());

    // Step 3: Install hooks (if agent supports them)
    if agents::supports_hooks(agent) {
        // Pre-build skillsets for each detected language
        let mut skillsets = std::collections::HashMap::new();
        for lang in &languages {
            if let Ok(skillset) = build_skillset(lang, config.as_ref()) {
                skillsets.insert(lang.clone(), skillset);
            }
        }

        let files = agents::install_agent_hooks(
            &scan_path,
            agent,
            &languages,
            role_name,
            &skillsets,
            force,
        )?;

        if !files.is_empty() {
            println!();
            println!(
                "{} Installed {} hooks:",
                "✓".green().bold(),
                agent.display_name().cyan()
            );
            for f in &files {
                println!(
                    "  {} {} {}",
                    "•".green(),
                    f.path.display(),
                    format!("— {}", f.description).dimmed()
                );
            }
        }
    }

    // Init .promptctl.toml if it doesn't exist
    let toml_path = scan_path.join(".promptctl.toml");
    if !toml_path.exists() && !global {
        Config::init(&scan_path, false).ok();
        println!();
        println!(
            "{} Created {} for custom prompts",
            "✓".green().bold(),
            ".promptctl.toml".dimmed()
        );
    }

    println!();
    println!(
        "{}",
        format!(
            "Done! {} will now use promptctl guidelines.",
            agent.display_name()
        )
        .green()
        .bold()
    );

    Ok(())
}

/// Build the full prompt content for an agent instruction file.
///
/// The base instruction file contains ONLY:
///   1. Role prefix (persona)
///   2. Project context (detected languages, frameworks, structure)
///   3. Hallucination guardrails (generic, not language-specific)
///
/// Language-specific skillsets are delivered via agent hooks (Cursor .mdc rules,
/// Copilot .instructions.md, Claude session hooks) — NOT baked into the base file.
fn build_agent_prompt(
    _language: &str,
    role: &Role,
    index: &ProjectIndex,
    _config: Option<&Config>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut content = String::new();

    // 1. Role prefix
    content.push_str(role.prompt_prefix());

    // 2. Project context
    let context = index.to_context_string();
    if !context.is_empty() {
        content.push_str("## Project Context\n\n");
        content.push_str(&context);
        content.push_str("\n\n");
    }

    // 3. Note about skillsets (so the AI knows they exist)
    let detected: Vec<String> = index.languages.keys().map(|l| l.to_lowercase()).collect();
    if !detected.is_empty() {
        content.push_str("## Language Skillsets\n\n");
        content.push_str(
            "This project uses language-specific coding guidelines loaded as separate skillsets.\n",
        );
        content.push_str(
            "Refer to each language's skillset for idiomatic patterns, error handling, \
             type system usage, testing, and tooling conventions.\n\n",
        );
        content.push_str("Detected languages: ");
        content.push_str(&detected.join(", "));
        content.push_str("\n\n");
    }

    // 4. Generic hallucination guardrails (no language-specific ones — those live in skillsets)
    content.push_str(GENERIC_GUARDRAILS);

    Ok(content)
}

/// Generic guardrails appended to every base instruction file.
/// Language-specific guardrails are part of each language's skillset.
const GENERIC_GUARDRAILS: &str = r#"## Hallucination Prevention

- **Never invent APIs, functions, or types** that do not exist in the language or library version.
- **Never fabricate package or module names**. Only reference dependencies that are documented and published.
- **If you are unsure whether a feature exists**, say so explicitly rather than guessing.
- **Pin to the language version** declared in project config (Cargo.toml, go.mod, package.json, etc.).
- **Do not hallucinate CLI flags, compiler options, or toolchain features**.
- **Verify struct fields, enum variants, and trait/interface methods** before referencing them.
- **When suggesting dependencies**, only suggest packages you are confident exist and are maintained.
- **Prefer standard library solutions** over third-party when the stdlib provides equivalent functionality.
- **Quote error messages exactly** when referencing compiler or runtime errors.
"#;

// ── show ─────────────────────────────────────────────────────────────────────

fn cmd_show(language: &str, role: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;

    // Build the skillset: structured prompt + custom merge + language guardrails
    let prompt = build_skillset(language, config.as_ref())?;

    if let Some(role_name) = role {
        let role = Role::from_str(role_name).ok_or_else(|| {
            format!(
                "unknown role: '{role_name}'. Available: {}",
                Role::all()
                    .iter()
                    .map(Role::name)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
        println!("{}", role.prompt_prefix());
    }

    println!("{prompt}");
    Ok(())
}

// ── list ─────────────────────────────────────────────────────────────────────

fn cmd_list() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;

    // Languages
    println!("{}", "Languages:".bold());
    for lang in prompts::available_languages() {
        println!("  {} {lang}", "•".green());
    }
    if let Some(ref cfg) = config {
        for lang in cfg.custom_languages() {
            if let Some(prompt) = cfg.get_prompt(lang) {
                let desc = prompt
                    .description
                    .as_deref()
                    .map(|d| format!(" - {d}"))
                    .unwrap_or_default();
                println!("  {} {}{}", "•".yellow(), lang, desc.dimmed());
            }
        }
    }

    // Agents
    println!();
    println!("{}", "Agents:".bold());
    for agent in Agent::all() {
        println!(
            "  {} {} {}",
            "•".green(),
            agent.name().cyan(),
            format!("→ {}", agent.instruction_file()).dimmed()
        );
    }

    // Roles
    println!();
    println!("{}", "Roles:".bold());
    for role in Role::all() {
        println!(
            "  {} {} {}",
            "•".green(),
            role.name().cyan(),
            format!("- {}", role.description()).dimmed()
        );
    }

    println!();
    println!(
        "{}",
        "Use 'promptctl init <agent>' to set up your project.".dimmed()
    );

    Ok(())
}

// ── clean ────────────────────────────────────────────────────────────────────

fn cmd_clean(agent_name: &str, path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let agent = Agent::from_str(agent_name)
        .ok_or_else(|| format!("unknown agent: '{agent_name}'"))?;

    let scan_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let mut removed = Vec::new();

    // Remove main instruction file
    if let Some(instr_path) = agent.resolve_path(&scan_path, false) {
        if instr_path.exists() {
            std::fs::remove_file(&instr_path)?;
            removed.push(instr_path);
        }
    }

    // Remove hooks
    if agents::supports_hooks(agent) {
        let hook_removed = agents::remove_agent_hooks(&scan_path, agent)?;
        removed.extend(hook_removed);
    }

    if removed.is_empty() {
        println!(
            "{}",
            format!("No promptctl files found for {}.", agent.display_name()).dimmed()
        );
    } else {
        println!(
            "{} Removed {} file{}:",
            "✓".green().bold(),
            removed.len(),
            if removed.len() == 1 { "" } else { "s" }
        );
        for p in &removed {
            println!("  {} {}", "✗".red(), p.display());
        }
    }

    Ok(())
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Preview hook file paths for dry-run output.
fn preview_hook_files(agent: Agent, languages: &[String], project_root: &std::path::Path) -> Vec<String> {
    let mut files = Vec::new();
    match agent {
        Agent::Claude => {
            files.push(format!(
                "{}",
                project_root.join(".claude/hooks/promptctl-session-start.sh").display()
            ));
            files.push(format!(
                "{}",
                project_root.join(".claude/hooks/promptctl-pre-write.sh").display()
            ));
            files.push(format!(
                "{}",
                project_root.join(".claude/settings.json").display()
            ));
        }
        Agent::Cursor => {
            for lang in languages {
                files.push(format!(
                    "{}",
                    project_root
                        .join(format!(".cursor/rules/promptctl-{lang}.mdc"))
                        .display()
                ));
            }
        }
        Agent::Copilot => {
            for lang in languages {
                files.push(format!(
                    "{}",
                    project_root
                        .join(format!(".github/instructions/promptctl-{lang}.instructions.md"))
                        .display()
                ));
            }
        }
        _ => {}
    }
    files
}

/// Apply custom prepend/append merge from config around a built-in prompt.
fn apply_custom_merge(language: &str, config: Option<&Config>, builtin: &str) -> String {
    let lang_lower = language.to_lowercase();
    let Some(cfg) = config else {
        return builtin.to_string();
    };
    let Some(custom) = cfg.get_prompt(&lang_lower) else {
        return builtin.to_string();
    };

    match custom.mode {
        PromptMode::Replace => custom.content.clone(),
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

/// Build a complete language skillset: structured prompt + custom merge + guardrails.
///
/// This is the self-contained output for `promptctl show <lang>` and agent hooks.
/// It includes everything an AI needs for a specific language — no overlap with the
/// base instruction file.
fn build_skillset(
    language: &str,
    config: Option<&Config>,
) -> Result<String, Box<dyn std::error::Error>> {
    let lang_lower = language.to_lowercase();

    // Try structured prompt first (smart + full for maximum skillset coverage)
    let base = if let Some(structured) = prompts::get_structured_prompt(&lang_lower) {
        let scan_path = std::env::current_dir().unwrap_or_default();
        let index = ProjectIndex::scan(&scan_path);
        let builder = PromptBuilder::new().size(PromptSize::Full).smart(true);

        format!(
            "# {} Development Guidelines\n\n{}",
            language.to_uppercase(),
            builder.build(&structured, Some(&index))
        )
    } else {
        // Fall back to raw builtin prompt
        let builtin = prompts::get_builtin_prompt(&lang_lower);

        if let Some(cfg) = config {
            if let Some(resolved) = cfg.resolve_prompt(&lang_lower, builtin) {
                return Ok(resolved);
            }
        }

        builtin
            .map(|s| s.to_string())
            .ok_or_else(|| {
                format!(
                    "unknown language: '{language}'. Use 'promptctl list' to see available prompts."
                )
            })?
    };

    // Apply custom merge if configured
    let merged = apply_custom_merge(&lang_lower, config, &base);

    // Append language-specific guardrails
    let guardrails = agents::hallucination_guardrails(&lang_lower);
    Ok(format!("{merged}\n\n{guardrails}"))
}
