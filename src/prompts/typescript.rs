//! TypeScript programming prompt template.

use crate::prompt_builder::{PromptSection, Section, StructuredPrompt};

pub const TYPESCRIPT_PROMPT: &str = r#"# TypeScript Development Guidelines (5.9)

## Language Version
- Target **TypeScript 5.9** with strict mode enabled
- Use **ES2024** target for modern JavaScript features
- Enable `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`
- Use `moduleResolution: "bundler"` or `"node16"` for modern resolution

## Type System Mastery

### Core Principles
- Prefer explicit types for public APIs, infer for implementation details
- Use `unknown` over `any`; narrow with type guards
- Leverage union types and discriminated unions for state modeling
- Use `as const` for literal inference and readonly tuples

### Advanced Types
```typescript
// Discriminated unions for state machines
type State =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "success"; data: Data }
  | { status: "error"; error: Error };

// Template literal types
type EventName = `on${Capitalize<string>}`;

// Conditional types
type Awaited<T> = T extends Promise<infer U> ? Awaited<U> : T;

// Mapped types with key remapping
type Getters<T> = {
  [K in keyof T as `get${Capitalize<string & K>}`]: () => T[K];
};

// Const assertions
const routes = ["home", "about", "contact"] as const;
type Route = typeof routes[number]; // "home" | "about" | "contact"
```

### Type Guards & Narrowing
```typescript
// Custom type guards
function isUser(value: unknown): value is User {
  return typeof value === "object" && value !== null && "id" in value;
}

// Assertion functions
function assertNonNull<T>(value: T): asserts value is NonNullable<T> {
  if (value == null) throw new Error("Value is null or undefined");
}

// Using `satisfies` for type checking without widening
const config = {
  port: 3000,
  host: "localhost",
} satisfies Config;
```

## Error Handling

### Modern Patterns
```typescript
// Result type pattern
type Result<T, E = Error> = { ok: true; value: T } | { ok: false; error: E };

function parseJson<T>(json: string): Result<T> {
  try {
    return { ok: true, value: JSON.parse(json) };
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e : new Error(String(e)) };
  }
}

// Custom error classes
class AppError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly cause?: Error
  ) {
    super(message);
    this.name = "AppError";
  }
}

// Never throw raw strings
// ❌ throw "Something went wrong"
// ✅ throw new Error("Something went wrong")
```

## Async Patterns

### Modern Async/Await
```typescript
// Proper async error handling
async function fetchData<T>(url: string): Promise<T> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new AppError(`HTTP ${response.status}`, "FETCH_ERROR");
  }
  return response.json();
}

// Concurrent operations
const [users, posts] = await Promise.all([fetchUsers(), fetchPosts()]);

// Race with timeout
async function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  const timeout = new Promise<never>((_, reject) =>
    setTimeout(() => reject(new Error("Timeout")), ms)
  );
  return Promise.race([promise, timeout]);
}

// AsyncIterator for streaming
async function* paginate<T>(fetcher: (page: number) => Promise<T[]>) {
  let page = 0;
  while (true) {
    const items = await fetcher(page++);
    if (items.length === 0) break;
    yield* items;
  }
}
```

## Project Structure

### Recommended Layout
```
src/
├── index.ts           # Public API exports
├── types/             # Shared type definitions
│   └── index.ts
├── utils/             # Pure utility functions
├── services/          # Business logic
├── repositories/      # Data access
└── __tests__/         # Test files (or colocated .test.ts)
```

### Module Organization
```typescript
// Barrel exports (src/utils/index.ts)
export { formatDate, parseDate } from "./date";
export { slugify, truncate } from "./string";

// Re-export types
export type { Config, Options } from "./types";

// Named exports preferred over default
export function createClient(options: Options): Client { }
```

## Testing

### Modern Testing Practices
```typescript
import { describe, it, expect, vi, beforeEach } from "vitest";

describe("UserService", () => {
  let service: UserService;
  let mockRepo: MockProxy<UserRepository>;

  beforeEach(() => {
    mockRepo = mock<UserRepository>();
    service = new UserService(mockRepo);
  });

  it("should create user with hashed password", async () => {
    mockRepo.save.mockResolvedValue({ id: "1", email: "test@example.com" });

    const user = await service.createUser({ email: "test@example.com", password: "secret" });

    expect(user.id).toBe("1");
    expect(mockRepo.save).toHaveBeenCalledWith(
      expect.objectContaining({ email: "test@example.com" })
    );
  });
});

// Type-safe mocks
const mockFn = vi.fn<[string, number], boolean>();
```

## Performance & Best Practices

