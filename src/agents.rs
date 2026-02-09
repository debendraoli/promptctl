//! Agent-specific instruction formatting and file conventions.
//!
//! Each AI coding agent (Copilot, Claude, Cursor, Codex, Aider, etc.) has its own
//! instruction file format, path conventions, and recommended practices. This module
//! defines the agent abstraction and provides formatting/emission logic.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Agent {
    Copilot,
    Claude,
    Cursor,
    Codex,
    Aider,
    Raw,
}

impl Agent {
    /// All available agents (excluding Raw)
    pub const fn all() -> &'static [Agent] {
        &[
            Agent::Copilot,
            Agent::Claude,
            Agent::Cursor,
            Agent::Codex,
            Agent::Aider,
        ]
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Agent::Copilot => "copilot",
            Agent::Claude => "claude",
            Agent::Cursor => "cursor",
            Agent::Codex => "codex",
            Agent::Aider => "aider",
            Agent::Raw => "raw",
        }
    }

    pub const fn display_name(&self) -> &'static str {
        match self {
            Agent::Copilot => "GitHub Copilot",
            Agent::Claude => "Claude Code",
            Agent::Cursor => "Cursor",
            Agent::Codex => "OpenAI Codex",
            Agent::Aider => "Aider",
            Agent::Raw => "Raw",
        }
    }

    #[allow(dead_code)]
    pub const fn description(&self) -> &'static str {
        match self {
            Agent::Copilot => "GitHub Copilot — .github/copilot-instructions.md",
            Agent::Claude => "Claude Code — CLAUDE.md at project root",
            Agent::Cursor => "Cursor IDE — .cursor/rules/promptctl.mdc",
            Agent::Codex => "OpenAI Codex — AGENTS.md at project root",
            Agent::Aider => "Aider — CONVENTIONS.md at project root",
            Agent::Raw => "Raw output — no agent wrapper",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "copilot" | "github-copilot" | "gh-copilot" => Some(Agent::Copilot),
            "claude" | "claude-code" | "anthropic" => Some(Agent::Claude),
            "cursor" => Some(Agent::Cursor),
            "codex" | "openai-codex" | "openai" => Some(Agent::Codex),
            "aider" => Some(Agent::Aider),
            "raw" | "none" | "generic" => Some(Agent::Raw),
            _ => None,
        }
    }

    pub fn instruction_file(&self) -> &'static str {
        match self {
            Agent::Copilot => ".github/copilot-instructions.md",
            Agent::Claude => "CLAUDE.md",
            Agent::Cursor => ".cursor/rules/promptctl.mdc",
            Agent::Codex => "AGENTS.md",
            Agent::Aider => "CONVENTIONS.md",
            Agent::Raw => "",
        }
    }

    pub fn global_instruction_file(&self) -> Option<&'static str> {
        match self {
            Agent::Copilot => Some(".github/copilot-instructions.md"),
            Agent::Claude => Some(".claude/CLAUDE.md"),
            Agent::Cursor => Some(".cursor/rules/promptctl.mdc"),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub const fn token_budget(&self) -> usize {
        match self {
            Agent::Copilot => 4000,
            Agent::Claude => 8000,
            Agent::Cursor => 4000,
            Agent::Codex => 6000,
            Agent::Aider => 4000,
            Agent::Raw => 8000,
        }
    }

    pub fn format_prompt(&self, content: &str, language: &str) -> String {
        match self {
            Agent::Copilot => format_copilot(content, language),
            Agent::Claude => format_claude(content, language),
            Agent::Cursor => format_cursor(content, language),
            Agent::Codex => format_codex(content, language),
            Agent::Aider => format_aider(content, language),
            Agent::Raw => content.to_string(),
        }
    }

    pub fn resolve_path(&self, project_root: &Path, global: bool) -> Option<PathBuf> {
        if *self == Agent::Raw {
            return None;
        }

        if global {
            let home = dirs::home_dir()?;
            let rel = self.global_instruction_file()?;
            Some(home.join(rel))
        } else {
            Some(project_root.join(self.instruction_file()))
        }
    }

    pub fn emit(
        &self,
        content: &str,
        project_root: &Path,
        global: bool,
        force: bool,
    ) -> Result<PathBuf, AgentError> {
        let path = self
            .resolve_path(project_root, global)
            .ok_or(AgentError::NoFilePath)?;

        if path.exists() && !force {
            return Err(AgentError::AlreadyExists(path));
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(AgentError::Io)?;
        }

        fs::write(&path, content).map_err(AgentError::Io)?;
        Ok(path)
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug)]
pub enum AgentError {
    Io(std::io::Error),
    AlreadyExists(PathBuf),
    NoFilePath,
    NoHookSupport(Agent),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentError::Io(e) => write!(f, "I/O error: {e}"),
            AgentError::AlreadyExists(p) => {
                write!(f, "file already exists: {} (use --force to overwrite)", p.display())
            }
            AgentError::NoFilePath => write!(f, "agent has no instruction file path"),
            AgentError::NoHookSupport(a) => {
                write!(f, "agent '{}' has no native hook support (use 'emit' instead)", a.name())
            }
        }
    }
}

