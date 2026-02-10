# Sprint 51: Standard Library Completeness

## Goal

Implement all functions documented in `reference.html` that are currently missing. This sprint closes the gap between documentation and implementation, ensuring every code example on the website actually works.

**Scope:** 18 new builtins + 3 stdlib functions (plus aliases), plus docs and website updates.

---

## Background

A thorough audit of `reference.html` against the actual codebase revealed 18 documented functions that don't exist (plus stdlib alignment work). These fall into five categories:

1. **Functional collection operations** — `map`, `filter`, `reduce`, `any`, `all` (highest visibility, shown in multiple code examples)
2. **Wrong/missing names** — functions exist under different names than documented (`str_replace` vs `str_replace_all`, `shuffle` vs `random_shuffle`, type-specific sorts vs generic `vec_sort`)
3. **Missing math** — `log2`, `asin`, `acos`, `atan2` (exist in LLVM codegen but not interpreter), plus `str_to_float`
4. **Missing I/O** — `file_read_lines`, `file_write_lines`, `file_read_bytes`, `file_write_bytes`
5. **Option/Result chaining** — `map_opt`, `flatten`, `and_then`

### Builtin vs Stdlib Split

FORMA's existing convention:
- **Builtins** (`interp.rs`): primitives that can't be written in FORMA — I/O, Rust stdlib access, closure invocation
- **Stdlib** (`std/`): higher-level functions written in FORMA that compose builtins

Applying this principle:

| Function | Layer | Reason |
|----------|-------|--------|
| `map`, `filter`, `reduce`, `any`, `all` | **Builtin** | Must invoke a closure argument — no `call(fn, arg)` primitive exists for stdlib to use |
| `vec_sort` (generic) | **Builtin** | Needs runtime type dispatch for comparison; existing `sort_ints`/`sort_floats` are builtins |
| `vec_index_of` (generic) | **Builtin** | Consistency with `vec_contains` (builtin); needs `Value` equality at runtime |
| `str_replace`, `random_shuffle` | **Builtin** (alias) | Aliasing existing builtins |
| `str_to_float`, `log2`, `asin`, `acos`, `atan2` | **Builtin** | Need Rust `f64` methods — can't be written in FORMA |
| `clamp` | **Stdlib** (already exists) | Pure arithmetic; already in `std/core.forma` as Int-only. Upgrade to support Float too |
| `file_read_lines`, `file_write_lines` | **Stdlib** | Composable from `file_read` + `str_split` and `str_join` + `file_write` |
| `file_read_bytes`, `file_write_bytes` | **Builtin** | Need raw byte access via `std::fs::read`/`std::fs::write` |
| `map_opt`, `and_then` | **Builtin** | Must invoke a closure argument |
| `flatten` | **Builtin** | Pure value inspection, but belongs with `map_opt`/`and_then` for consistency |

### Reference: What Exists Today

| Documented | Actual | Location |
|-----------|--------|----------|
| `str_replace(s, old, new)` | `str_replace_all(s, pat, rep)` | interp.rs:2848 |
| `vec_sort(arr)` | `sort_ints` / `sort_floats` / `sort_strings` | interp.rs:8934+ |
| `vec_index_of(arr, x)` | `int_vec_index_of` (stdlib, Int-only) | std/vec.forma:167 |
| `random_shuffle(arr)` | `shuffle(arr)` | interp.rs:9046 |
| `clamp(v, lo, hi)` | `clamp` (stdlib, Int-only) | std/core.forma:65 |
| `map`, `filter`, etc. | — | not implemented |
| `str_to_float` | — | LLVM only (llvm.rs:1404) |
| `log2`, `asin`, `acos`, `atan2` | — | LLVM only |
| `file_read_lines` etc. | — | not implemented |
| `map_opt`, `flatten`, `and_then` | — | not implemented |

---

## Scope

### 51.1 — Functional Collection Operations (P0)

**Priority:** P0 — these are shown in multiple prominent code examples on the website
**Layer:** Builtin
**File:** `src/mir/interp.rs`

Add 5 higher-order builtins that accept closure values as arguments:

