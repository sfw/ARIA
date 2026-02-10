# Sprint 49: Contract Pattern Correctness + Safety

## Goal

Fix correctness and safety regressions introduced in Sprint 48 contract-pattern expansion, and close test/documentation gaps before launch.

---

## Scope

### 49.1 Fix `@rotated` Runtime Safety + Semantics

**Priority:** P0  
**Files:** `src/mir/interp.rs`, `tests/forma/`, `tests/cli_tests.rs` (if needed)

### Problems

- Negative `k` in `is_rotated(a, b, k)` can panic via integer cast/overflow path.
- Empty-array behavior is currently unintuitive (`([], [], 0)` fails).

### Required Changes

- Remove panic path by avoiding unchecked `as usize` conversion from signed `k`.
- Normalize rotation using safe math (`rem_euclid`) and bounded indexing.
- Define and implement explicit empty-array semantics (recommended: empty vs empty is rotated for any `k`, or at minimum for `k == 0`), and document it.
- Ensure invalid cases produce contract `false` (or typed interpreter error) but never panic.

### Acceptance Criteria

1. `@rotated(..., -1)` never panics.
2. Empty-array rotation behavior is deterministic and documented.
3. Regression tests cover negative, large, and empty-array cases.

---

### 49.2 Fix `@stable` Correctness (No False Passes)

**Priority:** P1  
**Files:** `src/mir/interp.rs`, `tests/forma/`, `tests/` (Rust unit tests)

### Problems

- `stable(input, output, key)` currently checks relative order but does not enforce that `output` preserves all elements of `input`.
- False positives are possible when output drops elements.

### Required Changes

- Strengthen `stable` contract helper to require:
  - same length,
  - permutation/multiset preservation between input and output,
  - stable relative ordering for equal keys.
- Keep clear errors for invalid struct/key usage.
- Add explicit negative tests (dropped elements, duplicated elements, wrong key ordering).

### Acceptance Criteria

1. `@stable` fails when output is not a full permutation of input.
2. `@stable` still passes valid stable-sort outputs.
3. Tests include both pass and fail paths.

---

### 49.3 Align Spec and Docs with Actual Pattern Semantics

**Priority:** P2  
**Files:** `docs/reference.md`, `planning/sprints/SPRINT_48_CONTRACT_PATTERNS.md` (if retaining), optional inline comments in `src/parser/parser.rs`

### Problems

- Docs currently imply semantics that are broader/clearer than implementation in a few edge cases.
- Pattern postcondition inference rule text can be misread.

### Required Changes

- Update docs to reflect precise behavior for:
  - `@rotated` (including empty arrays and `k` normalization),
  - `@stable` (requires full output preservation + stable order),
  - postcondition inference rule wording (explicitly describe result-context rule implemented by parser).
- Ensure docs do not overclaim unsupported behavior.

### Acceptance Criteria

1. Reference docs match runtime behavior exactly for the above patterns.
2. No ambiguous wording around postcondition inference.

---

### 49.4 Test Coverage Closure for New Sprint-48 Patterns

**Priority:** P1  
**Files:** `tests/forma/test_sequence_patterns.forma`, `tests/forma/test_sorting_patterns.forma`, new/expanded Rust tests in `src/mir/interp.rs` test module

### Problems

- Several new helpers/patterns are not directly tested (`@rotated`, `@partitioned`, `@sorted_by`, `@stable`).
- Current tests are mostly happy-path.

### Required Changes

- Add direct integration coverage for:
  - `@rotated` (positive, zero, negative, empty, large `k`),
  - `@partitioned` (valid/invalid pivot and ordering),
  - `@sorted_by` (struct field ordering),
  - `@stable` (stable pass, unstable fail, missing element fail).
- Add focused Rust unit tests for helper functions where useful (especially `is_rotated` and `is_stable_sort`).

### Acceptance Criteria

1. Each new helper from Sprint 48 has at least one positive and one negative test.
2. Repro cases from Sprint 48 review are covered by regression tests.

---

## Verification Plan

Run and include in PR summary:

```bash
# Core checks
cargo test --all
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check

# Targeted pattern tests
./target/release/forma run --allow-all tests/forma/test_numeric_patterns.forma
./target/release/forma run --allow-all tests/forma/test_collection_patterns.forma
./target/release/forma run --allow-all tests/forma/test_sequence_patterns.forma
./target/release/forma run --allow-all tests/forma/test_sorting_patterns.forma

# New regression repros (must NOT panic)
./target/release/forma run --allow-all tests/forma/test_rotated_edge_cases.forma
./target/release/forma run --allow-all tests/forma/test_stable_pattern.forma
```

CI expectation: existing `.forma` glob discovery should pick up new test files automatically.

---

## Out of Scope

1. User-defined custom patterns
2. Calling arbitrary user functions from contract expressions (e.g., `@monotonic`)
3. SMT/static verification backend

---

## Definition of Done

1. No panic path remains for `@rotated`.
2. `@stable` rejects non-permutation outputs and preserves stable semantics.
3. Docs accurately describe implemented semantics and inference rules.
4. Missing pattern coverage is added and regression-proofed.
5. All verification commands pass with zero warnings.
