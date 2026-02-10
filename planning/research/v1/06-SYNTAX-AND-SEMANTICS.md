# ARIA Syntax and Semantics Specification

## Design Goals for Syntax

1. **Token Efficiency**: Minimize tokens per semantic unit
2. **Predictability**: Consistent patterns throughout
3. **Readability**: Clear at a glance
4. **Parseability**: Unambiguous grammar for AI generation
5. **Familiarity**: Leverage existing mental models

---

## 1. Lexical Structure

### 1.1 Character Set

ARIA source files are UTF-8 encoded. Identifiers support Unicode letters.

### 1.2 Comments

```aria
// Single-line comment

/*
   Multi-line comment
   Can span multiple lines
*/

/// Documentation comment for the following item
/// Supports Markdown formatting
fn documented_function() { }

//! Module-level documentation
//! Describes the current module
```

### 1.3 Identifiers

```aria
// Snake_case for functions and variables
let my_variable = 42
fn calculate_sum() { }

// PascalCase for types and traits
struct MyStruct { }
trait Comparable { }
enum MyEnum { }

// SCREAMING_SNAKE_CASE for constants
const MAX_SIZE: Int = 1000
```

### 1.4 Keywords

Reserved keywords (cannot be used as identifiers):

```
// Declarations
fn      struct    enum      trait     impl      type
const   static    let       mut       pub       mod
use     as        self      Self

// Control Flow
if      else      match     for       while     loop
break   continue  return    yield     await     async

// Types
true    false     none

// Safety
unsafe  move      ref

// Other
where   in        extern    dyn
```

### 1.5 Literals

```aria
// Integers
42              // Int (inferred size)
42i32           // Explicit 32-bit
42i64           // Explicit 64-bit
42u8            // Unsigned 8-bit
0xFF            // Hexadecimal
0b1010          // Binary
0o755           // Octal
1_000_000       // Underscores for readability

// Floats
3.14            // Float (inferred, defaults to f64)
3.14f32         // Explicit 32-bit
2.5e10          // Scientific notation
2.5e-3f32       // Scientific with explicit type

// Strings
"hello"                     // String literal
"line1\nline2"             // Escape sequences
"value: {x}"               // String interpolation
"value: {x + 1}"           // Expression interpolation
"""
    Multi-line string
    preserves formatting
"""

// Raw strings (no escapes processed)
r"C:\path\to\file"
r#"Can contain "quotes""#

// Characters
'a'             // Char
'\n'            // Escaped char
'\u{1F600}'     // Unicode escape

// Booleans
true
false

// None (absence of value)
none
```

---

## 2. Declarations

### 2.1 Variables

```aria
// Immutable by default
let x = 42
let y: Int = 42          // Explicit type

// Mutable variables
let mut counter = 0
counter = counter + 1

// Constants (compile-time evaluated)
const PI: Float = 3.14159265359
const MAX_ITEMS: Int = 100

// Static variables (global lifetime)
static mut COUNTER: Int = 0  // Requires unsafe to access
```

### 2.2 Functions

```aria
// Basic function
fn greet(name: String) -> String {
    "Hello, {name}!"
}

// No return value (returns Unit)
fn log(message: String) {
    print(message)
}

// Multiple parameters
fn add(a: Int, b: Int) -> Int {
    a + b
}

// Default parameters
fn connect(host: String, port: Int = 8080) -> Connection {
    // ...
}

// Named arguments at call site
connect(host: "localhost", port: 3000)
connect(host: "localhost")  // Uses default port

// Generic functions
fn first[T](list: &List[T]) -> Option[T] {
    if list.is_empty() {
        none
    } else {
        some(list[0])
    }
}

// Multiple return values via tuples
fn divmod(a: Int, b: Int) -> (Int, Int) {
    (a / b, a % b)
}

// Destructuring return
let (quotient, remainder) = divmod(17, 5)
```

### 2.3 Structs

