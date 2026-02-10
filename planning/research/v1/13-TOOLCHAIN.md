# ARIA Toolchain Design

## Design Philosophy

1. **Unified Experience**: Single `aria` command for all operations
2. **Fast by Default**: Prioritize compilation speed
3. **Helpful Errors**: AI-friendly error messages
4. **Zero Configuration**: Sensible defaults, optional customization
5. **Cross-Platform**: Consistent behavior across platforms

---

## 1. The `aria` Command

### 1.1 Command Structure

```bash
aria <command> [options] [arguments]

Commands:
  new         Create a new project
  build       Compile the current project
  run         Build and run the current project
  test        Run tests
  check       Check code without building
  fmt         Format source code
  lint        Run linter
  doc         Generate documentation
  pkg         Package management commands
  repl        Start interactive REPL
  lsp         Start language server
```

### 1.2 Project Creation

```bash
# Create binary project
aria new my_project
aria new my_project --bin

# Create library project
aria new my_lib --lib

# Create workspace
aria new my_workspace --workspace

# From template
aria new my_api --template web-api
aria new my_cli --template cli

# Project structure
my_project/
├── aria.toml
├── src/
│   └── main.aria
├── tests/
└── .gitignore
```

### 1.3 Build Commands

```bash
# Debug build (fast compilation)
aria build

# Release build (optimized)
aria build --release

# Specific target
aria build --target x86_64-unknown-linux-gnu
aria build --target wasm32-unknown-unknown

# Cross-compilation
aria build --target aarch64-apple-darwin

# Features
aria build --features "async,json"
aria build --no-default-features
aria build --all-features

# Verbose output
aria build -v
aria build -vv  # Extra verbose

# Build specific package in workspace
aria build -p my_lib
```

### 1.4 Run Commands

```bash
# Run binary
aria run

# Pass arguments to binary
aria run -- arg1 arg2

# Run specific binary
aria run --bin other_binary

# Run example
aria run --example my_example
```

### 1.5 Test Commands

```bash
# Run all tests
aria test

# Run specific test
aria test test_name

# Run tests in module
aria test utils::

# Run with output
aria test -- --nocapture

# Run ignored tests
aria test -- --ignored

# Run doc tests
aria test --doc

# Generate coverage report
aria test --coverage
```

---

## 2. Compiler Architecture

### 2.1 Compilation Pipeline

```
Source Code (.aria)
       ↓
    Lexer
       ↓
   Token Stream
       ↓
    Parser
       ↓
      AST
       ↓
  Name Resolution
       ↓
  Type Checking
       ↓
   Typed AST
       ↓
  Borrow Checking
       ↓
   MIR (Mid-level IR)
       ↓
  Optimization
       ↓
   Backend IR
       ↓
  Code Generation
       ↓
  Object Files (.o)
       ↓
    Linker
       ↓
  Executable
```

### 2.2 Incremental Compilation

```aria
// Query-based architecture (like Salsa)
// Each compilation stage is a query
queries {
    // Source → Tokens
    fn lex(file: FileId) -> TokenStream

    // Tokens → AST
    fn parse(file: FileId) -> Ast

    // AST → Types
    fn type_check(item: ItemId) -> TypedItem

    // Dependencies tracked automatically
    // Only recompute what changed
}
```

### 2.3 Compiler Backends

```bash
# LLVM backend (default for release)
aria build --release  # Uses LLVM

# Cranelift backend (default for debug)
aria build  # Uses Cranelift for speed

# Force backend
aria build --backend llvm
aria build --backend cranelift

# WebAssembly
aria build --target wasm32-unknown-unknown
```

---

## 3. Package Manager (aria pkg)

### 3.1 Dependency Commands

```bash
# Add dependency
aria pkg add serde
aria pkg add serde@1.0
aria pkg add tokio --features full
aria pkg add --dev test_helper
aria pkg add --build build_tool

# Remove dependency
aria pkg remove serde

# Update dependencies
aria pkg update
aria pkg update serde
aria pkg update --dry-run

# Show outdated
aria pkg outdated

# Audit security
aria pkg audit
```

### 3.2 Registry Commands

```bash
# Login to registry
aria pkg login

# Publish package
aria pkg publish
aria pkg publish --dry-run
aria pkg publish --allow-dirty

# Yank (discourage version)
aria pkg yank my_package@0.1.0

# Search
aria pkg search json parser

# Show package info
aria pkg info serde
```

### 3.3 Local Commands

