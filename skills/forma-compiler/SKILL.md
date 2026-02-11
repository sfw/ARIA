---
name: forma-compiler
description: >
  FORMA compiler internals and development patterns. Use when modifying the
  compiler source code, adding builtins, fixing bugs, writing tests, or
  working on the lexer, parser, type checker, MIR, or interpreter.
---

# FORMA Compiler Development Skill

The FORMA compiler is written in Rust. This skill covers architecture, patterns,
and conventions for compiler development.

## Architecture

```
Source (.forma)
  -> Lexer (scanner.rs)     -> Vec<Token>
  -> Parser (parser.rs)     -> AST (ast.rs)
  -> Type Checker (inference.rs) -> Typed AST
  -> MIR Lowering (mir/mod.rs)   -> MIR Program
  -> Optimizer (mir/optimize.rs) -> Optimized MIR
  -> Interpreter (mir/interp.rs) -> Execution
     OR
  -> LLVM Codegen (codegen/)     -> Native binary
```

## Key Files

| File | Purpose |
|------|---------|
| `src/lexer/token.rs` | TokenKind enum, keyword() fn, Display impl |
| `src/lexer/scanner.rs` | Scanner with indentation tracking |
| `src/parser/ast.rs` | AST types (Type, Expr, Item, etc.) |
| `src/parser/parser.rs` | Recursive descent parser |
| `src/types/types.rs` | Internal type representations (Ty enum) |
| `src/types/inference.rs` | InferenceEngine with type checking |
| `src/mir/mod.rs` | MIR types and lowering |
| `src/mir/optimize.rs` | MIR optimization passes |
| `src/mir/interp.rs` | MIR interpreter (~7000 lines, builtins) |
| `src/ffi/safe_ptr.rs` | SafePtr, MemoryArena for FFI safety |
| `src/main.rs` | CLI (clap), REPL, grammar export, contract explain |
| `src/lsp/` | LSP server implementation |
| `stdlib/` | FORMA stdlib modules (.forma files) |
| `runtime/` | Native runtime crate for LLVM codegen |

## Adding a New Keyword

1. Add variant to `TokenKind` enum in `src/lexer/token.rs`
2. Add to `keyword()` match in same file
3. Add to `is_keyword()` match in same file
4. Add to `Display` impl in same file
5. Use in parser as needed

Contextual single-letter keywords (`f`, `s`, `e`, `t`, `i`, `m`) are emitted as
`Ident` by the scanner; the parser decides based on context.

## Adding a New Builtin Function

1. **Interpreter** (`src/mir/interp.rs`): Add case to `call_builtin()` match.
   Follow the pattern of existing builtins for arg extraction and error handling.

2. **Type inference** (`src/types/inference.rs`): Add entry to `builtin_type()`
   match to specify the function signature.

3. **LSP completions** (`src/lsp/`): Add to the builtin completions list so
   IDEs can autocomplete it.

4. **LSP signature help**: Add function signature for hover/signature help.

5. **Docs** (`docs/ai-reference.md`): Add to the appropriate category.

6. **Tests**: Add unit test in the test module at the bottom of `interp.rs` and/or
   a `.forma` integration test in `tests/forma/`.

### Builtin Pattern

```rust
"my_builtin" => {
    check_arity!(args, 2, "my_builtin");
    let a = &args[0];
    let b = &args[1];
    // ... implementation ...
    Ok(Value::Int(result))
}
```

### Higher-Order Builtin Pattern (closures)

```rust
"map" => {
    check_arity!(args, 2, "map");
    let arr = match &args[0] {
        Value::Array(a) => a.clone(),
        _ => return Err(self.error("map: first arg must be array", span)),
    };
    let (func_name, captures) = match &args[1] {
        Value::Closure { func_name, captures, .. } => (func_name.clone(), captures.clone()),
        _ => return Err(self.error("map: second arg must be closure", span)),
    };
    let func = self.program.functions.iter()
        .find(|f| f.name == func_name)
        .ok_or_else(|| self.error("map: closure function not found", span))?
        .clone();
    let mut result = Vec::new();
    for elem in &arr {
        let mut call_args = captures.clone();
        call_args.push(elem.clone());
        let val = self.call_function_internal(&func, call_args, span)?;
        result.push(val);
    }
    Ok(Value::Array(result))
}
```

