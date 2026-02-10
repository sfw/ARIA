# ARIA Type System Specification

## Design Goals

1. **Strong Static Typing**: Catch errors at compile time
2. **Powerful Inference**: Minimize annotation burden
3. **AI-Friendly**: Predictable rules for code generation
4. **Memory Safety**: Types encode ownership and mutability
5. **Expressiveness**: Generics, traits, and algebraic types

---

## 1. Primitive Types

### 1.1 Numeric Types

```aria
// Signed integers
i8      // -128 to 127
i16     // -32,768 to 32,767
i32     // -2³¹ to 2³¹-1
i64     // -2⁶³ to 2⁶³-1
i128    // -2¹²⁷ to 2¹²⁷-1
Int     // Platform-native signed (i64 on 64-bit)

// Unsigned integers
u8      // 0 to 255
u16     // 0 to 65,535
u32     // 0 to 2³²-1
u64     // 0 to 2⁶⁴-1
u128    // 0 to 2¹²⁸-1
UInt    // Platform-native unsigned

// Floating point
f32     // 32-bit IEEE 754
f64     // 64-bit IEEE 754
Float   // Alias for f64
```

### 1.2 Text Types

```aria
Char    // Unicode scalar value (4 bytes)
String  // UTF-8 encoded, owned string
&str    // String slice (borrowed)
```

### 1.3 Boolean

```aria
Bool    // true or false
```

### 1.4 Unit Type

```aria
Unit    // Zero-size type, written as ()
        // Returned by functions with no meaningful return
```

### 1.5 Never Type

```aria
Never   // Type that can never be instantiated
        // For functions that never return (panic, infinite loop)
```

---

## 2. Compound Types

### 2.1 Tuples

```aria
// Fixed-size, heterogeneous collection
let pair: (Int, String) = (42, "hello")
let triple: (Int, Int, Int) = (1, 2, 3)

// Access by index
let first = pair.0
let second = pair.1

// Destructuring
let (x, y, z) = triple

// Unit is the empty tuple
let unit: () = ()
```

### 2.2 Arrays

```aria
// Fixed-size, homogeneous collection
let numbers: [Int; 5] = [1, 2, 3, 4, 5]
let zeros: [Int; 100] = [0; 100]  // 100 zeros

// Access by index
let first = numbers[0]

// Slices (borrowed view)
let slice: &[Int] = &numbers[1..4]
```

### 2.3 Structs

```aria
// Named fields
struct Point {
    x: Float
    y: Float
}

// Tuple struct
struct Color(u8, u8, u8)

// Unit struct
struct Marker

// Generic struct
struct Box[T] {
    value: T
}
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

// Enum with data
enum Message {
    Quit
    Move { x: Int, y: Int }
    Write(String)
    ChangeColor(u8, u8, u8)
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
```

---

## 3. Reference Types

### 3.1 Borrowing Rules

ARIA uses a simplified borrowing model based on **second-class references**:

1. References cannot be stored in structs or returned from functions (with exceptions)
2. References are always valid (no dangling references)
3. Either one mutable reference OR multiple immutable references

```aria
// Immutable borrow
fn read_data(data: &List[Int]) -> Int {
    data.sum()
}

// Mutable borrow
fn modify_data(data: &mut List[Int]) {
    data.push(42)
}

// Cannot return reference to local (compile error)
fn bad() -> &Int {
    let x = 42
    &x  // ERROR: cannot return reference to local
}
```

### 3.2 Reference Exceptions (Scoped Returns)

```aria
// Can return reference that lives as long as input
fn first[T](list: &List[T]) -> &T {
    &list[0]  // OK: reference derived from input
}

// Method can return reference to self
impl Container[T] {
    fn get(&self) -> &T {
        &self.value  // OK: tied to self's lifetime
    }
}
```

### 3.3 Smart Pointers

```aria
// Owned heap allocation
Box[T]      // Single owner, heap allocated

// Reference counted
Rc[T]       // Single-threaded reference counting
Arc[T]      // Atomic reference counting (thread-safe)

// Interior mutability
Cell[T]     // Single-threaded mutation of Copy types
RefCell[T]  // Single-threaded runtime borrow checking
Mutex[T]    // Thread-safe mutation with locking
RwLock[T]   // Reader-writer lock
```

---

## 4. Type Inference

### 4.1 Local Inference

```aria
// Type inferred from initialization
let x = 42          // Int
let y = 3.14        // Float
let z = "hello"     // String
let list = [1, 2, 3]  // [Int; 3]

// Type inferred from usage
let mut vec = []
vec.push(42)        // vec: List[Int]

// Type inferred from return
fn compute() -> Int {
    let result = complex_calculation()  // result: Int
    result
}
```

### 4.2 Bidirectional Inference

