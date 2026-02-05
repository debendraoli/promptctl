//! Solidity smart contract prompt template.

use crate::prompt_builder::{PromptSection, Section, StructuredPrompt};

pub const SOLIDITY_PROMPT: &str = r#"# Solidity Development Guidelines (0.8.28)

## Language Version
- Target **Solidity 0.8.28** (latest stable)
- Use `pragma solidity ^0.8.28;` for production contracts
- Enable optimizer with 200+ runs for deployed contracts
- Use `via-ir` pipeline for complex contracts

## Contract Structure

### Layout Convention
```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/// @title Token Contract
/// @author Your Name
/// @notice Brief description
/// @dev Implementation details
contract MyToken is ERC20, Ownable {
    // Type declarations
    struct User { address addr; uint256 balance; }
    enum Status { Active, Paused, Stopped }

    // State variables (storage layout order matters!)
    uint256 public constant MAX_SUPPLY = 1_000_000e18;
    uint256 public immutable deployedAt;
    uint256 private _totalMinted;
    mapping(address => User) private _users;

    // Events
    event Minted(address indexed to, uint256 amount);

    // Errors (gas efficient vs require strings)
    error InsufficientBalance(uint256 available, uint256 required);
    error Unauthorized(address caller);

    // Modifiers
    modifier onlyActive() {
        require(status == Status.Active, "Not active");
        _;
    }

    // Constructor
    constructor() ERC20("MyToken", "MTK") Ownable(msg.sender) {
        deployedAt = block.timestamp;
    }

    // External functions
    // Public functions
    // Internal functions
    // Private functions
}
```

## Type System

### Value Types
```solidity
// Fixed-size types
uint256 amount;      // 256-bit unsigned (use uint256 explicitly)
int128 delta;        // 128-bit signed
address owner;       // 20 bytes
address payable recipient;  // Can receive ETH
bool active;         // true/false
bytes32 hash;        // Fixed-size byte array

// User-defined value types (0.8.8+)
type TokenId is uint256;
type Price is uint128;

function process(TokenId id) external {
    uint256 raw = TokenId.unwrap(id);  // Extract underlying
    TokenId newId = TokenId.wrap(raw + 1);  // Create new
}
```

### Reference Types
```solidity
// Arrays
uint256[] dynamicArray;
uint256[10] fixedArray;

// Mappings (cannot iterate, no length)
mapping(address => uint256) balances;
mapping(address => mapping(address => uint256)) allowances;

// Structs
struct Order {
    address maker;
    uint256 amount;
    uint256 price;
}
```

## Error Handling

### Custom Errors (preferred)
```solidity
// Gas efficient: ~100 gas vs ~2000 for require with string
error InsufficientBalance(uint256 available, uint256 required);
error Unauthorized();
error InvalidInput(string reason);

function withdraw(uint256 amount) external {
    uint256 balance = balances[msg.sender];
    if (balance < amount) {
        revert InsufficientBalance(balance, amount);
    }
    // ...
}
```

### Require/Assert/Revert
```solidity
// require: Input validation and conditions
require(amount > 0, "Amount must be positive");
require(msg.sender == owner, "Not owner");

// assert: Invariants that should never fail
assert(totalSupply == sumOfBalances);

// revert: Complex conditions with custom errors
if (block.timestamp < unlockTime) {
    revert TooEarly(block.timestamp, unlockTime);
}

// try/catch for external calls
try externalContract.call() returns (uint256 result) {
    // Success
} catch Error(string memory reason) {
    // require/revert with string
} catch (bytes memory lowLevelData) {
    // Custom error or panic
}
```

## Security Patterns

### Reentrancy Protection
```solidity
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

contract Vault is ReentrancyGuard {
    mapping(address => uint256) private _balances;

    // Checks-Effects-Interactions pattern
    function withdraw(uint256 amount) external nonReentrant {
        // Checks
        uint256 balance = _balances[msg.sender];
        require(balance >= amount, "Insufficient");

        // Effects (state changes BEFORE external calls)
        _balances[msg.sender] = balance - amount;

        // Interactions (external calls LAST)
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Transfer failed");
    }
}
```

### Access Control
```solidity
import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";

contract MyContract is AccessControl {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");

    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
    }

    function mint(address to, uint256 amount) external onlyRole(MINTER_ROLE) {
        _mint(to, amount);
    }
}
```

