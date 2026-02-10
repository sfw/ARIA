# ARIA v2 Error Handling

## Design: Compact but Clear

Error handling uses the same semantics as v1 but with shorter syntax.

---

## 1. Result Type

### 1.1 Type Shortcuts

```
# Full form
Result[Data, Error]

# Shortcut: T!E
Data!Error

# Default error type: T!
Data!                    # Result[Data, Error]
Int!                     # Result[Int, Error]
()!                      # Result[(), Error]
```

### 1.2 Creating Results

```
# Success
ok 42                    # Ok(42)
ok()                     # Ok(())

# Failure
err "failed"             # Err("failed")
err NotFound             # Err(NotFound)
```

### 1.3 Methods

```
# Query
result.is_ok
result.is_err

# Extract (may panic)
result.unwrap            # panics if Err
result.expect "msg"      # panics with message

# Safe extract
result ?? default        # unwrap_or
result ?? || compute     # unwrap_or_else

# Transform
result | map f           # map Ok value
result | map_err f       # map Err value
result | and_then f      # chain operations

# Convert
result.ok                # Option[T]
result.err               # Option[E]
```

---

## 2. Option Type

### 2.1 Type Shortcut

```
# Full form
Option[Int]

# Shortcut
Int?
Str?
User?
```

### 2.2 Creating Options

```
# Present
Some 42
Some "hello"

# Absent
N                        # None shortcut
none                     # also valid
```

### 2.3 Methods

```
# Query
opt.is_some
opt.is_none

# Extract (may panic)
opt.unwrap
opt.expect "msg"

# Safe extract
opt ?? default           # unwrap_or
opt ?? || compute        # unwrap_or_else

# Transform
opt | map f
opt | and_then f
opt | filter pred

# Convert to Result
opt.ok_or err
opt.ok_or_else || make_err
```

---

## 3. The ? Operator

### 3.1 Basic Usage

```
# Propagates Err, unwraps Ok
f read_config -> Config!
    file = open "config.toml"?    # early return if Err
    content = file.read?
    parse content?

# Propagates None, unwraps Some
f find_email(id: Int) -> Str?
    user = find_user id?          # early return if None
    user.email
```

### 3.2 Chained Operations

```
# Pipeline with ?
f process(path: Str) -> Data!
    open path? | read? | parse? | validate?

# Multiple ? in sequence
data = open path?
    | read?
    | decompress?
    | parse?
```

### 3.3 Error Conversion

```
# Auto-converts error types via Into
f process -> AppError!
    file = open path?             # IoError -> AppError
    data = parse content?         # ParseError -> AppError
    ok data
```

---

## 4. Null Coalescing (??)

```
# Default value on None/Err
value = maybe_value ?? 0
name = user?.name ?? "Anonymous"
config = load_config ?? Config.default

# Chaining
value = first ?? second ?? third ?? default

# With computation
value = expensive_lookup ?? || compute_fallback
```

---

## 5. Custom Error Types

### 5.1 Simple Errors

```
s ParseError
    msg: Str
    line: Int
    col: Int

i Display for ParseError
    f display(&self) -> Str
        "Parse error at {self.line}:{self.col}: {self.msg}"

i Error for ParseError
```

### 5.2 Error Enums

```
e AppError
    Io(IoError)
    Parse(ParseError)
    Network(NetworkError)
    Custom(Str)

i Display for AppError
    f display(&self) -> Str
        m self
            Io e -> "IO: {e}"
            Parse e -> "Parse: {e}"
            Network e -> "Network: {e}"
            Custom msg -> msg

i Error for AppError
    f source(&self) -> Error?
        m self
            Io e -> Some e
            Parse e -> Some e
            Network e -> Some e
            Custom _ -> N

# Auto-conversion
i From[IoError] for AppError
    f from(e: IoError) -> AppError = Io e

i From[ParseError] for AppError
    f from(e: ParseError) -> AppError = Parse e
```

### 5.3 Derive Macro