```aria
// Type flows from annotation to expression
let numbers: List[i32] = [1, 2, 3]  // Literals typed as i32

// Type flows from context
fn takes_i32(x: i32) { }
takes_i32(42)  // 42 typed as i32

// Closure parameter inference
let numbers = [1, 2, 3]
numbers.map(|x| x * 2)  // x inferred as Int
```

### 4.3 When Annotations Are Required

```aria
// Public function signatures (always)
pub fn add(a: Int, b: Int) -> Int { a + b }

// Ambiguous literals
let x: i32 = 0     // Could be any integer type
let y: f32 = 1.0   // Could be f32 or f64

// Complex generic bounds
fn process[T: Display + Clone](value: T) { }

// Turbofish for generic function calls
let parsed = "42".parse::[Int]()?
```

---

## 5. Generics

### 5.1 Generic Functions

```aria
// Single type parameter
fn identity[T](x: T) -> T {
    x
}

// Multiple type parameters
fn swap[A, B](pair: (A, B)) -> (B, A) {
    (pair.1, pair.0)
}

// Const generics (compile-time values)
fn create_array[T, const N: usize]() -> [T; N]
where T: Default
{
    [T.default(); N]
}
```

### 5.2 Generic Types

```aria
// Generic struct
struct Stack[T] {
    items: List[T]
}

// Generic enum
enum Tree[T] {
    Leaf(T)
    Node(Box[Tree[T]], Box[Tree[T]])
}

// Multiple parameters with constraints
struct Map[K, V] where K: Hash + Eq {
    // ...
}
```

### 5.3 Associated Types

```aria
trait Iterator {
    type Item
    fn next(&mut self) -> Option[Self.Item]
}

impl Iterator for Counter {
    type Item = Int
    fn next(&mut self) -> Option[Int] {
        // ...
    }
}

// Using associated types in bounds
fn sum[I: Iterator](iter: I) -> I.Item
where I.Item: Add + Default
{
    let mut total = I.Item.default()
    for item in iter {
        total = total + item
    }
    total
}
```

---

## 6. Trait System

### 6.1 Trait Definition

```aria
// Basic trait
trait Display {
    fn display(&self) -> String
}

// Trait with default method
trait Greet {
    fn name(&self) -> String

    fn greet(&self) -> String {
        "Hello, {self.name()}!"
    }
}

// Trait with associated type
trait Container {
    type Item
    fn get(&self, index: Int) -> Option[Self.Item]
}

// Trait with associated constant
trait Bounded {
    const MIN: Self
    const MAX: Self
}

// Generic trait
trait Into[T] {
    fn into(self) -> T
}
```

### 6.2 Trait Bounds

```aria
// Single bound
fn print[T: Display](value: T) {
    print(value.display())
}

// Multiple bounds
fn compare[T: Ord + Display](a: T, b: T) {
    // ...
}

// Where clause
fn complex[T, U](t: T, u: U) -> T
where
    T: Clone + Into[U],
    U: Display
{
    // ...
}

// Bound on associated type
fn process[C: Container](c: C) -> C.Item
where
    C.Item: Clone
{
    // ...
}
```

### 6.3 Trait Inheritance

```aria
// Subtrait
trait Error: Display {
    fn source(&self) -> Option[&Error]
}

// Multiple supertraits
trait Serializable: Clone + Display {
    fn serialize(&self) -> Bytes
}
```

### 6.4 Blanket Implementations

```aria
// Implement for all types satisfying bound
impl[T: Display] ToString for T {
    fn to_string(&self) -> String {
        self.display()
    }
}

// Conditional implementation
impl[T: Clone] Clone for Option[T] {
    fn clone(&self) -> Self {
        match self {
            Some(v) => Some(v.clone())
            None => None
        }
    }
}
```

---

## 7. Special Types

### 7.1 Option Type

```aria
enum Option[T] {
    Some(T)
    None
}

// Methods
impl[T] Option[T] {
    fn is_some(&self) -> Bool
    fn is_none(&self) -> Bool
    fn unwrap(self) -> T                    // Panics if None
    fn unwrap_or(self, default: T) -> T
    fn map[U](self, f: fn(T) -> U) -> Option[U]
    fn and_then[U](self, f: fn(T) -> Option[U]) -> Option[U]
    fn ok_or[E](self, err: E) -> Result[T, E]
}

// Syntactic sugar
let maybe: Option[Int] = some(42)   // Option.Some(42)
let empty: Option[Int] = none       // Option.None

// Null coalescing
let value = maybe ?? 0  // Unwrap or default
```

### 7.2 Result Type