impl std::error::Error for AgentError {}

// ── Hallucination prevention guardrails ──────────────────────────────────────

/// Generate anti-hallucination instructions tailored to a language and agent.
/// These are appended to every generated prompt when not in raw mode.
pub fn hallucination_guardrails(language: &str) -> String {
    let lang_specific = match language.to_lowercase().as_str() {
        "rust" => RUST_GUARDRAILS,
        "go" => GO_GUARDRAILS,
        "typescript" | "ts" => TS_GUARDRAILS,
        "solidity" | "sol" => SOLIDITY_GUARDRAILS,
        "leo" => LEO_GUARDRAILS,
        _ => "",
    };

    let mut output = String::from(COMMON_GUARDRAILS);
    if !lang_specific.is_empty() {
        output.push('\n');
        output.push_str(lang_specific);
    }
    output
}

const COMMON_GUARDRAILS: &str = r#"## Hallucination Prevention

- **Never invent APIs, functions, or types** that do not exist in the language or library version specified above.
- **Never fabricate crate, package, or module names**. Only reference dependencies that are documented and published.
- **If you are unsure whether a feature exists**, say so explicitly rather than guessing. Prefer linking to official docs.
- **Pin to the language version** declared in project config (Cargo.toml, go.mod, package.json, etc.) — do not assume newer features.
- **Do not hallucinate CLI flags, compiler options, or toolchain features** that do not exist for the specified version.
- **Verify struct fields, enum variants, and trait/interface methods** before referencing them — do not assume from memory.
- **When suggesting dependencies**, only suggest crates/packages you are confident exist and are actively maintained.
- **Prefer standard library solutions** over third-party when the stdlib provides equivalent functionality.
- **Quote error messages exactly** when referencing compiler or runtime errors — do not paraphrase."#;

const RUST_GUARDRAILS: &str = r#"- Do not reference unstable features or nightly-only APIs unless the project explicitly uses nightly.
- Do not invent trait implementations — verify a type actually implements a trait before calling its methods.
- Do not fabricate `unsafe` justifications — every `unsafe` block must have a real, auditable safety comment."#;

const GO_GUARDRAILS: &str = r#"- Do not reference Go generics syntax from versions prior to 1.18 or features beyond the go.mod version.
- Do not invent methods on standard library types — verify with `go doc`.
- Do not fabricate build tags or linker flags."#;

const TS_GUARDRAILS: &str = r#"- Do not invent TypeScript compiler options — verify against the tsconfig.json reference.
- Do not reference DOM APIs in Node.js contexts or vice versa without checking the environment.
- Do not fabricate type utility names — verify they exist in `typescript` or `@types/*` packages."#;

