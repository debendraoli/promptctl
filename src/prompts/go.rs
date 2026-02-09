//! Go programming prompt template.

use crate::prompt_builder::{PromptSection, Section, StructuredPrompt};

pub const GO_PROMPT: &str = r#"# Go Development Guidelines (1.25)

## Language Version
- Target **Go 1.25**
- Use `go 1.25` in go.mod
- Leverage all features including enhanced generics and iterators

## Code Style & Idioms

### General Principles
- Write simple, readable, idiomatic Go code
- Prefer composition over inheritance
- Keep functions small and focused
- Use meaningful names; avoid abbreviations except for common ones (ctx, err, req)
- Accept interfaces, return concrete types

### Error Handling
- Always handle errors explicitly; never ignore with `_`
- Use `errors.Is()` and `errors.As()` for error inspection
- Wrap errors with context: `fmt.Errorf("operation failed: %w", err)`
- Create sentinel errors with `var ErrNotFound = errors.New("not found")`
- Use custom error types for rich error information

### Generics (Go 1.25)
- Use generics for type-safe containers and algorithms
- Prefer constraints from `constraints` package or `any`/`comparable`
- Don't over-genericize; use when it genuinely reduces duplication
- Use type inference when types are obvious

```go
func Map[T, U any](items []T, fn func(T) U) []U {
    result := make([]U, len(items))
    for i, item := range items {
        result[i] = fn(item)
    }
    return result
}
```

### Iterators (Go 1.25)
- Use range-over-func for custom iteration
- Implement iterator pattern with `iter.Seq[T]`
- Use `slices` and `maps` packages for common operations

```go
func (s *Store) All() iter.Seq[Item] {
    return func(yield func(Item) bool) {
        for _, item := range s.items {
            if !yield(item) {
                return
            }
        }
    }
}
```

### Concurrency
- Use goroutines for concurrent operations
- Communicate via channels; don't share memory
- Use `sync.WaitGroup` for coordination
- Prefer `context.Context` for cancellation and timeouts
- Use `sync.Mutex` or `sync.RWMutex` when shared state is unavoidable
- Always pass context as the first parameter

```go
func ProcessItems(ctx context.Context, items []Item) error {
    g, ctx := errgroup.WithContext(ctx)
    for _, item := range items {
        g.Go(func() error {
            return process(ctx, item)
        })
    }
    return g.Wait()
}
```

### Structured Logging
- Use `log/slog` for structured logging (standard library)
- Include context in log entries
- Use appropriate log levels (Debug, Info, Warn, Error)

```go
slog.Info("processing request",
    slog.String("method", r.Method),
    slog.String("path", r.URL.Path),
    slog.Duration("duration", elapsed),
)
```

### Testing
- Write table-driven tests
- Use `t.Parallel()` for independent tests
- Use `testify/assert` or standard library assertions
- Use `t.Helper()` in test helper functions
- Place tests in `*_test.go` files
- Use `testing/quick` for property-based testing

```go
func TestAdd(t *testing.T) {
    tests := []struct {
        name     string
        a, b     int
        expected int
    }{
        {"positive", 1, 2, 3},
        {"negative", -1, -2, -3},
        {"zero", 0, 0, 0},
    }
    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            t.Parallel()
            got := Add(tt.a, tt.b)
            if got != tt.expected {
                t.Errorf("Add(%d, %d) = %d; want %d", tt.a, tt.b, got, tt.expected)
            }
        })
    }
}
```

### Project Structure
```
project/
├── cmd/
│   └── myapp/
│       └── main.go         # Application entry point
├── internal/               # Private packages
│   ├── config/
│   ├── handler/
│   └── service/
├── pkg/                    # Public packages (optional)
├── go.mod
├── go.sum
└── Makefile
```

### Interface Design
- Keep interfaces small (1-3 methods)
- Define interfaces where they're used, not implemented
- Use `io.Reader`, `io.Writer` and standard interfaces
- Prefer function types over single-method interfaces