### Immutability
```typescript
// Use readonly for immutable data
interface Config {
  readonly apiUrl: string;
  readonly timeout: number;
}

// Readonly arrays and tuples
function process(items: readonly string[]): void { }

// Object.freeze with type narrowing
const frozen = Object.freeze({ a: 1, b: 2 }) as Readonly<typeof frozen>;
```

### Memory & Performance
- Use `WeakMap`/`WeakSet` for object metadata to prevent memory leaks
- Prefer `for...of` or array methods over traditional for loops
- Use `Set` for unique collections, `Map` for key-value pairs
- Leverage lazy evaluation with generators for large datasets

### Tree Shaking
```typescript
// ✅ Named exports enable tree shaking
export { formatDate } from "./date";

// ❌ Avoid namespace re-exports
export * from "./utils"; // Prevents tree shaking
```

## Tooling & Configuration

### tsconfig.json Essentials
```json
{
  "compilerOptions": {
    "target": "ES2024",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "verbatimModuleSyntax": true,
    "isolatedModules": true,
    "skipLibCheck": true
  }
}
```

### ESLint + Prettier
```bash
# Essential packages
npm i -D eslint @typescript-eslint/parser @typescript-eslint/eslint-plugin
npm i -D prettier eslint-config-prettier
```

### Runtime Validation
```typescript
// Use zod for runtime type validation
import { z } from "zod";

const UserSchema = z.object({
  id: z.string().uuid(),
  email: z.string().email(),
  age: z.number().int().positive().optional(),
});

type User = z.infer<typeof UserSchema>;

// Validate at boundaries
const user = UserSchema.parse(await response.json());
```
"#;