```bash
# Show dependency tree
aria pkg tree
aria pkg tree --duplicates
aria pkg tree --invert serde

# Verify lockfile
aria pkg verify

# Clean cache
aria pkg cache clean

# Show cache location
aria pkg cache path
```

---

## 4. Code Formatting (aria fmt)

### 4.1 Formatter Commands

```bash
# Format all files
aria fmt

# Check formatting (CI)
aria fmt --check

# Format specific files
aria fmt src/main.aria src/lib.aria

# Format stdin
echo "fn main(){}" | aria fmt --stdin
```

### 4.2 Format Configuration

```toml
# aria.toml
[format]
max_width = 100
indent_size = 4
use_tabs = false
newline_style = "auto"  # auto, unix, windows

# Imports
group_imports = "crate"  # preserve, crate, module
imports_granularity = "crate"  # preserve, crate, module, item

# Control flow
single_line_if = true
single_line_let_else = true
```

### 4.3 Format Rules

```aria
// Input
fn foo(a:Int,b:Int)->Int{a+b}

// Output
fn foo(a: Int, b: Int) -> Int {
    a + b
}

// Input
let x = if cond {1} else {2};

// Output (single_line_if = true)
let x = if cond { 1 } else { 2 }

// Output (single_line_if = false)
let x = if cond {
    1
} else {
    2
}
```

---

## 5. Linter (aria lint)

### 5.1 Lint Commands

```bash
# Run all lints
aria lint

# Auto-fix issues
aria lint --fix

# Specific lint category
aria lint --category style
aria lint --category correctness
aria lint --category performance

# Allow/deny specific lints
aria lint --allow unused_variable
aria lint --deny unsafe_code

# Explain lint
aria lint --explain unused_must_use
```

### 5.2 Built-in Lints

```aria
// Correctness
#[allow(dead_code)]
fn unused() {}

#[allow(unused_variable)]
let x = 42

// Style
#[allow(non_snake_case)]
fn BadName() {}

// Performance
#[allow(unnecessary_clone)]
let y = x.clone()  // x is Copy

// Security
#[deny(unsafe_code)]
mod safe_only {
    // No unsafe allowed here
}
```

### 5.3 Lint Configuration

```toml
# aria.toml
[lint]
# Workspace-wide settings
dead_code = "warn"
unused_variable = "warn"
unsafe_code = "deny"

[lint.style]
non_snake_case = "warn"

[lint.performance]
unnecessary_clone = "warn"
```

---

## 6. Documentation (aria doc)

### 6.1 Documentation Commands

```bash
# Generate documentation
aria doc

# Open in browser
aria doc --open

# Include private items
aria doc --private

# Include dependencies
aria doc --deps

# Specific package
aria doc -p my_lib
```

### 6.2 Documentation Format

```aria
/// A point in 2D space.
///
/// # Examples
///
/// ```
/// let p = Point.new(1.0, 2.0)
/// assert_eq(p.x, 1.0)
/// ```
///
/// # Panics
///
/// Panics if coordinates are NaN.
pub struct Point {
    /// The x coordinate.
    pub x: Float
    /// The y coordinate.
    pub y: Float
}

/// Creates a new point.
///
/// # Arguments
///
/// * `x` - The x coordinate
/// * `y` - The y coordinate
///
/// # Returns
///
/// A new `Point` instance.
pub fn new(x: Float, y: Float) -> Point {
    Point { x, y }
}
```

### 6.3 Doc Tests

```aria
/// Adds two numbers.
///
/// ```
/// let result = add(2, 3)
/// assert_eq(result, 5)
/// ```
///
/// Handles negative numbers:
///
/// ```
/// let result = add(-1, 1)
/// assert_eq(result, 0)
/// ```
pub fn add(a: Int, b: Int) -> Int {
    a + b
}

// Doc tests are compiled and run with `aria test --doc`
```

---

## 7. Language Server (LSP)

### 7.1 LSP Commands

```bash
# Start language server
aria lsp

# With debug logging
aria lsp --log-level debug