```go
// Good: small, focused interface
type Validator interface {
    Validate() error
}

// Good: function type for simple cases
type Handler func(ctx context.Context, req Request) (Response, error)
```

### Resource Management
- Use `defer` for cleanup immediately after acquiring resources
- Implement `io.Closer` for types that hold resources
- Use `context.Context` for cancellation propagation

### HTTP Services
- Use `http.ServeMux` (enhanced in Go 1.22+) or chi/gorilla/echo
- Implement middleware for cross-cutting concerns
- Use `http.Handler` interface consistently

```go
mux := http.NewServeMux()
mux.HandleFunc("GET /users/{id}", getUser)
mux.HandleFunc("POST /users", createUser)
```

### Common Patterns
```go
// Functional options
type Option func(*Config)

func WithTimeout(d time.Duration) Option {
    return func(c *Config) { c.Timeout = d }
}

func New(opts ...Option) *Client {
    cfg := defaultConfig()
    for _, opt := range opts {
        opt(&cfg)
    }
    return &Client{config: cfg}
}

// Constructor with validation
func NewUser(name, email string) (*User, error) {
    if name == "" {
        return nil, errors.New("name is required")
    }
    return &User{Name: name, Email: email}, nil
}
```

### Tools & Linting
- Run `go fmt` and `goimports` before committing
- Use `golangci-lint` with strict configuration
- Enable `go vet` in CI
- Use `govulncheck` for security scanning
- Run `go mod tidy` to clean dependencies
"#;

