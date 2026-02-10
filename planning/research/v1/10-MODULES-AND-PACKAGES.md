# ARIA Module and Package System

## Design Goals

1. **Simplicity**: Clear, predictable module resolution
2. **Encapsulation**: Strong privacy boundaries
3. **Reproducibility**: Lockfiles and semantic versioning
4. **Performance**: Parallel compilation, incremental builds
5. **Tooling**: Integrated package manager (`aria pkg`)

---

## 1. Module System

### 1.1 Module Declaration

```aria
// In-file module
mod utils {
    pub fn helper() -> Int { 42 }
    fn private() { }  // Not visible outside
}

// File-based module
// Declaring `mod utils` looks for:
//   1. utils.aria (single file module)
//   2. utils/mod.aria (directory module)
mod utils
```

### 1.2 File Structure

```
my_project/
├── aria.toml           # Package manifest
├── src/
│   ├── main.aria       # Binary entry point
│   ├── lib.aria        # Library root (optional)
│   ├── utils.aria      # Module file
│   └── parser/
│       ├── mod.aria    # Directory module root
│       ├── lexer.aria  # Submodule
│       └── ast.aria    # Submodule
└── tests/
    └── test_utils.aria # Test files
```

### 1.3 Module Paths

```aria
// Absolute path from crate root
use crate.utils.helper
use crate.parser.lexer.Token

// Relative path
use self.helper        // Current module
use super.other_fn     // Parent module

// External crate
use std.collections.Map
use serde.Serialize
```

### 1.4 Importing Items

```aria
// Single item
use std.collections.HashMap

// Multiple items
use std.collections.{HashMap, HashSet, BTreeMap}

// All public items (use sparingly)
use std.collections.*

// Rename on import
use std.collections.HashMap as Map
use std.io.Result as IoResult

// Re-export
pub use self.internal.PublicType
pub use crate.utils.{helper, format}
```

---

## 2. Visibility

### 2.1 Visibility Modifiers

```aria
// Private (default) - visible only in current module
fn private_fn() { }
struct PrivateStruct { }

// Public - visible everywhere
pub fn public_fn() { }
pub struct PublicStruct { }

// Crate-public - visible within the crate only
pub(crate) fn crate_fn() { }

// Parent-public - visible to parent module
pub(super) fn parent_fn() { }

// Path-public - visible to specific module
pub(in crate.utils) fn utils_fn() { }
```

### 2.2 Struct Field Visibility

```aria
pub struct User {
    pub name: String      // Public field
    email: String         // Private field (default)
    pub(crate) id: Int   // Crate-visible field
}

// Private fields require constructor
impl User {
    pub fn new(name: String, email: String) -> User {
        User { name, email, id: generate_id() }
    }
}
```

### 2.3 Privacy Rules

```aria
// Child modules can see parent's private items
mod parent {
    fn private_fn() { }

    mod child {
        fn use_parent() {
            super.private_fn()  // OK
        }
    }
}

// But not vice versa
mod parent {
    mod child {
        fn child_private() { }
    }

    fn try_use() {
        // child.child_private()  // ERROR: private
    }
}
```

---

## 3. Package Management

### 3.1 Package Manifest (aria.toml)

```toml
[package]
name = "my_project"
version = "0.1.0"
edition = "2026"
authors = ["Author Name <author@example.com>"]
license = "MIT"
description = "A brief description"
repository = "https://github.com/user/my_project"
keywords = ["keyword1", "keyword2"]
categories = ["category"]

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
local_pkg = { path = "../local_pkg" }
git_pkg = { git = "https://github.com/user/repo" }

[dev-dependencies]
test_helper = "0.1"

[build-dependencies]
build_tool = "0.2"

[features]
default = ["std"]
std = []
async = ["tokio"]
full = ["std", "async"]

[[bin]]
name = "my_binary"
path = "src/main.aria"

[lib]
name = "my_library"
path = "src/lib.aria"

[profile.release]
opt_level = 3
lto = true

[profile.dev]
opt_level = 0
debug = true
```

### 3.2 Lockfile (aria.lock)