#### `map(arr, fn) -> [T]`
```
map([1, 2, 3], |x| x * 2)  # [2, 4, 6]
```
- Takes an array and a closure
- Returns a new array with `fn` applied to each element
- The function argument is a `Value::Closure` — use existing closure invocation via `call_function_internal(...)` (pattern used by `http_serve`)

#### `filter(arr, fn) -> [T]`
```
filter([1, 2, 3, 4], |x| x % 2 == 0)  # [2, 4]
```
- Takes an array and a predicate closure
- Returns a new array containing only elements where `fn(elem)` returns `true`

#### `reduce(arr, init, fn) -> T`
```
reduce([1, 2, 3], 0, |acc, x| acc + x)  # 6
```
- Takes an array, initial accumulator, and a 2-argument function
- Folds left: `fn(fn(fn(init, arr[0]), arr[1]), arr[2])`

#### `any(arr, fn) -> Bool`
```
any([1, 2, 3], |x| x > 2)  # true
```
- Returns `true` if `fn` returns `true` for any element
- Short-circuits on first `true`

#### `all(arr, fn) -> Bool`
```
all([1, 2, 3], |x| x > 0)  # true
```
- Returns `true` if `fn` returns `true` for every element
- Short-circuits on first `false`

#### Implementation Notes

The key challenge is invoking a `Value::Closure` from within `call_builtin`. Use the same closure call pattern already used in `http_serve` (`captures + runtime args`, then `call_function_internal(...)`). The function value is passed as a regular argument.

Approach:
1. Extract the closure value from args
2. For each element, invoke it using the interpreter's existing call machinery
3. Collect results into new array

These builtins do NOT need capability gating (pure computation).

---

### 51.2 — Generic `vec_sort` and `vec_index_of` (P0)

**Priority:** P0 — documented as generic in reference.html
**Layer:** Builtin
**File:** `src/mir/interp.rs`

#### `vec_sort(arr) -> [T]`
```
vec_sort([3, 1, 2])        # [1, 2, 3]
vec_sort([3.1, 1.2, 2.3])  # [1.2, 2.3, 3.1]
vec_sort(["c", "a", "b"])  # ["a", "b", "c"]
```
- New generic builtin that dispatches on element type at runtime
- Inspect `arr[0]` to determine type, then sort accordingly:
  - `Value::Int` → sort by i64 comparison
  - `Value::Float` → sort by f64 comparison (handle NaN: push to end)
  - `Value::Str` → sort lexicographically
  - `Value::Char` → sort by char value
  - Other types → return error "vec_sort: unsupported element type"
- Returns a new sorted array (does not mutate)
- Keep existing `sort_ints` / `sort_floats` / `sort_strings` / `sort_strings_desc` / `sort_ints_desc` as-is — do NOT remove them

#### `vec_index_of(arr, target) -> Int?`
```
vec_index_of([10, 20, 30], 20)  # Some(1)
vec_index_of([10, 20, 30], 99)  # None
```
- Generic builtin using `Value` equality (`==` / `PartialEq`)
- Works for any element type (Int, Float, Str, Char, Bool, etc.)
- Returns `Some(index)` for first match, `None` if not found

---

### 51.3 — Name Aliases (P1)

**Priority:** P1
**Layer:** Builtin
**File:** `src/mir/interp.rs`

Add match arms that delegate to existing builtins:

#### `str_replace(s, pattern, replacement) -> Str`
- Alias for `str_replace_all` — same semantics (replace all occurrences)
- Add `"str_replace" =>` match arm that falls through to or duplicates `str_replace_all` logic
- reference.html shows it as replacing all occurrences, so `str_replace_all` semantics are correct

#### `random_shuffle(arr) -> [T]`
- Alias for `shuffle` — identical behavior
- Add `"random_shuffle" =>` arm that delegates to `shuffle` logic

Both are trivial — just add match arms that call the same code.

---

### 51.4 — Missing Math Builtins (P1)

**Priority:** P1
**Layer:** Builtin
**File:** `src/mir/interp.rs`

These exist in LLVM codegen but are missing from the interpreter:

