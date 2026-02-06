# promptctl

A personal CLI tool for managing coding prompts across projects with project indexing, role-based customization, and token optimization.

## Features

- **Built-in prompts** for Rust (1.93), Go (1.25), Leo (Aleo smart contracts), TypeScript (5.9), and Solidity (0.8.28) with modern, idiomatic guidelines
- **Agent-specific output** — emit instructions for Copilot, Claude, Cursor, Codex, or Aider in their native format
- **Hallucination prevention** — auto-appended guardrails that prevent AI agents from inventing APIs or fabricating dependencies
- **Prompt merging** — extend built-in prompts with project-specific rules (prepend, append, or merge) instead of replacing them
- **Agent hooks** — native lifecycle hooks for Claude (SessionStart, PreToolUse), Cursor (per-language rules), and Copilot (path-specific instructions)
- **Project indexing** - Automatically detects languages, frameworks, and project structure
- **Role selection** - Apply different personas (developer, reviewer, security, etc.)
- **Tiered sizes** - Minimal (~500 tokens), compact (~1500), or full (~3000) prompts
- **Section selection** - Include only the sections you need
- **Smart filtering** - Auto-include relevant sections based on project analysis
- **Presets** - Save and reuse your favorite configurations
- **Custom prompts** via `.promptctl.toml` configuration files
- **Clipboard support** for quick copying

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap debendraoli/promptctl
brew install promptctl
```

### From crates.io

```bash
cargo install promptctl
```

### From source

```bash
git clone https://github.com/debendraoli/promptctl.git
cd promptctl
cargo install --path .
```

## Usage

### List available prompts

```bash
promptctl list
```

### Show a prompt

```bash
promptctl show rust
promptctl show go
promptctl show leo
promptctl show typescript
promptctl show solidity

# With a specific role
promptctl show rust --role reviewer
promptctl show go --role security
promptctl show solidity --role senior
```

### Copy prompt to clipboard

```bash
promptctl copy rust
promptctl copy go --role senior
promptctl copy leo --role security
```

### List available roles

```bash
promptctl roles
```

Available roles:

- **developer** - General development with focus on clean, working code
- **senior** - Architecture decisions, design patterns, and technical leadership
- **reviewer** - Code review with focus on quality and standards
- **security** - Security auditing and vulnerability detection
- **performance** - Performance optimization and efficiency
- **documentation** - Clear documentation and API design
- **mentor** - Teaching concepts with detailed explanations
- **devops** - CI/CD, deployment, and infrastructure

### Scan a project

```bash
promptctl scan
promptctl scan --path /path/to/project
```

This analyzes your project and detects:

- Programming languages and versions
- Frameworks and libraries
- Project structure (tests, docs, CI)

### Generate context-aware prompt

```bash
# Auto-detect language, use developer role
promptctl generate

# Specify role
promptctl generate --role reviewer

# Specify language
promptctl generate --language rust --role security

# Copy to clipboard
promptctl generate --role senior --copy

# Plain text output (no markdown)
promptctl generate --format plain
```

### Token-Optimized Prompts

Control prompt size to reduce token usage:

```bash
# Minimal (~500 tokens) - essential rules only
promptctl generate --size minimal

# Compact (~1500 tokens) - balanced guidelines (default)
promptctl generate --size compact

# Full (~3000 tokens) - comprehensive
promptctl generate --size full
```

### Section Selection

Include only specific sections:

```bash
# Show available sections
promptctl sections

# Select specific sections
promptctl generate --sections error-handling,testing,async

# Combine with size for fine control
promptctl generate --size minimal --sections concurrency
```

### Smart Filtering

Auto-include relevant sections based on project:

```bash
# Enable smart filtering
promptctl generate --smart

# Smart filtering adds sections based on:
# - Async frameworks detected → includes async section
# - Tests directory exists → includes testing section
# - Large project → includes structure, dependencies
```

### Presets

Save and reuse configurations:

```bash
# List available presets
promptctl preset list

