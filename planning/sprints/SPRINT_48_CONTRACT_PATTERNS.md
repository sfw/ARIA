# Sprint 48: Comprehensive Contract Patterns

**Goal:** Expand FORMA's contract pattern library to match industry-standard verification languages (Dafny, Eiffel, JML).

**Estimated Effort:** 5-7 days

**Key Deliverables:**
- 20+ new named contract patterns
- Full parity with common verification idioms
- Comprehensive test coverage
- Documentation updates

---

## Context

FORMA currently has 12 named contract patterns. While useful, this falls short of what developers expect from verification languages like [Dafny](https://dafny.org/dafny/DafnyRef/DafnyRef), [Eiffel](https://www.eiffel.org/doc/eiffel/ET-_Design_by_Contract_(tm),_Assertions_and_Exceptions), and [JML](https://www.openjml.org/documentation/JML_Reference_Manual.pdf).

This sprint adds patterns commonly used in formal specification, making FORMA's contract system more expressive and familiar to developers with verification experience.

---

## Current Patterns (12)

| Pattern | Expansion | Context |
|---------|-----------|---------|
| `@nonempty(x)` | `x.len() > 0` | Pre |
| `@nonnegative(x)` | `x >= 0` | Both |
| `@positive(x)` | `x > 0` | Both |
| `@nonzero(x)` | `x != 0` | Both |
| `@bounded(x, lo, hi)` | `x >= lo && x <= hi` | Both |
| `@sorted(x)` | `forall i in 0..x.len()-1: x[i] <= x[i+1]` | Both |
| `@sorted_desc(x)` | `forall i in 0..x.len()-1: x[i] >= x[i+1]` | Both |
| `@unique(x)` | `forall i,j: i != j => x[i] != x[j]` | Both |
| `@same_length(a, b)` | `a.len() == b.len()` | Both |
| `@permutation(a, b)` | `permutation(a, b)` (builtin) | Both |
| `@unchanged(x)` | `x == old(x)` | Post |
| `@pure` | (marker - no side effects) | Post |

---

## New Patterns (24)

### Tier 1: Essential (P0) - 10 patterns

These patterns are commonly needed and frequently requested.

#### 48.1 Numeric Patterns

| Pattern | Expansion | Context | Use Case |
|---------|-----------|---------|----------|
| `@even(x)` | `x % 2 == 0` | Both | Parity checks |
| `@odd(x)` | `x % 2 != 0` | Both | Parity checks |
| `@divisible(x, n)` | `x % n == 0` | Both | Divisibility |
| `@in_range(x, lo, hi)` | `x > lo && x < hi` | Both | Exclusive bounds |

#### 48.2 Collection Membership Patterns

| Pattern | Expansion | Context | Use Case |
|---------|-----------|---------|----------|
| `@contains(arr, elem)` | `elem in arr` | Both | Membership tests |
| `@all_positive(arr)` | `forall x in arr: x > 0` | Both | Bulk checks |
| `@all_nonnegative(arr)` | `forall x in arr: x >= 0` | Both | Bulk checks |
| `@all_nonzero(arr)` | `forall x in arr: x != 0` | Both | Bulk checks |

#### 48.3 Index Patterns

| Pattern | Expansion | Context | Use Case |
|---------|-----------|---------|----------|
| `@valid_index(arr, i)` | `i >= 0 && i < arr.len()` | Both | Bounds checking |
| `@valid_range(arr, lo, hi)` | `lo >= 0 && hi <= arr.len() && lo <= hi` | Both | Slice bounds |

---

### Tier 2: Collection Relationships (P1) - 8 patterns

These patterns express relationships between collections.

#### 48.4 Set-like Patterns

| Pattern | Expansion | Context | Use Case |
|---------|-----------|---------|----------|
| `@subset(a, b)` | `forall x in a: x in b` | Both | Subset relationship |
| `@superset(a, b)` | `forall x in b: x in a` | Both | Superset relationship |
| `@disjoint(a, b)` | `forall x in a: !(x in b)` | Both | No overlap |
| `@equals(a, b)` | `subset(a, b) && subset(b, a)` | Both | Set equality |

#### 48.5 Sequence Patterns

| Pattern | Expansion | Context | Use Case |
|---------|-----------|---------|----------|
| `@prefix(a, b)` | `a.len() <= b.len() && forall i in 0..a.len(): a[i] == b[i]` | Both | Prefix check |
| `@suffix(a, b)` | `a.len() <= b.len() && forall i in 0..a.len(): a[i] == b[b.len()-a.len()+i]` | Both | Suffix check |
| `@reversed(a, b)` | `a.len() == b.len() && forall i in 0..a.len(): a[i] == b[a.len()-1-i]` | Both | Reverse check |
| `@rotated(a, b, k)` | `a.len() == b.len() && forall i in 0..a.len(): a[i] == b[(i+k) % b.len()]` | Both | Rotation |

---

### Tier 3: Sorting & Ordering (P1) - 4 patterns

These patterns support sorting algorithms and ordered data.

| Pattern | Expansion | Context | Use Case |
|---------|-----------|---------|----------|
| `@strictly_sorted(x)` | `forall i in 0..x.len()-1: x[i] < x[i+1]` | Both | Strictly increasing |
| `@strictly_sorted_desc(x)` | `forall i in 0..x.len()-1: x[i] > x[i+1]` | Both | Strictly decreasing |
| `@sorted_by(arr, field)` | `forall i in 0..arr.len()-1: arr[i].field <= arr[i+1].field` | Both | Sort by field |
| `@partitioned(arr, pivot_idx)` | `forall i in 0..pivot_idx: forall j in pivot_idx+1..arr.len(): arr[i] <= arr[pivot_idx] && arr[pivot_idx] <= arr[j]` | Both | Quicksort partition |

---

### Tier 4: Stability & Advanced (P2) - 2 patterns

These are complex patterns requiring additional implementation work.

| Pattern | Expansion | Context | Use Case |
|---------|-----------|---------|----------|
| `@stable(input, output, key)` | (see implementation below) | Post | Stable sort |
| `@monotonic(f, x, y)` | `x <= y => f(x) <= f(y)` | Both | Function monotonicity |

---

## Implementation Details

### 48.1-48.3: Simple Expansion Patterns

Most patterns are simple expansions in `parser.rs`. Add after line 608:

```rust
// === Sprint 48: New Numeric Patterns ===
"even" => {
    let x = Self::pattern_arg_name(attr, 0)?;
    (
        format!("{} % 2 == 0", x),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}
"odd" => {
    let x = Self::pattern_arg_name(attr, 0)?;
    (
        format!("{} % 2 != 0", x),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}
"divisible" => {
    let x = Self::pattern_arg_name(attr, 0)?;
    let n = Self::pattern_arg_name(attr, 1)?;
    (
        format!("{} % {} == 0", x, n),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}
"in_range" => {
    let x = Self::pattern_arg_name(attr, 0)?;
    let lo = Self::pattern_arg_name(attr, 1)?;
    let hi = Self::pattern_arg_name(attr, 2)?;
    (
        format!("{} > {} && {} < {}", x, lo, x, hi),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}

// === Sprint 48: Collection Membership Patterns ===
"contains" => {
    let arr = Self::pattern_arg_name(attr, 0)?;
    let elem = Self::pattern_arg_name(attr, 1)?;
    (
        format!("{} in {}", elem, arr),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}
"all_positive" => {
    let arr = Self::pattern_arg_name(attr, 0)?;
    (
        format!("forall x in {}: x > 0", arr),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}
"all_nonnegative" => {
    let arr = Self::pattern_arg_name(attr, 0)?;
    (
        format!("forall x in {}: x >= 0", arr),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}
"all_nonzero" => {
    let arr = Self::pattern_arg_name(attr, 0)?;
    (
        format!("forall x in {}: x != 0", arr),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}

// === Sprint 48: Index Patterns ===
"valid_index" => {
    let arr = Self::pattern_arg_name(attr, 0)?;
    let i = Self::pattern_arg_name(attr, 1)?;
    (
        format!("{} >= 0 && {} < {}.len()", i, i, arr),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}
"valid_range" => {
    let arr = Self::pattern_arg_name(attr, 0)?;
    let lo = Self::pattern_arg_name(attr, 1)?;
    let hi = Self::pattern_arg_name(attr, 2)?;
    (
        format!("{} >= 0 && {} <= {}.len() && {} <= {}", lo, hi, arr, lo, hi),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}

// === Sprint 48: Set-like Patterns ===
"subset" => {
    let a = Self::pattern_arg_name(attr, 0)?;
    let b = Self::pattern_arg_name(attr, 1)?;
    (
        format!("forall x in {}: x in {}", a, b),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}
"superset" => {
    let a = Self::pattern_arg_name(attr, 0)?;
    let b = Self::pattern_arg_name(attr, 1)?;
    (
        format!("forall x in {}: x in {}", b, a),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}
"disjoint" => {
    let a = Self::pattern_arg_name(attr, 0)?;
    let b = Self::pattern_arg_name(attr, 1)?;
    (
        format!("forall x in {}: !(x in {})", a, b),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}

// === Sprint 48: Sequence Patterns ===
"prefix" => {
    let a = Self::pattern_arg_name(attr, 0)?;
    let b = Self::pattern_arg_name(attr, 1)?;
    (
        format!("{}.len() <= {}.len() && forall i in 0..{}.len(): {}[i] == {}[i]", a, b, a, a, b),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}
"suffix" => {
    let a = Self::pattern_arg_name(attr, 0)?;
    let b = Self::pattern_arg_name(attr, 1)?;
    (
        format!("{}.len() <= {}.len() && forall i in 0..{}.len(): {}[i] == {}[{}.len()-{}.len()+i]",
                a, b, a, a, b, b, a),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}
"reversed" => {
    let a = Self::pattern_arg_name(attr, 0)?;
    let b = Self::pattern_arg_name(attr, 1)?;
    (
        format!("{}.len() == {}.len() && forall i in 0..{}.len(): {}[i] == {}[{}.len()-1-i]",
                a, b, a, a, b, a),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}

// === Sprint 48: Sorting Patterns ===
"strictly_sorted" => {
    let x = Self::pattern_arg_name(attr, 0)?;
    (
        format!("forall i in 0..{}.len()-1: {}[i] < {}[i+1]", x, x, x),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}
"strictly_sorted_desc" => {
    let x = Self::pattern_arg_name(attr, 0)?;
    (
        format!("forall i in 0..{}.len()-1: {}[i] > {}[i+1]", x, x, x),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}
"sorted_by" => {
    let arr = Self::pattern_arg_name(attr, 0)?;
    let field = Self::pattern_arg_name(attr, 1)?;
    (
        format!("forall i in 0..{}.len()-1: {}[i].{} <= {}[i+1].{}", arr, arr, field, arr, field),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}
"partitioned" => {
    let arr = Self::pattern_arg_name(attr, 0)?;
    let pivot = Self::pattern_arg_name(attr, 1)?;
    (
        format!("forall i in 0..{}: {}[i] <= {}[{}] && forall j in {}+1..{}.len(): {}[{}] <= {}[j]",
                pivot, arr, arr, pivot, pivot, arr, arr, pivot, arr),
        Self::is_result_context(attr),
        PatternContext::Both,
    )
}
```

### 48.5: `@stable` Pattern (Complex)

The `@stable` pattern requires runtime support in `interp.rs`:

```rust
// In eval_contract_expr, add to the function call match:
"stable" => {
    // stable(input, output, key_field)
    // Checks that elements with equal key values maintain their relative order
    let mut arg_vals = Vec::new();
    for arg in args {
        arg_vals.push(self.eval_contract_expr(&arg.value)?);
    }
    if arg_vals.len() != 3 {
        return Err(InterpError {
            message: "stable() takes 3 arguments: input, output, key_field".to_string(),
        });
    }
    match (&arg_vals[0], &arg_vals[1], &arg_vals[2]) {
        (Value::Array(input), Value::Array(output), Value::Str(key)) => {
            Ok(Value::Bool(Self::is_stable_sort(input, output, key)?))
        }
        _ => Err(InterpError {
            message: "stable() expects (Array, Array, Str)".to_string(),
        }),
    }
}

// Helper function:
fn is_stable_sort(input: &[Value], output: &[Value], key: &str) -> Result<bool, InterpError> {
    // For each pair in output with equal keys,
    // check that their relative order matches input

    // Build a map: (key_value, occurrence_index) -> original_position
    let mut input_positions: HashMap<(Value, usize), usize> = HashMap::new();
    let mut key_counts: HashMap<Value, usize> = HashMap::new();

    for (pos, val) in input.iter().enumerate() {
        let k = Self::get_field_value(val, key)?;
        let count = key_counts.entry(k.clone()).or_insert(0);
        input_positions.insert((k, *count), pos);
        *key_counts.get_mut(&k).unwrap() += 1;
    }

    // Reset counts for output traversal
    key_counts.clear();
    let mut prev_positions: HashMap<Value, usize> = HashMap::new();

    for val in output {
        let k = Self::get_field_value(val, key)?;
        let count = key_counts.entry(k.clone()).or_insert(0);

        if let Some(&orig_pos) = input_positions.get(&(k.clone(), *count)) {
            // Check stability: for same key, positions should be increasing
            if let Some(&prev_pos) = prev_positions.get(&k) {
                if orig_pos <= prev_pos {
                    return Ok(false); // Stability violated
                }
            }
            prev_positions.insert(k, orig_pos);
        }
        *key_counts.get_mut(&k).unwrap() += 1;
    }

    Ok(true)
}
```

Parser addition for `@stable`:

```rust
"stable" => {
    let input = Self::pattern_arg_name(attr, 0)?;
    let output = Self::pattern_arg_name(attr, 1)?;
    let key = Self::pattern_arg_name(attr, 2)?;
    (
        format!("stable({}, {}, \"{}\")", input, output, key),
        true,  // Always postcondition
        PatternContext::PostOnly,
    )
}
```

---

## English Translations for `forma explain`

Add to the explain translation logic in `main.rs`:

```rust
// Pattern-specific English translations
fn pattern_to_english(pattern: &str, args: &[String]) -> String {
    match pattern {
        "even" => format!("{} is even", args[0]),
        "odd" => format!("{} is odd", args[0]),
        "divisible" => format!("{} is divisible by {}", args[0], args[1]),
        "in_range" => format!("{} is strictly between {} and {}", args[0], args[1], args[2]),
        "contains" => format!("{} contains {}", args[0], args[1]),
        "all_positive" => format!("all elements in {} are positive", args[0]),
        "all_nonnegative" => format!("all elements in {} are non-negative", args[0]),
        "all_nonzero" => format!("all elements in {} are non-zero", args[0]),
        "valid_index" => format!("{} is a valid index for {}", args[1], args[0]),
        "valid_range" => format!("[{}, {}) is a valid range for {}", args[1], args[2], args[0]),
        "subset" => format!("{} is a subset of {}", args[0], args[1]),
        "superset" => format!("{} is a superset of {}", args[0], args[1]),
        "disjoint" => format!("{} and {} have no common elements", args[0], args[1]),
        "prefix" => format!("{} is a prefix of {}", args[0], args[1]),
        "suffix" => format!("{} is a suffix of {}", args[0], args[1]),
        "reversed" => format!("{} is the reverse of {}", args[0], args[1]),
        "strictly_sorted" => format!("{} is strictly sorted (no duplicates)", args[0]),
        "strictly_sorted_desc" => format!("{} is strictly sorted descending (no duplicates)", args[0]),
        "sorted_by" => format!("{} is sorted by {} field", args[0], args[1]),
        "partitioned" => format!("{} is partitioned at index {}", args[0], args[1]),
        "stable" => format!("relative order of equal-{} elements is preserved from {} to {}",
                           args[2], args[0], args[1]),
        _ => format!("@{}({})", pattern, args.join(", ")),
    }
}
```

---

## Test Files

### `tests/forma/test_numeric_patterns.forma`

```forma
# Numeric pattern tests

@even(n)
@post(result == n / 2)
f half_even(n: Int) -> Int
    n / 2

@odd(n)
@post(@odd(result))
f double_odd(n: Int) -> Int
    n * 2 + 1

@divisible(n, 3)
@post(@divisible(result, 3))
f triple(n: Int) -> Int
    n * 3

@in_range(x, 0, 100)
@post(@bounded(result, 1, 99))
f clamp_exclusive(x: Int) -> Int
    x

f main() -> Int
    _ := half_even(10)
    _ := double_odd(5)
    _ := triple(9)
    _ := clamp_exclusive(50)
    0
```

### `tests/forma/test_collection_patterns.forma`

```forma
# Collection pattern tests

@all_positive(items)
@post(@all_positive(result))
f double_all(items: [Int]) -> [Int]
    result := vec_new()
    for x in items
        vec_push(result, x * 2)
    result

@contains(items, target)
@post(@contains(result, target))
f with_target(items: [Int], target: Int) -> [Int]
    items

@nonempty(items)
@post(@valid_index(result, 0))
f first_index_valid(items: [Int]) -> [Int]
    items

@subset(a, b)
f check_subset(a: [Int], b: [Int]) -> Bool
    true

@disjoint(a, b)
f check_disjoint(a: [Int], b: [Int]) -> Bool
    true

f main() -> Int
    _ := double_all([1, 2, 3])
    _ := with_target([1, 2, 3], 2)
    _ := first_index_valid([1])
    0
```

### `tests/forma/test_sequence_patterns.forma`

```forma
# Sequence pattern tests

@nonempty(items)
@post(@prefix(items, result))
f append_one(items: [Int]) -> [Int]
    vec_push(vec_clone(items), 0)

@same_length(a, b)
@post(@reversed(a, result))
f reverse_check(a: [Int], b: [Int]) -> [Int]
    result := vec_new()
    i := a.len() - 1
    wh i >= 0
        vec_push(result, a[i])
        i := i - 1
    result

f main() -> Int
    _ := append_one([1, 2])
    _ := reverse_check([1, 2, 3], [3, 2, 1])
    0
```

### `tests/forma/test_sorting_patterns.forma`

```forma
# Sorting pattern tests

@nonempty(items)
@strictly_sorted(result)
@permutation(items, result)
f sort_unique(items: [Int]) -> [Int]
    # For items known to be unique
    sort_ints(items)

s Pair { key: Int, value: Int }

@nonempty(items)
@sorted_by(result, key)
@permutation(items, result)
f sort_by_key(items: [Pair]) -> [Pair]
    # insertion sort by key field
    result := vec_clone(items)
    i := 1
    wh i < vec_len(result)
        j := i
        wh j > 0 and result[j-1].key > result[j].key
            temp = result[j]
            result[j] := result[j-1]
            result[j-1] := temp
            j := j - 1
        i := i + 1
    result

@nonempty(items)
@sorted_by(result, key)
@stable(items, result, "key")
f stable_sort_by_key(items: [Pair]) -> [Pair]
    # insertion sort is naturally stable
    result := vec_clone(items)
    i := 1
    wh i < vec_len(result)
        current = result[i]
        j := i - 1
        wh j >= 0 and result[j].key > current.key
            result[j + 1] := result[j]
            j := j - 1
        result[j + 1] := current
        i := i + 1
    result

f main() -> Int
    _ := sort_unique([3, 1, 2])
    pairs := [Pair { key: 2, value: 1 }, Pair { key: 1, value: 2 }, Pair { key: 1, value: 3 }]
    _ := sort_by_key(pairs)
    _ := stable_sort_by_key(pairs)
    0
```

---

## Documentation Updates

### Update `docs/reference.md` Contracts Section

Add new patterns table:

```markdown
### Named Contract Patterns

FORMA provides 36 named patterns for common specifications:

#### Numeric Patterns
| Pattern | Meaning | Example |
|---------|---------|---------|
| `@positive(x)` | x > 0 | `@positive(count)` |
| `@nonnegative(x)` | x >= 0 | `@nonnegative(age)` |
| `@nonzero(x)` | x != 0 | `@nonzero(divisor)` |
| `@even(x)` | x % 2 == 0 | `@even(array_size)` |
| `@odd(x)` | x % 2 != 0 | `@odd(n)` |
| `@divisible(x, n)` | x % n == 0 | `@divisible(total, 10)` |
| `@bounded(x, lo, hi)` | lo <= x <= hi | `@bounded(percent, 0, 100)` |
| `@in_range(x, lo, hi)` | lo < x < hi | `@in_range(temp, 0, 100)` |

#### Collection Patterns
| Pattern | Meaning | Example |
|---------|---------|---------|
| `@nonempty(x)` | x.len() > 0 | `@nonempty(items)` |
| `@contains(arr, elem)` | elem in arr | `@contains(result, target)` |
| `@all_positive(arr)` | all elements > 0 | `@all_positive(prices)` |
| `@all_nonnegative(arr)` | all elements >= 0 | `@all_nonnegative(counts)` |
| `@all_nonzero(arr)` | all elements != 0 | `@all_nonzero(divisors)` |
| `@valid_index(arr, i)` | 0 <= i < arr.len() | `@valid_index(items, idx)` |
| `@valid_range(arr, lo, hi)` | valid slice bounds | `@valid_range(arr, start, end)` |

#### Set Relationships
| Pattern | Meaning | Example |
|---------|---------|---------|
| `@subset(a, b)` | all of a in b | `@subset(selected, available)` |
| `@superset(a, b)` | all of b in a | `@superset(all_items, subset)` |
| `@disjoint(a, b)` | no overlap | `@disjoint(evens, odds)` |
| `@same_length(a, b)` | a.len() == b.len() | `@same_length(keys, values)` |
| `@permutation(a, b)` | same multiset | `@permutation(input, output)` |

#### Sequence Relationships
| Pattern | Meaning | Example |
|---------|---------|---------|
| `@prefix(a, b)` | a starts b | `@prefix(header, message)` |
| `@suffix(a, b)` | a ends b | `@suffix(extension, filename)` |
| `@reversed(a, b)` | a is reverse of b | `@reversed(result, input)` |
| `@unique(x)` | no duplicates | `@unique(ids)` |

#### Ordering Patterns
| Pattern | Meaning | Example |
|---------|---------|---------|
| `@sorted(x)` | ascending order | `@sorted(result)` |
| `@sorted_desc(x)` | descending order | `@sorted_desc(rankings)` |
| `@strictly_sorted(x)` | ascending, no equals | `@strictly_sorted(unique_ids)` |
| `@strictly_sorted_desc(x)` | descending, no equals | `@strictly_sorted_desc(scores)` |
| `@sorted_by(arr, field)` | sorted by field | `@sorted_by(users, age)` |
| `@partitioned(arr, pivot)` | quicksort partition | `@partitioned(arr, mid)` |
| `@stable(in, out, key)` | stable sort | `@stable(input, result, "priority")` |

#### State Patterns
| Pattern | Meaning | Example |
|---------|---------|---------|
| `@unchanged(x)` | x == old(x) | `@unchanged(config)` |
| `@pure` | no side effects | `@pure` |
```

---

## Verification Checklist

```bash
# Build and test
cargo build --release
cargo test --all
cargo clippy --all-targets -- -D warnings

# Pattern-specific tests
./target/release/forma run tests/forma/test_numeric_patterns.forma
./target/release/forma run tests/forma/test_collection_patterns.forma
./target/release/forma run tests/forma/test_sequence_patterns.forma
./target/release/forma run tests/forma/test_sorting_patterns.forma
./target/release/forma run tests/forma/test_patterns.forma  # existing

# Explain command with new patterns
./target/release/forma explain tests/forma/test_sorting_patterns.forma

# Verify report
./target/release/forma verify --report tests/forma/
```

---

## Summary: Final Pattern Count

| Category | Current | New | Total |
|----------|---------|-----|-------|
| Numeric | 4 | 4 | 8 |
| Collection | 2 | 6 | 8 |
| Set Relations | 2 | 4 | 6 |
| Sequence | 1 | 4 | 5 |
| Ordering | 2 | 5 | 7 |
| State | 2 | 0 | 2 |
| **Total** | **12** | **24** | **36** |

---

## Definition of Done

1. All 24 new patterns implemented in parser
2. `@stable` builtin function implemented in interpreter
3. English translations for all patterns in `forma explain`
4. Unit tests for each new pattern
5. Integration tests pass
6. Documentation updated (reference.md, website)
7. All existing tests still pass
8. `forma verify --report` works with new patterns

---

## References

- [Dafny Reference Manual](https://dafny.org/dafny/DafnyRef/DafnyRef)
- [Eiffel Design by Contract](https://www.eiffel.org/doc/eiffel/ET-_Design_by_Contract_(tm),_Assertions_and_Exceptions)
- [JML Reference Manual](https://www.openjml.org/documentation/JML_Reference_Manual.pdf)
- [Design by Contract - Wikipedia](https://en.wikipedia.org/wiki/Design_by_contract)