#### `str_to_float(s: Str) -> Float?`
```
str_to_float("3.14")  # Some(3.14)
str_to_float("abc")   # None
```
- Parse string to f64 using `s.parse::<f64>()`
- Return `Some(Value::Float(f))` on success, `None` on failure
- Pattern: same as `str_to_int` but for floats

#### `log2(x: Float) -> Float`
```
log2(8.0)  # 3.0
```
- `f64::log2()` — one-liner

#### `asin(x: Float) -> Float`
```
asin(0.0)  # 0.0
```
- `f64::asin()` — one-liner

#### `acos(x: Float) -> Float`
```
acos(1.0)  # 0.0
```
- `f64::acos()` — one-liner

#### `atan2(y: Float, x: Float) -> Float`
```
atan2(1.0, 1.0)  # 0.785...
```
- `f64::atan2(other)` — takes 2 args

All math builtins: no capability gating needed (pure computation).

---

### 51.5 — File I/O Extensions (P1)

**Layer:** Mixed — `file_read_bytes`/`file_write_bytes` are builtins, `file_read_lines`/`file_write_lines` are stdlib

#### Builtins (`src/mir/interp.rs`)

##### `file_read_bytes(path: Str) -> Result[[Int], Str]`
```
bytes = file_read_bytes("image.png")?
```
- Read file as raw bytes, return array of Int (0-255)
- Implementation: `std::fs::read(path)` → map bytes to `Value::Int`
- Gate: `require_capability("read")`

##### `file_write_bytes(path: Str, bytes: [Int]) -> Result[(), Str]`
```
file_write_bytes("output.bin", bytes)?
```
- Write array of Int (0-255) as raw bytes to file
- Implementation: collect `Value::Int` → `u8`, `std::fs::write`
- Validate each value is 0-255, error otherwise
- Gate: `require_capability("write")`

#### Stdlib (`std/io.forma`)

##### `file_read_lines(path: Str) -> Result[[Str], Str]`
```
lines = file_read_lines("data.txt")?
```
- Implementation in FORMA: `file_read(path)` then `str_split(content, "\n")`
- Wraps existing builtins — no new Rust code needed

##### `file_write_lines(path: Str, lines: [Str]) -> Result[(), Str]`
```
file_write_lines("output.txt", ["line1", "line2"])?
```
- Implementation in FORMA: `str_join(lines, "\n")` then `file_write(path, content)`
- Wraps existing builtins — no new Rust code needed

---

### 51.6 — Option/Result Chaining (P1)

**Priority:** P1
**Layer:** Builtin
**File:** `src/mir/interp.rs`

#### `map_opt(opt, fn) -> Option[T]`
```
map_opt(Some(5), |x| x * 2)   # Some(10)
map_opt(None, |x| x * 2)      # None
```
- If `Some(v)`, apply `fn(v)` and wrap in `Some`
- If `None`, return `None`
- Uses same closure-invocation pattern as `map`/`filter`

#### `flatten(opt) -> Option[T]`
```
flatten(Some(Some(42)))  # Some(42)
flatten(Some(None))      # None
flatten(None)            # None
```
- If `Some(Some(v))`, return `Some(v)`
- If `Some(None)` or `None`, return `None`
- Pure value inspection, no closure needed

#### `and_then(opt, fn) -> Option[T]`
```
and_then(Some(5), |x| if x > 0 then Some(x * 2) else None)  # Some(10)
and_then(None, |x| Some(x * 2))                               # None
```
- If `Some(v)`, apply `fn(v)` (which returns an Option) — do NOT wrap in extra Some
- If `None`, return `None`
- Difference from `map_opt`: the function itself returns `Option`, so no double-wrapping

**Note:** The `.and_then().map()` method chaining syntax shown in reference.html is a separate language feature (method call syntax on Option/Result values). This is **out of scope** for Sprint 51 — these are implemented as standalone functions, not method calls. The reference.html examples using method chaining syntax should be updated to use the function-call form. Add a TODO note for a future sprint to consider method syntax sugar.

---

### 51.7 — Stdlib: `clamp` Upgrade + `file_read_lines`/`file_write_lines` (P1)