# Use a built-in preset
promptctl generate --preset quick     # Minimal for quick fixes
promptctl generate --preset review    # Code review focused
promptctl generate --preset security  # Security audit
promptctl generate --preset learn     # Full with mentor role
promptctl generate --preset perf      # Performance focused
promptctl generate --preset daily     # Daily dev with smart filtering

# Save a custom preset
promptctl preset save myreview --role reviewer --size compact --sections error-handling,testing

# Use your preset
promptctl generate --preset myreview --copy

# Show preset details
promptctl preset show review

# Delete a preset
promptctl preset delete myreview
```

### Initialize config file

```bash
promptctl init
```

This creates a `.promptctl.toml` in the current directory for custom prompts.

### Agent Instructions

Emit instructions directly to the file each AI agent reads:

```bash
# List supported agents
promptctl agents

# Emit to agent instruction files
promptctl emit copilot              # writes .github/copilot-instructions.md
promptctl emit claude               # writes CLAUDE.md
promptctl emit cursor               # writes .cursor/rules/promptctl.mdc
promptctl emit codex                # writes AGENTS.md
promptctl emit aider                # writes CONVENTIONS.md

# Emit to global location (home dir)
promptctl emit copilot --global     # writes ~/.github/copilot-instructions.md
promptctl emit claude --global      # writes ~/.claude/CLAUDE.md

# Preview without writing
promptctl emit claude --dry-run

# Force overwrite
promptctl emit copilot --force

# Use with presets and roles
promptctl emit claude --role senior --size full --smart
```

### Agent-Formatted Generate

Format output for a specific agent without writing files:

```bash
# Format for Copilot (adds HTML comment header)
promptctl generate --agent copilot

# Format for Cursor (adds MDC frontmatter)
promptctl generate --agent cursor --copy

# Format for Claude and copy
promptctl generate --agent claude --role reviewer --copy
```

### Hallucination Prevention

Prompts now include anti-hallucination guardrails by default:

- Never invent APIs, functions, or types that don't exist
- Never fabricate crate/package names
- Pin to the language version in project config
- Language-specific guardrails (Rust, Go, TypeScript, Solidity, Leo)

```bash
# Disable guardrails if you want raw guidelines only
promptctl generate --no-guardrails
promptctl emit copilot --no-guardrails
```

### Agent Hooks

Install native lifecycle hooks that run inside each agent's own loop:

```bash
# Claude Code — SessionStart + PreToolUse (Write|Edit) hooks
promptctl hooks install claude

# Cursor — per-language .mdc rules with glob scoping
promptctl hooks install cursor

# Copilot — path-specific .instructions.md files
promptctl hooks install copilot

# Preview what would be generated
promptctl hooks install claude --dry-run

# List installed agent hooks
promptctl hooks list

