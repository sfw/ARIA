# ARIA v2 Syntax Specification

## Design Principles

1. **Minimum tokens** for any construct
2. **Indentation-significant** (braces optional)
3. **Short keywords** (1-3 characters)
4. **Implicit where possible** (no `let`, smart defaults)
5. **Pipeline-first** for data transformation

---

## 1. Lexical Elements

### 1.1 Keywords

| Keyword | Meaning | Notes |
|---------|---------|-------|
| `f` | function | |
| `s` | struct | |
| `e` | enum | |
| `t` | trait | |
| `i` | impl | |
| `m` | match | |
| `if` | conditional | kept for readability |
| `for` | loop | kept for readability |
| `wh` | while | |
| `lp` | infinite loop | |
| `br` | break | |
| `ct` | continue | |
| `ret` | return | often implicit |
| `aw` | await | |
| `as` | async | |
| `mut` | mutable borrow | |
| `mv` | move | |
| `us` | use/import | |
| `md` | module | |
| `pub` | public | |
| `un` | unsafe | |

### 1.2 Operators

```
# Arithmetic
+  -  *  /  %

# Comparison
== != < <= > >=

# Logical
&& || !

# Bitwise
& | ^ << >>

# Assignment
=     # immutable binding
:=    # mutable binding
+=  -=  *=  /=

# Special
?     # error propagation / optional
!     # Result type suffix
|     # pipeline / pattern or
->    # function return / pattern branch
=>    # fat arrow (closures)
..    # range exclusive
..=   # range inclusive
```

### 1.3 Literals

```
# Integers
42        # Int (inferred)
42i32     # explicit i32
42u8      # explicit u8
0xFF      # hex
0b1010    # binary
1_000_000 # underscores ok

# Floats
3.14      # Float (f64)
3.14f32   # explicit f32
2.5e10    # scientific

# Strings
"hello"           # String
"value: {x}"      # interpolation
"sum: {a + b}"    # expression interpolation
`raw string`      # no escapes
```
multiline
string
```           # triple backtick

# Characters
'a'  '\n'  '\u{1F600}'

# Boolean
T  F        # true/false shortcuts (or true/false)

# None
N           # None shortcut (or none)
```

---

## 2. Declarations

### 2.1 Variables

```
# Immutable (default)
x = 42
name = "Alice"

# Mutable
count := 0
count += 1

# With type annotation (when needed)
x: Int = 42
items: [Str] = []

# Constants (compile-time)
PI :: 3.14159
MAX :: 1000
```

### 2.2 Functions

```
# Basic function
f add(a: Int, b: Int) -> Int
    a + b

# No return value
f log(msg: Str)
    print msg

# Single expression (inline)
f double(n: Int) -> Int = n * 2

# With braces (optional)
f complex(x: Int) -> Int {
    y = x * 2
    z = y + 1
    z
}

# Generic
f first[T](list: [T]) -> T?
    if list.empty then N else list[0]

# Multiple returns (tuple)
f divmod(a: Int, b: Int) -> (Int, Int)
    (a / b, a % b)

# Default parameters
f connect(host: Str, port: Int = 8080) -> Conn
    # ...

# Async
as f fetch(url: Str) -> Data!
    resp = aw http.get url?
    aw resp.json
```

### 2.3 Structs

```
# Basic struct
s Point
    x: Float
    y: Float

# Single line
s Color(r: u8, g: u8, b: u8)

# With defaults
s Config
    host: Str = "localhost"
    port: Int = 8080
    debug: B = F

# Generic
s Box[T]
    value: T

# Tuple struct
s UserId(Int)

# Unit struct
s Marker
```

### 2.4 Enums

```
# Simple enum
e Direction
    North
    South
    East
    West

# With data
e Shape
    Circle(r: Float)
    Rect(w: Float, h: Float)
    Point

# Generic
e Option[T]
    Some(T)
    None

e Result[T, E]
    Ok(T)
    Err(E)

# Single line
e Bool = True | False
```

### 2.5 Traits