### Safe Math & Overflow
```solidity
// 0.8.0+ has built-in overflow checks
uint256 a = type(uint256).max;
uint256 b = a + 1;  // Reverts automatically!

// unchecked for gas optimization (when safe)
function incrementCounter() external {
    unchecked {
        counter++;  // Saves ~100 gas if overflow impossible
    }
}
```

## Gas Optimization

### Storage Patterns
```solidity
// Pack variables (32-byte slots)
// Bad: 3 slots
uint256 a;    // slot 0
uint128 b;    // slot 1
uint128 c;    // slot 2

// Good: 2 slots
uint256 a;    // slot 0
uint128 b;    // slot 1 (first half)
uint128 c;    // slot 1 (second half)

// Use immutable for constructor-set values
uint256 public immutable deployTime;

// Use constant for compile-time constants
uint256 public constant FEE_DENOMINATOR = 10000;

// Transient storage (0.8.24+) - cleared after transaction
contract Lock {
    bytes32 constant LOCK_SLOT = keccak256("lock");

    modifier nonReentrantTransient() {
        assembly {
            if tload(LOCK_SLOT) { revert(0, 0) }
            tstore(LOCK_SLOT, 1)
        }
        _;
        assembly {
            tstore(LOCK_SLOT, 0)
        }
    }
}
```

### Calldata vs Memory
```solidity
// Use calldata for read-only external function params
function process(bytes calldata data) external pure returns (bytes32) {
    return keccak256(data);  // No copy needed
}

// Use memory when modification needed
function modify(uint256[] memory arr) internal pure {
    arr[0] = 100;
}
```

## Events & Logging

```solidity
// Index up to 3 parameters for filtering
event Transfer(
    address indexed from,
    address indexed to,
    uint256 value  // Not indexed, in data
);

// Anonymous events save gas but lose signature
event AnonymousLog(uint256 value) anonymous;

function transfer(address to, uint256 amount) external {
    // ...
    emit Transfer(msg.sender, to, amount);
}
```

## Testing

```solidity
// Foundry test
import {Test, console} from "forge-std/Test.sol";

contract TokenTest is Test {
    Token token;
    address alice = makeAddr("alice");
    address bob = makeAddr("bob");

    function setUp() public {
        token = new Token();
        deal(address(token), alice, 1000e18);
    }

    function test_Transfer() public {
        vm.prank(alice);
        token.transfer(bob, 100e18);
        assertEq(token.balanceOf(bob), 100e18);
    }

    function testFuzz_Transfer(uint256 amount) public {
        amount = bound(amount, 0, token.balanceOf(alice));
        vm.prank(alice);
        token.transfer(bob, amount);
        assertEq(token.balanceOf(bob), amount);
    }

    function testRevert_InsufficientBalance() public {
        vm.prank(alice);
        vm.expectRevert(abi.encodeWithSelector(
            Token.InsufficientBalance.selector, 1000e18, 2000e18
        ));
        token.transfer(bob, 2000e18);
    }
}
```

## CLI & Tooling

```bash
# Foundry (recommended)
forge init my_project
forge build
forge test -vvv
forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast

# Hardhat alternative
npx hardhat compile
npx hardhat test
npx hardhat run scripts/deploy.js --network mainnet
```
"#;

