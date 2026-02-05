//! Leo programming language prompt template for Aleo smart contracts.

use crate::prompt_builder::{PromptSection, Section, StructuredPrompt};

pub const LEO_PROMPT: &str = r#"# Leo Development Guidelines (Aleo Smart Contracts)

## Language Version
- Target **Leo 3.4.0** (latest stable)
- Use `leo new` for project scaffolding
- Leverage snarkVM's proof system for privacy-preserving computation
- Install: `cargo install leo-lang`

## Core Concepts

### Program Structure
```leo
program token.aleo {
    // Records - private state owned by users
    record Token {
        owner: address,
        amount: u64,
    }

    // Mappings - public on-chain state
    mapping balances: address => u64;

    // Async transitions - return Future for on-chain state changes
    async transition mint(amount: u64) -> (Token, Future) {
        let token: Token = Token {
            owner: self.caller,
            amount: amount,
        };
        let f: Future = finalize_mint(self.caller, amount);
        return (token, f);
    }

    // Async function - executes on-chain after proof verification
    async function finalize_mint(owner: address, amount: u64) {
        let current: u64 = Mapping::get_or_use(balances, owner, 0u64);
        Mapping::set(balances, owner, current + amount);
    }
}
```

### Type System

#### Primitive Types
- Integers: `u8`, `u16`, `u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`
- Field elements: `field`, `scalar`, `group`
- Boolean: `bool`
- Address: `address`
- Signature: `signature`
- Future: `Future` (async operation handle)

#### Composite Types
- Arrays: `[u64; 4]` - fixed-size, supports repeater syntax `[0u8; 32]`
- Empty arrays: `[u8; 0]` - supported for generic programming
- Tuples: `(u64, bool, address)`
- Structs: named product types
- Records: private state with `owner` field

```leo
struct Point {
    x: u64,
    y: u64,
}

record NFT {
    owner: address,
    token_id: field,
    metadata: [u8; 32],
}

// Array repeater syntax (v3.0+)
let zeros: [u8; 32] = [0u8; 32];
```

### Async & Futures (Leo 3.x)

Leo uses `async transition` and `async function` with `Future` for on-chain state changes.

#### Async Transition
Returns a `Future` that must be awaited in an async function:

```leo
program counter.aleo {
    mapping counts: address => u64;

    // Async transition returns (outputs, Future)
    async transition increment() -> Future {
        return finalize_increment(self.caller);
    }

    // Async function handles on-chain state
    async function finalize_increment(user: address) {
        let count: u64 = Mapping::get_or_use(counts, user, 0u64);
        Mapping::set(counts, user, count + 1u64);
    }
}
```

#### Awaiting Futures
```leo
import other_program.aleo;

program orchestrator.aleo {
    async transition do_both(a: u64, b: u64) -> Future {
        // Call external async transitions
        let (result1, f1): (u64, Future) = other_program.aleo/action_a(a);
        let (result2, f2): (u64, Future) = other_program.aleo/action_b(b);
        
        // Return future that awaits both
        return finalize_both(f1, f2);
    }

    async function finalize_both(f1: Future, f2: Future) {
        // Await futures - order matters for execution
        f1.await();
        f2.await();
    }
}
```

#### Future Rules
- `async transition` must return exactly one `Future` as the last output
- `async function` cannot call other async functions directly
- All futures must be statically awaited (no conditional awaiting)
- Access future input args via tuple syntax: `f.0`, `f.1`
- Use `f.await()` or `Future::await(f)` syntax

### Visibility & Privacy

#### Record Privacy
- Records are **private by default** - only owner can read
- Use records for sensitive user data (balances, credentials)
- Records are consumed and produced (UTXO model)

#### Mapping Publicity
- Mappings are **public on-chain state**
- Readable by anyone
- Only modifiable in `async function` blocks

#### Transition Visibility
```leo
// Private inputs (default)
transition transfer(amount: u64) -> Token { }

// Public inputs - visible on-chain
transition transfer(public amount: u64) -> Token { }
```

### Functions vs Transitions

#### Transitions
- Entry points callable externally
- Generate zero-knowledge proofs
- Can produce/consume records
- Use `async transition` for on-chain state changes

```leo
transition send(
    sender: Token,
    receiver: address,
    amount: u64
) -> (Token, Token) {
    assert(sender.amount >= amount);
    
    let remaining: Token = Token {
        owner: sender.owner,
        amount: sender.amount - amount,
    };
    
    let sent: Token = Token {
        owner: receiver,
        amount: amount,
    };
    
    return (remaining, sent);
}
```