```
@derive(Error)
e DataError
    @error("File not found: {path}")
    NotFound(path: Str)

    @error("Invalid format at line {line}")
    InvalidFormat(line: Int, @source cause: ParseError)

    @error("Permission denied")
    PermissionDenied

    @error(transparent)
    Other(Box[Error])
```

---

## 6. Panic

### 6.1 When to Panic

```
# Programming bugs
f get(idx: Int) -> T
    if idx >= self.len
        panic "index {idx} >= len {self.len}"
    self.data[idx]

# Assertions
assert condition
assert condition, "details: {info}"
assert_eq actual, expected
assert_ne actual, other

# Debug only
debug_assert expensive_check

# Unreachable
unreachable
unreachable "should never happen"

# Placeholder
todo
todo "implement this"
```

### 6.2 Catching Panics

```
# At thread boundaries
result = catch_panic ||
    risky_operation

m result
    Ok value -> use value
    Err info -> log "panic: {info}"
```

---

## 7. Error Patterns

### 7.1 Early Return

```
f process(input: Str) -> Output!
    if input.empty
        ret err InvalidInput "empty"

    parsed = parse input?
    validated = validate parsed?
    transform validated
```

### 7.2 Fallback Chain

```
f get_config -> Config
    read_file_config "./config.toml"
        ?? || read_env_config
        ?? || read_default_config
        ?? Config.default
```

### 7.3 Collect Results

```
# Fail fast
results: [Data]! = items | map process | collect

# Collect all
(successes, failures) = items
    | map process
    | partition_results
```

### 7.4 Retry

```
f fetch_retry(url: Str, max: Int) -> Response!
    attempts := 0
    lp
        attempts += 1
        m fetch url
            Ok resp -> ret ok resp
            Err e if attempts < max ->
                sleep Duration.secs attempts
                ct
            Err e -> ret err e
```

---

## 8. Context and Chaining

```
# Add context to errors
f read_config(path: Str) -> Config!
    content = fs.read path
        | context "failed to read {path}"?

    parse content
        | context "failed to parse config"?

# Error chain
f main
    m run_app
        Ok _ -> ()
        Err e ->
            eprint "Error: {e}"
            source := e.source
            wh Some cause = source
                eprint "Caused by: {cause}"
                source = cause.source
```

---

## 9. Testing Errors

```
@test
f test_parse_error
    result = parse "invalid"
    assert result.is_err
    err = result.unwrap_err
    assert err.msg.contains "invalid"

@test
f test_specific_error
    result = divide 10, 0
    assert_matches result, Err DivByZero

@test
@should_panic
f test_index_panic
    list = [1, 2, 3]
    _ = list[10]

@test
@should_panic("out of bounds")
f test_specific_panic
    list = [1, 2, 3]
    _ = list[10]
```

---

## 10. Complete Example

```
us std.fs
us std.json

@derive(Error)
e ConfigError
    @error("Config file not found: {path}")
    NotFound(path: Str)

    @error("Failed to read config")]
    ReadError(@source IoError)

    @error("Invalid JSON at line {line}")]
    ParseError(line: Int, @source JsonError)

    @error("Missing required field: {field}")]
    MissingField(field: Str)

s Config
    host: Str
    port: Int
    debug: B = F

f load_config(path: Str) -> Config!ConfigError
    # Check exists
    if !fs.exists path
        ret err NotFound path

    # Read file
    content = fs.read path
        | map_err ReadError?

    # Parse JSON
    data = json.parse content
        | map_err |e| ParseError(e.line, e)?

    # Extract fields
    host = data.get "host"
        ?? || ret err MissingField "host"

    port = data.get "port"
        ?? || ret err MissingField "port"

    debug = data.get "debug" ?? F

    ok Config(host, port, debug)

f main
    m load_config "app.toml"
        Ok config ->
            print "Loaded: {config.host}:{config.port}"
        Err e ->
            eprint "Failed to load config: {e}"
            exit 1
```
