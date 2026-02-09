# promptctl

A CLI tool for setting up AI coding agent instructions across projects.

One command scans your project, writes agent-native instruction files, and installs language-specific skillset hooks.

## Features

- **One-command setup** — `promptctl init <agent>` does everything: scan, emit, hooks
- **Two-layer architecture** — lightweight base file (role + project context + guardrails) plus full language skillsets as hooks
- **Built-in skillsets** for Rust, Go, Leo, TypeScript, and Solidity
- **Agent-native output** — writes to the file each agent reads (Copilot, Claude, Cursor, Codex, Aider)
- **Agent-specific formatting** — Copilot markers, Claude XML tags, Cursor MDC frontmatter
- **8 roles** — developer, senior, reviewer, security, performance, documentation, mentor, devops
- **Lifecycle hooks** — Claude session hooks, Cursor per-language rules, Copilot path-specific instructions
- **Hallucination prevention** — generic guardrails in the base file; language-specific guardrails in skillsets
- **Prompt merging** — extend built-in skillsets with project-specific rules via `.promptctl.toml`
- **Project indexing** — auto-detects languages, frameworks, and project structure

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

## Quick Start

```bash
# Set up Copilot for your project (scans, writes instructions, installs hooks)
promptctl init copilot

# Set up Claude Code
promptctl init claude

# Set up Cursor
promptctl init cursor

# Preview what would be generated
promptctl init copilot --dry-run
```

That's it. One command.

## Architecture

`promptctl init` generates two layers of content:

### Base instruction file

Contains **no language-specific content**. Only:

1. **Role prefix** — persona and priorities for the chosen role
2. **Project context** — detected languages, frameworks, CI presence
3. **Language skillsets note** — tells the agent that language rules are loaded separately
4. **Hallucination prevention** — generic guardrails (don't invent APIs, pin versions, etc.)

### Skillset hooks (per language)

Full language-specific guidelines covering:

- Code style and idioms
- Error handling patterns
- Type system usage
- Testing conventions
- Concurrency and async
- Security best practices
- Tooling and linting
- Dependencies and project structure

Hooks are installed as agent-native files:

- **Copilot**: `.github/instructions/promptctl-<lang>.instructions.md` (MDC frontmatter with `applyTo` globs)
- **Claude**: `.claude/hooks/promptctl-<lang>.sh` (session hooks that call `promptctl show`)
- **Cursor**: `.cursor/rules/promptctl-<lang>.mdc` (MDC rules with glob patterns)

This separation keeps the base file small (~500 tokens) while delivering comprehensive language guidelines through hooks that activate only for matching files.

## Commands

### `init` — Set up an agent

Scans your project, writes the agent's instruction file, and installs skillset hooks.

```bash
promptctl init <agent>                   # copilot, claude, cursor, codex, aider
promptctl init copilot --role security   # security-focused guidelines
promptctl init claude --role reviewer    # code review persona
promptctl init copilot --force           # overwrite existing files
promptctl init copilot --global          # write to ~/  instead of project
promptctl init claude --dry-run          # preview without writing
```

### `show` — View a language skillset

```bash
promptctl show rust
promptctl show go --role security
promptctl show typescript --role reviewer
```

### `list` — See what's available

```bash
promptctl list
```

Shows all built-in languages, supported agents, and available roles.

### `clean` — Remove generated files

```bash
promptctl clean copilot
promptctl clean claude
```

Removes the instruction file and any hooks installed by `init`.

## Roles

| Role | Focus |
|------|-------|
| `developer` | Clean, working, idiomatic code (default) |
| `senior` | Architecture, design decisions, code ownership, mentorship |
| `reviewer` | Code quality, maintainability, standards enforcement |
| `security` | Vulnerability detection, secure coding, auditing |
| `performance` | Profiling, optimization, benchmarking, resource efficiency |
| `documentation` | API docs, guides, READMEs, code comments |
| `mentor` | Teaching, explaining concepts, pair programming guidance |
| `devops` | CI/CD, infrastructure, deployment, monitoring |

Aliases: `senior` accepts `sr`, `lead`, `architect`; `reviewer` accepts `review`; `documentation` accepts `docs`, `doc`; `devops` accepts `ops`, `infra`, `ci`.

## Supported Agents

| Agent | Instruction File | Hooks |
|-------|-----------------|-------|
| Copilot | `.github/copilot-instructions.md` | `.github/instructions/*.instructions.md` |
| Claude | `CLAUDE.md` | `.claude/hooks/promptctl-*.sh` + settings.json |
| Cursor | `.cursor/rules/promptctl.mdc` | `.cursor/rules/promptctl-<lang>.mdc` |
| Codex | `AGENTS.md` | — |
| Aider | `CONVENTIONS.md` | — |

### Agent-specific formatting

- **Copilot** — base file wrapped in `<!-- COPILOT INSTRUCTIONS START/END -->` markers
- **Claude** — base file wrapped in `<instructions>` XML tags
- **Cursor** — MDC frontmatter with `description` and `globs`

## Configuration

Create a `.promptctl.toml` to customize skillsets (auto-created by `init`):

```toml
# Set a default agent
default_agent = "copilot"

# Extend the built-in Rust skillset with project rules
[prompts.rust]
name = "Rust"
mode = "append"
append = """
## Project-Specific Rules
- Use workspace dependencies from root Cargo.toml
- All public APIs must have doc comments
"""
```

### Prompt Modes

| Mode | Behavior |
|------|----------|
| `replace` | Fully replace the built-in skillset (default) |
| `prepend` | Add custom content before the built-in |
| `append` | Add custom content after the built-in |
| `merge` | Both prepend and append around the built-in |

## Built-in Language Skillsets

- **Rust** (1.93) — ownership, error handling (`thiserror`/`anyhow`), type system, async/await, testing, clippy, security
- **Go** (1.25) — generics, error handling, concurrency, `slog`, testing, architecture, security
- **Leo** (Aleo) — program structure, privacy patterns, cross-program calls, ZK circuits
- **TypeScript** (5.9) — strict types, async patterns, framework awareness (Next.js, React, vitest), security
- **Solidity** (0.8.33) — adversarial mindset, CEI pattern, access control, Foundry-first testing

## License

MIT