#### Helper Functions
- Internal computation only
- Cannot be called externally
- No proof generation
- Use `inline` for gas optimization

```leo
function calculate_fee(amount: u64) -> u64 {
    return amount / 100u64; // 1% fee
}

inline function min(a: u64, b: u64) -> u64 {
    return a < b ? a : b;
}
```

### Control Flow

```leo
// Conditionals (must be deterministic)
let result: u64 = condition ? value_if_true : value_if_false;

// Bounded loops (compile-time known bounds)
for i: u8 in 0u8..10u8 {
    // Loop body
}

// Empty loops supported (v3.4+) - useful for generic code
for i: u32 in 0u32..N {  // When N=0, loop is skipped
    xs[i] = 1u8;
}

// Assertions (circuit constraints)
assert(amount > 0u64);
assert_eq(a, b);
assert_neq(a, b);
```

### Mappings & Async Functions

```leo
program staking.aleo {
    mapping stakes: address => u64;
    mapping total_staked: u8 => u64;  // Use 0u8 as singleton key

    async transition stake(amount: u64) -> Future {
        return finalize_stake(self.caller, amount);
    }

    async function finalize_stake(staker: address, amount: u64) {
        // Mapping operations (public state)
        let current: u64 = Mapping::get_or_use(stakes, staker, 0u64);
        Mapping::set(stakes, staker, current + amount);
        
        let total: u64 = Mapping::get_or_use(total_staked, 0u8, 0u64);
        Mapping::set(total_staked, 0u8, total + amount);
    }
}
```

### Cryptographic Primitives

```leo
// Hashing
let hash: field = BHP256::hash_to_field(data);
let hash: field = Poseidon2::hash_to_field(data);
let hash: field = Keccak256::hash_to_field(data);  // EVM compatible

// Hash to bits (v3.4+ renamed from hash_native)
let bits: [bool; 256] = BHP256::hash_to_bits(data);
let bits: [bool; 256] = Poseidon2::hash_to_bits(data);

// Commitment schemes
let commit: field = BHP256::commit_to_field(data, randomness);
let commit: group = Pedersen64::commit_to_group(data, randomness);

// Signature verification
let is_valid: bool = signature::verify(sig, addr, message);

// Random number generation (in async function only)
let rand: field = ChaCha::rand_field();
```

### Cross-Program Calls

```leo
import credits.aleo;
import other_program.aleo;

program my_program.aleo {
    // Sync cross-program call
    transition purchase(
        payment: credits.aleo/credits,
        price: u64
    ) -> credits.aleo/credits {
        let change: credits.aleo/credits = credits.aleo/transfer_private(
            payment, 
            aleo1merchant..., 
            price
        );
        return change;
    }

    // Async cross-program call with Future chaining
    async transition purchase_and_track(
        payment: credits.aleo/credits,
        price: u64
    ) -> (credits.aleo/credits, Future) {
        let (change, pay_future): (credits.aleo/credits, Future) = 
            credits.aleo/transfer_private_with_fee(payment, aleo1merchant..., price);
        
        return (change, finalize_purchase(pay_future, self.caller, price));
    }

    async function finalize_purchase(pay_future: Future, buyer: address, amount: u64) {
        pay_future.await();
        // Additional on-chain tracking...
    }
}
```

### Error Handling

```leo
// Assertions are the primary error mechanism
transition withdraw(token: Token, amount: u64) -> Token {
    // Will fail proof generation if false
    assert(token.amount >= amount);
    assert(self.caller == token.owner);
    
    return Token {
        owner: token.owner,
        amount: token.amount - amount,
    };
}

// Use descriptive variable names for implicit error context
let sufficient_balance: bool = token.amount >= amount;
assert(sufficient_balance);
```

### Testing

```leo
@test
transition test_mint() {
    let token: Token = mint(100u64);
    assert_eq(token.amount, 100u64);
}

@test 
transition test_transfer() {
    let sender_token: Token = Token {
        owner: aleo1sender...,
        amount: 100u64,
    };
    
    let (remaining, sent): (Token, Token) = transfer(
        sender_token, 
        aleo1receiver..., 
        30u64
    );
    
    assert_eq(remaining.amount, 70u64);
    assert_eq(sent.amount, 30u64);
}
```

