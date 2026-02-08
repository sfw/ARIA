# Sprint 36: Launch Hardening

**Goal:** Close launch-blocking security and reliability gaps identified in the deep code review.
**Estimated Effort:** 1-2 days

---

## Context

A pre-launch deep review found concrete blockers in capability enforcement, runtime robustness, module loading, and LLVM feature readiness.

Key risks to fix in this sprint:
1. Capability model is bypassable for multiple privileged builtins.
2. Some HTTP paths can panic the entire process.
3. Module loader does not resolve transitive imports.
4. LLVM feature build is currently broken due to non-exhaustive MIR statement handling.
5. Runtime string slicing can panic in FFI path.

---

## Agent Instructions

1. Treat this as a launch-blocking hardening sprint, not a refactor sprint.
2. Keep fixes minimal and targeted. Avoid broad architectural changes unless required to pass acceptance criteria.
3. Do not modify unrelated files.
4. Preserve existing CLI UX unless explicitly listed below.
5. Add tests for every fixed defect class.
6. If a task exposes follow-up work too large for this sprint, document it under "Deferred" in your PR notes.

---

## Task 36.1: Complete Capability Enforcement

**Priority:** P0

**Files (likely):**
- `src/mir/interp.rs`
- `src/main.rs` (only if CLI capability flags must expand)
- tests in `src/mir/interp.rs` test module and/or `tests/`

### Requirements

1. Enforce capability checks consistently for privileged builtins.
2. At minimum, remove known bypasses verified in review:
- `file_append`
- `exec`
- `http_post`
- `http_post_json`
- `http_put`
- `http_delete`
- `http_serve`
- `udp_bind` and UDP send/recv operations
- `tls_connect`
- destructive file/dir operations (`file_remove`, `file_move`, `file_copy`, `dir_remove`, `dir_remove_all`, `dir_create`, `dir_create_all`, `chdir`)
- sqlite file-backed operations (`db_open` and related disk-touching calls)
3. Keep error behavior consistent: denied operations must return `InterpError::capability_denied(...)`, not panic.
4. Centralize capability mapping logic to reduce drift (single helper or table).

### Acceptance Criteria

1. Running privileged builtins without grants fails with capability-denied errors.
2. Running same builtins with expected grants succeeds.
3. No previously-protected builtin regresses (`file_read`, `file_write`, `http_get`, `tcp_connect`, `tcp_listen`).

---

## Task 36.2: Harden HTTP Builtins Against Panics

**Priority:** P1

**Files (likely):**
- `src/mir/interp.rs`

### Requirements

1. Ensure HTTP builtins never crash the interpreter process due to internal client/runtime panics.
2. Convert failure paths into `Result::Err` values or `InterpError`, preserving interpreter stability.
3. Cover `http_get`, `http_post`, `http_post_json`, `http_put`, `http_delete`.

### Acceptance Criteria

1. Invalid/host-problematic HTTP calls do not panic the process.
2. Failures are surfaced as runtime errors/`Result::Err` values.

---

## Task 36.3: Fix Transitive Module Import Loading

**Priority:** P1

**Files (likely):**
- `src/module/loader.rs`
- tests under `tests/` or module loader unit tests

### Requirements

1. Resolve imports transitively (`main -> a -> b`) instead of only top-level `use` items.
2. Preserve cycle detection semantics.
3. Ensure `loading` state is correctly cleaned up even when parse/lex/read errors happen.

### Acceptance Criteria

1. Transitive import scenario compiles/runs.
2. Circular import scenario produces deterministic circular dependency error.

---

## Task 36.4: Restore LLVM Feature Build Health

**Priority:** P1

**Files (likely):**
- `src/codegen/llvm.rs`
- optional docs update in `README.md` if limitations remain

### Requirements

1. Handle `StatementKind::IndexAssign` in LLVM codegen match so feature build compiles.
2. Keep behavior explicit: implement support or return clear codegen error where not yet supported.
3. Eliminate compile-blocking non-exhaustive matches.

### Acceptance Criteria

1. `cargo test --features llvm --no-run` passes.
2. No new warnings elevated to errors in LLVM path.

---

## Task 36.5: Remove FFI String Substring Panic Path

**Priority:** P2

**Files (likely):**
- `runtime/src/string.rs`
- runtime tests (add if missing)

### Requirements

1. `forma_str_substr` must not panic on non-char-boundary ranges.
2. Return safe output (or null) consistently for invalid boundaries.
3. Preserve null-terminated allocation contract.

### Acceptance Criteria

1. UTF-8 multibyte input does not crash runtime substring function.
2. New tests validate boundary behavior.

---

## Verification Checklist

Run all of the following and include results in PR summary:

1. `cargo test`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test --features llvm --no-run`
4. Capability smoke checks via CLI (deny then allow cases for file append, exec, and network write path)
5. Transitive import repro case (`main -> a -> b`) now passes

---

## Out of Scope (This Sprint)

1. Full capability redesign (path-scoped and host-scoped policy engine).
2. Full LLVM parity for every MIR feature beyond compile health and explicit handling.
3. LSP feature quality improvements not tied to launch blockers.

---

## Definition of Done

1. All P0/P1 tasks above are implemented and verified.
2. No known capability bypass from the review remains.
3. LLVM feature build no longer fails at compile time.
4. Transitive imports work and cycle detection still works.
5. PR includes a concise "before vs after" risk summary.