```aria
enum Result[T, E] {
    Ok(T)
    Err(E)
}

// Methods
impl[T, E] Result[T, E] {
    fn is_ok(&self) -> Bool
    fn is_err(&self) -> Bool
    fn unwrap(self) -> T                    // Panics if Err
    fn unwrap_err(self) -> E                // Panics if Ok
    fn map[U](self, f: fn(T) -> U) -> Result[U, E]
    fn map_err[F](self, f: fn(E) -> F) -> Result[T, F]
    fn and_then[U](self, f: fn(T) -> Result[U, E]) -> Result[U, E]
    fn ok(self) -> Option[T]
    fn err(self) -> Option[E]
}

// Syntactic sugar
let success: Result[Int, String] = ok(42)
let failure: Result[Int, String] = err("failed")

// Error propagation
fn process() -> Result[Data, Error] {
    let file = open("data.txt")?    // Returns early if Err
    let content = file.read()?
    parse(content)
}
```

### 7.3 Collection Types

```aria
// Dynamic array
List[T]

// Hash map
Map[K, V] where K: Hash + Eq

// Hash set
Set[T] where T: Hash + Eq

// Double-ended queue
Deque[T]

// Ordered map (tree-based)
OrderedMap[K, V] where K: Ord

// Ordered set (tree-based)
OrderedSet[T] where T: Ord
```

---

## 8. Type Aliases and Newtypes

### 8.1 Type Aliases

```aria
// Simple alias (same type, different name)
type UserId = Int
type Callback = fn(Int) -> Bool
type StringResult = Result[String, Error]

// Generic alias
type Pair[T] = (T, T)
type StringMap[V] = Map[String, V]
```

### 8.2 Newtypes (Distinct Types)

```aria
// Newtype pattern using tuple struct
struct UserId(Int)
struct Email(String)

// These are distinct types
let user: UserId = UserId(42)
let email: Email = Email("test@example.com")

// Cannot be confused
fn get_user(id: UserId) -> User { }
// get_user(42)        // ERROR: expected UserId, got Int
// get_user(email)     // ERROR: expected UserId, got Email
get_user(UserId(42))   // OK
```

---

## 9. Type Coercion

### 9.1 Implicit Coercions

ARIA performs minimal implicit coercion to maintain predictability:

```aria
// Reference coercion (deref)
&String -> &str
&List[T] -> &[T]
&Box[T] -> &T

// Mutability weakening
&mut T -> &T

// Never coercion
Never -> T  // Never can become any type
```

### 9.2 Explicit Conversions

```aria
// Using 'as' for primitive casts
let x: i32 = 42
let y: i64 = x as i64

// Using Into/From traits
let s: String = "hello".into()
let n: Int = Int.from("42")?

// Parse methods
let n: Int = "42".parse()?
let f: Float = "3.14".parse()?
```

---

## 10. Variance

### 10.1 Covariance

```aria
// If Child <: Parent, then Container[Child] <: Container[Parent]
// Applies to: immutable references, return types

fn process(animals: &List[Animal]) { }
let dogs: List[Dog] = get_dogs()
process(&dogs)  // OK: &List[Dog] coerces to &List[Animal]
```

### 10.2 Contravariance

```aria
// If Child <: Parent, then Fn(Parent) <: Fn(Child)
// Applies to: function parameters

fn apply(f: fn(Dog) -> Unit, dog: Dog) {
    f(dog)
}
let handler: fn(Animal) -> Unit = |a| print(a)
apply(handler, my_dog)  // OK: fn(Animal) works where fn(Dog) expected
```

### 10.3 Invariance

```aria
// If Child <: Parent, Container[Child] is NOT related to Container[Parent]
// Applies to: mutable references

fn modify(animals: &mut List[Animal]) {
    animals.push(Cat())  // Could add a Cat!
}
let mut dogs: List[Dog] = get_dogs()
// modify(&mut dogs)  // ERROR: would allow Cat in List[Dog]
```

---

## 11. Type System Features for AI

### 11.1 Predictable Rules

- No implicit conversions except documented coercions
- Consistent type inference algorithm
- Clear error messages with specific locations

### 11.2 Structured Error Messages

```aria
// Error format for AI consumption
Error[E0001]: Type mismatch
  --> file.aria:10:5
   |
10 |     let x: Int = "hello"
   |            ^^^   ^^^^^^^ found String
   |            |
   |            expected Int
   |
Help: Use parse() to convert String to Int:
   |
10 |     let x: Int = "hello".parse()?
   |                         ^^^^^^^^
```

### 11.3 Grammar-Constrained Generation

Type syntax is designed to be parseable with DFA:

```
Type := PrimitiveType
      | Identifier
      | Identifier '[' TypeList ']'
      | '(' TypeList ')'
      | '&' Type
      | '&' 'mut' Type
      | 'fn' '(' TypeList ')' '->' Type
      | '[' Type ';' Expr ']'

TypeList := Type (',' Type)*
```