### Project Structure
```
my_program/
├── program.json       # Program manifest
├── src/
│   └── main.leo      # Main program file
├── build/            # Compiled artifacts
│   ├── main.aleo     # Aleo instructions
│   └── program.json
└── outputs/          # Execution outputs
```

Note: Input files (`.in`) are no longer used as of Leo 3.0. Pass arguments directly to CLI.

### Best Practices

#### Security
- Always validate record ownership: `assert(self.caller == record.owner)`
- Use records for sensitive balances, mappings for public aggregates
- Validate all arithmetic to prevent overflow
- Use `field` for IDs/hashes requiring collision resistance

#### Gas Optimization
- Use `inline` functions for repeated small computations
- Minimize async function complexity
- Prefer records over mappings when privacy allows
- Batch operations to reduce proof overhead

#### Privacy Patterns
```leo
// Private transfer - use records
transition private_transfer(token: Token, to: address, amount: u64) -> (Token, Token) {
    // Only proves valid state transition, reveals nothing
    assert(token.amount >= amount);
    return (
        Token { owner: token.owner, amount: token.amount - amount },
        Token { owner: to, amount: amount }
    );
}

// Shielding - convert public to private
async transition shield(public amount: u64) -> (Token, Future) {
    let token: Token = Token { owner: self.caller, amount };
    return (token, finalize_shield(self.caller, amount));
}

async function finalize_shield(owner: address, amount: u64) {
    let balance: u64 = Mapping::get(balances, owner);
    Mapping::set(balances, owner, balance - amount);
}

// Unshielding - convert private to public  
async transition unshield(token: Token, public amount: u64) -> Future {
    assert(token.amount >= amount);
    return finalize_unshield(self.caller, amount);
}

async function finalize_unshield(owner: address, amount: u64) {
    let balance: u64 = Mapping::get_or_use(balances, owner, 0u64);
    Mapping::set(balances, owner, balance + amount);
}
```

### CLI Commands
```bash
# Create new project
leo new my_program

# Build program
leo build

# Run locally (no proof)
leo run transition_name arg1 arg2

# Execute with proof generation
leo execute transition_name arg1 arg2

# Deploy to network
leo deploy --network mainnet

# Test
leo test

# Update Leo
leo update
```

### Common Patterns

#### Token Standard
```leo
program token.aleo {
    record Token {
        owner: address,
        amount: u64,
    }
    
    mapping total_supply: u8 => u64;
    
    async transition mint(amount: u64) -> (Token, Future) {
        let token: Token = Token { owner: self.caller, amount };
        return (token, finalize_mint(amount));
    }
    
    async function finalize_mint(amount: u64) {
        let supply: u64 = Mapping::get_or_use(total_supply, 0u8, 0u64);
        Mapping::set(total_supply, 0u8, supply + amount);
    }
    
    transition transfer(
        sender: Token, 
        receiver: address, 
        amount: u64
    ) -> (Token, Token) {
        assert(sender.amount >= amount);
        
        let remaining: Token = Token {
            owner: sender.owner,
            amount: sender.amount - amount,
        };
        
        let sent: Token = Token {
            owner: receiver,
            amount: amount,
        };
        
        return (remaining, sent);
    }
}
```

#### Access Control
```leo
program ownable.aleo {
    mapping owner: u8 => address;
    
    async transition initialize() -> Future {
        return finalize_initialize(self.caller);
    }
    
    async function finalize_initialize(caller: address) {
        let exists: bool = Mapping::contains(owner, 0u8);
        assert(!exists);  // Can only initialize once
        Mapping::set(owner, 0u8, caller);
    }
    
    async transition admin_action() -> Future {
        return finalize_admin_action(self.caller);
    }
    
    async function finalize_admin_action(caller: address) {
        let current_owner: address = Mapping::get(owner, 0u8);
        assert_eq(caller, current_owner);
        // Perform admin action
    }
}
```

#### Generic Array Builder (v3.4+)
```leo
// Empty arrays and loops enable generic programming
inline build_default::[N: u32]() -> [u8; N] {
    let xs: [u8; N] = [0u8; N];
    for i: u32 in 0u32..N {
        xs[i] = 1u8;
    }
    return xs;
}

let empty: [u8; 0] = build_default::[0]();  // []
let three: [u8; 3] = build_default::[3]();  // [1, 1, 1]
```
"#;

