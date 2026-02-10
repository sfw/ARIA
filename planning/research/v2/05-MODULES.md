# ARIA v2 Module System

## Design: Minimal Keywords, Maximum Clarity

---

## 1. Imports

### 1.1 Basic Import

```
# Single item
us std.collections.Map

# Multiple items
us std.collections.{Map, Set, List}

# All public items
us std.collections.*

# Rename on import
us std.collections.HashMap -> Map
us std.io.Result -> IoResult

# Path forms
us std.fs               # absolute from std
us crate.utils          # from current crate
us self.helper          # from current module
us super.other          # from parent module
```

### 1.2 Shorthand Imports

```
# Common standard library
us std.*                # everything from std
us std.io               # io module
us std.fs               # filesystem
us std.net              # networking
us std.json             # JSON (from extended lib)
```

---

## 2. Module Definition

### 2.1 Inline Module

```
md utils
    pub f helper -> Int = 42
    f private_fn -> Int = 0   # not visible outside

md parser
    pub s Token
        kind: TokenKind
        text: Str

    pub e TokenKind
        Ident
        Number
        Symbol
```

### 2.2 File-Based Module

```
# In main.ar or lib.ar:
md utils     # loads utils.ar or utils/mod.ar
md parser    # loads parser.ar or parser/mod.ar

# File structure:
src/
├── main.ar
├── utils.ar
└── parser/
    ├── mod.ar
    ├── lexer.ar
    └── ast.ar
```

---

## 3. Visibility

### 3.1 Visibility Modifiers

```
# Private (default)
f private_fn -> Int = 42
s PrivateStruct
    value: Int

# Public
pub f public_fn -> Int = 42
pub s PublicStruct
    value: Int

# Crate-public
pub(crate) f crate_fn -> Int = 42

# Parent-public
pub(super) f parent_fn -> Int = 42

# Path-public
pub(in crate.utils) f utils_fn -> Int = 42
```

### 3.2 Struct Field Visibility

```
pub s User
    pub name: Str         # public field
    email: Str            # private field
    pub(crate) id: Int    # crate-visible

# Private fields require constructor
i User
    pub f new(name: Str, email: Str) -> User
        User(name, email, id: gen_id)
```

---

## 4. Re-exporting

```
# In lib.ar - clean public API
pub us self.config.Config
pub us self.db.{Connection, Query}
pub us self.api.Server

# Internal modules
md config
md db
md api

# Users import:
us my_crate.Config       # not my_crate.config.Config
us my_crate.Server
```

### 4.1 Prelude Pattern

```
# prelude.ar
pub us crate.{Config, Error, Result}
pub us crate.traits.{Read, Write}
pub us crate.types.{Option, Result, List, Map}

# Users:
us my_crate.prelude.*
```

---

## 5. Package Manifest

### 5.1 aria.toml

```toml
[package]
name = "my_project"
version = "0.1.0"
edition = "2026"
authors = ["Name <email@example.com>"]
license = "MIT"
description = "A brief description"

[deps]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
local = { path = "../local" }
git_dep = { git = "https://github.com/user/repo" }

[dev-deps]
test_helper = "0.1"

[features]
default = ["std"]
std = []
async = ["tokio"]
full = ["std", "async"]

[[bin]]
name = "my_app"
path = "src/main.ar"

[lib]
name = "my_lib"
path = "src/lib.ar"
```

### 5.2 Version Requirements

```toml
# Exact
dep = "=1.0.0"

# Caret (default)
dep = "1.0"         # ^1.0.0, allows 1.x.x

# Tilde
dep = "~1.0.0"      # >=1.0.0, <1.1.0

# Range
dep = ">=1.0, <2.0"
```

---

## 6. CLI Commands

```bash
# Create project
aria new my_project
aria new --lib my_lib

# Build
aria build
aria build --release

# Run
aria run
aria run -- args

# Test
aria test
aria test module::

# Package management
aria add serde
aria add tokio --features full
aria rm serde
aria update

# Other
aria fmt
aria lint
aria doc
```

---

## 7. Workspaces

### 7.1 Workspace Config

```toml
# Root aria.toml
[workspace]
members = [
    "core",
    "cli",
    "server",
]

[workspace.deps]
serde = "1.0"
tokio = "1.0"

[workspace.package]
version = "0.1.0"
edition = "2026"
```

### 7.2 Member Package

```toml
# core/aria.toml
[package]
name = "my_core"
version.workspace = true
edition.workspace = true

[deps]
serde.workspace = true
```

---

## 8. Conditional Compilation

```
# OS
@cfg(os = "linux")
f linux_fn
    # ...

@cfg(os = "windows")
f windows_fn
    # ...

# Architecture
@cfg(arch = "x86_64")
f x86_fn
    # ...

# Features
@cfg(feature = "async")
md async_impl
    # ...

# Build mode
@cfg(debug)
f debug_only
    # ...

@cfg(release)
f release_only
    # ...

# Combinations
@cfg(all(os = "linux", arch = "x86_64"))
f linux_x86
    # ...

@cfg(any(os = "linux", os = "macos"))
f unix_like
    # ...

@cfg(not(os = "windows"))
f not_windows
    # ...
```

---

## 9. Features

### 9.1 Defining Features

```toml
[features]
default = ["std"]
std = []
async = ["dep:tokio"]
json = ["dep:serde_json"]
full = ["std", "async", "json"]

[deps]
tokio = { version = "1.0", optional = true }
serde_json = { version = "1.0", optional = true }
```

### 9.2 Using Features

```
@cfg(feature = "async")
pub md async_impl
    pub as f fetch(url: Str) -> Data!
        # ...

@cfg(not(feature = "async"))
pub md sync_impl
    pub f fetch(url: Str) -> Data!
        # ...

@cfg(feature = "json")
pub f parse_json(s: &str) -> Data!
    # ...
```

---

## 10. Complete Example

```
# Project structure
my_app/
├── aria.toml
├── src/
│   ├── main.ar
│   ├── lib.ar
│   ├── config.ar
│   ├── db/
│   │   ├── mod.ar
│   │   ├── connection.ar
│   │   └── query.ar
│   └── api/
│       ├── mod.ar
│       ├── routes.ar
│       └── handlers.ar
└── tests/
    └── integration.ar
```

```
# src/lib.ar
//! My application library

pub us self.config.Config
pub us self.db.{Connection, Query}
pub us self.api.Server

md config
md db
md api
```

```
# src/config.ar
us std.fs
us std.json

pub s Config
    pub host: Str
    pub port: Int
    pub debug: B = F

i Config
    pub f load(path: Str) -> Config!
        content = fs.read path?
        json.parse content
```

```
# src/db/mod.ar
pub us self.connection.Connection
pub us self.query.Query

md connection
md query
```

```
# src/main.ar
us my_app.{Config, Server}

f main
    config = Config.load "app.toml"
        ?? || panic "failed to load config"

    server = Server.new &config
    server.run
```