/// Create a structured TypeScript prompt with sections
pub fn structured_prompt() -> StructuredPrompt {
    StructuredPrompt {
        language: "typescript".to_string(),
        sections: vec![
            PromptSection {
                section: Section::Version,
                title: "Language Version".to_string(),
                content: r#"- Target **TypeScript 5.9** with strict mode enabled
- Use **ES2024** target for modern JavaScript features
- Enable `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`
- Use `moduleResolution: "bundler"` or `"node16"` for modern resolution"#
                    .to_string(),
                relevance_keywords: vec!["typescript", "version", "tsconfig", "strict"],
            },
            PromptSection {
                section: Section::Types,
                title: "Type System Mastery".to_string(),
                content: r#"- Prefer explicit types for public APIs, infer for implementation details
- Use `unknown` over `any`; narrow with type guards
- Leverage union types and discriminated unions for state modeling
- Use `as const` for literal inference and readonly tuples

```typescript
// Discriminated unions for state machines
type State =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "success"; data: Data }
  | { status: "error"; error: Error };

// Template literal types
type EventName = `on${Capitalize<string>}`;

// Type guards
function isUser(value: unknown): value is User {
  return typeof value === "object" && value !== null && "id" in value;
}

// Using `satisfies` for type checking without widening
const config = { port: 3000 } satisfies Partial<Config>;
```"#
                    .to_string(),
                relevance_keywords: vec![
                    "type",
                    "interface",
                    "union",
                    "generic",
                    "guard",
                    "narrowing",
                ],
            },
            PromptSection {
                section: Section::ErrorHandling,
                title: "Error Handling".to_string(),
                content: r#"- Use Result type pattern for recoverable errors
- Create custom Error classes with error codes
- Never throw raw strings; always use Error objects
- Handle Promise rejections explicitly

```typescript
type Result<T, E = Error> = { ok: true; value: T } | { ok: false; error: E };

class AppError extends Error {
  constructor(message: string, public readonly code: string, public readonly cause?: Error) {
    super(message);
    this.name = "AppError";
  }
}
```"#
                    .to_string(),
                relevance_keywords: vec!["error", "exception", "throw", "catch", "result"],
            },
            PromptSection {
                section: Section::Async,
                title: "Async Patterns".to_string(),
                content: r#"- Use async/await over raw Promises
- Handle errors with try/catch, not .catch()
- Use Promise.all() for concurrent operations
- Implement timeouts for network requests

```typescript
// Concurrent execution
const [users, posts] = await Promise.all([fetchUsers(), fetchPosts()]);

// Timeout wrapper
async function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  const timeout = new Promise<never>((_, reject) =>
    setTimeout(() => reject(new Error("Timeout")), ms)
  );
  return Promise.race([promise, timeout]);
}

// AsyncIterator for streaming
async function* paginate<T>(fetcher: (page: number) => Promise<T[]>) {
  let page = 0;
  while (true) {
    const items = await fetcher(page++);
    if (items.length === 0) break;
    yield* items;
  }
}
```"#
                    .to_string(),
                relevance_keywords: vec!["async", "await", "promise", "concurrent", "fetch"],
            },
            PromptSection {
                section: Section::Testing,
                title: "Testing".to_string(),
                content: r#"- Use Vitest or Jest with TypeScript support
- Type-safe mocks with `vi.fn<[Args], Return>()`
- Test behavior, not implementation details
- Use fixtures and factories for test data

```typescript
import { describe, it, expect, vi } from "vitest";

describe("UserService", () => {
  it("should create user", async () => {
    const mockRepo = { save: vi.fn().mockResolvedValue({ id: "1" }) };
    const service = new UserService(mockRepo);

    const user = await service.createUser({ email: "test@example.com" });

    expect(user.id).toBe("1");
    expect(mockRepo.save).toHaveBeenCalled();
  });
});
```"#
                    .to_string(),
                relevance_keywords: vec!["test", "vitest", "jest", "mock", "expect"],
            },
            PromptSection {
                section: Section::Structure,
                title: "Project Structure".to_string(),
                content: r#"```
src/
├── index.ts           # Public API exports
├── types/             # Shared type definitions
├── utils/             # Pure utility functions
├── services/          # Business logic
├── repositories/      # Data access
└── __tests__/         # Test files
```

- Use barrel exports (index.ts) for clean imports
- Prefer named exports over default exports
- Re-export types explicitly: `export type { Config }`"#
                    .to_string(),
                relevance_keywords: vec!["structure", "module", "import", "export", "barrel"],
            },
            PromptSection {
                section: Section::Memory,
                title: "Performance".to_string(),
                content: r#"- Use `readonly` for immutable data structures
- Use `WeakMap`/`WeakSet` for object metadata
- Prefer `for...of` over traditional for loops
- Enable tree shaking with named exports
- Use generators for lazy evaluation of large datasets"#
                    .to_string(),
                relevance_keywords: vec![
                    "performance",
                    "memory",
                    "readonly",
                    "immutable",
                    "optimization",
                ],
            },
            PromptSection {
                section: Section::Tooling,
                title: "Tooling & Configuration".to_string(),
                content: r#"**tsconfig.json:**
```json
{
  "compilerOptions": {
    "target": "ES2024",
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "verbatimModuleSyntax": true
  }
}
```

**Runtime Validation (zod):**
```typescript
const UserSchema = z.object({
  id: z.string().uuid(),
  email: z.string().email(),
});
type User = z.infer<typeof UserSchema>;
```"#
                    .to_string(),
                relevance_keywords: vec!["tsconfig", "eslint", "prettier", "zod", "validation"],
            },
            PromptSection {
                section: Section::Security,
                title: "Security".to_string(),
                content: r#"- Always use `zod` (or similar) for schema validation at API boundaries
- Never use `any`; use `unknown` with narrowing if necessary
- Sanitize all dangerous HTML (use DOMPurify or equivalent)
- Validate all API inputs on the server side, never trust client
- Use `helmet` for HTTP security headers in Node.js
- Escape user input in templates to prevent XSS
- Use `crypto.randomUUID()` for IDs, never `Math.random()`
- Implement CSRF protection on state-changing endpoints
- Use Content Security Policy headers"#
                    .to_string(),
                relevance_keywords: vec!["security", "xss", "csrf", "validation", "sanitize", "helmet"],
            },
            PromptSection {
                section: Section::Patterns,
                title: "Common Patterns".to_string(),
                content: r#"```typescript
// Builder pattern
class QueryBuilder<T> {
  private filters: Filter[] = [];
  where(filter: Filter): this { this.filters.push(filter); return this; }
  build(): Query<T> { /* ... */ }
}

// Factory functions
function createUser(data: Partial<User>): User {
  return { id: crypto.randomUUID(), createdAt: new Date(), ...data };
}

// Branded types for type safety
type UserId = string & { readonly __brand: unique symbol };
function toUserId(id: string): UserId { return id as UserId; }
```"#
                    .to_string(),
                relevance_keywords: vec!["pattern", "builder", "factory", "brand"],
            },
            PromptSection {
                section: Section::Dependencies,
                title: "Framework Awareness".to_string(),
                content: r#"**Next.js:** Use Server Components (RSC) by default. Use `'use client'` only for interactivity (event handlers, hooks, browser APIs). Prefer Server Actions for mutations.

**React:** Prefer Composition over Context for state where possible. Use `tanstack-query` for async server state. Minimize `useEffect` — derive state instead.

**Node.js:** Use `node:` protocol for builtins (`import fs from "node:fs"`). Prefer `fetch` over `axios`. Use `dotenv` for config, never hardcode secrets.

**Testing:** Prefer `vitest` and `@testing-library` patterns (user-centric testing). Use `msw` for API mocking."#
                    .to_string(),
                relevance_keywords: vec!["next", "react", "node", "framework", "server", "component"],
            },
        ],
    }
}
