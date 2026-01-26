# FORMA v1.0 Known Limitations

**Last Updated:** January 26, 2026
**Status:** Post-Sprint 14.7

This document lists known limitations in FORMA v1.0, categorized by severity and planned resolution timeline.

---

## Summary

| Category | Fixed | Remaining | Severity |
|----------|-------|-----------|----------|
| Language Features | 2 | 3 | Medium |
| Type System | 3 | 3 | High |
| Tooling | 1 | 5 | Medium-High |
| Standard Library | 2 | 2 | Low-Medium |
| Parser/Lexer | 2 | 1 | Low |

---

## Recently Fixed (Sprint 14-14.7)

These issues were identified in code reviews and have been resolved:

| Issue | Sprint | Resolution |
|-------|--------|------------|
| Enum discriminant hash collision | 14 | Index-based registry |
| Call stack unwrap panics | 14.7 | Helper methods with Result |
| MIR lowerer unwrap panics | 14.7 | Helper methods with Result |
| Borrow checker unwrap panics | 14.7 | Helper methods with Result |
| String/Bool/Char/Float pattern matching | 14.7 | Added literal handlers in lower_match() |
| Import system silent failures | 14.6 | Returns ModuleError, searches std/ |
| JSON type mapping | 14.5 | Added "Json" => Ty::Json |
| Async example broken | 14.5 | Fixed vec_push reassignment |
| Contextual keyword conflicts (m/s/f/e/t/i) | 12 | Context-aware parser lookahead |

---

## Language Features

### 1. Struct Update Syntax (Medium Priority)
**Status:** Parsed but not implemented
**Location:** `src/mir/lower.rs` line 962-964

The struct update syntax `{ ..base, field: value }` is parsed but the MIR lowering has a TODO comment and doesn't generate correct code.

```forma
# This parses but doesn't work correctly:
new_point := { ..old_point, x: 10 }
```

**Workaround:** Create a new struct with all fields specified explicitly.

**Planned:** v1.1

---

### 2. Loop Labels (Low Priority)
**Status:** Not implemented
**Location:** `src/parser/parser.rs` line 1770

Loop labels for `break` and `continue` (`break 'outer`) are not implemented. The parser has a placeholder `let label = None`.

```forma
# Not supported:
'outer: for x in items
    for y in other
        if condition
            break 'outer  # Can't break to outer loop
```

**Workaround:** Restructure code to avoid needing labeled breaks, or use a boolean flag.

**Planned:** v1.2

---

### 3. Indirect Closure Calls (Medium Priority)
**Status:** Interpreter works, LLVM codegen doesn't
**Location:** `src/codegen/llvm.rs` line 582

LLVM codegen doesn't support indirect calls for closures stored in variables.

```forma
# Works in interpreter, not in compiled code:
callback := |x| x + 1
result := callback(5)  # Fails in LLVM
```

**Workaround:** Use direct function calls or interpreter mode.

**Planned:** v1.1

---

## Type System

### 4. Trait Implementation Checking (High Priority)
**Status:** Minimal validation
**Location:** `src/types/inference.rs` line 3104

User-defined impl blocks and trait implementations aren't fully checked. The type checker doesn't validate that impl methods match trait signatures.

```forma
t Printable
    f to_string(self) -> Str

s Point { x: Int, y: Int }

i Printable for Point
    # Missing to_string - should error but doesn't
```

**Workaround:** Manually ensure impl blocks are complete.

**Planned:** v1.1

---

### 5. Higher-Kinded Types (Not Planned)
**Status:** Not supported

Higher-kinded types (type constructors as parameters) are not supported and not planned for v1.x.

```forma
# Not supported:
f map_functor[F[_], A, B](fa: F[A], f: (A) -> B) -> F[B]
```

**Rationale:** Complexity vs. benefit for AI code generation use case.

**Planned:** v2.0 (maybe)

---

### 6. Enum Pattern Validation (Medium Priority)
**Status:** Not validated at compile time
**Location:** `src/types/inference.rs` lines 4331-4470

Enum patterns in match expressions don't validate variant names or field types at compile time. Invalid patterns compile but fail at runtime.

```forma
e Color { Red, Green, Blue }

m my_color
    Color::Purple -> ...  # Invalid variant - should error at compile time
```

**Workaround:** Be careful with enum variant names.

**Planned:** v1.1

---

## Tooling

### 7. Formatter Incomplete (High Priority)
**Status:** Partially implemented
**Location:** `src/fmt/mod.rs`

The formatter has catch-all cases that output `"..."` or `"?"` for unhandled constructs:
- Line 78: `Use` items output as "us ..."
- Line 247: Unhandled type kinds (Ptr, Fn, etc.)
- Line 369: Unhandled expression kinds
- Line 443: Unhandled pattern kinds