const SOLIDITY_GUARDRAILS: &str = r#"- Assume every external caller is a malicious contract — apply adversarial mindset.
- Enforce Checks-Effects-Interactions (CEI) pattern on every state-changing function that makes external calls.
- Every public/external state-changing function MUST have explicit access control — missing modifiers are critical vulnerabilities.
- Do not reference Solidity features from versions higher than the pragma specifies.
- Do not invent EIPs or precompile addresses — verify they exist on the target chain.
- Do not hallucinate storage slot layouts, ABI encoding details, or OpenZeppelin API surfaces."#;

const LEO_GUARDRAILS: &str = r#"- Do not invent Leo/Aleo instructions, opcodes, or record fields that do not exist.
- Do not fabricate program IDs or deployment addresses.
- Do not reference Aleo network features that are not yet on mainnet."#;

// ── Agent-specific formatters ────────────────────────────────────────────────

fn format_copilot(content: &str, _language: &str) -> String {
    format!(
        r#"<!-- Generated by promptctl — GitHub Copilot instructions -->
<!-- Regenerate: promptctl init copilot -->

<!-- COPILOT INSTRUCTIONS START -->
{content}
<!-- COPILOT INSTRUCTIONS END -->
"#
    )
}

fn format_claude(content: &str, _language: &str) -> String {
    format!(
        r#"<!-- Generated by promptctl — Claude Code instructions -->
<!-- Regenerate: promptctl init claude -->

<instructions>
{content}
</instructions>
"#
    )
}

fn format_cursor(content: &str, language: &str) -> String {
    // Cursor uses MDC (Markdown Components) format with YAML frontmatter.
    format!(
        r#"---
description: "{language} development guidelines generated by promptctl"
globs:
alwaysApply: true
---

{content}
"#,
        language = language,
        content = content,
    )
}

fn format_codex(content: &str, _language: &str) -> String {
    format!(
        r#"<!-- Generated by promptctl — OpenAI Codex agent instructions -->
<!-- Regenerate: promptctl init codex -->

{content}
"#
    )
}

fn format_aider(content: &str, _language: &str) -> String {
    format!(
        r#"<!-- Generated by promptctl — Aider conventions -->
<!-- Regenerate: promptctl init aider -->

{content}
"#
    )
}

// ── Agent-native hooks ───────────────────────────────────────────────────────
//
// "Agent hooks" are lifecycle hooks that run *inside* the AI agent's own loop,
// not git hooks. Each agent that supports them has a different format:
//
//   • Claude Code — `.claude/settings.json` + `.claude/hooks/*.sh` scripts
//     Events: SessionStart, PreToolUse (Write|Edit), PostToolUse, Stop, etc.
//
//   • Cursor — `.cursor/rules/*.mdc` with per-language globs
//     Rules are applied when files matching the glob are in context.
//
//   • Copilot — `.github/instructions/*.instructions.md` with `applyTo` globs
//     Path-specific instruction files scoped to file types.
//
//   • Codex / Aider — no native hook system (use `emit` for static files).

/// Which agents support native hooks?
pub fn supports_hooks(agent: Agent) -> bool {
    matches!(agent, Agent::Claude | Agent::Cursor | Agent::Copilot)
}

/// Metadata about a single file written by the hooks installer.
#[derive(Debug)]
pub struct HookFile {
    pub path: PathBuf,
    pub description: String,
}

/// Install agent-native hooks for the given agent.
///
/// `languages` is the set of detected project languages (lowercased).
/// `skillsets` maps language name → pre-built skillset content (from `build_skillset`).
/// Returns a list of files written.
pub fn install_agent_hooks(
    project_root: &Path,
    agent: Agent,
    languages: &[String],
    role: &str,
    skillsets: &std::collections::HashMap<String, String>,
    force: bool,
) -> Result<Vec<HookFile>, AgentError> {
    match agent {
        Agent::Claude => install_claude_hooks(project_root, role, force),
        Agent::Cursor => install_cursor_hooks(project_root, languages, role, skillsets, force),
        Agent::Copilot => install_copilot_hooks(project_root, languages, role, skillsets, force),
        _ => Err(AgentError::NoHookSupport(agent)),
    }
}