```
# Basic trait
t Display
    f display(&self) -> Str

# With default implementation
t Greet
    f name(&self) -> Str
    f greet(&self) -> Str = "Hello, {self.name}!"

# With associated type
t Iterator
    type Item
    f next(&mut self) -> Self.Item?

# Generic trait
t Into[T]
    f into(self) -> T
```

### 2.6 Implementations

```
# Implement trait
i Display for Point
    f display(&self) -> Str
        "({self.x}, {self.y})"

# Inherent methods
i Point
    f new(x: Float, y: Float) -> Point
        Point(x, y)

    f distance(&self, other: &Point) -> Float
        dx = self.x - other.x
        dy = self.y - other.y
        (dx*dx + dy*dy).sqrt

# Generic implementation
i[T] Display for [T] where T: Display
    f display(&self) -> Str
        items = self | map .display | join ", "
        "[{items}]"
```

---

## 3. Expressions

### 3.1 Basic Expressions

```
# Arithmetic
a + b
a - b
a * b
a / b
a % b

# Comparison
a == b
a != b
a < b
a <= b

# Logical
a && b
a || b
!a

# Grouping
(a + b) * c
```

### 3.2 Control Flow

```
# If expression
result = if cond then a else b

# Multi-line if
if cond
    do_something
else if other
    do_other
else
    do_default

# Match expression
m value
    0 -> "zero"
    1..=9 -> "digit"
    n if n < 0 -> "negative"
    _ -> "other"

# Match with destructuring
m point
    Point(0, 0) -> "origin"
    Point(x, 0) -> "x-axis at {x}"
    Point(0, y) -> "y-axis at {y}"
    Point(x, y) -> "({x}, {y})"

# Inline match
x | 0 -> "zero" | _ -> "other"

# For loop
for item in items
    process item

for i in 0..10
    print i

for (idx, val) in items.enum
    print "{idx}: {val}"

# While loop
wh condition
    do_work

# Infinite loop
lp
    if done then br
    work
```

### 3.3 Pipeline Operator

The pipeline `|` is the primary way to chain operations:

```
# Basic pipeline
items | filter(> 0) | map(* 2) | sum

# Equivalent to
sum(map(filter(items, |x| x > 0), |x| x * 2))

# With method shorthand
users | filter .active | map .name | sort

# Equivalent to
users.filter(|u| u.active).map(|u| u.name).sort()

# Pipeline with blocks
data
    | parse
    | validate?
    | transform
    | save?

# Partial application in pipeline
numbers | map(+ 10)      # add 10 to each
numbers | filter(> 0)    # keep positive
numbers | fold(0, +)     # sum
```

### 3.4 Closures

```
# Full syntax
|a: Int, b: Int| -> Int { a + b }

# Inferred types
|a, b| a + b

# Single parameter
|x| x * 2

# No parameters
|| print "hello"

# Shorthand for field/method access
.name           # |x| x.name
.len            # |x| x.len
.is_empty       # |x| x.is_empty

# Shorthand for operators
(+ 10)          # |x| x + 10
(* 2)           # |x| x * 2
(> 0)           # |x| x > 0
(== "test")     # |x| x == "test"
```

### 3.5 Error Handling Expressions

```
# Propagate error
value?

# Default on None/Err
value ?? default

# Map and propagate
result | map(process)?

# Chain operations
open path? | read? | parse?
```

---

## 4. Pattern Matching

### 4.1 Pattern Types

```
# Literal
m x
    0 -> "zero"
    1 -> "one"

# Variable binding
m opt
    Some v -> use v
    None -> default

# Wildcard
m x
    _ -> "anything"

# Or pattern
m x
    0 | 1 | 2 -> "small"
    _ -> "big"

# Range
m x
    0..=9 -> "digit"
    'a'..='z' -> "lower"

# Guard
m x
    n if n < 0 -> "negative"
    n if n > 0 -> "positive"
    _ -> "zero"

# Struct destructure
m point
    Point(x, y) -> "{x}, {y}"

# Nested
m data
    Some(Ok v) -> use v
    Some(Err e) -> handle e
    None -> default

# Binding with @
m x
    n @ 1..=10 -> "small: {n}"
    n @ _ -> "other: {n}"
```