```aria
// Basic struct
struct Point {
    x: Float
    y: Float
}

// Instantiation
let p = Point { x: 1.0, y: 2.0 }

// Field access
let x_coord = p.x

// Struct with visibility
pub struct User {
    pub name: String
    email: String        // Private by default
    age: Int
}

// Tuple structs
struct Color(Int, Int, Int)
let red = Color(255, 0, 0)

// Unit struct
struct Marker

// Generic structs
struct Pair[A, B] {
    first: A
    second: B
}

// Struct update syntax
let p2 = Point { x: 5.0, ..p }  // Copy p, override x
```

### 2.4 Enums (Sum Types)

```aria
// Simple enum
enum Direction {
    North
    South
    East
    West
}

// Enum with associated data
enum Shape {
    Circle(radius: Float)
    Rectangle(width: Float, height: Float)
    Triangle(a: Float, b: Float, c: Float)
}

// Generic enum
enum Option[T] {
    Some(T)
    None
}

enum Result[T, E] {
    Ok(T)
    Err(E)
}

// Using enums
let shape = Shape.Circle(radius: 5.0)
let maybe_value: Option[Int] = Option.Some(42)

// Shorthand when type is known
let maybe: Option[Int] = some(42)
let result: Result[Int, String] = ok(42)
```

### 2.5 Type Aliases

```aria
type UserId = Int
type Callback = fn(Int) -> Bool
type StringResult = Result[String, Error]
```

---

## 3. Expressions

### 3.1 Expression-Oriented

ARIA is expression-oriented. Most constructs return values.

```aria
// If is an expression
let max = if a > b { a } else { b }

// Match is an expression
let description = match shape {
    Circle(r) => "Circle with radius {r}"
    Rectangle(w, h) => "Rectangle {w}x{h}"
    Triangle(_, _, _) => "Triangle"
}

// Blocks are expressions (last expression is the value)
let result = {
    let x = compute_x()
    let y = compute_y()
    x + y  // No semicolon = return value
}
```

### 3.2 Operators

```aria
// Arithmetic
a + b       // Addition
a - b       // Subtraction
a * b       // Multiplication
a / b       // Division
a % b       // Remainder
-a          // Negation

// Comparison
a == b      // Equality
a != b      // Inequality
a < b       // Less than
a <= b      // Less or equal
a > b       // Greater than
a >= b      // Greater or equal

// Logical
a && b      // And (short-circuit)
a || b      // Or (short-circuit)
!a          // Not

// Bitwise
a & b       // And
a | b       // Or
a ^ b       // Xor
!a          // Not (context-dependent)
a << n      // Left shift
a >> n      // Right shift

// Assignment
a = b       // Assign
a += b      // Add-assign
a -= b      // Subtract-assign
a *= b      // Multiply-assign
a /= b      // Divide-assign

// Range
0..10       // Exclusive range [0, 10)
0..=10      // Inclusive range [0, 10]

// Pipe (for chaining)
data |> transform |> filter |> collect

// Error propagation
value?      // Early return on error

// Null coalescing
value ?? default
```

### 3.3 Control Flow

```aria
// If-else
if condition {
    do_something()
} else if other_condition {
    do_other()
} else {
    do_default()
}

// Match (pattern matching)
match value {
    0 => "zero"
    1..=9 => "single digit"
    n if n < 0 => "negative"
    _ => "other"
}

// Match with destructuring
match point {
    Point { x: 0, y: 0 } => "origin"
    Point { x: 0, y } => "on y-axis at {y}"
    Point { x, y: 0 } => "on x-axis at {x}"
    Point { x, y } => "at ({x}, {y})"
}

// For loops
for item in collection {
    process(item)
}

for i in 0..10 {
    print(i)
}

for (index, value) in collection.enumerate() {
    print("{index}: {value}")
}

// While loops
while condition {
    do_work()
}

// Loop (infinite, use break to exit)
loop {
    if done() {
        break
    }
    work()
}

// Loop with value
let result = loop {
    if found {
        break value  // Returns value from loop
    }
}

// Labeled loops
'outer: for i in 0..10 {
    for j in 0..10 {
        if condition {
            break 'outer  // Breaks outer loop
        }
    }
}
```

---

## 4. Pattern Matching

### 4.1 Pattern Types