/// Remove previously installed agent hooks.
pub fn remove_agent_hooks(project_root: &Path, agent: Agent) -> Result<Vec<PathBuf>, AgentError> {
    match agent {
        Agent::Claude => remove_claude_hooks(project_root),
        Agent::Cursor => remove_cursor_hooks(project_root),
        Agent::Copilot => remove_copilot_hooks(project_root),
        _ => Err(AgentError::NoHookSupport(agent)),
    }
}

/// List currently installed agent hooks.
#[allow(dead_code)]
pub fn list_agent_hooks(project_root: &Path) -> Vec<(Agent, Vec<PathBuf>)> {
    let mut results = Vec::new();

    // Claude
    let claude_hooks_dir = project_root.join(".claude/hooks");
    if claude_hooks_dir.exists() {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(&claude_hooks_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("promptctl-")) {
                    files.push(p);
                }
            }
        }
        let settings = project_root.join(".claude/settings.json");
        if settings.exists() {
            if let Ok(content) = fs::read_to_string(&settings) {
                if content.contains("promptctl") {
                    files.push(settings);
                }
            }
        }
        if !files.is_empty() {
            results.push((Agent::Claude, files));
        }
    }

    // Cursor
    let cursor_rules_dir = project_root.join(".cursor/rules");
    if cursor_rules_dir.exists() {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(&cursor_rules_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("promptctl-")) {
                    files.push(p);
                }
            }
        }
        if !files.is_empty() {
            results.push((Agent::Cursor, files));
        }
    }

    // Copilot
    let copilot_instr_dir = project_root.join(".github/instructions");
    if copilot_instr_dir.exists() {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(&copilot_instr_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("promptctl-")) {
                    files.push(p);
                }
            }
        }
        if !files.is_empty() {
            results.push((Agent::Copilot, files));
        }
    }

    results
}

// ── Claude Code hooks ────────────────────────────────────────────────────────

fn install_claude_hooks(project_root: &Path, role: &str, force: bool) -> Result<Vec<HookFile>, AgentError> {
    let hooks_dir = project_root.join(".claude/hooks");
    fs::create_dir_all(&hooks_dir).map_err(AgentError::Io)?;

    let mut written = Vec::new();

    // 1. SessionStart script — injects promptctl guidelines on session start
    let session_script = CLAUDE_SESSION_START_SCRIPT.replace(
        "--role developer",
        &format!("--role {role}"),
    );
    let session_script_path = hooks_dir.join("promptctl-session-start.sh");
    write_hook_file(&session_script_path, &session_script, force)?;
    make_executable(&session_script_path)?;
    written.push(HookFile {
        path: session_script_path,
        description: "Injects promptctl guidelines on session start".into(),
    });

    // 2. PreToolUse script — reminds guidelines before Write/Edit
    let pre_write_script = CLAUDE_PRE_WRITE_SCRIPT.replace("--role developer", &format!("--role {role}"));
    let pre_write_path = hooks_dir.join("promptctl-pre-write.sh");
    write_hook_file(&pre_write_path, &pre_write_script, force)?;
    make_executable(&pre_write_path)?;
    written.push(HookFile {
        path: pre_write_path,
        description: "Validates language guidelines before file writes".into(),
    });

    // 3. Merge into .claude/settings.json
    let settings_path = project_root.join(".claude/settings.json");
    let merged = merge_claude_settings(&settings_path)?;
    fs::write(&settings_path, &merged).map_err(AgentError::Io)?;
    written.push(HookFile {
        path: settings_path,
        description: "Claude Code hook configuration".into(),
    });

    Ok(written)
}

