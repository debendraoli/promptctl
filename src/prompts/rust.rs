//! Rust programming prompt template.

use crate::prompt_builder::{PromptSection, Section, StructuredPrompt};

pub const RUST_PROMPT: &str = r#"# Rust Development Guidelines (1.92)

## Language Version
- Target **Rust 1.92** stable
- Use **edition = "2024"** in Cargo.toml
- Leverage all stable features available in 1.92

## Code Style & Idioms

### General Principles
- Write idiomatic, expressive Rust code
- Prefer zero-cost abstractions
- Embrace ownership and borrowing—avoid unnecessary cloning
- Use `#[must_use]` on functions returning values that shouldn't be ignored
- Prefer `impl Trait` in argument and return positions for flexibility

### Error Handling
- Use `thiserror` for library error types, `anyhow` for applications
- Implement `std::error::Error` for custom error types
- Use `?` operator for propagation; avoid `.unwrap()` in production code
- Provide context with `.context()` or `.with_context()`
- Use `Result<T, E>` as the primary error handling mechanism

### Type System
- Leverage the type system to make invalid states unrepresentable
- Use newtypes for domain modeling (`struct UserId(u64)`)
- Prefer enums over boolean flags for clarity
- Use `Option<T>` instead of sentinel values
- Implement `From`/`Into` for type conversions

### Memory & Performance
- Prefer stack allocation; use `Box` only when necessary
- Use `Cow<'_, T>` for flexible ownership
- Prefer `&str` over `String` in function parameters
- Use `Vec::with_capacity` when size is known
- Leverage iterators and lazy evaluation

### Concurrency
- Use `std::sync` primitives (`Mutex`, `RwLock`, `Arc`)
- Prefer channels (`mpsc`, `crossbeam`) for communication
- Use `rayon` for data parallelism
- Consider `tokio` or `async-std` for async I/O
- Avoid `unsafe` unless absolutely necessary; document safety invariants

### Async Rust
- Use `async`/`await` with `tokio` runtime (or `async-std`)
- Prefer `tokio::spawn` for concurrent tasks
- Use `Select` for racing futures
- Handle cancellation properly with `tokio::select!`
- Use `async-trait` when needed for trait methods

### Testing
- Write unit tests in the same file with `#[cfg(test)]`
- Use `#[should_panic]` for expected panics
- Leverage `proptest` or `quickcheck` for property-based testing
- Use `mockall` or `mockito` for mocking
- Integration tests go in `tests/` directory

### Project Structure
```
src/
├── main.rs          # Binary entry point
├── lib.rs           # Library root (optional)
├── error.rs         # Error types
├── config.rs        # Configuration
└── modules/         # Feature modules
    ├── mod.rs
    └── feature.rs
```

### Dependencies Best Practices
- Use `cargo-audit` for security vulnerabilities
- Prefer well-maintained crates with good documentation
- Pin versions in `Cargo.lock` for reproducibility
- Use workspace for multi-crate projects
- Enable LTO and strip in release profile