```aria
// Literal patterns
match x {
    0 => "zero"
    1 => "one"
    _ => "other"
}

// Variable binding
match opt {
    Some(value) => use(value)
    None => default()
}

// Struct patterns
match point {
    Point { x, y } => "({x}, {y})"
}

// Tuple patterns
match pair {
    (0, 0) => "origin"
    (x, 0) => "x-axis"
    (0, y) => "y-axis"
    (x, y) => "({x}, {y})"
}

// Or patterns
match x {
    0 | 1 | 2 => "small"
    _ => "large"
}

// Range patterns
match x {
    0..=9 => "digit"
    'a'..='z' => "lowercase"
    _ => "other"
}

// Guard patterns
match x {
    n if n < 0 => "negative"
    n if n > 0 => "positive"
    _ => "zero"
}

// Nested patterns
match data {
    Some(Ok(value)) => use(value)
    Some(Err(e)) => handle(e)
    None => default()
}

// @ binding (bind while matching)
match x {
    n @ 1..=10 => "small number: {n}"
    n @ _ => "other: {n}"
}
```

### 4.2 Let Patterns

```aria
// Destructuring in let
let (a, b) = get_pair()
let Point { x, y } = get_point()
let [first, second, ..rest] = get_list()

// If-let for single pattern
if let Some(value) = maybe_value {
    use(value)
}

// While-let
while let Some(item) = iterator.next() {
    process(item)
}
```

---

## 5. Traits and Implementations

### 5.1 Trait Definition

```aria
// Basic trait
trait Display {
    fn display(&self) -> String
}

// Trait with default implementation
trait Greet {
    fn name(&self) -> String

    fn greet(&self) -> String {
        "Hello, {self.name()}!"
    }
}

// Trait with associated types
trait Iterator {
    type Item

    fn next(&mut self) -> Option[Self.Item]
}

// Trait with generic methods
trait Convertible {
    fn convert[T](&self) -> T where T: From[Self]
}
```

### 5.2 Implementations

```aria
// Implement trait for struct
impl Display for Point {
    fn display(&self) -> String {
        "({self.x}, {self.y})"
    }
}

// Inherent implementation (methods on type)
impl Point {
    // Constructor (associated function)
    fn new(x: Float, y: Float) -> Point {
        Point { x, y }
    }

    // Method
    fn distance(&self, other: &Point) -> Float {
        let dx = self.x - other.x
        let dy = self.y - other.y
        (dx * dx + dy * dy).sqrt()
    }

    // Mutable method
    fn translate(&mut self, dx: Float, dy: Float) {
        self.x += dx
        self.y += dy
    }
}

// Generic implementation
impl[T] Display for List[T] where T: Display {
    fn display(&self) -> String {
        let items = self.map(|item| item.display()).join(", ")
        "[{items}]"
    }
}
```

### 5.3 Derive (Automatic Implementation)

```aria
// Automatic trait implementation
#[derive(Debug, Clone, Eq, Hash)]
struct User {
    name: String
    age: Int
}

// Available derives:
// - Debug: Debug formatting
// - Clone: Deep copy
// - Copy: Bitwise copy (for small types)
// - Eq: Equality comparison
// - Ord: Ordering comparison
// - Hash: Hash computation
// - Default: Default value
// - Serialize, Deserialize: Serialization
```

---

## 6. Closures and Functions

### 6.1 Closure Syntax

```aria
// Full syntax
let add = |a: Int, b: Int| -> Int { a + b }

// Type inference
let add = |a, b| a + b

// No parameters
let greet = || print("Hello!")

// Multi-statement
let process = |x| {
    let y = x * 2
    let z = y + 1
    z
}

// Capturing variables
let multiplier = 3
let multiply = |x| x * multiplier
```

### 6.2 Function Types

```aria
// Function type syntax
type BinaryOp = fn(Int, Int) -> Int

// As parameter
fn apply(f: fn(Int) -> Int, x: Int) -> Int {
    f(x)
}

// With closure (captures environment)
fn apply_closure(f: Fn(Int) -> Int, x: Int) -> Int {
    f(x)
}

// Mutable closure
fn apply_mut(f: FnMut(Int) -> Int, x: Int) -> Int {
    f(x)
}

// Consuming closure (takes ownership)
fn apply_once(f: FnOnce(Int) -> Int, x: Int) -> Int {
    f(x)
}
```

