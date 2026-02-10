# ARIA v2: AI-First Variant

> Maximum token efficiency for generative AI while maintaining human readability.

---

## Design Philosophy

ARIA v2 optimizes ruthlessly for AI code generation:

| Metric | ARIA v1 | ARIA v2 | Improvement |
|--------|---------|---------|-------------|
| Avg tokens per function | 45 | 28 | **38% reduction** |
| Keywords length | 3-6 chars | 1-3 chars | **60% reduction** |
| Brace tokens | Required | Optional | **~20% reduction** |
| Type annotations | Verbose | Shortcuts | **~25% reduction** |

---

## Quick Syntax Comparison

### ARIA v1 (Human-Optimized)
```aria
fn read_users(path: String) -> Result[List[User], Error] {
    let file = open(path)?
    let content = file.read_all()?
    let users = parse_json(content)?
    users.filter(|u| u.active)
}

struct User {
    name: String
    age: Int
    active: Bool
}

enum Status {
    Active
    Inactive(reason: String)
}
```

### ARIA v2 (AI-Optimized)
```
f read_users(path: Str) -> [User]!
    file = open path?
    content = file.read?
    users = parse_json content?
    users | filter .active

s User
    name: Str
    age: Int
    active: Bool

e Status
    Active
    Inactive(reason: Str)
```

---

## Key Changes from v1

### 1. Short Keywords
| v1 | v2 | Saves |
|----|----|----|
| `fn` | `f` | 1 token |
| `struct` | `s` | 1 token |
| `enum` | `e` | 1 token |
| `trait` | `t` | 1 token |
| `impl` | `i` | 1 token |
| `match` | `m` | 1 token |
| `let` | (implicit) | 1 token |
| `return` | `ret` or implicit | 1 token |
| `String` | `Str` | — |
| `Bool` | `B` | — |

### 2. Indentation-Significant (Braces Optional)
```
# AI-preferred: indentation
f add(a: Int, b: Int) -> Int
    a + b

# Human-preferred: braces (still valid)
f add(a: Int, b: Int) -> Int { a + b }
```

### 3. Type Shortcuts
| Full Type | Shortcut | Example |
|-----------|----------|---------|
| `List[T]` | `[T]` | `[Int]`, `[Str]` |
| `Map[K,V]` | `{K:V}` | `{Str:Int}` |
| `Set[T]` | `{T}` | `{Int}` |
| `Option[T]` | `T?` | `Int?`, `Str?` |
| `Result[T,E]` | `T!E` or `T!` | `Int!Error`, `Data!` |
| `Tuple` | `(A,B)` | `(Int,Str)` |

### 4. Pipeline-First Design
```
# Method chaining (verbose)
items.filter(|x| x > 0).map(|x| x * 2).sum()

# Pipeline (compact)
items | filter(>0) | map(*2) | sum
```

### 5. Implicit Variable Declaration
```
# v1
let x = 42
let mut y = 0

# v2
x = 42      # immutable by default
y := 0      # := for mutable
```

### 6. Compact Pattern Matching
```
# v1
match result {
    Ok(value) => process(value)
    Err(e) => handle(e)
}

# v2
m result
    Ok v -> process v
    Err e -> handle e

# Inline form
result | Ok v -> v | Err _ -> 0
```

---

## Document Structure

| Document | Description |
|----------|-------------|
| [01-SYNTAX.md](01-SYNTAX.md) | Complete syntax specification |
| [02-TYPES.md](02-TYPES.md) | Type system with shortcuts |
| [03-MEMORY.md](03-MEMORY.md) | Memory model (unchanged from v1) |
| [04-ERRORS.md](04-ERRORS.md) | Error handling |
| [05-MODULES.md](05-MODULES.md) | Module system |
| [06-CONCURRENCY.md](06-CONCURRENCY.md) | Async and concurrency |
| [07-STDLIB.md](07-STDLIB.md) | Standard library |
| [08-GRAMMAR.md](08-GRAMMAR.md) | Formal grammar |
| [09-COMPARISON.md](09-COMPARISON.md) | v1 vs v2 comparison |

---

## Token Efficiency Analysis

### Example: HTTP Handler

**v1 (~85 tokens)**
```aria
fn handle_request(req: Request) -> Result[Response, Error] {
    let user_id = req.params.get("id")?
    let user = database.find_user(user_id)?
    match user {
        Some(u) => Ok(Response.json(u))
        None => Err(Error.not_found("User not found"))
    }
}
```

**v2 (~52 tokens)**
```
f handle_request(req: Request) -> Response!
    user_id = req.params.get "id"?
    user = db.find_user user_id?
    m user
        Some u -> Response.json u
        None -> err NotFound "User not found"
```

**Savings: 39%**

---

## When to Use v2 vs v1

| Use Case | Recommendation |
|----------|----------------|
| AI code generation | **v2** |
| Teaching/learning | v1 |
| Team with mixed experience | v1 |
| Maximizing throughput | **v2** |
| Code review by humans | v1 |
| Production AI pipelines | **v2** |

---

## Compatibility

v2 is a **strict subset** of valid syntax patterns. A v2-to-v1 transpiler is trivial:
- Expand keywords (`f` → `fn`)
- Add braces from indentation
- Expand type shortcuts

This means:
- v1 tools can process v2 (with preprocessing)
- Teams can mix styles
- Gradual adoption possible