/// Merge promptctl hooks into existing .claude/settings.json without clobbering
/// user settings. We do a simple JSON-level merge.
fn merge_claude_settings(settings_path: &Path) -> Result<String, AgentError> {
    let existing: serde_json::Value = if settings_path.exists() {
        let raw = fs::read_to_string(settings_path).map_err(AgentError::Io)?;
        serde_json::from_str(&raw).unwrap_or(serde_json::Value::Object(Default::default()))
    } else {
        serde_json::Value::Object(Default::default())
    };

    let mut root = match existing {
        serde_json::Value::Object(m) => m,
        _ => serde_json::Map::new(),
    };

    // Build our hooks object
    let our_hooks: serde_json::Value = serde_json::json!({
        "hooks": {
            "SessionStart": [{
                "matcher": "startup",
                "hooks": [{
                    "type": "command",
                    "command": "\"$CLAUDE_PROJECT_DIR\"/.claude/hooks/promptctl-session-start.sh",
                    "statusMessage": "Loading promptctl guidelines…"
                }]
            }],
            "PreToolUse": [{
                "matcher": "Write|Edit",
                "hooks": [{
                    "type": "command",
                    "command": "\"$CLAUDE_PROJECT_DIR\"/.claude/hooks/promptctl-pre-write.sh"
                }]
            }]
        }
    });

    // Merge: if user already has "hooks", merge our event arrays in
    if let Some(serde_json::Value::Object(existing_hooks)) = root.get("hooks") {
        let mut merged_hooks = existing_hooks.clone();
        if let serde_json::Value::Object(our) = &our_hooks["hooks"] {
            for (event, value) in our {
                // Only add our entry if the event doesn't already have promptctl hooks
                if let Some(serde_json::Value::Array(existing_arr)) = merged_hooks.get(event) {
                    let has_promptctl = existing_arr.iter().any(|v| {
                        v.to_string().contains("promptctl")
                    });
                    if has_promptctl {
                        continue; // Already installed
                    }
                    let mut combined = existing_arr.clone();
                    if let serde_json::Value::Array(new_arr) = value {
                        combined.extend(new_arr.clone());
                    }
                    merged_hooks.insert(event.clone(), serde_json::Value::Array(combined));
                } else {
                    merged_hooks.insert(event.clone(), value.clone());
                }
            }
        }
        root.insert("hooks".into(), serde_json::Value::Object(merged_hooks));
    } else {
        root.insert("hooks".into(), our_hooks["hooks"].clone());
    }

    serde_json::to_string_pretty(&serde_json::Value::Object(root))
        .map_err(|e| AgentError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
}

fn remove_claude_hooks(project_root: &Path) -> Result<Vec<PathBuf>, AgentError> {
    let mut removed = Vec::new();

    let hooks_dir = project_root.join(".claude/hooks");
    for name in ["promptctl-session-start.sh", "promptctl-pre-write.sh"] {
        let p = hooks_dir.join(name);
        if p.exists() {
            fs::remove_file(&p).map_err(AgentError::Io)?;
            removed.push(p);
        }
    }

    // Remove our entries from settings.json
    let settings_path = project_root.join(".claude/settings.json");
    if settings_path.exists() {
        let raw = fs::read_to_string(&settings_path).map_err(AgentError::Io)?;
        if let Ok(serde_json::Value::Object(mut root)) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(serde_json::Value::Object(hooks)) = root.get_mut("hooks") {
                for (_event, value) in hooks.iter_mut() {
                    if let serde_json::Value::Array(arr) = value {
                        arr.retain(|v| !v.to_string().contains("promptctl"));
                    }
                }
                // Remove empty arrays
                hooks.retain(|_, v| {
                    !matches!(v, serde_json::Value::Array(a) if a.is_empty())
                });
            }
            // If hooks is now empty, remove it
            if let Some(serde_json::Value::Object(hooks)) = root.get("hooks") {
                if hooks.is_empty() {
                    root.remove("hooks");
                }
            }
            let pretty = serde_json::to_string_pretty(&serde_json::Value::Object(root))
                .map_err(|e| AgentError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            fs::write(&settings_path, pretty).map_err(AgentError::Io)?;
            removed.push(settings_path);
        }
    }

    Ok(removed)
}

const CLAUDE_SESSION_START_SCRIPT: &str = r#"#!/bin/bash
# promptctl — Claude Code SessionStart hook
# Injects project-aware coding guidelines into the session context.

if ! command -v promptctl >/dev/null 2>&1; then
  exit 0
fi

GUIDELINES=$(promptctl show rust 2>/dev/null || promptctl list 2>/dev/null)
if [ -z "$GUIDELINES" ]; then
  exit 0
fi

# Return guidelines as additionalContext so Claude sees them
jq -n --arg ctx "$GUIDELINES" '{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": $ctx
  }
}'
"#;

