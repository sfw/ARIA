# ARIA v2 Type System

## Design Goals

1. **Maximum inference** — annotate only at boundaries
2. **Short type names** — `Str` not `String`, `B` not `Bool`
3. **Type shortcuts** — `[T]` for List, `T?` for Option
4. **Predictable rules** — easy for AI to generate

---

## 1. Primitive Types

### 1.1 Numeric Types

```
# Signed integers
i8   i16   i32   i64   i128
Int                          # platform native (i64 on 64-bit)

# Unsigned integers
u8   u16   u32   u64   u128
UInt                         # platform native

# Floating point
f32   f64
Float                        # alias for f64

# Shorthand in literals
42        # Int
42i32     # i32
42u8      # u8
3.14      # Float
3.14f32   # f32
```

### 1.2 Text Types

```
Str       # UTF-8 string (owned)
&str      # string slice (borrowed)
Char      # Unicode scalar value
```

### 1.3 Boolean

```
B         # Boolean type
T         # true literal
F         # false literal

# Also valid
Bool      # full name
true      # full literal
false     # full literal
```

### 1.4 Unit and Never

```
()        # Unit type (no value)
!         # Never type (never returns)
```

---

## 2. Type Shortcuts

### 2.1 Collections

| Full Type | Shortcut | Example |
|-----------|----------|---------|
| `List[T]` | `[T]` | `[Int]`, `[Str]` |
| `Map[K,V]` | `{K:V}` | `{Str:Int}` |
| `Set[T]` | `{T}` | `{Int}` |
| `Array[T,N]` | `[T;N]` | `[Int;5]` |
| `Slice[T]` | `&[T]` | `&[Int]` |

```
# Examples
numbers: [Int] = [1, 2, 3]
scores: {Str:Int} = {"alice": 100, "bob": 95}
unique: {Int} = {1, 2, 3}
fixed: [Int;5] = [1, 2, 3, 4, 5]
```

### 2.2 Option and Result

| Full Type | Shortcut | Example |
|-----------|----------|---------|
| `Option[T]` | `T?` | `Int?`, `Str?` |
| `Result[T,E]` | `T!E` | `Int!Error` |
| `Result[T,Error]` | `T!` | `Data!` (default Error) |

```
# Option
maybe: Int? = Some 42
empty: Str? = N

# Result with explicit error
result: Int!ParseError = Ok 42

# Result with default Error type
data: Data! = fetch url?
```

### 2.3 Tuples

```
# Tuple types
pair: (Int, Str) = (42, "hello")
triple: (Int, Int, Int) = (1, 2, 3)

# Access
first = pair.0
second = pair.1

# Destructure
(a, b) = pair
```

### 2.4 Function Types

```
# Function type syntax
callback: Int -> Bool
transform: (Int, Int) -> Int
producer: () -> Str
consumer: Str -> ()

# With shortcuts
handler: Request -> Response!
parser: Str -> Data!ParseError
```

---

## 3. Composite Types

### 3.1 Structs

```
s Point
    x: Float
    y: Float

s User
    id: Int
    name: Str
    email: Str
    active: B = T     # default value

# Usage
p = Point(1.0, 2.0)
u = User(id: 1, name: "Alice", email: "a@b.com")
```

### 3.2 Enums

```
e Direction
    North
    South
    East
    West

e Shape
    Circle(r: Float)
    Rect(w: Float, h: Float)

# Usage
dir = Direction.North
shape = Shape.Circle(5.0)
```

### 3.3 Type Aliases

```
# Simple alias
type UserId = Int
type Handler = Request -> Response!

# Generic alias
type Pair[T] = (T, T)
type StrMap[V] = {Str:V}
```

### 3.4 Newtypes

```
# Distinct types via tuple struct
s UserId(Int)
s Email(Str)

# These are different types
id: UserId = UserId(42)
email: Email = Email("test@example.com")

# Type safety
f get_user(id: UserId) -> User?   # can't pass raw Int
```

---

## 4. Generic Types

### 4.1 Generic Definitions

```
# Generic struct
s Box[T]
    value: T

# Multiple type parameters
s Pair[A, B]
    first: A
    second: B

# Generic enum
e Option[T]
    Some(T)
    None

e Result[T, E]
    Ok(T)
    Err(E)
```

### 4.2 Trait Bounds

```
# Single bound
f print_all[T: Display](items: [T])
    for item in items
        print item.display

# Multiple bounds
f compare[T: Ord + Display](a: T, b: T)
    if a < b
        print "{a} < {b}"

# Where clause (for complex bounds)
f process[T, U](t: T, u: U) -> T
    where T: Clone + Into[U]
          U: Display
    # ...
```

### 4.3 Associated Types

```
t Iterator
    type Item
    f next(&mut self) -> Self.Item?

t Container
    type Elem
    f get(&self, idx: Int) -> Self.Elem?

# Implementation
i Iterator for Counter
    type Item = Int
    f next(&mut self) -> Int?
        # ...
```

---

## 5. Reference Types

### 5.1 Borrows

```
# Immutable borrow
f read(data: &[Int]) -> Int
    data.sum

# Mutable borrow
f modify(data: &mut [Int])
    data.push 42

# Reference in expression
ref = &value
mut_ref = &mut value
```