/// Create a structured Go prompt with sections
pub fn structured_prompt() -> StructuredPrompt {
    StructuredPrompt {
        language: "go".to_string(),
        sections: vec![
            PromptSection {
                section: Section::Version,
                title: "Language Version".to_string(),
                content: r#"- Target **Go 1.25**
- Use `go 1.25` in go.mod
- Leverage all features including enhanced generics and iterators"#
                    .to_string(),
                relevance_keywords: vec!["go", "version", "go.mod"],
            },
            PromptSection {
                section: Section::Style,
                title: "Code Style & Idioms".to_string(),
                content: r#"- Write simple, readable, idiomatic Go code
- Prefer composition over inheritance
- Keep functions small and focused
- Use meaningful names; avoid abbreviations except for common ones (ctx, err, req)
- Accept interfaces, return concrete types"#
                    .to_string(),
                relevance_keywords: vec!["style", "idiom", "naming", "composition"],
            },
            PromptSection {
                section: Section::ErrorHandling,
                title: "Error Handling".to_string(),
                content: r#"- Always handle errors explicitly; never ignore with `_`
- Use `errors.Is()` and `errors.As()` for error inspection
- Wrap errors with context: `fmt.Errorf("operation failed: %w", err)`
- Create sentinel errors with `var ErrNotFound = errors.New("not found")`
- Use custom error types for rich error information"#
                    .to_string(),
                relevance_keywords: vec!["error", "errors", "wrap", "sentinel"],
            },
            PromptSection {
                section: Section::Types,
                title: "Generics & Types".to_string(),
                content: r#"- Use generics for type-safe containers and algorithms
- Prefer constraints from `constraints` package or `any`/`comparable`
- Don't over-genericize; use when it genuinely reduces duplication
- Use type inference when types are obvious
- Keep interfaces small (1-3 methods)
- Define interfaces where they're used, not implemented"#
                    .to_string(),
                relevance_keywords: vec!["generic", "type", "interface", "constraint"],
            },
            PromptSection {
                section: Section::Concurrency,
                title: "Concurrency".to_string(),
                content: r#"- Use goroutines for concurrent operations
- Communicate via channels; don't share memory
- Use `sync.WaitGroup` for coordination
- Prefer `context.Context` for cancellation and timeouts
- Use `sync.Mutex` or `sync.RWMutex` when shared state is unavoidable
- Always pass context as the first parameter"#
                    .to_string(),
                relevance_keywords: vec!["goroutine", "channel", "mutex", "context", "concurrent"],
            },
            PromptSection {
                section: Section::Async,
                title: "Iterators & Channels".to_string(),
                content: r#"- Use range-over-func for custom iteration (Go 1.25)
- Implement iterator pattern with `iter.Seq[T]`
- Use `slices` and `maps` packages for common operations
- Use buffered channels for producer-consumer patterns"#
                    .to_string(),
                relevance_keywords: vec!["iterator", "range", "channel", "iter"],
            },
            PromptSection {
                section: Section::Testing,
                title: "Testing".to_string(),
                content: r#"- Write table-driven tests
- Use `t.Parallel()` for independent tests
- Use `testify/assert` or standard library assertions
- Use `t.Helper()` in test helper functions
- Place tests in `*_test.go` files
- Use `testing/quick` for property-based testing"#
                    .to_string(),
                relevance_keywords: vec!["test", "testing", "assert", "parallel"],
            },
            PromptSection {
                section: Section::Structure,
                title: "Project Structure".to_string(),
                content: r#"```
project/
├── cmd/
│   └── myapp/
│       └── main.go         # Application entry point
├── internal/               # Private packages
│   ├── config/
│   ├── handler/
│   └── service/
├── pkg/                    # Public packages (optional)
├── go.mod
└── Makefile
```"#
                    .to_string(),
                relevance_keywords: vec!["structure", "project", "cmd", "internal", "pkg"],
            },
            PromptSection {
                section: Section::Documentation,
                title: "Documentation".to_string(),
                content: r#"- Use `log/slog` for structured logging (standard library)
- Include context in log entries
- Document exported types and functions with comments
- Use `go doc` style comments starting with the name"#
                    .to_string(),
                relevance_keywords: vec!["doc", "slog", "logging", "comment"],
            },
            PromptSection {
                section: Section::Patterns,
                title: "Common Patterns".to_string(),
                content: r#"```go
// Functional options
type Option func(*Config)

func WithTimeout(d time.Duration) Option {
    return func(c *Config) { c.Timeout = d }
}

func New(opts ...Option) *Client {
    cfg := defaultConfig()
    for _, opt := range opts {
        opt(&cfg)
    }
    return &Client{config: cfg}
}
```"#
                    .to_string(),
                relevance_keywords: vec!["pattern", "option", "functional", "example"],
            },
            PromptSection {
                section: Section::Tooling,
                title: "Tools & Linting".to_string(),
                content: r#"- Run `go fmt` and `goimports` before committing
- Use `golangci-lint` with strict configuration
- Enable `go vet` in CI
- Use `govulncheck` for security scanning
- Run `go mod tidy` to clean dependencies"#
                    .to_string(),
                relevance_keywords: vec!["fmt", "lint", "vet", "golangci", "goimports"],
            },
            PromptSection {
                section: Section::Security,
                title: "Security".to_string(),
                content: r#"- Use `context.Context` for cancellation and timeouts on all external calls
- Validate all user input; use `validator` or custom validation
- Use `crypto/rand` not `math/rand` for security-sensitive randomness
- Sanitize SQL with parameterized queries (`sqlx`, `database/sql`)
- Use `govulncheck` for dependency vulnerability scanning
- Prevent goroutine leaks: always ensure goroutines can exit
- Use `net/http` timeouts: `ReadTimeout`, `WriteTimeout`, `IdleTimeout`
- Never log sensitive data (tokens, passwords, PII)"#
                    .to_string(),
                relevance_keywords: vec!["security", "validate", "crypto", "sql", "goroutine"],
            },
            PromptSection {
                section: Section::Dependencies,
                title: "Architecture & Dependencies".to_string(),
                content: r#"- Follow Standard Go Project Layout (`cmd/`, `internal/`, `pkg/`)
- Use dependency injection via constructor functions, not frameworks
- Avoid ORMs if `sqlx` or `database/sql` serves better
- Accept interfaces, return structs — keeps packages decoupled
- Keep `main()` thin: parse config, wire dependencies, start server
- Use `wire` or manual DI for complex dependency graphs"#
                    .to_string(),
                relevance_keywords: vec!["architecture", "dependency", "injection", "layout", "sqlx"],
            },
        ],
    }
}
