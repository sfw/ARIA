# ARIA Error Handling Specification

## Design Philosophy

1. **Errors as Values**: Recoverable errors use `Result[T, E]`
2. **Explicit Propagation**: The `?` operator for ergonomic error bubbling
3. **Unrecoverable Errors**: `panic` for truly exceptional situations
4. **Rich Context**: Error chains and backtraces for debugging
5. **AI-Friendly**: Structured errors for self-correction

---

## 1. The Result Type

### 1.1 Definition

```aria
enum Result[T, E] {
    Ok(T)
    Err(E)
}
```

### 1.2 Creating Results

```aria
// Success
let success: Result[Int, String] = ok(42)
let success = Result.Ok(42)

// Failure
let failure: Result[Int, String] = err("something went wrong")
let failure = Result.Err("something went wrong")

// From computation
fn parse_int(s: String) -> Result[Int, ParseError] {
    // ...implementation
}
```

### 1.3 Core Methods

```aria
impl[T, E] Result[T, E] {
    // Querying
    fn is_ok(&self) -> Bool
    fn is_err(&self) -> Bool

    // Extracting values (may panic)
    fn unwrap(self) -> T           // Panics if Err
    fn unwrap_err(self) -> E       // Panics if Ok
    fn expect(self, msg: String) -> T  // Panics with message if Err

    // Safe extraction
    fn unwrap_or(self, default: T) -> T
    fn unwrap_or_else(self, f: fn(E) -> T) -> T
    fn unwrap_or_default(self) -> T where T: Default

    // Transformation
    fn map[U](self, f: fn(T) -> U) -> Result[U, E]
    fn map_err[F](self, f: fn(E) -> F) -> Result[T, F]
    fn and_then[U](self, f: fn(T) -> Result[U, E]) -> Result[U, E]
    fn or_else[F](self, f: fn(E) -> Result[T, F]) -> Result[T, F]

    // Conversion
    fn ok(self) -> Option[T]
    fn err(self) -> Option[E]
}
```

### 1.4 The ? Operator

The `?` operator provides ergonomic error propagation:

```aria
// Without ?
fn read_config() -> Result[Config, Error] {
    let file = match open("config.toml") {
        Ok(f) => f
        Err(e) => return err(e.into())
    }
    let content = match file.read_all() {
        Ok(c) => c
        Err(e) => return err(e.into())
    }
    match parse_toml(content) {
        Ok(config) => ok(config)
        Err(e) => err(e.into())
    }
}

// With ?
fn read_config() -> Result[Config, Error] {
    let file = open("config.toml")?
    let content = file.read_all()?
    let config = parse_toml(content)?
    ok(config)
}

// Even more concise
fn read_config() -> Result[Config, Error] {
    open("config.toml")?.read_all() |> parse_toml
}
```

### 1.5 Error Conversion

The `?` operator automatically converts errors:

```aria
trait Into[T] {
    fn into(self) -> T
}

// If IoError implements Into[AppError]:
fn process() -> Result[Data, AppError] {
    let file = open("data.txt")?  // IoError converted to AppError
    ok(parse(file.read()?))
}
```

---

## 2. The Option Type

### 2.1 Definition

```aria
enum Option[T] {
    Some(T)
    None
}
```

### 2.2 Creating Options

```aria
// Present value
let present: Option[Int] = some(42)
let present = Option.Some(42)

// Absent value
let absent: Option[Int] = none
let absent = Option.None
```

### 2.3 Core Methods

```aria
impl[T] Option[T] {
    // Querying
    fn is_some(&self) -> Bool
    fn is_none(&self) -> Bool

    // Extracting (may panic)
    fn unwrap(self) -> T
    fn expect(self, msg: String) -> T

    // Safe extraction
    fn unwrap_or(self, default: T) -> T
    fn unwrap_or_else(self, f: fn() -> T) -> T
    fn unwrap_or_default(self) -> T where T: Default

    // Transformation
    fn map[U](self, f: fn(T) -> U) -> Option[U]
    fn and_then[U](self, f: fn(T) -> Option[U]) -> Option[U]
    fn filter(self, predicate: fn(&T) -> Bool) -> Option[T]
    fn or(self, other: Option[T]) -> Option[T]
    fn or_else(self, f: fn() -> Option[T]) -> Option[T]

    // Conversion to Result
    fn ok_or[E](self, err: E) -> Result[T, E]
    fn ok_or_else[E](self, f: fn() -> E) -> Result[T, E]
}
```

### 2.4 Null Coalescing

```aria
// ?? operator for default values
let value = maybe_value ?? default_value

// Equivalent to
let value = maybe_value.unwrap_or(default_value)

// Chaining
let value = first ?? second ?? third ?? default
```

### 2.5 Option and ?

```aria
fn find_user(id: UserId) -> Option[User] { ... }
fn get_email(user: &User) -> Option[String] { ... }

// Chain operations with ?
fn get_user_email(id: UserId) -> Option[String] {
    let user = find_user(id)?
    get_email(&user)
}

// Or with and_then
fn get_user_email(id: UserId) -> Option[String] {
    find_user(id).and_then(|u| get_email(&u))
}
```