const CLAUDE_PRE_WRITE_SCRIPT: &str = r#"#!/bin/bash
# promptctl — Claude Code PreToolUse hook (Write|Edit)
# Adds a reminder of the relevant language guidelines before file writes.

if ! command -v promptctl >/dev/null 2>&1; then
  exit 0
fi

# Extract the file path from the tool input on stdin
INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

if [ -z "$FILE_PATH" ]; then
  exit 0
fi

# Detect extension → language
EXT="${FILE_PATH##*.}"
case "$EXT" in
  rs)   LANG="rust" ;;
  go)   LANG="go" ;;
  ts|tsx) LANG="typescript" ;;
  sol)  LANG="solidity" ;;
  leo)  LANG="leo" ;;
  *)    exit 0 ;;
esac

# Fetch a minimal reminder for the target language
REMINDER=$(promptctl show "$LANG" --role developer 2>/dev/null | head -30)
if [ -z "$REMINDER" ]; then
  exit 0
fi

jq -n --arg ctx "Guidelines reminder for $LANG: follow the project coding standards." '{
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "additionalContext": $ctx
  }
}'
"#;

// ── Cursor hooks (per-language .mdc rules) ───────────────────────────────────

fn install_cursor_hooks(
    project_root: &Path,
    languages: &[String],
    role: &str,
    skillsets: &std::collections::HashMap<String, String>,
    force: bool,
) -> Result<Vec<HookFile>, AgentError> {
    let rules_dir = project_root.join(".cursor/rules");
    fs::create_dir_all(&rules_dir).map_err(AgentError::Io)?;

    let mut written = Vec::new();

    for lang in languages {
        let Some(info) = cursor_language_info(lang) else {
            continue;
        };

        let skillset_content = skillsets
            .get(lang.as_str())
            .map(|s| s.as_str())
            .unwrap_or("<!-- No skillset available — run 'promptctl show' to check -->");

        let filename = format!("promptctl-{lang}.mdc");
        let path = rules_dir.join(&filename);

        let content = format!(
            r#"---
description: "{lang} coding guidelines from promptctl — applied when editing {ext} files"
globs: "{globs}"
alwaysApply: false
---

<!-- Generated by promptctl init cursor --role {role} -->
<!-- Regenerate: promptctl init cursor --role {role} --force -->

{skillset_content}
"#,
            lang = lang,
            ext = info.ext,
            globs = info.globs,
            role = role,
            skillset_content = skillset_content,
        );

        write_hook_file(&path, &content, force)?;
        written.push(HookFile {
            path,
            description: format!("{lang} skillset for {ext} files", ext = info.ext),
        });
    }

    if written.is_empty() {
        return Err(AgentError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "no supported languages detected — run from a project directory",
        )));
    }

    Ok(written)
}

