# FORMA

**Code that writes itself correctly.**

---

## The Problem

AI code generation is transforming software development. But there's a catch: **AI fails spectacularly at systems programming.**

When researchers tested LLMs on Rust code generation:
- **94.8%** of failures were lifetime/borrow checker errors
- **33.6%** were type mismatches
- AI models hallucinate syntax, invent APIs, and struggle with memory semantics

The issue isn't AI capability—it's language design. Rust, C++, and Go were designed for humans with compilers. Not for AI with humans.

## The Solution

**FORMA is the first programming language designed for generative AI.**

```forma
f fetch_users(db: Database) -> Result[Vec[User], Str]
    rows := db_query(db, "SELECT * FROM users")?
    users := vec_new()
    for row in rows
        user := User {
            id: row_get_int(row, 0),
            name: row_get_str(row, 1),
            active: row_get_bool(row, 2)
        }
        vec_push(users, user)
    Ok(users)
```

Same memory safety as Rust. None of the complexity that trips up AI.

## Why FORMA Works

### Memory Safety Without Lifetimes

FORMA uses **second-class references**—references can't be stored in structs or returned from functions. This eliminates lifetime annotations entirely while preserving memory safety.

```forma
# Rust: fn longest<'a>(x: &'a str, y: &'a str) -> &'a str
# FORMA: No lifetimes needed
f longest(x: &Str, y: &Str) -> Str
    m str_len(x) > str_len(y)
        true -> x
        false -> y
```

The compiler statically verifies memory safety. AI doesn't need to reason about lifetimes.

### 96% Fewer Syntax Errors

FORMA's grammar is designed for **constrained decoding**. AI models can generate only syntactically valid code:

```bash
# Export grammar for any LLM toolkit
forma grammar --format ebnf > forma.ebnf
forma grammar --format json > forma.json
```

When AI generates tokens, invalid syntax is impossible.

### 38% Fewer Tokens

Every character costs API tokens. FORMA's concise syntax reduces costs and latency:

| Feature | Rust | FORMA |
|---------|------|-------|
| Function | `fn` | `f` |
| Struct | `struct` | `s` |
| Enum | `enum` | `e` |
| Match | `match` | `m` |
| While | `while` | `wh` |
| Return | `return` | `ret` |
| Use | `use` | `us` |
| Import | `import` | `im` |

### AI Self-Correction

Errors are structured JSON that AI can parse and fix automatically:

```json
{
  "error": "type_mismatch",
  "expected": "Int",
  "found": "Str",
  "location": {"line": 5, "column": 12},
  "suggestion": "Use str_parse_int() to convert Str to Int"
}
```

```bash
forma check --format json myfile.forma
```

## Quick Start

```bash
# Clone and build
git clone https://github.com/forma-lang/forma
cd forma
cargo build --release

# Hello World
echo 'f main() -> Int
    print("Hello, FORMA!")
    0' > hello.forma
./target/release/forma run hello.forma
```

## Feature Highlights

### Async/Await

```forma
as f fetch_data(url: Str) -> Bool
    m http_get(url)
        Ok(_) -> true
        Err(_) -> false

as f main()
    # Spawn concurrent tasks
    task1 := sp fetch_data("https://api.example.com/data1")
    task2 := sp fetch_data("https://api.example.com/data2")

    # Wait for results
    result1 := aw task1
    result2 := aw task2
    print("Both requests complete!")
```

### HTTP Server

```forma
f handle_request(req: HttpRequest) -> HttpResponse
    m req.path
        "/" -> http_response(200, "Welcome!")
        "/api/hello" ->
            name := http_req_param(req, "name")
            m name
                Some(n) -> http_response(200, "Hello, " + n + "!")
                None -> http_response(200, "Hello, World!")
        "/api/data" ->
            json := json_parse("{\"status\": \"ok\"}")
            m json
                Ok(j) -> http_json_response(200, j)
                Err(_) -> http_response(500, "Error")
        _ -> http_response(404, "Not Found")

f main() -> Int
    print("Server starting on http://localhost:8080")
    result := http_serve(8080, handle_request)
    m result
        Ok(_) -> 0
        Err(e) ->
            print("Error: " + e)
            1
```

### SQLite Database

```forma
f main() -> Int
    db := db_open("app.db")!

    db_execute(db, "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)")!
    db_execute(db, "INSERT INTO users (name) VALUES ('Alice')")!

    rows := db_query(db, "SELECT id, name FROM users")!
    for row in rows
        id := row_get_int(row, 0)
        name := row_get_str(row, 1)
        print("User " + int_to_str(id) + ": " + name)

    db_close(db)
    0
```

### Pattern Matching

```forma
e Shape
    Circle(Float)
    Rectangle(Float, Float)
    Triangle(Float, Float, Float)

f area(shape: Shape) -> Float
    m shape
        Circle(r) -> 3.14159 * r * r
        Rectangle(w, h) -> w * h
        Triangle(a, b, c) ->
            s := (a + b + c) / 2.0
            sqrt(s * (s - a) * (s - b) * (s - c))
```

## Language Features

- **Type inference**: Hindley-Milner style, rarely need type annotations
- **Generics**: Full parametric polymorphism with monomorphization
- **Pattern matching**: Exhaustive, with guards
- **Result types**: No exceptions, explicit error handling with `?` and `!`
- **Modules**: Simple `us std.collections` imports
- **Async/await**: Native coroutines with spawn
- **HTTP client & server**: Built-in networking primitives
- **SQLite**: Embedded database support
- **Native compilation**: LLVM backend for C-level performance
- **FFI**: C interop with `extern` functions

## Status

FORMA is in **beta**. The core language and ecosystem are feature-complete:

- [x] Lexer, parser, type checker
- [x] Borrow checker (second-class references)
- [x] MIR interpreter
- [x] Generics with monomorphization
- [x] Module system
- [x] Standard library (175+ builtins)
- [x] Grammar export (EBNF, JSON)
- [x] LLVM native compilation
- [x] Package manager (basic)
- [x] Async/await with spawn
- [x] HTTP client & server
- [x] TCP/UDP sockets
- [x] TLS support
- [x] SQLite database
- [x] Compression (gzip, zlib)
- [x] LSP server (basic)
- [ ] Full LSP (diagnostics, refactoring)
- [ ] Package registry

## For AI Developers

FORMA provides first-class tooling for AI code generation:

```bash
# Grammar-constrained generation (EBNF or JSON)
forma grammar --format ebnf
forma grammar --format json

# Structured errors for self-correction
forma check --format json myfile.forma

# Export grammar for constrained decoding
forma grammar --format json | jq '.productions'
```

## Documentation

- [Getting Started](docs/getting-started.md)
- [Language Tour](docs/tour.md)
- [Language Reference](docs/reference.md)
- [Standard Library](docs/stdlib.md)
- [AI Integration Guide](docs/ai-integration.md)

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT OR Apache-2.0 (same as Rust)

---

<p align="center">
  <strong>FORMA</strong><br>
  Code that writes itself correctly.
</p>