# Remove hooks for an agent
promptctl hooks remove claude
```

**What gets generated:**

| Agent | Files | How it works |
|-------|-------|--------------|
| Claude | `.claude/hooks/promptctl-*.sh` + `.claude/settings.json` | Injects guidelines on session start; reminds context before file writes |
| Cursor | `.cursor/rules/promptctl-<lang>.mdc` | Per-language rules applied when editing matching files (e.g. `**/*.rs`) |
| Copilot | `.github/instructions/promptctl-<lang>.instructions.md` | Path-specific instructions scoped to file types via `applyTo` globs |

## Configuration

Create a `.promptctl.toml` file to add custom prompts:

```toml
# Optional: set a default AI agent for this project
default_agent = "copilot"  # copilot, claude, cursor, codex, aider

# Replace mode (default) — fully replace the built-in prompt
[prompts.python]
name = "Python"
description = "Python development guidelines"
content = """
# Python Development Guidelines

- Use Python 3.12+
- Follow PEP 8 style guide
- Use type hints everywhere
- Prefer dataclasses for data containers
"""

# Append mode — add project rules AFTER the built-in prompt
[prompts.rust]
name = "Rust"
mode = "append"
append = """
## Project-Specific Rules
- Use workspace dependencies from root Cargo.toml
- All public APIs must have doc comments
- Integration tests go in tests/ not src/
"""

# Prepend mode — add context BEFORE the built-in prompt
[prompts.typescript]
name = "TypeScript"
mode = "prepend"
prepend = """
## Project Context
This is a Next.js 15 app with App Router.
Use server components by default.
"""

# Merge mode — both prepend and append around the built-in
[prompts.go]
name = "Go"
mode = "merge"
prepend = """
## Our Go Standards
- All services use our internal pkg/errors package
"""
append = """
## Repo-Specific Patterns
- Use sqlc for database queries
- Proto files live in api/proto/
"""
```

### Prompt Modes

| Mode | Behavior |
|------|----------|
| `replace` | Fully replace the built-in prompt (default, backward-compatible) |
| `prepend` | Add custom content before the built-in prompt |
| `append` | Add custom content after the built-in prompt |
| `merge` | Use both `prepend` and `append` around the built-in prompt |

The tool searches for config files:

1. Current directory and parent directories
2. Home directory (`~/.promptctl.toml`)

Custom prompts override (or extend, with merge modes) built-in prompts with the same name.

## Built-in Prompts

### Rust (1.92)

Modern Rust development guidelines covering:

- Error handling with `thiserror`/`anyhow`
- Type system best practices
- Concurrency patterns
- Async Rust with tokio
- Testing strategies
- Project structure

### Go (1.25)

Idiomatic Go development guidelines covering:

- Generics and iterators
- Error handling patterns
- Concurrency with goroutines and channels
- Structured logging with `slog`
- Table-driven tests
- Project structure

### Leo (Aleo Smart Contracts)

Leo 3.4.0 development guidelines for privacy-preserving smart contracts:

- Program structure (records, mappings, async transitions, async functions)
- Async & Futures (`async transition`, `async function`, `Future`, `.await()`)
- Type system (primitives, field, address, structs, records, Future)
- Privacy patterns (shielding, unshielding, UTXO model)
- Cross-program calls with Future chaining
- Cryptographic operations (BHP256, Poseidon2, hash_to_bits, signatures)
- Testing with `@test` transitions
- CLI commands (`leo build`, `leo execute`, `leo deploy`)
- Common patterns (token standard, access control, generic arrays)

## Example Workflow

```bash
# 1. Scan your project to see what's detected
promptctl scan

# 2. Quick fix? Use minimal preset (~400 tokens)
promptctl generate --preset quick --copy

# 3. Code review? Use review preset
promptctl generate --preset review --copy

# 4. Daily development with smart filtering
promptctl generate --preset daily --copy

# 5. Save your own preset for repeated use
promptctl preset save mywork --role senior --size compact --smart
promptctl generate --preset mywork --copy
```

## Agent Usage Examples

AI coding agents can use `promptctl` to fetch language-specific guidelines before generating code. Here are common patterns:

### One-Command Setup Per Agent

```bash
# Set up Copilot for your project
promptctl emit copilot --smart

# Set up Claude Code
promptctl emit claude --role senior --smart

# Set up Cursor
promptctl emit cursor --size full

# Set up all agents at once
for agent in copilot claude cursor codex aider; do
    promptctl emit $agent --smart --force
done
```

### Agent Hooks — Deep Integration

```bash
# Install agent-native lifecycle hooks
promptctl hooks install claude       # SessionStart + PreToolUse hooks
promptctl hooks install cursor       # Per-language .mdc rules
promptctl hooks install copilot      # Path-specific instruction files

# Preview before writing
promptctl hooks install claude --dry-run

# Manage hooks
promptctl hooks list
promptctl hooks remove cursor
```

### Fetch Guidelines Before Coding

```bash
# Agent fetches Rust guidelines before writing Rust code
promptctl show rust

# Fetch with specific role for the task
promptctl show rust --role security    # For security-sensitive code
promptctl show rust --role reviewer    # Before reviewing PRs
promptctl show go --role performance   # For performance-critical Go code
```

### Project-Aware Generation

```bash
# Let promptctl detect project language and generate appropriate guidelines
cd /path/to/project
promptctl generate

# With role based on task
promptctl generate --role senior       # Architecture decisions
promptctl generate --role security     # Security audit
promptctl generate --role mentor       # Learning/explaining

# Format for a specific agent
promptctl generate --agent copilot --copy
promptctl generate --agent claude --role reviewer --copy
```

### Token-Efficient Prompts for Agents

```bash
# Minimal guidelines to save context window
promptctl generate --size minimal

# Only specific sections relevant to the task
promptctl generate --sections error-handling,testing

# Quick preset for simple fixes
promptctl generate --preset quick
```

### Smart Contract Development

```bash
# Solidity for EVM chains
promptctl show solidity
promptctl show sol --role security     # Smart contract auditing

# Leo for Aleo privacy chains
promptctl show leo
```

### Agent Workflow Example

```bash
# 1. Detect project type
promptctl scan --json 2>/dev/null || promptctl scan

# 2. Emit instructions for your agent
promptctl emit copilot --smart

# 3. Install agent hooks for deep integration
promptctl hooks install claude         # Lifecycle hooks
promptctl hooks install cursor         # Per-language rules

# 4. Or fetch guidelines for ad-hoc use
GUIDELINES=$(promptctl generate --size compact)

# 5. For specific tasks, use targeted sections
promptctl generate --sections async,concurrency --role performance
```

### Combining with Project Context

```bash
# Generate context-aware prompt with smart filtering
promptctl generate --smart --role developer

# Full guidelines for complex architectural decisions
promptctl generate --size full --role senior

# Security review prompt
promptctl generate --preset security --sections error-handling
```

### Copy-Paste for Agent Prompts

Add this to your AI agent's system prompt or MCP server instructions:

```
You have access to `promptctl` CLI for fetching language-specific coding guidelines.

Before writing code in any supported language, fetch the appropriate guidelines:
- `promptctl show <language>` - Get full guidelines (rust, go, leo, typescript, solidity)
- `promptctl show <language> --role <role>` - Role-specific (developer, senior, reviewer, security, performance)
- `promptctl generate --size minimal` - Token-efficient guidelines (~500 tokens)
- `promptctl generate --sections <sections>` - Specific sections (error-handling, testing, async, concurrency, types, memory, security)
- `promptctl emit <agent>` - Write native instruction files (copilot, claude, cursor, codex, aider)
- `promptctl hooks install <agent>` - Install agent-native lifecycle hooks (claude, cursor, copilot)

Workflow:
1. Detect project language: `promptctl scan`
2. Install agent hooks: `promptctl hooks install claude` (or cursor/copilot)
3. Emit agent instructions: `promptctl emit copilot --smart`
4. Or fetch guidelines before coding: `promptctl show rust` or `promptctl generate`
5. For security-sensitive code: `promptctl show <lang> --role security`
6. For code reviews: `promptctl show <lang> --role reviewer`

Always follow the fetched guidelines when generating code.
```

## Supported Agents

| Agent | Instruction File | Global File | Native Hooks |
|-------|-----------------|-------------|----|
| Copilot | `.github/copilot-instructions.md` | `~/.github/copilot-instructions.md` | `.github/instructions/*.instructions.md` |
| Claude | `CLAUDE.md` | `~/.claude/CLAUDE.md` | `.claude/hooks/promptctl-*.sh` + settings.json |
| Cursor | `.cursor/rules/promptctl.mdc` | `~/.cursor/rules/promptctl.mdc` | `.cursor/rules/promptctl-<lang>.mdc` |
| Codex | `AGENTS.md` | — | — |
| Aider | `CONVENTIONS.md` | — | — |

## Token Estimates

| Size | Tokens | Use Case |
|------|--------|----------|
| minimal | ~400-600 | Quick fixes, simple tasks |
| compact | ~1200-1800 | Daily development |
| full | ~2500-3500 | Learning, complex architecture |

## License

MIT
