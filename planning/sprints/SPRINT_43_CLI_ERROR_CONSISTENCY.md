# Sprint 43: CLI Error Consistency + Import Validation

**Goal:** Close remaining CLI correctness gaps so `run`, `check`, and `build` have consistent import handling and JSON error behavior.
**Estimated Effort:** 0.5-1 day

---

## Context

Post-Sprint-42 verification still shows two launch-relevant issues:

1. `check --error-format json` reports success for missing imports.
2. `build --error-format json` can fail silently (non-zero exit, empty JSON output) on early errors.

There is also process confusion around workspace/commit state, so this sprint includes explicit hygiene checks in CI/local verification.

---

## Agent Instructions

1. Treat this as a correctness sprint, not a refactor sprint.
2. Keep behavior aligned across `run`, `check`, and `build` for module loading and JSON error output.
3. Add regression tests first-class (must fail before fix, pass after fix).
4. Do not weaken or skip tests to get green.

---

## Task 43.1: Make `check` Validate Imports

**Priority:** P0

**Files (likely):**
- `src/main.rs`
- `src/module/loader.rs` (only if needed)
- `tests/cli_tests.rs`
- fixtures under `tests/fixtures/`

### Requirements

1. In `check(...)`, load imports the same way `run(...)` and `build(...)` do.
2. Missing imports must produce an error (not success) in both human and JSON modes.
3. Preserve existing `--partial` behavior, but include module errors in the structured response.

### Acceptance Criteria

1. `forma check --error-format json <file-with-missing-import>` returns non-zero and JSON with `success: false` and at least one `MODULE` error.
2. `forma check <file-with-missing-import>` prints human-readable module error and exits non-zero.
3. Existing successful-check flows still pass.

---

## Task 43.2: Ensure `build` Always Emits JSON Errors on Failure

**Priority:** P0

**Files (likely):**
- `src/main.rs`
- `tests/cli_tests.rs`

### Requirements

1. For all early `build(...)` failure paths (lex/parse/module/type/codegen/link), if `--error-format json` is set, emit a JSON error payload before returning.
2. Eliminate silent JSON failures (non-zero with empty stdout/stderr JSON body).
3. Keep current human-format output unchanged.

### Acceptance Criteria

1. `forma build --error-format json <file-with-missing-import>` returns non-zero and prints structured JSON error output.
2. At least one regression test asserts that build failure in JSON mode is never silent.
3. No regression in successful build JSON output.

---

## Task 43.3: CLI Parity Regression Tests

**Priority:** P1

**Files (likely):**
- `tests/cli_tests.rs`
- `tests/fixtures/missing_import.forma`

### Requirements

1. Add tests covering missing-import behavior for:
- `run --error-format json`
- `check --error-format json`
- `build --error-format json`
2. Assert parity of key fields (`success`, `code`, non-empty `errors`, exit status).

### Acceptance Criteria

1. All three commands fail consistently on missing import with structured JSON errors.
2. New tests are deterministic and pass in CI.

---

## Task 43.4: Workspace Hygiene Guardrail

**Priority:** P1

**Files (likely):**
- `.github/workflows/ci.yml` (optional)
- docs/PR checklist (optional)

### Requirements

1. Add a lightweight verification step/checklist item in sprint output requiring:
- `git status --short` capture before final summary
- explicit list of intentionally untracked files (if any)
2. Ensure sprint completion summaries clearly distinguish:
- committed changes
- uncommitted workspace edits

### Acceptance Criteria

1. Final verification output includes commit hash and clean/dirty status.
2. No ambiguity about whether fixes are committed.

---

## Verification Checklist

Run and include results in PR summary:

1. `cargo test --all`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo clippy --all-features --all-targets -- -D warnings`
4. `cargo fmt --all -- --check`
5. `cargo build --release`
6. Targeted CLI repros:
- `forma check --error-format json tests/fixtures/missing_import.forma`
- `forma build --error-format json tests/fixtures/missing_import.forma`
- `forma run --error-format json tests/fixtures/missing_import.forma`
7. `git status --short` and `git log --oneline -n 1`

---

## Out of Scope

1. New module system features.
2. Capability model changes.
3. Broad CLI redesign.

---

## Definition of Done

1. `check` no longer reports false success on missing imports.
2. `build --error-format json` never fails silently.
3. CLI parity tests cover and lock this behavior.
4. Sprint summary includes explicit commit/worktree state.

---

## Coding Agent Prompt

```text
Implement Sprint 43: CLI Error Consistency + Import Validation.

Fix these exact defects:
1) `forma check --error-format json` currently returns success on missing imports.
2) `forma build --error-format json` can fail silently (non-zero exit with no JSON error output) on early failures.

Requirements:
- Make `check` load/validate imports consistently with `run` and `build`.
- Ensure `build` emits JSON-formatted errors for all failure paths when `--error-format json` is set.
- Add regression tests in tests/cli_tests.rs using a missing-import fixture.
- Keep human-readable output behavior intact.

Validation to run:
- cargo test --all
- cargo clippy --all-targets -- -D warnings
- cargo clippy --all-features --all-targets -- -D warnings
- cargo fmt --all -- --check
- cargo build --release
- forma check/build/run --error-format json on missing-import fixture
- git status --short
- git log --oneline -n 1

Deliverables:
- code changes
- regression tests
- concise summary with file list, command results, and commit hash
```
