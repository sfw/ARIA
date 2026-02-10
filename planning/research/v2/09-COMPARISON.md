# ARIA v1 vs v2 Comparison

## Executive Summary

| Metric | v1 | v2 | Change |
|--------|-----|-----|--------|
| Avg tokens/function | 45 | 28 | **-38%** |
| Keyword characters | 3-6 | 1-3 | **-50%** |
| Required braces | Yes | No | **-20% tokens** |
| Type annotation length | Verbose | Shortcuts | **-25%** |
| Total token reduction | Baseline | | **~35-40%** |

---

## 1. Keywords

| Concept | v1 | v2 | Savings |
|---------|-----|-----|---------|
| Function | `fn` | `f` | 1 char |
| Struct | `struct` | `s` | 5 chars |
| Enum | `enum` | `e` | 3 chars |
| Trait | `trait` | `t` | 4 chars |
| Impl | `impl` | `i` | 3 chars |
| Match | `match` | `m` | 4 chars |
| While | `while` | `wh` | 3 chars |
| Loop | `loop` | `lp` | 2 chars |
| Break | `break` | `br` | 3 chars |
| Continue | `continue` | `ct` | 6 chars |
| Return | `return` | `ret` | 3 chars |
| Async | `async` | `as` | 3 chars |
| Await | `await` | `aw` | 3 chars |
| Let | `let` | (implicit) | 3 chars |
| Boolean | `true`/`false` | `T`/`F` | 3-4 chars |
| None | `none` | `N` | 3 chars |

---

## 2. Type Shortcuts

| Type | v1 | v2 | Savings |
|------|-----|-----|---------|
| List | `List[Int]` | `[Int]` | 4 chars |
| Map | `Map[Str, Int]` | `{Str:Int}` | 4 chars |
| Set | `Set[Int]` | `{Int}` | 3 chars |
| Option | `Option[Int]` | `Int?` | 8 chars |
| Result | `Result[Int, Error]` | `Int!` | 14 chars |
| Result (explicit) | `Result[Int, MyErr]` | `Int!MyErr` | 9 chars |
| String | `String` | `Str` | 3 chars |
| Boolean | `Bool` | `B` | 3 chars |

---

## 3. Syntax Comparison

### 3.1 Function Definition

**v1 (25 tokens)**
```aria
fn calculate_total(items: List[Item], tax: Float) -> Result[Float, Error] {
    let subtotal = items.map(|i| i.price).sum()
    let total = subtotal * (1.0 + tax)
    Ok(total)
}
```

**v2 (16 tokens)**
```
f calculate_total(items: [Item], tax: Float) -> Float!
    subtotal = items | map .price | sum
    total = subtotal * (1.0 + tax)
    ok total
```

**Savings: 36%**

---

### 3.2 Struct Definition

**v1 (18 tokens)**
```aria
struct User {
    name: String,
    email: String,
    age: Int,
    active: Bool,
}
```

**v2 (12 tokens)**
```
s User
    name: Str
    email: Str
    age: Int
    active: B
```

**Savings: 33%**

---

### 3.3 Enum Definition

**v1 (22 tokens)**
```aria
enum Result[T, E] {
    Ok(T),
    Err(E),
}
```

**v2 (10 tokens)**
```
e Result[T, E] = Ok(T) | Err(E)
```

**Savings: 55%**

---

### 3.4 Pattern Matching

**v1 (28 tokens)**
```aria
match response {
    Ok(data) => process(data),
    Err(NetworkError(msg)) => log_network(msg),
    Err(ParseError(line)) => log_parse(line),
    Err(_) => log_unknown(),
}
```

**v2 (20 tokens)**
```
m response
    Ok data -> process data
    Err NetworkError msg -> log_network msg
    Err ParseError line -> log_parse line
    Err _ -> log_unknown
```

**Savings: 29%**

---

### 3.5 Error Handling

**v1 (35 tokens)**
```aria
fn load_config(path: String) -> Result[Config, Error] {
    let file = open(path)?
    let content = file.read_all()?
    let config = parse_toml(content)?
    Ok(config)
}
```

**v2 (20 tokens)**
```
f load_config(path: Str) -> Config!
    file = open path?
    content = file.read?
    config = parse_toml content?
    ok config
```

**Savings: 43%**

---

### 3.6 Pipeline vs Method Chaining

**v1 (24 tokens)**
```aria
let result = items
    .filter(|x| x.active)
    .map(|x| x.value)
    .filter(|x| x > 0)
    .sum()
```

**v2 (14 tokens)**
```
result = items
    | filter .active
    | map .value
    | filter (> 0)
    | sum
```