```toml
# Auto-generated, do not edit
[[package]]
name = "my_project"
version = "0.1.0"

[[package]]
name = "serde"
version = "1.0.152"
source = "registry+https://packages.aria-lang.org"
checksum = "sha256:abc123..."
dependencies = [
    "serde_derive",
]

[[package]]
name = "serde_derive"
version = "1.0.152"
source = "registry+https://packages.aria-lang.org"
checksum = "sha256:def456..."
```

### 3.3 Version Requirements

```toml
# Exact version
serde = "=1.0.152"

# Caret (default, allows minor updates)
serde = "1.0"      # Equivalent to "^1.0"
serde = "^1.0.0"   # >=1.0.0, <2.0.0

# Tilde (allows patch updates)
serde = "~1.0.0"   # >=1.0.0, <1.1.0

# Wildcard
serde = "1.*"      # >=1.0.0, <2.0.0

# Range
serde = ">=1.0, <2.0"
serde = ">1.0, <=1.5"

# Multiple requirements
serde = ">=1.0, <1.5, !=1.3.0"
```

---

## 4. CLI Commands

### 4.1 Package Commands

```bash
# Create new project
aria new my_project
aria new --lib my_library

# Build project
aria build
aria build --release
aria build --target wasm32-unknown-unknown

# Run project
aria run
aria run --release
aria run -- arg1 arg2

# Run tests
aria test
aria test utils::     # Test specific module
aria test --doc       # Run doc tests

# Generate documentation
aria doc
aria doc --open

# Check without building
aria check

# Format code
aria fmt
aria fmt --check

# Lint code
aria lint
aria lint --fix
```

### 4.2 Dependency Commands

```bash
# Add dependency
aria add serde
aria add serde@1.0
aria add tokio --features full
aria add --dev test_helper

# Remove dependency
aria remove serde

# Update dependencies
aria update
aria update serde
aria update --dry-run

# Show dependency tree
aria tree
aria tree --duplicates
aria tree --invert serde

# Audit for security issues
aria audit
```

### 4.3 Publishing

```bash
# Login to registry
aria login

# Publish package
aria publish
aria publish --dry-run

# Yank version (discourage use)
aria yank my_package@0.1.0

# Search packages
aria search json parser
```

---

## 5. Workspaces

### 5.1 Workspace Configuration

```toml
# Root aria.toml
[workspace]
members = [
    "core",
    "cli",
    "server",
    "shared/*",  # Glob patterns
]
exclude = ["experiments"]

# Shared dependencies for all members
[workspace.dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }

[workspace.package]
version = "0.1.0"
edition = "2026"
license = "MIT"
```

### 5.2 Member Package

```toml
# Member aria.toml (core/aria.toml)
[package]
name = "my_project_core"
version.workspace = true    # Inherit from workspace
edition.workspace = true

[dependencies]
serde.workspace = true      # Use workspace version
local_dep = { path = "../shared/utils" }
```

### 5.3 Workspace Commands

```bash
# Build all workspace members
aria build --workspace

# Build specific member
aria build -p core

# Test all members
aria test --workspace

# Run member binary
aria run -p cli -- args
```

---

## 6. Features

### 6.1 Defining Features

```toml
[features]
# Default features (enabled by default)
default = ["std"]

# Feature flags
std = []
async = ["dep:tokio"]
json = ["dep:serde_json"]
full = ["std", "async", "json"]

# Optional dependencies as features
serde = { version = "1.0", optional = true }
tokio = { version = "1.0", optional = true }
```

### 6.2 Using Features in Code

```aria
// Conditional compilation
#[cfg(feature = "async")]
pub mod async_impl {
    pub async fn fetch() { ... }
}

#[cfg(not(feature = "async"))]
pub mod sync_impl {
    pub fn fetch() { ... }
}

// Feature-gated items
#[cfg(feature = "json")]
pub fn parse_json(s: &str) -> Result[Value, Error] { ... }

// Feature-gated derives
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Config { ... }
```

### 6.3 Depending on Features