# Specific transport
aria lsp --stdio        # Default
aria lsp --tcp 9000
```

### 7.2 LSP Features

```
Supported capabilities:
- textDocument/completion       Auto-complete
- textDocument/hover           Type information
- textDocument/definition      Go to definition
- textDocument/references      Find all references
- textDocument/rename          Rename symbol
- textDocument/formatting      Format document
- textDocument/codeAction      Quick fixes
- textDocument/diagnostic      Real-time errors
- workspace/symbol            Symbol search
```

### 7.3 Editor Integration

```json
// VS Code settings.json
{
    "aria.server.path": "aria",
    "aria.server.args": ["lsp"],
    "aria.formatting.enable": true,
    "aria.lint.enable": true,
    "aria.inlayHints.enable": true
}
```

---

## 8. REPL

### 8.1 REPL Commands

```bash
# Start REPL
aria repl

# With imports
aria repl --import std.collections
```

### 8.2 REPL Features

```aria
>>> let x = 42
>>> x * 2
84

>>> fn double(n: Int) -> Int { n * 2 }
>>> double(21)
42

>>> struct Point { x: Float, y: Float }
>>> let p = Point { x: 1.0, y: 2.0 }
>>> p
Point { x: 1.0, y: 2.0 }

// Special commands
>>> :help           Show help
>>> :quit           Exit REPL
>>> :type expr      Show type of expression
>>> :clear          Clear screen
>>> :load file.aria Load and execute file
>>> :reset          Reset REPL state
```

---

## 9. Error Messages

### 9.1 Error Format

```
Error[E0308]: Mismatched types
  --> src/main.aria:10:18
   |
10 |     let x: Int = "hello"
   |            ---   ^^^^^^^ expected `Int`, found `String`
   |            |
   |            expected due to this type annotation
   |
Help: Consider parsing the string:
   |
10 |     let x: Int = "hello".parse()?
   |                         ++++++++
```

### 9.2 Error Categories

```
E0xxx - Type errors
E1xxx - Borrow checker errors
E2xxx - Name resolution errors
E3xxx - Syntax errors
E4xxx - Trait errors
E5xxx - Macro errors
W0xxx - Warnings
```

### 9.3 Machine-Readable Output

```bash
# JSON output for tools
aria build --message-format json

# Output format
{
    "message": "Mismatched types",
    "code": "E0308",
    "level": "error",
    "spans": [{
        "file": "src/main.aria",
        "line_start": 10,
        "line_end": 10,
        "column_start": 18,
        "column_end": 25,
        "label": "expected `Int`, found `String`"
    }],
    "suggestions": [{
        "message": "Consider parsing the string",
        "replacements": [{
            "span": {...},
            "text": ".parse()?"
        }]
    }]
}
```

---

## 10. Cross-Compilation

### 10.1 Target Triples

```bash
# List available targets
aria target list

# Add target
aria target add x86_64-unknown-linux-musl
aria target add wasm32-unknown-unknown
aria target add aarch64-apple-darwin

# Build for target
aria build --target x86_64-unknown-linux-musl
```

### 10.2 Platform Configuration

```toml
# aria.toml
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"

[target.aarch64-apple-darwin]
linker = "aarch64-apple-darwin-clang"

[target.wasm32-unknown-unknown]
runner = "wasm-bindgen-test-runner"
```

### 10.3 Conditional Dependencies

```toml
[target.'cfg(target_os = "linux")'.dependencies]
linux_specific = "1.0"

[target.'cfg(target_os = "windows")'.dependencies]
windows_specific = "1.0"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm_bindgen = "0.2"
```

---

## 11. Build Profiles

### 11.1 Profile Configuration

```toml
# aria.toml

[profile.dev]
opt_level = 0
debug = true
overflow_checks = true
lto = false
incremental = true

[profile.release]
opt_level = 3
debug = false
overflow_checks = false
lto = "thin"
incremental = false
strip = true

[profile.test]
inherits = "dev"
opt_level = 1

[profile.bench]
inherits = "release"
debug = true
```

### 11.2 Custom Profiles

```toml
[profile.release-with-debug]
inherits = "release"
debug = true
strip = false

# Usage: aria build --profile release-with-debug
```

---

## 12. Continuous Integration

### 12.1 GitHub Actions Example

```yaml
name: CI

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: aria-lang/setup-aria@v1
        with:
          aria-version: stable

      - name: Check formatting
        run: aria fmt --check

      - name: Lint
        run: aria lint

      - name: Build
        run: aria build --release

      - name: Test
        run: aria test

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: aria-lang/setup-aria@v1
      - name: Generate coverage
        run: aria test --coverage
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

### 12.2 Pre-commit Hooks

```bash
# Install hooks
aria hook install

# .aria/hooks/pre-commit
#!/bin/sh
aria fmt --check || exit 1
aria lint || exit 1
aria test || exit 1
```