---

## 3. Error Types

### 3.1 The Error Trait

```aria
trait Error: Display {
    // Optional source error (for chaining)
    fn source(&self) -> Option[&Error] {
        none
    }

    // Backtrace if available
    fn backtrace(&self) -> Option[&Backtrace] {
        none
    }
}
```

### 3.2 Defining Custom Errors

```aria
// Simple error
struct ParseError {
    message: String
    line: Int
    column: Int
}

impl Display for ParseError {
    fn display(&self) -> String {
        "Parse error at {self.line}:{self.column}: {self.message}"
    }
}

impl Error for ParseError { }

// Error with source
struct ConfigError {
    path: String
    source: Box[Error]
}

impl Error for ConfigError {
    fn source(&self) -> Option[&Error] {
        some(&*self.source)
    }
}
```

### 3.3 Error Enums

```aria
enum AppError {
    Io(IoError)
    Parse(ParseError)
    Network(NetworkError)
    Custom(String)
}

impl Display for AppError {
    fn display(&self) -> String {
        match self {
            Io(e) => "I/O error: {e}"
            Parse(e) => "Parse error: {e}"
            Network(e) => "Network error: {e}"
            Custom(msg) => msg
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option[&Error] {
        match self {
            Io(e) => some(e)
            Parse(e) => some(e)
            Network(e) => some(e)
            Custom(_) => none
        }
    }
}

// Automatic From implementations
impl From[IoError] for AppError {
    fn from(e: IoError) -> AppError {
        AppError.Io(e)
    }
}
```

### 3.4 Derive Macro for Errors

```aria
#[derive(Error)]
enum DataError {
    #[error("File not found: {path}")]
    NotFound { path: String }

    #[error("Invalid format at line {line}")]
    InvalidFormat { line: Int, #[source] cause: ParseError }

    #[error("Permission denied")]
    PermissionDenied

    #[error(transparent)]
    Other(Box[Error])
}
```

---

## 4. Panic and Unrecoverable Errors

### 4.1 When to Panic

Panic is for truly unrecoverable situations:

```aria
// Programming errors (bugs)
fn get(index: Int) -> T {
    if index >= self.len {
        panic("Index out of bounds: {index} >= {self.len}")
    }
    // ...
}

// Invariant violations
fn process(data: &Data) {
    assert(data.is_valid(), "Invalid data state")
    // ...
}

// Impossible states
match value {
    Some(x) => x
    None => unreachable("Value should always be Some here")
}
```

### 4.2 Panic Functions

```aria
// Basic panic
panic("Something went terribly wrong")

// Assertion
assert(condition)
assert(condition, "Condition failed: {details}")

// Debug assertion (only in debug builds)
debug_assert(expensive_check())

// Equality assertion
assert_eq(actual, expected)
assert_ne(actual, other)

// Unreachable code marker
unreachable()
unreachable("This should never happen")

// Unimplemented placeholder
todo()
todo("Implement this feature")
```

### 4.3 Panic Behavior

```aria
// Default: Unwind the stack, run destructors
fn main() {
    let file = open("important.txt")
    do_something_that_panics()
    // file's destructor still runs during unwinding
}

// Configuration: Abort immediately (faster, smaller binary)
// In aria.toml:
// [profile.release]
// panic = "abort"
```

### 4.4 Catching Panics

```aria
// Catch panic in a thread boundary
let result = catch_panic(|| {
    risky_operation()
})

match result {
    Ok(value) => use(value)
    Err(panic_info) => {
        log_error("Thread panicked: {panic_info}")
    }
}

// Note: Catching panics is discouraged for normal control flow
// Use Result for recoverable errors
```

---

## 5. Error Context and Chaining

### 5.1 Adding Context

```aria
fn read_config(path: String) -> Result[Config, Error] {
    let content = fs.read_string(&path)
        .context("Failed to read config file: {path}")?

    parse_config(&content)
        .context("Failed to parse config")?
}

// context() wraps the error with additional information
```

### 5.2 Error Chains

```aria
fn main() {
    match run_app() {
        Ok(_) => {}
        Err(e) => {
            eprintln("Error: {e}")

            // Print error chain
            let mut source = e.source()
            while let Some(cause) = source {
                eprintln("Caused by: {cause}")
                source = cause.source()
            }

            // Print backtrace if available
            if let Some(bt) = e.backtrace() {
                eprintln("Backtrace:\n{bt}")
            }
        }
    }
}
```

### 5.3 The anyhow Pattern

```aria
// For applications where error type doesn't matter
use anyhow.{Result, Context}

fn main() -> Result[Unit] {
    let config = read_config("app.toml")
        .context("Failed to load configuration")?

    let db = connect_database(&config.db_url)
        .context("Failed to connect to database")?

    run_server(&config)?

    ok(())
}
```

---

## 6. Error Handling Patterns

### 6.1 Early Return Pattern

```aria
fn process_data(input: String) -> Result[Output, Error] {
    // Validate early
    if input.is_empty() {
        return err(Error.InvalidInput("Input cannot be empty"))
    }

    // Process with ? for propagation
    let parsed = parse(input)?
    let validated = validate(parsed)?
    let transformed = transform(validated)?

    ok(transformed)
}
```