## Adding a Field to AST Structs

Use the constructor method pattern (e.g., `Type::new()`) to avoid touching
hundreds of struct literal sites. `InterpError` has 541+ construction sites;
never add required fields directly — use helper methods.

## Capability System

Builtins requiring system access must call `self.require_capability(cap, span)?`.

| Capability | Flag | Builtins |
|-----------|------|----------|
| `"read"` | `--allow-read` | file_read, dir_list, etc. |
| `"write"` | `--allow-write` | file_write, dir_create, etc. |
| `"network"` | `--allow-network` | http_*, tcp_*, udp_*, tls_*, dns_* |
| `"exec"` | `--allow-exec` | exec, pid, args, cwd, etc. |
| `"env"` | `--allow-env` | env_get, env_set, env_remove, env_vars |
| `"unsafe"` | `--allow-unsafe` | alloc, dealloc, ptr_*, mem_*, ffi |
| `"all"` | `--allow-all` | All of the above |

## Size Validation

Use `validate_size(n, name, max)` and `validate_port(n, name)` free functions
for user-supplied sizes and ports. Constants: `MAX_BUFFER_SIZE`, `MAX_ALLOC_SIZE`,
`MAX_PORT`.

## MIR Optimization

`src/mir/optimize.rs` runs between lowering and interpretation/codegen.
Passes: constant fold, copy propagation (block-local), dead block elimination, peephole.
Fixed-point iteration (max 3 rounds). `--no-optimize` CLI flag disables it.

Copy propagation is restricted to compiler temps (`LocalDecl.name == None`).
Block-local only — mappings do not cross block boundaries.

## Contract System

- `@pre(expr)` / `@post(expr)` attributes on functions
- `old(expr)` in post-conditions captures entry state
- `result` refers to return value in post-conditions
- 35 named patterns expand to expressions at check time
- `--no-check-contracts` disables; contracts are ON by default
- `check_contracts` field on Interpreter, `set_check_contracts()` setter

## Reference Parameters

- `PassMode` enum in `ast.rs` and `mir.rs`: `ByValue`, `ByRef`, `ByRefMut`
- `ref_bindings: HashMap` in `Frame` struct tracks reference bindings
- `resolve_local()`: always check `ref_bindings` before `locals`
- Borrow checking: double-mutable-borrow detection in `inference.rs`

## Testing

```bash
cargo test                          # all Rust tests (~518)
cargo test --all                    # include runtime crate
cargo clippy --all-targets -- -D warnings  # zero warnings required

# .forma integration tests (56 files)
./target/release/forma run --allow-all tests/forma/test_*.forma

# CLI integration tests
cargo test --test cli_tests

# Optimization equivalence
forma run --allow-all tests/forma/test_optimization.forma > /tmp/opt.out
forma run --allow-all --no-optimize tests/forma/test_optimization.forma > /tmp/noopt.out
diff /tmp/opt.out /tmp/noopt.out
```

Test conventions:
- `.forma` test files use `then 0 else 1` exit convention (0 = success)
- CI uses `--allow-all` for .forma tests
- Zero compiler warnings, zero clippy warnings required

## Important Lessons

- If a test that should not fail, fails: **fix the bug**, never change the test
- Builtin functions can shadow user-defined ones; check user-defined FIRST in Call
- `resolve_local()`: always check ref_bindings before locals
- Negative `i64 -> usize` casts wrap silently; always guard with `>= 0` check
- Discriminant hash: byte-sum collides for anagram variants; use FNV-like hash
- Match guards were silently dropped by MIR lowerer (fixed Sprint 39)
- PatternKind::Ident("None") can shadow the None constructor — check get_enum_for_variant

## LLVM Build

```bash
# macOS: needs zstd linking
LIBRARY_PATH="$(brew --prefix)/lib" cargo build --features llvm

# Opaque pointers only (Context::ptr_type), no typed ptr_type calls
```

## Crate Allows

```rust
// src/lib.rs
#![allow(clippy::result_large_err, clippy::module_inception)]

// runtime/src/lib.rs
#![allow(clippy::missing_safety_doc, clippy::not_unsafe_ptr_arg_deref)]
```

## Module System

- `load_module_recursive()` handles transitive imports with cycle detection
- `ModuleError` has `span: Option<Span>` for diagnostic propagation
- `us` statement span is attached to module load errors