**Impact:** Running `forma fmt --write` may corrupt source files with complex constructs.

**Workaround:** Review formatted output before committing.

**Planned:** v1.1

---

### 8. LSP Go-to-Definition (High Priority)
**Status:** Not implemented
**Location:** `src/lsp/mod.rs` lines 384-391

The `textDocument/definition` LSP method explicitly returns `None` with a TODO comment. Requires tracking definition locations in the type checker.

**Impact:** IDE users can't jump to definitions.

**Workaround:** Use grep/search.

**Planned:** v1.1

---

### 9. REPL Type Display (Medium Priority)
**Status:** Shows "well-typed" instead of actual type
**Location:** `src/main.rs` lines 2009-2010

The `:type` REPL command just prints "Expression is well-typed" instead of showing the actual inferred type.

```
forma> :type [1, 2, 3]
Expression is well-typed  # Should show: Vec[Int]
```

**Planned:** v1.1

---

### 10. Grammar Export Gaps (Low Priority)
**Status:** Mostly complete with some gaps
**Location:** `src/main.rs` lines 1295-1770

The EBNF grammar export is comprehensive but missing:
- Detailed shorthand keyword rules (f/s/e/t/i/m)
- Full indentation rule specification
- Some operator precedence details

**Impact:** External tools may have edge cases.

**Planned:** v1.1

---

### 11. Multi-Error Reporting (Medium Priority)
**Status:** Infrastructure exists but only first error returned
**Location:** `src/parser/parser.rs` lines 26, 41-42, 50-51

The parser has error recovery (`synchronize()` method at lines 3312-3340) but only stores and returns the first error encountered.

```rust
if first_error.is_none() {
    first_error = Some(e);
}
```

**Impact:** Users must fix errors one at a time.

**Planned:** v1.1

---

## Standard Library

### 12. Async is Synchronous (Medium Priority)
**Status:** By design for v1.0
**Location:** `src/mir/interp.rs` lines 917-955

The `sp` (spawn) and `aw` (await) keywords work but execute synchronously. True parallelism is not implemented.

```forma
# Runs sequentially, not in parallel:
task1 := sp fetch_url(url1)
task2 := sp fetch_url(url2)
results := await_all([task1, task2])
```

**Note:** Useful for structuring async-style code. Actual parallelism planned for v1.1.

**Planned:** v1.1

---

### 13. Iterator Encoding Hack (Low Priority)
**Status:** Known workaround
**Location:** `std/iter.forma` lines 181, 190-191

The `enumerate` function uses `idx * 1000000 + value` encoding due to lack of tuple support in iterators.

```forma
# Only works for values < 1,000,000
for encoded in enumerate(items)
    idx := decode_index(encoded)
    val := decode_value(encoded)
```

**Impact:** Breaks silently for large values.

**Planned:** v1.2 (proper tuple iteration)

---

## Parser/Lexer

### 14. Multiline Expression Edge Cases (Low Priority)
**Status:** Known quirk

Some multiline expressions with trailing operators require parentheses:

```forma
# May fail:
result := a &&
    b

# Works:
result := (a) && (b)

# Or on single line:
result := a && b
```

**Workaround:** Use parentheses or single-line expressions.

**Planned:** v1.2

---

## Items NOT Considered Limitations

These were fixed and verified:

| Item | Status | Sprint |
|------|--------|--------|
| Enum discriminant collisions | ✅ Fixed | 14 |
| Method/field type checking | ✅ Working | 9 |
| Option/Result unification | ✅ Working | 9 |
| String pattern matching | ✅ Fixed | 14.7 |
| Import system | ✅ Working | 14.6 |
| JSON stdlib | ✅ Working | 14.5 |
| Async example | ✅ Working | 14.5 |
| Duration builtins | ✅ Implemented | 13 |
| pow() negative check | ✅ Fixed | 13 |

---

## Version Roadmap

### v1.1 (Target: Q2 2026)
- Trait implementation checking
- Formatter completeness
- LSP go-to-definition
- Multi-error reporting
- REPL type display
- True async parallelism
- Enum pattern validation
- Indirect closure calls (LLVM)

### v1.2 (Target: Q3 2026)
- Loop labels
- Proper tuple iteration
- Multiline expression improvements
- Grammar export completeness

### v2.0 (Future)
- Higher-kinded types (maybe)
- Full dependent types (research)

---

## How to Report New Limitations

If you discover a limitation not listed here:

1. Check if it's already in the GitHub issues
2. Create a minimal reproduction case
3. File an issue with the `limitation` label
4. Include FORMA version and expected vs. actual behavior

---

*"Transparency about limitations builds trust."*