fn remove_cursor_hooks(project_root: &Path) -> Result<Vec<PathBuf>, AgentError> {
    let rules_dir = project_root.join(".cursor/rules");
    let mut removed = Vec::new();
    if rules_dir.exists() {
        if let Ok(entries) = fs::read_dir(&rules_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("promptctl-")) {
                    fs::remove_file(&p).map_err(AgentError::Io)?;
                    removed.push(p);
                }
            }
        }
    }
    Ok(removed)
}

struct LangGlobInfo {
    globs: &'static str,
    ext: &'static str,
}

fn cursor_language_info(lang: &str) -> Option<LangGlobInfo> {
    match lang.to_lowercase().as_str() {
        "rust" => Some(LangGlobInfo { globs: "**/*.rs", ext: ".rs" }),
        "go" => Some(LangGlobInfo { globs: "**/*.go", ext: ".go" }),
        "typescript" | "ts" => Some(LangGlobInfo { globs: "**/*.ts,**/*.tsx", ext: ".ts/.tsx" }),
        "solidity" | "sol" => Some(LangGlobInfo { globs: "**/*.sol", ext: ".sol" }),
        "leo" => Some(LangGlobInfo { globs: "**/*.leo", ext: ".leo" }),
        _ => None,
    }
}

// ── Copilot hooks (path-specific .instructions.md) ───────────────────────────

fn install_copilot_hooks(
    project_root: &Path,
    languages: &[String],
    role: &str,
    skillsets: &std::collections::HashMap<String, String>,
    force: bool,
) -> Result<Vec<HookFile>, AgentError> {
    let instr_dir = project_root.join(".github/instructions");
    fs::create_dir_all(&instr_dir).map_err(AgentError::Io)?;

    let mut written = Vec::new();

    for lang in languages {
        let Some(info) = cursor_language_info(lang) else {
            continue;
        };

        let skillset_content = skillsets
            .get(lang.as_str())
            .map(|s| s.as_str())
            .unwrap_or("<!-- No skillset available — run 'promptctl show' to check -->");

        let filename = format!("promptctl-{lang}.instructions.md");
        let path = instr_dir.join(&filename);

        let content = format!(
            r#"---
applyTo: "{globs}"
---

<!-- COPILOT INSTRUCTIONS START — {lang} skillset -->
<!-- Generated by promptctl init copilot --role {role} -->
<!-- Regenerate: promptctl init copilot --role {role} --force -->

{skillset_content}

<!-- COPILOT INSTRUCTIONS END -->
"#,
            globs = info.globs,
            lang = lang,
            role = role,
            skillset_content = skillset_content,
        );

        write_hook_file(&path, &content, force)?;
        written.push(HookFile {
            path,
            description: format!("{lang} skillset for {ext} files", ext = info.ext),
        });
    }

    if written.is_empty() {
        return Err(AgentError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "no supported languages detected — run from a project directory",
        )));
    }

    Ok(written)
}

fn remove_copilot_hooks(project_root: &Path) -> Result<Vec<PathBuf>, AgentError> {
    let instr_dir = project_root.join(".github/instructions");
    let mut removed = Vec::new();
    if instr_dir.exists() {
        if let Ok(entries) = fs::read_dir(&instr_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("promptctl-")) {
                    fs::remove_file(&p).map_err(AgentError::Io)?;
                    removed.push(p);
                }
            }
        }
    }
    Ok(removed)
}

// ── Shared helpers ───────────────────────────────────────────────────────────

fn write_hook_file(path: &Path, content: &str, force: bool) -> Result<(), AgentError> {
    if path.exists() && !force {
        return Err(AgentError::AlreadyExists(path.to_path_buf()));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(AgentError::Io)?;
    }
    fs::write(path, content).map_err(AgentError::Io)?;
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), AgentError> {
    use std::os::unix::fs::PermissionsExt;
    let perms = fs::Permissions::from_mode(0o755);
    fs::set_permissions(path, perms).map_err(AgentError::Io)
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), AgentError> {
    Ok(())
}