### 6.2 Collect Results

```aria
// Fail fast: Stop on first error
let results: Result[List[T], E] = items
    .map(|item| process(item))
    .collect()

// Collect all: Gather all errors
let (successes, failures): (List[T], List[E]) = items
    .map(|item| process(item))
    .partition_results()
```

### 6.3 Fallback Pattern

```aria
fn get_config() -> Config {
    // Try multiple sources, use first success
    read_file_config("./config.toml")
        .or_else(|_| read_env_config())
        .or_else(|_| read_default_config())
        .unwrap_or(Config.default())
}
```

### 6.4 Retry Pattern

```aria
fn fetch_with_retry(url: String, max_attempts: Int) -> Result[Response, Error] {
    let mut attempts = 0
    loop {
        attempts += 1
        match fetch(&url) {
            Ok(response) => return ok(response)
            Err(e) if attempts < max_attempts => {
                sleep(Duration.seconds(attempts))
                continue
            }
            Err(e) => return err(e)
        }
    }
}
```

---

## 7. AI-Optimized Error Messages

### 7.1 Structured Error Format

```aria
// Error messages designed for AI parsing and self-correction
Error[E0308]: Mismatched types
  --> src/main.aria:15:9
   |
15 |     let x: Int = "hello"
   |            ---   ^^^^^^^ expected `Int`, found `String`
   |            |
   |            expected due to this type annotation
   |
Help: Consider parsing the string:
   |
15 |     let x: Int = "hello".parse()?
   |                         ++++++++

Suggestion[primary]: Add .parse()? to convert String to Int
  --> src/main.aria:15:21
  insert: ".parse()?"
```

### 7.2 Machine-Readable Errors

```aria
// JSON error format for tooling
{
    "code": "E0308",
    "severity": "error",
    "message": "Mismatched types",
    "spans": [{
        "file": "src/main.aria",
        "line_start": 15,
        "line_end": 15,
        "column_start": 18,
        "column_end": 25,
        "label": "expected `Int`, found `String`"
    }],
    "suggestions": [{
        "message": "Consider parsing the string",
        "replacements": [{
            "span": { "line": 15, "column_start": 25, "column_end": 25 },
            "text": ".parse()?"
        }]
    }]
}
```

### 7.3 Error Categories

```aria
// Errors categorized by AI fix-ability
enum ErrorCategory {
    // Likely fixable by AI
    TypeMismatch        // Add conversion, change type
    MissingImport       // Add use statement
    UnusedVariable      // Remove or use variable
    MissingField        // Add missing struct field

    // May need human input
    AmbiguousType       // Multiple valid interpretations
    DesignDecision      // Architecture choice needed

    // Requires human intervention
    LogicError          // Semantic issue
    SecurityIssue       // Needs review
}
```

---

## 8. Testing Error Conditions

### 8.1 Testing for Errors

```aria
#[test]
fn test_parse_error() {
    let result = parse_int("not a number")
    assert(result.is_err())

    let err = result.unwrap_err()
    assert(err.message.contains("invalid"))
}

#[test]
fn test_specific_error() {
    let result = divide(10, 0)
    assert_matches(result, Err(MathError.DivisionByZero))
}
```

### 8.2 Testing for Panics

```aria
#[test]
#[should_panic]
fn test_index_panic() {
    let list = [1, 2, 3]
    let _ = list[10]  // Should panic
}

#[test]
#[should_panic(expected: "out of bounds")]
fn test_specific_panic() {
    let list = [1, 2, 3]
    let _ = list[10]
}
```

---

## 9. Best Practices

### 9.1 When to Use What

| Situation | Approach |
|-----------|----------|
| Recoverable failure | `Result[T, E]` |
| Optional value | `Option[T]` |
| Programming bug | `panic!` |
| Impossible state | `unreachable!` |
| Not yet implemented | `todo!` |
| Contract violation | `assert!` |

### 9.2 Error Type Guidelines

```aria
// Library: Use specific error types
pub enum ParseError {
    UnexpectedToken { expected: String, found: String }
    UnterminatedString { line: Int }
    InvalidEscape { char: Char }
}

// Application: Use anyhow for convenience
use anyhow.Result

fn main() -> Result[Unit] {
    // ...
}

// Interface boundary: Define clear error types
pub trait Storage {
    type Error: Error

    fn get(&self, key: &str) -> Result[Option[Value], Self.Error]
    fn set(&mut self, key: String, value: Value) -> Result[Unit, Self.Error]
}
```

### 9.3 Documentation

```aria
/// Parses an integer from a string.
///
/// # Errors
///
/// Returns `ParseError.InvalidDigit` if the string contains non-digit characters.
/// Returns `ParseError.Overflow` if the number is too large for `Int`.
///
/// # Examples
///
/// ```
/// let n = parse_int("42")?  // Ok(42)
/// let e = parse_int("abc")  // Err(ParseError.InvalidDigit)
/// ```
pub fn parse_int(s: &str) -> Result[Int, ParseError] {
    // ...
}
```