### 5.2 Smart Pointers

```
Box[T]        # Owned heap allocation
Rc[T]         # Reference counted (single-thread)
Arc[T]        # Atomic reference counted (multi-thread)
Cell[T]       # Interior mutability (Copy types)
RefCell[T]    # Interior mutability (runtime checked)
Mutex[T]      # Thread-safe interior mutability
RwLock[T]     # Reader-writer lock
```

### 5.3 Raw Pointers (unsafe)

```
*T            # Raw immutable pointer
*mut T        # Raw mutable pointer

un
    ptr: *Int = get_ptr
    val = *ptr
```

---

## 6. Type Inference

### 6.1 Where Inference Works

```
# Variable initialization
x = 42                    # Int
name = "Alice"            # Str
items = [1, 2, 3]        # [Int]

# Closure parameters
items | map |x| x * 2     # x: Int inferred

# Generic instantiation
first([1, 2, 3])         # T = Int inferred

# Return type from body
f double(n: Int) = n * 2  # return Int inferred
```

### 6.2 Where Annotation Required

```
# Empty collections
items: [Int] = []
map: {Str:Int} = {}

# Ambiguous literals
x: i32 = 0               # could be any int type
y: f32 = 1.0             # could be f32 or f64

# Public function signatures
pub f add(a: Int, b: Int) -> Int
    a + b

# Turbofish for ambiguous generics
parsed = "42".parse[Int]?
```

---

## 7. Type Coercion

### 7.1 Implicit Coercions (Minimal)

```
# Reference coercions
&Str -> &str              # owned to slice
&[T] -> &[T]              # deref
&mut T -> &T              # mut to immut

# Never coercion
! -> T                    # never becomes any type
```

### 7.2 Explicit Conversions

```
# Numeric casts with 'as'
x: i32 = 42
y: i64 = x as i64

# Using Into/From
s: Str = "hello".into
n: Int = Int.from "42"?

# Parse method
n: Int = "42".parse?
f: Float = "3.14".parse?
```

---

## 8. Built-in Traits

### 8.1 Marker Traits

```
t Copy          # bitwise copyable
t Clone         # can be cloned
t Send          # can transfer across threads
t Sync          # can be shared across threads
t Sized         # has known size at compile time
```

### 8.2 Operator Traits

```
t Add           # a + b
t Sub           # a - b
t Mul           # a * b
t Div           # a / b
t Eq            # a == b
t Ord           # a < b, etc.
t Index         # a[i]
t Deref         # *a
```

### 8.3 Conversion Traits

```
t Into[T]       # convert self to T
t From[T]       # create Self from T
t TryInto[T]    # fallible conversion to T
t TryFrom[T]    # fallible creation from T
t AsRef[T]      # borrow as &T
```

### 8.4 Formatting Traits

```
t Display       # user-facing string
t Debug         # programmer-facing string
```

### 8.5 Derive

```
@derive(Debug, Clone, Eq, Hash)
s User
    name: Str
    age: Int

# Available derives
Debug           # debug formatting
Clone           # deep copy
Copy            # bitwise copy
Eq              # equality
Ord             # ordering
Hash            # hash computation
Default         # default value
```

---

## 9. Type Grammar

```ebnf
Type = PrimitiveType
     | CollectionShortcut
     | OptionShortcut
     | ResultShortcut
     | TupleType
     | FunctionType
     | ReferenceType
     | PathType

PrimitiveType = 'Int' | 'i8' | 'i16' | 'i32' | 'i64' | 'i128'
              | 'UInt' | 'u8' | 'u16' | 'u32' | 'u64' | 'u128'
              | 'Float' | 'f32' | 'f64'
              | 'Str' | 'Char' | 'B' | 'Bool'
              | '()' | '!'

CollectionShortcut = '[' Type ']'              # List
                   | '[' Type ';' Expr ']'     # Array
                   | '{' Type ':' Type '}'      # Map
                   | '{' Type '}'               # Set

OptionShortcut = Type '?'

ResultShortcut = Type '!' Type?

TupleType = '(' Type (',' Type)* ')'

FunctionType = Type '->' Type
             | '(' TypeList? ')' '->' Type

ReferenceType = '&' 'mut'? Type
              | '*' 'mut'? Type

PathType = IDENT GenericArgs?
         | IDENT '::' PathType

GenericArgs = '[' Type (',' Type)* ']'
```

---

## 10. Examples

```
# Type shortcut examples

# Collections
items: [Int] = [1, 2, 3]
matrix: [[Int]] = [[1, 2], [3, 4]]
cache: {Str:Data} = {}
ids: {Int} = {1, 2, 3}

# Options
maybe_name: Str? = Some "Alice"
no_value: Int? = N

# Results
result: Int!ParseError = parse "42"
simple: Data! = fetch url?

# Functions
transform: Int -> Str = |n| "{n}"
binary: (Int, Int) -> Int = |a, b| a + b
predicate: Int -> B = (> 0)

# Complex types
handlers: {Str: Request -> Response!} = {
    "get": handle_get
    "post": handle_post
}

callbacks: [(Int -> B)] = [
    (> 0)
    (< 100)
    |n| n % 2 == 0
]
```