/// Create a structured Leo prompt with sections
pub fn structured_prompt() -> StructuredPrompt {
    StructuredPrompt {
        language: "leo".to_string(),
        sections: vec![
            PromptSection {
                section: Section::Version,
                title: "Language Version".to_string(),
                content: r#"- Target **Leo 3.4.0** (latest stable)
- Use `leo new` for project scaffolding
- Leverage snarkVM's proof system for privacy-preserving computation
- Install: `cargo install leo-lang`
- Breaking: `hash_native` renamed to `hash_to_bits`"#
                    .to_string(),
                relevance_keywords: vec!["leo", "aleo", "snarkvm", "version"],
            },
            PromptSection {
                section: Section::Style,
                title: "Program Structure & Style".to_string(),
                content: r#"```leo
program token.aleo {
    // Records - private state owned by users
    record Token {
        owner: address,
        amount: u64,
    }

    // Mappings - public on-chain state
    mapping balances: address => u64;

    // Async transition - returns Future for on-chain state
    async transition mint(amount: u64) -> (Token, Future) {
        let token: Token = Token { owner: self.caller, amount };
        return (token, finalize_mint(self.caller, amount));
    }

    // Async function - on-chain state changes
    async function finalize_mint(owner: address, amount: u64) {
        Mapping::set(balances, owner, amount);
    }
}
```"#
                    .to_string(),
                relevance_keywords: vec!["program", "structure", "record", "mapping", "transition", "async"],
            },
            PromptSection {
                section: Section::Types,
                title: "Type System".to_string(),
                content: r#"**Primitives:** `u8`-`u128`, `i8`-`i128`, `field`, `scalar`, `group`, `bool`, `address`, `signature`, `Future`

**Composites:**
- Arrays: `[u64; 4]` (fixed-size), `[0u8; 32]` (repeater syntax)
- Empty arrays: `[u8; 0]` (v3.4+ for generics)
- Tuples: `(u64, bool, address)`
- Structs: named product types
- Records: private state with mandatory `owner: address`

```leo
struct Point { x: u64, y: u64 }

record NFT {
    owner: address,
    token_id: field,
    metadata: [u8; 32],
}

let zeros: [u8; 32] = [0u8; 32];  // Array repeater
```"#
                    .to_string(),
                relevance_keywords: vec!["type", "struct", "record", "field", "address", "future", "array"],
            },
            PromptSection {
                section: Section::ErrorHandling,
                title: "Error Handling & Assertions".to_string(),
                content: r#"- Assertions are the primary error mechanism (fail proof generation)
- Use descriptive variable names for implicit error context
- All constraints must be satisfiable for valid proofs

```leo
transition withdraw(token: Token, amount: u64) -> Token {
    // Circuit constraints - fails if false
    assert(token.amount >= amount);
    assert(self.caller == token.owner);
    assert_eq(a, b);
    assert_neq(a, b);
    
    return Token {
        owner: token.owner,
        amount: token.amount - amount,
    };
}
```"#
                    .to_string(),
                relevance_keywords: vec!["assert", "error", "constraint", "proof"],
            },
            PromptSection {
                section: Section::Memory,
                title: "Privacy & Visibility".to_string(),
                content: r#"**Records:** Private by default, UTXO model (consumed/produced)
**Mappings:** Public on-chain state, readable by anyone

```leo
// Private inputs (default)
transition transfer(amount: u64) -> Token { }

// Public inputs - visible on-chain
transition transfer(public amount: u64) -> Token { }

// Shielding (public → private) with async
async transition shield(public amount: u64) -> (Token, Future) {
    let token: Token = Token { owner: self.caller, amount };
    return (token, finalize_shield(self.caller, amount));
}

// Unshielding (private → public) with async
async transition unshield(token: Token, public amount: u64) -> Future {
    assert(token.amount >= amount);
    return finalize_unshield(self.caller, amount);
}
```"#
                    .to_string(),
                relevance_keywords: vec!["privacy", "record", "mapping", "public", "private", "shield"],
            },
            PromptSection {
                section: Section::Concurrency,
                title: "Mappings & Async Functions".to_string(),
                content: r#"```leo
mapping stakes: address => u64;
mapping total_staked: u8 => u64;  // Singleton pattern

async transition stake(amount: u64) -> Future {
    return finalize_stake(self.caller, amount);
}

async function finalize_stake(staker: address, amount: u64) {
    // Mapping operations (public state)
    let current: u64 = Mapping::get_or_use(stakes, staker, 0u64);
    Mapping::set(stakes, staker, current + amount);
    
    let exists: bool = Mapping::contains(stakes, staker);
    Mapping::remove(stakes, staker);  // Delete entry
}
```"#
                    .to_string(),
                relevance_keywords: vec!["mapping", "async", "function", "state", "get", "set"],
            },
            PromptSection {
                section: Section::Async,
                title: "Async & Futures".to_string(),
                content: r#"Leo 3.x uses `async transition` + `async function` with `Future` for on-chain state:

```leo
import other.aleo;

program orchestrator.aleo {
    async transition do_both(a: u64, b: u64) -> Future {
        // Call external async transitions
        let (r1, f1): (u64, Future) = other.aleo/action_a(a);
        let (r2, f2): (u64, Future) = other.aleo/action_b(b);
        
        return finalize_both(f1, f2);
    }

    async function finalize_both(f1: Future, f2: Future) {
        f1.await();  // Await futures
        f2.await();
    }
}
```

**Rules:**
- `async transition` returns exactly one `Future` (last output)
- All futures must be statically awaited
- Access future args: `f.0`, `f.1`"#
                    .to_string(),
                relevance_keywords: vec!["async", "await", "future", "transition", "function"],
            },
            PromptSection {
                section: Section::Testing,
                title: "Testing".to_string(),
                content: r#"```leo
@test
transition test_mint() {
    let token: Token = mint(100u64);
    assert_eq(token.amount, 100u64);
}

@test 
transition test_transfer() {
    let sender: Token = Token { owner: aleo1..., amount: 100u64 };
    let (remaining, sent) = transfer(sender, aleo1receiver..., 30u64);
    assert_eq(remaining.amount, 70u64);
    assert_eq(sent.amount, 30u64);
}
```

CLI: `leo test`, `leo run transition_name args...`, `leo execute transition_name args...`"#
                    .to_string(),
                relevance_keywords: vec!["test", "assert", "run", "execute"],
            },
            PromptSection {
                section: Section::Structure,
                title: "Project Structure".to_string(),
                content: r#"```
my_program/
├── program.json       # Program manifest
├── src/
│   └── main.leo      # Main program file
├── build/            # Compiled artifacts
│   ├── main.aleo     # Aleo instructions
│   └── program.json
└── outputs/          # Execution outputs
```

Note: Input files (.in) removed in v3.0. Pass args directly to CLI.

CLI: `leo new project`, `leo build`, `leo deploy --network mainnet`"#
                    .to_string(),
                relevance_keywords: vec!["project", "structure", "build", "deploy"],
            },
            PromptSection {
                section: Section::Patterns,
                title: "Common Patterns".to_string(),
                content: r#"**Token Standard:**
```leo
record Token { owner: address, amount: u64 }
async transition mint(amount: u64) -> (Token, Future) {
    return (Token { owner: self.caller, amount }, finalize_mint(amount));
}
async function finalize_mint(amount: u64) {
    let supply: u64 = Mapping::get_or_use(total_supply, 0u8, 0u64);
    Mapping::set(total_supply, 0u8, supply + amount);
}
```

**Access Control:**
```leo
mapping owner: u8 => address;
async function finalize_admin_action(caller: address) {
    let current_owner: address = Mapping::get(owner, 0u8);
    assert_eq(caller, current_owner);
}
```

**Cryptographic Operations:**
```leo
let hash: field = BHP256::hash_to_field(data);
let bits: [bool; 256] = BHP256::hash_to_bits(data);  // v3.4+
let commit: field = BHP256::commit_to_field(data, randomness);
let valid: bool = signature::verify(sig, addr, message);
```"#
                    .to_string(),
                relevance_keywords: vec!["pattern", "token", "access", "hash", "signature"],
            },
            PromptSection {
                section: Section::Tooling,
                title: "CLI & Tooling".to_string(),
                content: r#"```bash
leo new my_program      # Create project
leo build               # Compile to Aleo instructions
leo run transition args # Run locally (no proof)
leo execute transition args # Execute with proof generation
leo test                # Run tests
leo deploy --network mainnet # Deploy to network
leo update              # Update Leo
```

**Best Practices:**
- Use `inline` functions for gas optimization
- Validate ownership: `assert(self.caller == record.owner)`
- Use `field` for IDs requiring collision resistance
- Prefer records for privacy, mappings for public aggregates
- Empty arrays/loops (v3.4+) enable generic programming"#
                    .to_string(),
                relevance_keywords: vec!["cli", "build", "deploy", "test", "leo"],
            },
        ],
    }
}