### 4.2 Pattern Contexts

```
# Let destructuring
(a, b) = get_pair
Point(x, y) = get_point
[first, second, ..rest] = list

# If-let
if Some v = maybe_value
    use v

# While-let
wh Some item = iter.next
    process item

# For pattern
for (k, v) in map
    print "{k}: {v}"
```

---

## 5. Modules and Imports

```
# Import
us std.collections.Map
us std.io.{read, write}
us std.*

# Rename
us std.collections.HashMap -> Map

# Module definition
md utils
    pub f helper -> Int = 42

# File-based module
md utils  # loads utils.ar or utils/mod.ar

# Visibility
pub f public_fn -> Int = 42
pub(crate) f crate_fn -> Int = 42
f private_fn -> Int = 42
```

---

## 6. Attributes

```
# Item attributes
@test
f test_add
    assert_eq(add(1, 2), 3)

@inline
f hot_path -> Int = 42

@derive(Debug, Clone, Eq)
s User
    name: Str
    age: Int

# Conditional compilation
@cfg(os = "linux")
f linux_only
    # ...

@cfg(debug)
f debug_only
    # ...
```

---

## 7. Generics and Constraints

```
# Generic function
f first[T](list: [T]) -> T?
    # ...

# Multiple type parameters
f swap[A, B](pair: (A, B)) -> (B, A)
    (pair.1, pair.0)

# Trait bounds
f print_all[T: Display](items: [T])
    for item in items
        print item.display

# Multiple bounds
f compare[T: Ord + Display](a: T, b: T)
    # ...

# Where clause
f process[T, U](t: T, u: U) -> T
    where T: Clone + Into[U]
          U: Display
    # ...
```

---

## 8. Async/Await

```
# Async function
as f fetch(url: Str) -> Data!
    resp = aw http.get url?
    aw resp.json

# Await expression
data = aw fetch "https://api.example.com"

# Concurrent execution
(a, b) = aw join(fetch url1, fetch url2)

# Async block
future = as
    x = aw compute_x
    y = aw compute_y
    x + y

# Spawn task
spawn as
    aw background_work
```

---

## 9. Unsafe Code

```
# Unsafe block
un
    ptr = get_raw_ptr
    *ptr = 42

# Unsafe function
un f dangerous
    # entire body is unsafe

# Unsafe trait
un t RawAccess
    f raw_ptr(&self) -> *u8

# Implement unsafe trait
un i RawAccess for Buffer
    f raw_ptr(&self) -> *u8
        self.data.as_ptr
```

---

## 10. Complete Example

```
# A complete program in ARIA v2

us std.io
us std.json

@derive(Debug, Clone)
s User
    id: Int
    name: Str
    email: Str
    active: B = T

t Repository
    type Item
    f find(&self, id: Int) -> Self.Item?
    f save(&mut self, item: Self.Item) -> Result!
    f delete(&mut self, id: Int) -> Result!

s UserRepo
    users: {Int: User}

i Repository for UserRepo
    type Item = User

    f find(&self, id: Int) -> User?
        self.users.get id

    f save(&mut self, user: User) -> Result!
        self.users.insert(user.id, user)
        ok()

    f delete(&mut self, id: Int) -> Result!
        self.users.remove id
        ok()

as f handle_request(req: Request, repo: &mut UserRepo) -> Response!
    m req.method
        GET ->
            id = req.param "id" | parse?
            user = repo.find id ?? ret Response.not_found
            Response.json user

        POST ->
            user = req.body | json.parse?
            repo.save user?
            Response.created user

        DELETE ->
            id = req.param "id" | parse?
            repo.delete id?
            Response.ok

        _ -> Response.method_not_allowed

f main
    repo := UserRepo(users: {})

    # Add some users
    repo.save User(1, "Alice", "alice@example.com")?
    repo.save User(2, "Bob", "bob@example.com")?

    # Query
    for (id, user) in repo.users
        print "User {id}: {user.name}"

    # Filter active users
    active = repo.users.values
        | filter .active
        | map .name
        | collect

    print "Active users: {active}"
```