/// Create a structured Solidity prompt with sections
pub fn structured_prompt() -> StructuredPrompt {
    StructuredPrompt {
        language: "solidity".to_string(),
        sections: vec![
            PromptSection {
                section: Section::Version,
                title: "Language Version".to_string(),
                content: r#"- Target **Solidity 0.8.28** (latest stable)
- Use `pragma solidity ^0.8.28;` for production contracts
- Enable optimizer with 200+ runs for deployed contracts
- Use `via-ir` pipeline for complex contracts"#
                    .to_string(),
                relevance_keywords: vec!["solidity", "version", "pragma", "evm"],
            },
            PromptSection {
                section: Section::Structure,
                title: "Contract Structure".to_string(),
                content: r#"Layout order: License → Pragma → Imports → Interfaces → Libraries → Contracts

Within contract: Type declarations → State variables → Events → Errors → Modifiers → Constructor → Functions (external → public → internal → private)

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

contract MyContract {
    uint256 public constant MAX = 100;
    uint256 public immutable deployedAt;
    uint256 private _value;

    event ValueChanged(uint256 newValue);
    error InvalidValue(uint256 value);

    constructor() { deployedAt = block.timestamp; }
}
```"#
                    .to_string(),
                relevance_keywords: vec!["contract", "structure", "layout", "import"],
            },
            PromptSection {
                section: Section::Types,
                title: "Type System".to_string(),
                content: r#"**Value Types:** `uint256`, `int256`, `address`, `address payable`, `bool`, `bytes32`
**User-Defined Value Types:** `type TokenId is uint256;`
**Reference Types:** `mapping`, arrays, structs

```solidity
type Price is uint128;
mapping(address => uint256) balances;
struct Order { address maker; uint256 amount; }
```"#
                    .to_string(),
                relevance_keywords: vec!["type", "mapping", "struct", "uint", "address"],
            },
            PromptSection {
                section: Section::ErrorHandling,
                title: "Error Handling".to_string(),
                content: r#"**Custom errors** (gas efficient, ~100 gas vs ~2000 for require strings):
```solidity
error InsufficientBalance(uint256 available, uint256 required);
error Unauthorized();

function withdraw(uint256 amount) external {
    if (balances[msg.sender] < amount) {
        revert InsufficientBalance(balances[msg.sender], amount);
    }
}
```

Use `require` for input validation, `assert` for invariants, try/catch for external calls."#
                    .to_string(),
                relevance_keywords: vec!["error", "require", "revert", "assert", "catch"],
            },
            PromptSection {
                section: Section::Security,
                title: "Security Patterns".to_string(),
                content: r#"**Reentrancy:** Use ReentrancyGuard or Checks-Effects-Interactions pattern
```solidity
function withdraw(uint256 amount) external nonReentrant {
    uint256 balance = _balances[msg.sender];
    require(balance >= amount);
    _balances[msg.sender] = balance - amount;  // Effect before interaction
    (bool success,) = msg.sender.call{value: amount}("");
    require(success);
}
```

**Access Control:** Use OpenZeppelin's AccessControl or Ownable
**Overflow:** Built-in since 0.8.0; use `unchecked` only when safe"#
                    .to_string(),
                relevance_keywords: vec![
                    "security",
                    "reentrancy",
                    "access",
                    "overflow",
                    "audit",
                ],
            },
            PromptSection {
                section: Section::Memory,
                title: "Gas Optimization".to_string(),
                content: r#"- Pack storage variables (32-byte slots)
- Use `immutable` for constructor-set values, `constant` for compile-time
- Use `calldata` for read-only external params
- Use `unchecked` for safe arithmetic
- Use transient storage (0.8.24+) for reentrancy locks

```solidity
// Bad: 3 slots          // Good: 2 slots
uint256 a;               uint256 a;
uint128 b;               uint128 b;
uint128 c;               uint128 c;  // Same slot as b
```"#
                    .to_string(),
                relevance_keywords: vec![
                    "gas",
                    "optimization",
                    "storage",
                    "memory",
                    "calldata",
                ],
            },
            PromptSection {
                section: Section::Testing,
                title: "Testing".to_string(),
                content: r#"```solidity
// Foundry test
import {Test} from "forge-std/Test.sol";

contract TokenTest is Test {
    Token token;
    address alice = makeAddr("alice");

    function setUp() public {
        token = new Token();
        deal(address(token), alice, 1000e18);
    }

    function test_Transfer() public {
        vm.prank(alice);
        token.transfer(bob, 100e18);
        assertEq(token.balanceOf(bob), 100e18);
    }

    function testFuzz_Transfer(uint256 amount) public {
        amount = bound(amount, 0, 1000e18);
        // ...
    }
}
```"#
                    .to_string(),
                relevance_keywords: vec!["test", "foundry", "forge", "fuzz", "assert"],
            },
            PromptSection {
                section: Section::Tooling,
                title: "Tooling".to_string(),
                content: r#"```bash
# Foundry (recommended)
forge init my_project
forge build
forge test -vvv
forge script script/Deploy.s.sol --rpc-url $RPC --broadcast

# Hardhat
npx hardhat compile
npx hardhat test
```"#
                    .to_string(),
                relevance_keywords: vec!["foundry", "hardhat", "forge", "deploy", "compile"],
            },
            PromptSection {
                section: Section::Patterns,
                title: "Common Patterns".to_string(),
                content: r#"```solidity
// ERC20 Token
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
contract MyToken is ERC20 {
    constructor() ERC20("Name", "SYM") { _mint(msg.sender, 1000000e18); }
}

// Proxy/Upgradeable
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

// Factory pattern
function createPair(address t0, address t1) external returns (address pair) {
    pair = address(new Pair(t0, t1));
}
```"#
                    .to_string(),
                relevance_keywords: vec!["pattern", "erc20", "proxy", "factory", "upgradeable"],
            },
        ],
    }
}