**Priority:** P1
**Layer:** Stdlib
**Files:** `std/core.forma`, `std/io.forma`

#### `clamp` in `std/core.forma`
The existing `clamp(value: Int, min_val: Int, max_val: Int) -> Int` is Int-only. Add a Float version:

```forma
# Clamp a float between min and max
f clamp_float(value: Float, min_val: Float, max_val: Float) -> Float
    if value < min_val then min_val
    else if value > max_val then max_val
    else value
```

**Decision:** Keep `clamp` as Int-only in stdlib (already works). Add `clamp_float` alongside it. reference.html should show both. No builtin needed — these are pure arithmetic expressible in FORMA.

#### `file_read_lines` and `file_write_lines` in `std/io.forma`

```forma
# Read file as array of lines
f file_read_lines(path: Str) -> Result[[Str], Str]
    m file_read(path)
        Ok(content) -> Ok(str_split(content, "\n"))
        Err(e) -> Err(e)

# Write array of lines to file
f file_write_lines(path: Str, lines: [Str]) -> Result[(), Str]
    content := str_join(lines, "\n")
    file_write(path, content)
```

`str_join` is in `std/string.forma`, so this sprint must explicitly resolve module visibility behavior.

**Task 51.7a — Module visibility contract (required):**
- Decide and document one supported behavior:
  - `us std.io` is sufficient because stdlib modules are globally visible at compile/load time, or
  - callers must also `us std.string` before using `file_write_lines`
- Add a regression test that fails if the documented behavior changes.
- Update docs/examples to match the chosen behavior exactly.

---

### 51.8 — Type Inference Updates (P1)

**Priority:** P1
**File:** `src/types/inference.rs`

Add type signatures for all new builtins to the type checker builtin environment (`TypeEnv::with_builtins()`) so type inference works correctly:

- `map`: `([T], (T) -> U) -> [U]`
- `filter`: `([T], (T) -> Bool) -> [T]`
- `reduce`: `([T], U, (U, T) -> U) -> U`
- `any`: `([T], (T) -> Bool) -> Bool`
- `all`: `([T], (T) -> Bool) -> Bool`
- `vec_sort`: `([T]) -> [T]`
- `vec_index_of`: `([T], T) -> Option[Int]`
- `str_replace`: `(Str, Str, Str) -> Str`
- `random_shuffle`: `([T]) -> [T]`
- `str_to_float`: `(Str) -> Option[Float]`
- `log2`, `asin`, `acos`: `(Float) -> Float`
- `atan2`: `(Float, Float) -> Float`
- `file_read_bytes`: `(Str) -> Result[[Int], Str]`
- `file_write_bytes`: `(Str, [Int]) -> Result[(), Str]`
- `map_opt`: `(Option[T], (T) -> U) -> Option[U]`
- `flatten`: `(Option[Option[T]]) -> Option[T]`
- `and_then`: `(Option[T], (T) -> Option[U]) -> Option[U]`

Check the existing `env.bindings.insert(...)` builtin entries in `TypeEnv::with_builtins()` for the pattern.

Note: `file_read_lines`, `file_write_lines`, `clamp`, `clamp_float` are stdlib functions — they don't need entries in the builtin type registry (they get their types from FORMA source).

---

### 51.9 — LSP Updates (P2)

**Priority:** P2
**File:** `src/lsp/mod.rs`

Update LSP completion items for new builtins. The LSP already has aspirational entries for `map` and `filter` — update them with accurate signatures and add entries for all other new builtins.

Also update `get_signature_help()` for the new builtins so hover/signature help shows correct parameter info.

---

### 51.10 — Tests (P0)

**Priority:** P0
**Files:** `src/mir/interp.rs` (unit tests), `tests/cli_tests.rs`, `tests/forma/`