---

## 7. Modules and Visibility

### 7.1 Module Declaration

```aria
// In-file module
mod utils {
    pub fn helper() -> Int { 42 }

    fn private_helper() { }  // Not visible outside
}

// File-based module (utils.aria)
mod utils  // Loads from utils.aria or utils/mod.aria

// Use items from module
use utils.helper
use utils.{helper, another}
use utils.*  // Import all public items

// Rename on import
use std.collections.HashMap as Map
```

### 7.2 Visibility

```aria
// Private (default)
fn internal() { }

// Public
pub fn external() { }

// Public within crate
pub(crate) fn crate_visible() { }

// Public within parent module
pub(super) fn parent_visible() { }

// Public within specific module
pub(in crate.utils) fn utils_visible() { }
```

---

## 8. Attributes

```aria
// Item attributes
#[test]
fn test_addition() {
    assert_eq(1 + 1, 2)
}

#[inline]
fn hot_path() { }

#[deprecated(since: "2.0", note: "Use new_function instead")]
fn old_function() { }

// Conditional compilation
#[cfg(target_os: "linux")]
fn linux_only() { }

#[cfg(debug)]
fn debug_only() { }

// Documentation
#[doc("This function does something important")]
fn important() { }

// Multiple attributes
#[derive(Debug, Clone)]
#[repr(C)]
struct FFIStruct { }
```

---

## 9. Generics and Constraints

### 9.1 Generic Types

```aria
// Generic struct
struct Container[T] {
    value: T
}

// Multiple type parameters
struct Pair[A, B] {
    first: A
    second: B
}

// Generic enum
enum Option[T] {
    Some(T)
    None
}
```

### 9.2 Trait Bounds

```aria
// Single bound
fn print_all[T: Display](items: &List[T]) {
    for item in items {
        print(item.display())
    }
}

// Multiple bounds
fn compare_and_show[T: Ord + Display](a: T, b: T) {
    if a < b {
        print("{a.display()} < {b.display()}")
    }
}

// Where clause for complex bounds
fn process[T, U](t: T, u: U) -> Result[T, U]
where
    T: Clone + Display,
    U: Error
{
    // ...
}

// Associated type bounds
fn sum[I: Iterator](iter: I) -> I.Item
where
    I.Item: Add
{
    // ...
}
```

---

## 10. Async/Await

```aria
// Async function
async fn fetch(url: String) -> Result[Response, Error] {
    let connection = await connect(url)?
    await connection.get()
}

// Await expression
let data = await fetch("https://example.com")

// Concurrent execution
let (a, b) = await join(fetch(url1), fetch(url2))

// Async blocks
let future = async {
    let x = await compute_x()
    let y = await compute_y()
    x + y
}

// Spawn task
spawn(async {
    await do_background_work()
})
```

---

## 11. Unsafe Code

```aria
// Unsafe block
unsafe {
    // Raw pointer dereference
    let value = *raw_ptr

    // Call unsafe function
    external_c_function()
}

// Unsafe function
unsafe fn dangerous_operation() {
    // Entire body is unsafe context
}

// Unsafe trait
unsafe trait RawAccess {
    fn raw_ptr(&self) -> *const u8
}

// Implement unsafe trait
unsafe impl RawAccess for Buffer {
    fn raw_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }
}
```

---

## 12. Semantic Rules

### 12.1 Name Resolution

1. Local variables shadow outer scopes
2. Module items resolved by path
3. Trait methods resolved by type + trait in scope
4. Generic types resolved at instantiation

### 12.2 Type Inference

1. Local inference within functions
2. Global inference for generic bounds
3. Explicit annotations at API boundaries
4. Bidirectional type flow

### 12.3 Move Semantics

1. Assignment moves by default (non-Copy types)
2. Copy types are copied on assignment
3. Borrows create references without moving
4. Destructors run when owner goes out of scope
