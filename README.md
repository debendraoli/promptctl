# promptctl

A personal CLI tool for managing coding prompts across projects with project indexing, role-based customization, and token optimization.

## Features

- **Built-in prompts** for Rust (1.92), Go (1.25), and Leo (Aleo smart contracts) with modern, idiomatic guidelines
- **Project indexing** - Automatically detects languages, frameworks, and project structure
- **Role selection** - Apply different personas (developer, reviewer, security, etc.)
- **Tiered sizes** - Minimal (~500 tokens), compact (~1500), or full (~3000) prompts
- **Section selection** - Include only the sections you need
- **Smart filtering** - Auto-include relevant sections based on project analysis
- **Presets** - Save and reuse your favorite configurations
- **Custom prompts** via `.promptctl.toml` configuration files
- **Clipboard support** for quick copying

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
# Binary will be at target/release/promptctl
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

# With a specific role
promptctl show rust --role reviewer
promptctl show go --role security
promptctl show leo --role senior
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

## Configuration

Create a `.promptctl.toml` file to add custom prompts:

```toml
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

[prompts.typescript]
name = "TypeScript"
description = "TypeScript best practices"
content = """
# TypeScript Guidelines

- Use strict mode
- Prefer interfaces over types for object shapes
- Use const assertions where applicable
"""
```

The tool searches for config files:

1. Current directory and parent directories
2. Home directory (`~/.promptctl.toml`)

Custom prompts override built-in prompts with the same name.

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

## Token Estimates

| Size | Tokens | Use Case |
|------|--------|----------|
| minimal | ~400-600 | Quick fixes, simple tasks |
| compact | ~1200-1800 | Daily development |
| full | ~2500-3500 | Learning, complex architecture |

## License

MIT