#### Unit tests in interp.rs (Rust)
Add `call_builtin` unit tests for each new builtin (minimum 2 per builtin):
- `map`: basic transform, empty array
- `filter`: basic filter, empty result
- `reduce`: sum, empty array with init
- `any`: true case, false case
- `all`: true case, false case
- `vec_sort`: Int array, Float array, Str array, empty array
- `vec_index_of`: found, not found, empty array
- `str_replace`: basic replace, no match
- `random_shuffle`: length preserved (can't test order due to randomness)
- `str_to_float`: valid, invalid, edge cases (negative, scientific notation)
- `log2`: known value (8.0→3.0)
- `asin`, `acos`, `atan2`: known values
- `clamp`: Int below/above/in-range (via .forma test since it's stdlib)
- `file_read_bytes`, `file_write_bytes`: round-trip test
- `map_opt`: Some case, None case
- `flatten`: Some(Some), Some(None), None
- `and_then`: Some→Some, Some→None, None

#### Required negative-path tests (Rust + integration)
- Higher-order builtins (`map`, `filter`, `reduce`, `any`, `all`, `map_opt`, `and_then`):
  - non-closure function argument
  - wrong closure arity (e.g., unary closure passed to `reduce`)
  - wrong closure return type (`filter` predicate not Bool, `and_then` not Option)
- `vec_sort`:
  - mixed-type arrays rejected with deterministic error
  - unsupported element type rejected
  - NaN ordering behavior tested and documented
- `file_write_bytes`:
  - byte values < 0 or > 255 rejected
  - non-Int array elements rejected
- Capability enforcement:
  - `file_read_bytes` denied without `read`
  - `file_write_bytes` denied without `write`

#### Integration tests (.forma files)
- `tests/forma/test_functional.forma` — NEW: exercises `map`, `filter`, `reduce`, `any`, `all` with various types
- `tests/forma/test_option_chaining.forma` — NEW: exercises `map_opt`, `flatten`, `and_then`
- `tests/forma/test_file_io_extended.forma` — NEW: exercises `file_read_lines`, `file_write_lines`, `file_read_bytes`, `file_write_bytes`

#### CLI integration tests
- `tests/cli_tests.rs`: add tests for the new .forma fixtures

---

### 51.11 — Documentation & Website Updates (P1)

**Priority:** P1

#### reference.html (forma-website)
- Fix the `.and_then().map()` method chaining example to use function-call syntax:
  ```
  # Before (method chaining — not yet supported)
  file_read(path)
      .and_then(|c| json_parse(c))
      .map(|v| transform(v))

  # After (function-call form)
  and_then(file_read(path), |content|
      map_opt(json_parse(content), |data|
          transform(data)))
  ```
- Verify all other code examples compile and run correctly
- Update `clamp` example to show it requires `us std.core` (or show both Int stdlib and note about Float)
- Update `file_read_lines`/`file_write_lines` to note they're in `std/io` (require `us std.io`)
- Add note about `std/` library modules (currently not mentioned on website)

#### docs/reference.md (forma repo)
- Add entries for all new builtins in the appropriate sections
- Add a "Functional Operations" section documenting `map`, `filter`, `reduce`, `any`, `all`
- Add `map_opt`, `flatten`, `and_then` to the Option/Result section
- Document stdlib modules (`std/core`, `std/io`, `std/vec`, `std/string`, `std/map`, `std/iter`)

#### Website technical.html
- No changes needed (already fixed in Sprint 50.1)

#### Coverage audit
- Update `scripts/builtin_coverage.sh` if needed to reflect new builtin count
- Verify coverage percentage doesn't regress

---

## Implementation Order

1. **51.4** — Math builtins + `str_to_float` (simplest, no closures, warm up)
2. **51.3** — Name aliases (`str_replace`, `random_shuffle`)
3. **51.2** — Generic `vec_sort` and `vec_index_of`
4. **51.5** — File I/O (builtins: `file_read_bytes`/`file_write_bytes`; stdlib: `file_read_lines`/`file_write_lines`)
5. **51.7** — Stdlib upgrades (`clamp_float` in core.forma)
6. **51.6** — Option/Result chaining (`map_opt`, `flatten`, `and_then`)
7. **51.1** — Functional collection operations (`map`, `filter`, `reduce`, `any`, `all`) — most complex, needs closure invocation
8. **51.8** — Type inference updates
9. **51.9** — LSP updates
10. **51.10** — Tests (write alongside each section above)
11. **51.11** — Documentation & website

---

## Verification

```bash
# Build and lint
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings

# All tests pass
cargo test --all

# New .forma integration tests
./target/release/forma run --allow-all tests/forma/test_functional.forma
./target/release/forma run --allow-all tests/forma/test_option_chaining.forma
./target/release/forma run --allow-all tests/forma/test_file_io_extended.forma

# Coverage audit
bash scripts/builtin_coverage.sh

# Verify reference.html examples work (spot check)
# Create temp .forma files from reference.html code blocks and run them
```

---

## Out of Scope

- **Method syntax on Option/Result** (`.map()`, `.and_then()`, `.filter()`) — requires parser/type system changes for method dispatch on enum types. Future sprint.
- **Lazy iterators** — `map`/`filter` return concrete arrays, not lazy iterators. Future sprint if performance matters.
- **Custom comparators for `vec_sort`** — `vec_sort(arr, |a, b| compare(a, b))` is a possible future extension. For now, only natural ordering.
- **`sort_by` with key function** — e.g., `sort_by(arr, |x| x.age)`. Future sprint.
- **`vec_sort_desc`** — generic descending sort. Can be added if needed but `vec_reverse(vec_sort(arr))` works today.
- **Generic `call(fn, args)` builtin** — would let stdlib implement `map`/`filter` etc. in FORMA. Interesting but adds complexity; builtins are simpler for now.

---

## File Summary

| File | Change | Layer |
|------|--------|-------|
| `src/mir/interp.rs` | ~18 new builtin match arms + unit tests | Builtin |
| `src/types/inference.rs` | Type signatures for new builtins in `TypeEnv::with_builtins()` | Builtin |
| `src/lsp/mod.rs` | Completion items and signature help for new builtins | Builtin |
| `std/core.forma` | Add `clamp_float` | Stdlib |
| `std/io.forma` | Add `file_read_lines`, `file_write_lines` | Stdlib |
| `tests/forma/test_functional.forma` | NEW — map/filter/reduce/any/all integration tests | Test |
| `tests/forma/test_option_chaining.forma` | NEW — map_opt/flatten/and_then integration tests | Test |
| `tests/forma/test_file_io_extended.forma` | NEW — file I/O extension tests | Test |
| `tests/cli_tests.rs` | CLI tests for new .forma fixtures | Test |
| `docs/reference.md` | New entries, functional ops section, stdlib module docs | Docs |
| `reference.html` (website) | Fix method chaining examples, add stdlib notes | Docs |
| `scripts/builtin_coverage.sh` | Update expected builtin count if needed | Infra |

### New Builtins (interp.rs): 18

| Builtin | Args | Category |
|---------|------|----------|
| `map` | `(arr, fn)` | Functional |
| `filter` | `(arr, fn)` | Functional |
| `reduce` | `(arr, init, fn)` | Functional |
| `any` | `(arr, fn)` | Functional |
| `all` | `(arr, fn)` | Functional |
| `vec_sort` | `(arr)` | Collection |
| `vec_index_of` | `(arr, target)` | Collection |
| `str_replace` | `(s, pat, rep)` | Alias → `str_replace_all` |
| `random_shuffle` | `(arr)` | Alias → `shuffle` |
| `str_to_float` | `(s)` | Conversion |
| `log2` | `(x)` | Math |
| `asin` | `(x)` | Math |
| `acos` | `(x)` | Math |
| `atan2` | `(y, x)` | Math |
| `file_read_bytes` | `(path)` | I/O |
| `file_write_bytes` | `(path, bytes)` | I/O |
| `map_opt` | `(opt, fn)` | Option |
| `flatten` | `(opt)` | Option |
| `and_then` | `(opt, fn)` | Option |

### New Stdlib Functions: 3

| Function | File | Category |
|----------|------|----------|
| `clamp_float` | `std/core.forma` | Math |
| `file_read_lines` | `std/io.forma` | I/O |
| `file_write_lines` | `std/io.forma` | I/O |