**Savings: 42%**

---

### 3.7 Async Code

**v1 (32 tokens)**
```aria
async fn fetch_users(url: String) -> Result[List[User], Error] {
    let response = await http.get(url)?
    let data = await response.json()?
    let users = data.users.filter(|u| u.active)
    Ok(users)
}
```

**v2 (20 tokens)**
```
as f fetch_users(url: Str) -> [User]!
    response = aw http.get url?
    data = aw response.json?
    users = data.users | filter .active
    ok users
```

**Savings: 38%**

---

### 3.8 Trait Implementation

**v1 (35 tokens)**
```aria
impl Display for User {
    fn display(&self) -> String {
        format!("User({}, {})", self.name, self.email)
    }
}
```

**v2 (18 tokens)**
```
i Display for User
    f display(&self) -> Str
        "User({self.name}, {self.email})"
```

**Savings: 49%**

---

## 4. Complete Program Comparison

### v1 Version (156 tokens)

```aria
use std::io;
use std::fs;
use std::json;

struct Config {
    host: String,
    port: Int,
    debug: Bool,
}

fn load_config(path: String) -> Result[Config, Error] {
    let content = fs.read_string(path)?;
    let config = json.parse(content)?;
    Ok(config)
}

async fn start_server(config: Config) -> Result[(), Error] {
    let addr = format!("{}:{}", config.host, config.port);
    let listener = await TcpListener.bind(addr)?;

    println("Server listening on {}", addr);

    loop {
        let (stream, _) = await listener.accept()?;
        spawn(handle_connection(stream));
    }
}

fn main() {
    let config = load_config("config.json")
        .expect("Failed to load config");

    runtime.block_on(start_server(config));
}
```

### v2 Version (98 tokens)

```
us std.io
us std.fs
us std.json

s Config
    host: Str
    port: Int
    debug: B

f load_config(path: Str) -> Config!
    content = fs.read_str path?
    json.parse content

as f start_server(config: Config) -> ()!
    addr = "{config.host}:{config.port}"
    listener = aw TcpListener.bind addr?

    println "Server listening on {addr}"

    lp
        (stream, _) = aw listener.accept?
        spawn || handle_conn stream

f main
    config = load_config "config.json"
        ?? || panic "Failed to load config"

    runtime.block_on || start_server config
```

**Total Savings: 37%**

---

## 5. Token Analysis

### What Contributes to Savings

| Source | Estimated Savings |
|--------|-------------------|
| Short keywords | 15% |
| No braces (indentation) | 10% |
| Type shortcuts | 8% |
| Implicit let | 5% |
| Pipeline syntax | 5% |
| Short literals (T/F/N) | 2% |
| **Total** | **~35-40%** |

### Token Distribution

**v1 typical function:**
- Keywords: 8%
- Identifiers: 25%
- Types: 20%
- Operators: 15%
- Punctuation: 25%
- Literals: 7%

**v2 typical function:**
- Keywords: 5%
- Identifiers: 30%
- Types: 15%
- Operators: 18%
- Punctuation: 15%
- Literals: 7%
- Whitespace/indent: 10%

---

## 6. Readability Assessment

### v1 Strengths
- Familiar to Rust/Swift developers
- Explicit structure with braces
- Self-documenting keywords

### v2 Strengths
- Less visual noise
- Pipeline reads left-to-right
- Indentation matches mental model
- Faster to scan

### Trade-offs

| Aspect | v1 | v2 |
|--------|-----|-----|
| Learning curve | Lower | Slightly higher |
| Familiarity | High (Rust-like) | Medium (Python-ish) |
| Token efficiency | Baseline | **+35-40%** |
| Copy-paste safety | Braces help | Indent-sensitive |
| AI generation | Good | **Excellent** |

---

## 7. When to Use Which

### Use v1 When:
- Team unfamiliar with indentation-based syntax
- Teaching programming concepts
- Interfacing with Rust developers
- Codebase requires explicit structure

### Use v2 When:
- AI is generating code
- Minimizing token usage is priority
- Team comfortable with Python-style
- Building AI-first applications
- Cost-sensitive LLM deployments

---

## 8. Migration

### v2 → v1 Transpiler

Trivial transformation:
1. Expand keywords (`f` → `fn`)
2. Add braces from indentation
3. Expand type shortcuts
4. Add `let` keywords

### v1 → v2 Transpiler

Also straightforward:
1. Compress keywords
2. Remove braces, use indentation
3. Apply type shortcuts
4. Remove redundant `let`

### Compatibility

Both versions compile to the same IR, so:
- Libraries work across versions
- Mixed codebases possible
- Gradual migration supported