### Documentation
- Document public APIs with `///` doc comments
- Use `//!` for module-level documentation
- Include examples in doc comments (they're tested!)
- Use `#[doc(hidden)]` for internal-only public items
- Generate docs with `cargo doc --open`

### Common Patterns
```rust
// Builder pattern with typestate
pub struct RequestBuilder<State> {
    inner: Request,
    _state: PhantomData<State>,
}

// Extension traits
pub trait StringExt {
    fn truncate_ellipsis(&self, max_len: usize) -> Cow<'_, str>;
}

// Newtype pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(pub u64);

// Error with context
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("failed to read config: {0}")]
    Config(#[from] std::io::Error),
    #[error("invalid input: {message}")]
    Validation { message: String },
}
```

### Clippy & Formatting
- Run `cargo fmt` before committing
- Enable pedantic clippy: `#![warn(clippy::pedantic)]`
- Address all warnings; use `#[allow(...)]` sparingly with justification
- Use `rustfmt.toml` for team-wide formatting consistency
"#;

/// Create a structured Rust prompt with sections
pub fn structured_prompt() -> StructuredPrompt {
    StructuredPrompt {
        language: "rust".to_string(),
        sections: vec![
            PromptSection {
                section: Section::Version,
                title: "Language Version".to_string(),
                content: r#"- Target **Rust 1.92** stable
- Use **edition = "2024"** in Cargo.toml
- Leverage all stable features available in 1.92"#
                    .to_string(),
                relevance_keywords: vec!["rust", "edition", "version", "cargo"],
            },
            PromptSection {
                section: Section::Style,
                title: "Code Style & Idioms".to_string(),
                content: r#"- Write idiomatic, expressive Rust code
- Prefer zero-cost abstractions
- Embrace ownership and borrowing—avoid unnecessary cloning
- Use `#[must_use]` on functions returning values that shouldn't be ignored
- Prefer `impl Trait` in argument and return positions for flexibility"#
                    .to_string(),
                relevance_keywords: vec!["style", "idiom", "ownership", "borrow"],
            },
            PromptSection {
                section: Section::ErrorHandling,
                title: "Error Handling".to_string(),
                content: r#"- Use `thiserror` for library error types, `anyhow` for applications
- Implement `std::error::Error` for custom error types
- Use `?` operator for propagation; avoid `.unwrap()` in production code
- Provide context with `.context()` or `.with_context()`
- Use `Result<T, E>` as the primary error handling mechanism"#
                    .to_string(),
                relevance_keywords: vec!["error", "result", "thiserror", "anyhow", "unwrap"],
            },
            PromptSection {
                section: Section::Types,
                title: "Type System".to_string(),
                content: r#"- Leverage the type system to make invalid states unrepresentable
- Use newtypes for domain modeling (`struct UserId(u64)`)
- Prefer enums over boolean flags for clarity
- Use `Option<T>` instead of sentinel values
- Implement `From`/`Into` for type conversions"#
                    .to_string(),
                relevance_keywords: vec!["type", "enum", "struct", "newtype", "option"],
            },
            PromptSection {
                section: Section::Memory,
                title: "Memory & Performance".to_string(),
                content: r#"- Prefer stack allocation; use `Box` only when necessary
- Use `Cow<'_, T>` for flexible ownership
- Prefer `&str` over `String` in function parameters
- Use `Vec::with_capacity` when size is known
- Leverage iterators and lazy evaluation"#
                    .to_string(),
                relevance_keywords: vec![
                    "memory",
                    "performance",
                    "allocation",
                    "box",
                    "cow",
                    "iterator",
                ],
            },
            PromptSection {
                section: Section::Concurrency,
                title: "Concurrency".to_string(),
                content: r#"- Use `std::sync` primitives (`Mutex`, `RwLock`, `Arc`)
- Prefer channels (`mpsc`, `crossbeam`) for communication
- Use `rayon` for data parallelism
- Avoid `unsafe` unless absolutely necessary; document safety invariants"#
                    .to_string(),
                relevance_keywords: vec![
                    "concurrency",
                    "mutex",
                    "arc",
                    "channel",
                    "thread",
                    "rayon",
                ],
            },
            PromptSection {
                section: Section::Async,
                title: "Async Rust".to_string(),
                content: r#"- Use `async`/`await` with `tokio` runtime (or `async-std`)
- Prefer `tokio::spawn` for concurrent tasks
- Use `Select` for racing futures
- Handle cancellation properly with `tokio::select!`
- Use `async-trait` when needed for trait methods"#
                    .to_string(),
                relevance_keywords: vec!["async", "await", "tokio", "future", "spawn"],
            },
            PromptSection {
                section: Section::Testing,
                title: "Testing".to_string(),
                content: r#"- Write unit tests in the same file with `#[cfg(test)]`
- Use `#[should_panic]` for expected panics
- Leverage `proptest` or `quickcheck` for property-based testing
- Use `mockall` or `mockito` for mocking
- Integration tests go in `tests/` directory"#
                    .to_string(),
                relevance_keywords: vec!["test", "testing", "mock", "proptest", "assert"],
            },
            PromptSection {
                section: Section::Structure,
                title: "Project Structure".to_string(),
                content: r#"```
src/
├── main.rs          # Binary entry point
├── lib.rs           # Library root (optional)
├── error.rs         # Error types
├── config.rs        # Configuration
└── modules/         # Feature modules
    ├── mod.rs
    └── feature.rs
```"#
                    .to_string(),
                relevance_keywords: vec!["structure", "project", "module", "organization"],
            },
            PromptSection {
                section: Section::Dependencies,
                title: "Dependencies Best Practices".to_string(),
                content: r#"- Use `cargo-audit` for security vulnerabilities
- Prefer well-maintained crates with good documentation
- Pin versions in `Cargo.lock` for reproducibility
- Use workspace for multi-crate projects
- Enable LTO and strip in release profile"#
                    .to_string(),
                relevance_keywords: vec!["dependency", "cargo", "crate", "workspace"],
            },
            PromptSection {
                section: Section::Documentation,
                title: "Documentation".to_string(),
                content: r#"- Document public APIs with `///` doc comments
- Use `//!` for module-level documentation
- Include examples in doc comments (they're tested!)
- Use `#[doc(hidden)]` for internal-only public items
- Generate docs with `cargo doc --open`"#
                    .to_string(),
                relevance_keywords: vec!["doc", "documentation", "comment", "rustdoc"],
            },
            PromptSection {
                section: Section::Patterns,
                title: "Common Patterns".to_string(),
                content: r#"```rust
// Builder pattern with typestate
pub struct RequestBuilder<State> {
    inner: Request,
    _state: PhantomData<State>,
}

// Newtype pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(pub u64);

// Error with context
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("failed to read config: {0}")]
    Config(#[from] std::io::Error),
}
```"#
                    .to_string(),
                relevance_keywords: vec!["pattern", "builder", "newtype", "example"],
            },
            PromptSection {
                section: Section::Tooling,
                title: "Clippy & Formatting".to_string(),
                content: r#"- Run `cargo fmt` before committing
- Enable pedantic clippy: `#![warn(clippy::pedantic)]`
- Address all warnings; use `#[allow(...)]` sparingly with justification
- Use `rustfmt.toml` for team-wide formatting consistency"#
                    .to_string(),
                relevance_keywords: vec!["clippy", "fmt", "format", "lint", "rustfmt"],
            },
        ],
    }
}