```toml
# In dependent package
[dependencies]
my_lib = { version = "1.0", features = ["async", "json"] }
my_lib = { version = "1.0", default-features = false }
my_lib = { version = "1.0", default-features = false, features = ["json"] }
```

---

## 7. Build Scripts

### 7.1 Build Script

```aria
// build.aria - runs before compilation
fn main() {
    // Generate code
    let out_dir = env("OUT_DIR")
    generate_bindings("{out_dir}/bindings.aria")

    // Link native library
    println("cargo:rustc-link-lib=native")
    println("cargo:rustc-link-search=/usr/local/lib")

    // Rerun if file changes
    println("cargo:rerun-if-changed=wrapper.h")

    // Set environment variable for main build
    println("cargo:rustc-env=VERSION=1.0.0")
}
```

### 7.2 Including Generated Code

```aria
// In main code
mod generated {
    include!(concat!(env!("OUT_DIR"), "/bindings.aria"))
}
```

---

## 8. Conditional Compilation

### 8.1 Target Configuration

```aria
// Operating system
#[cfg(target_os = "linux")]
fn platform_specific() { ... }

#[cfg(target_os = "windows")]
fn platform_specific() { ... }

#[cfg(target_os = "macos")]
fn platform_specific() { ... }

// Architecture
#[cfg(target_arch = "x86_64")]
fn arch_specific() { ... }

#[cfg(target_arch = "aarch64")]
fn arch_specific() { ... }

// Pointer width
#[cfg(target_pointer_width = "64")]
fn pointer_size() { ... }

// Endianness
#[cfg(target_endian = "little")]
fn endian_specific() { ... }
```

### 8.2 Build Configuration

```aria
// Debug vs release
#[cfg(debug)]
fn debug_only() { ... }

#[cfg(release)]
fn release_only() { ... }

// Test configuration
#[cfg(test)]
mod tests { ... }

// Documentation generation
#[cfg(doc)]
fn doc_only() { ... }
```

### 8.3 Combining Conditions

```aria
// AND
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]

// OR
#[cfg(any(target_os = "linux", target_os = "macos"))]

// NOT
#[cfg(not(target_os = "windows"))]

// Complex combinations
#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64"),
    not(feature = "minimal")
))]
```

---

## 9. Module Best Practices

### 9.1 Module Organization

```
// Good: Flat structure for small projects
src/
├── main.aria
├── config.aria
├── database.aria
└── handlers.aria

// Good: Hierarchical for larger projects
src/
├── lib.aria
├── config/
│   ├── mod.aria
│   ├── file.aria
│   └── env.aria
├── database/
│   ├── mod.aria
│   ├── connection.aria
│   └── queries.aria
└── api/
    ├── mod.aria
    ├── routes.aria
    └── handlers.aria
```

### 9.2 Re-exporting

```aria
// lib.aria - Clean public API
pub use self.config.Config
pub use self.database.{Connection, Query}
pub use self.api.Server

// Internal modules
mod config
mod database
mod api

// Users import:
use my_crate.Config
use my_crate.Server
// Not: use my_crate.config.Config
```

### 9.3 Prelude Pattern

```aria
// prelude.aria - Common imports
pub use crate.{Config, Error, Result}
pub use crate.traits.{Read, Write, Display}
pub use crate.types.{Option, Result, List, Map}

// Users can do:
use my_crate.prelude.*
```

---

## 10. Semantic Versioning

### 10.1 Version Numbering

```
MAJOR.MINOR.PATCH

1.0.0 - Initial stable release
1.0.1 - Patch: Bug fixes, no API changes
1.1.0 - Minor: New features, backward compatible
2.0.0 - Major: Breaking changes
```

### 10.2 What Constitutes Breaking Changes

**Breaking (requires major bump):**
- Removing public items
- Changing function signatures
- Changing struct fields (public)
- Changing enum variants
- Tightening trait bounds
- Changing behavior in incompatible ways

**Non-breaking (minor bump):**
- Adding new public items
- Adding new optional parameters with defaults
- Loosening trait bounds
- Adding trait implementations
- Deprecating items (without removing)

**Patch-level:**
- Bug fixes
- Performance improvements
- Documentation updates
- Internal refactoring
