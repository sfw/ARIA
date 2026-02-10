# ARIA Memory Model Specification

## Design Philosophy

ARIA's memory model prioritizes:
1. **Simplicity**: Learnable rules, no lifetime annotations
2. **Safety**: Compile-time guarantees, no undefined behavior
3. **Predictability**: Consistent behavior for AI code generation
4. **Performance**: Zero-cost abstractions where possible

The key innovation is **second-class references with scoped returns** — references cannot be stored in data structures but can be returned if derived from inputs.

---

## 1. Ownership

### 1.1 Core Rules

Every value in ARIA has exactly one owner:

```aria
let s1 = String.from("hello")  // s1 owns the string
let s2 = s1                     // Ownership moves to s2
// print(s1)                    // ERROR: s1 no longer valid

let s3 = s2.clone()             // s3 owns a copy
print(s2)                       // OK: s2 still valid
print(s3)                       // OK: s3 also valid
```

### 1.2 Move Semantics

By default, assignment moves ownership for non-Copy types:

```aria
fn take_ownership(s: String) {
    // s is now owned here
    print(s)
}  // s is dropped here

let my_string = String.from("hello")
take_ownership(my_string)
// my_string is no longer valid here
```

### 1.3 Copy Types

Types that implement `Copy` are duplicated on assignment instead of moved:

```aria
// Copy types: all primitives, tuples of Copy types, small fixed-size types
let x: Int = 42
let y = x       // x is copied to y
print(x)        // OK: x is still valid
print(y)        // OK: y has its own copy

// Explicit Copy trait
#[derive(Copy, Clone)]
struct Point {
    x: Float
    y: Float
}
```

### 1.4 Drop and Destructors

Values are dropped when their owner goes out of scope:

```aria
struct FileHandle {
    handle: RawHandle
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        close_raw_handle(self.handle)
    }
}

fn process() {
    let file = FileHandle.open("data.txt")
    // use file...
}  // file is dropped here, close_raw_handle called automatically
```

---

## 2. Borrowing

### 2.1 Second-Class References

ARIA uses **second-class references** — references that cannot be stored in data structures:

```aria
// ALLOWED: References as function parameters
fn process(data: &List[Int]) -> Int {
    data.sum()
}

// ALLOWED: References in local variables
fn compute(data: &Data) {
    let x = &data.field
    use(x)
}

// NOT ALLOWED: References in structs
struct Bad {
    reference: &Int  // ERROR: Cannot store reference in struct
}

// NOT ALLOWED: References in collections
let refs: List[&Int] = []  // ERROR: Cannot store references
```

### 2.2 Scoped Returns

References CAN be returned if they're derived from input references:

```aria
// OK: Return reference derived from input
fn first[T](list: &List[T]) -> &T {
    &list[0]
}

// OK: Multiple input references, return derived from one
fn longer[T](a: &List[T], b: &List[T]) -> &List[T] {
    if a.len() > b.len() { a } else { b }
}

// OK: Method returning reference to self
impl Container[T] {
    fn get(&self, index: Int) -> &T {
        &self.items[index]
    }
}

// ERROR: Cannot return reference to local
fn bad() -> &Int {
    let x = 42
    &x  // ERROR: x will be dropped
}
```

### 2.3 Borrowing Rules

```aria
// Rule 1: Multiple immutable borrows allowed
let list = [1, 2, 3]
let a = &list
let b = &list
print(a.len() + b.len())  // OK

// Rule 2: Only one mutable borrow at a time
let mut list = [1, 2, 3]
let a = &mut list
// let b = &mut list  // ERROR: already mutably borrowed
a.push(4)             // OK

// Rule 3: No immutable borrow while mutable borrow exists
let mut list = [1, 2, 3]
let a = &mut list
// let b = &list     // ERROR: mutable borrow active
a.push(4)
// After a's last use, can borrow again
let b = &list
print(b.len())
```

### 2.4 Reborrowing

```aria
fn takes_ref(x: &Int) { }

let mut value = 42
let r = &mut value
takes_ref(r)    // Implicitly reborrows as &Int
*r = 43         // Can still use r after reborrow
```

---

## 3. The Borrow Checker

### 3.1 Simplified Analysis

Because references are second-class, the borrow checker is dramatically simpler:

```aria
// Traditional Rust: Needs lifetime annotations
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str { ... }

// ARIA: No annotations needed, compiler infers from structure
fn longest(a: &String, b: &String) -> &String {
    if a.len() > b.len() { a } else { b }
}
```

### 3.2 Function-Local Analysis

Most borrow checking happens within a single function:

```aria
fn process(data: &mut Data) {
    let x = &data.field1
    let y = &data.field2
    // Borrow checker verifies x and y don't overlap
    print(x)
    print(y)

    let z = &mut data.field1
    // Borrow checker knows x is no longer used
    *z = 42
}
```

### 3.3 Split Borrows

Borrowing different fields is allowed:

```aria
struct Data {
    a: Int
    b: String
}

fn process(data: &mut Data) {
    let a_ref = &mut data.a
    let b_ref = &mut data.b  // OK: different fields
    *a_ref = 42
    b_ref.push('!')
}
```

---

## 4. Escape Hatches

### 4.1 Smart Pointers

When you need shared ownership or interior mutability:

```aria
// Reference counting for shared ownership
let shared = Rc.new(Data { ... })
let clone1 = shared.clone()
let clone2 = shared.clone()
// All three can access the data

// Thread-safe reference counting
let shared = Arc.new(Data { ... })
let handle = spawn(move || {
    // shared can be used across threads
})

// Interior mutability (single-threaded)
let cell = RefCell.new(42)
*cell.borrow_mut() = 43
print(*cell.borrow())

// Thread-safe interior mutability
let mutex = Mutex.new(42)
{
    let mut guard = mutex.lock()
    *guard = 43
}
```

### 4.2 Arenas

For complex data structures with internal references:

```aria
// Arena allocates memory that lives together
let arena = Arena.new()
let node1 = arena.alloc(Node { value: 1 })
let node2 = arena.alloc(Node { value: 2, next: node1 })
// All nodes valid as long as arena exists

// Typed arena for homogeneous allocations
let arena = TypedArena[Node].new()
```

### 4.3 Unsafe Code

For cases the safe model cannot express:

```aria
unsafe {
    // Raw pointer dereference
    let ptr: *mut Int = get_raw_pointer()
    *ptr = 42

    // Transmute between types
    let bytes: [u8; 4] = transmute(42i32)

    // Call C functions
    libc.malloc(size)
}
```

---

## 5. Memory Layout

### 5.1 Size and Alignment

```aria
// Query type size and alignment
const SIZE: usize = size_of[Int]()
const ALIGN: usize = align_of[Int]()

// Sized vs unsized types
trait Sized { }  // Most types are Sized

// Unsized types (dynamically sized)
[T]     // Slice (unknown length)
dyn Trait  // Trait object (unknown concrete type)
String  // Actually sized, but str is not
```

### 5.2 Struct Layout

```aria
// Default layout: Rust-like optimization
struct Point {
    x: Float  // 8 bytes
    y: Float  // 8 bytes
}  // Total: 16 bytes, alignment: 8

// C-compatible layout
#[repr(C)]
struct CPoint {
    x: Float
    y: Float
}

// Packed layout (no padding)
#[repr(packed)]
struct Packed {
    a: u8
    b: u32
}  // Total: 5 bytes (normally would be 8 with padding)
```

### 5.3 Enum Layout

```aria
// Discriminant + largest variant
enum Value {
    Int(i64)      // 8 bytes
    Float(f64)    // 8 bytes
    String(String) // 24 bytes (ptr + len + cap)
}
// Size: 1 byte discriminant + 24 bytes = 32 bytes (with alignment)

// Null pointer optimization
enum Option[T] {
    Some(T)
    None
}
// Option[&T] is same size as &T (None uses null)
// Option[Box[T]] is same size as Box[T]
```

---

## 6. Allocation Strategies

### 6.1 Stack Allocation

```aria
fn stack_example() {
    let x: Int = 42              // Stack
    let arr: [Int; 100] = [0; 100]  // Stack (fixed size)
    let point = Point { x: 1.0, y: 2.0 }  // Stack
}  // All deallocated when function returns
```

### 6.2 Heap Allocation

```aria
fn heap_example() {
    let boxed = Box.new(42)      // Heap
    let list = List.new()        // Heap (dynamic size)
    let string = String.from("hello")  // Heap
}  // Heap memory freed when owners dropped
```

### 6.3 Custom Allocators

```aria
// Global allocator
#[global_allocator]
static ALLOCATOR: CustomAllocator = CustomAllocator.new()

// Per-collection allocator
let list = List.with_allocator(bump_allocator)

// Arena-based allocation
fn parse(arena: &Arena) -> &Ast {
    let node = arena.alloc(AstNode { ... })
    // node lives as long as arena
}
```

---

## 7. Thread Safety

### 7.1 Send and Sync Traits

```aria
// Send: Can be transferred to another thread
trait Send { }

// Sync: Can be shared between threads (&T is Send)
trait Sync { }

// Most types are Send + Sync
// Exceptions:
// - Rc[T]: Not Send or Sync (not atomic)
// - RefCell[T]: Not Sync (runtime borrow checking not thread-safe)
// - Raw pointers: Not Send or Sync by default
```

### 7.2 Thread-Safe Types

```aria
// Arc for shared ownership across threads
let data = Arc.new(Data { ... })
let data_clone = data.clone()

spawn(move || {
    // Use data_clone in new thread
})

// Mutex for mutable shared state
let counter = Arc.new(Mutex.new(0))
let counter_clone = counter.clone()

spawn(move || {
    let mut guard = counter_clone.lock()
    *guard += 1
})
```

### 7.3 Data Race Prevention

```aria
// Compile-time prevention
let mut data = vec![1, 2, 3]

spawn(move || {
    data.push(4)  // data moved to thread
})

// data.push(5)  // ERROR: data was moved

// Safe sharing requires explicit synchronization
let data = Arc.new(Mutex.new(vec![1, 2, 3]))
let d1 = data.clone()
let d2 = data.clone()

spawn(move || { d1.lock().push(4) })
spawn(move || { d2.lock().push(5) })
```

---

## 8. Memory Safety Guarantees

### 8.1 What ARIA Prevents

1. **Use after free**: Ownership ensures values exist while used
2. **Double free**: Single owner means single drop
3. **Dangling pointers**: Second-class references cannot outlive data
4. **Data races**: Send/Sync traits + borrow rules prevent concurrent mutation
5. **Buffer overflows**: Bounds checking on array access
6. **Null pointer dereference**: No null; use Option instead

### 8.2 What ARIA Allows (in unsafe)

```aria
unsafe {
    // Raw pointer manipulation
    let ptr: *mut Int = ...
    *ptr = 42

    // Unchecked array access
    array.get_unchecked(index)

    // Type transmutation
    let bytes: [u8; 4] = transmute(value)

    // FFI calls
    extern_c_function()
}
```

### 8.3 Safe Abstractions

The pattern for unsafe code is to encapsulate it in safe interfaces:

```aria
pub struct SafeBuffer {
    ptr: *mut u8
    len: usize
}

impl SafeBuffer {
    pub fn new(size: usize) -> SafeBuffer {
        unsafe {
            let ptr = alloc(size)
            SafeBuffer { ptr, len: size }
        }
    }

    pub fn get(&self, index: usize) -> Option[u8] {
        if index < self.len {
            unsafe { some(*self.ptr.add(index)) }
        } else {
            none
        }
    }
}

impl Drop for SafeBuffer {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr, self.len) }
    }
}
```

---

## 9. Comparison with Other Models

### 9.1 vs Rust

| Feature | Rust | ARIA |
|---------|------|------|
| Lifetime annotations | Required for complex cases | Never required |
| References in structs | Allowed with lifetimes | Not allowed (use Rc/Arc) |
| Borrow checker complexity | ~10,000 lines | ~600 lines |
| Self-referential structs | Difficult (Pin/unsafe) | Use arenas |
| Learning curve | Steep | Moderate |

### 9.2 vs Go

| Feature | Go | ARIA |
|---------|-----|------|
| Memory management | GC | Ownership |
| GC pauses | Yes | No |
| Memory overhead | Higher | Lower |
| Latency predictability | Variable | Predictable |

### 9.3 vs C++

| Feature | C++ | ARIA |
|---------|-----|------|
| Memory safety | Manual | Guaranteed |
| Use after free | Possible | Prevented |
| Data races | Possible | Prevented |
| Undefined behavior | Common | None in safe code |

---

## 10. Error Messages for Memory Issues

### 10.1 Move Errors

```
Error[E0382]: Use of moved value
  --> file.aria:10:5
   |
 8 |     let s1 = String.from("hello")
   |         -- value defined here
 9 |     let s2 = s1
   |              -- value moved here
10 |     print(s1)
   |           ^^ value used after move
   |
Help: Consider cloning the value:
   |
 9 |     let s2 = s1.clone()
   |                 ++++++++
```

### 10.2 Borrow Errors

```
Error[E0502]: Cannot borrow as mutable, already borrowed as immutable
  --> file.aria:12:5
   |
10 |     let r1 = &data
   |              ----- immutable borrow occurs here
11 |     let r2 = &data
12 |     let r3 = &mut data
   |              ^^^^^^^^^ mutable borrow attempted here
13 |     print(r1)
   |           -- immutable borrow later used here
   |
Help: Ensure immutable borrows are no longer used before mutable borrow:
   |
10 |     let r1 = &data
11 |     let r2 = &data
12 |     print(r1)
13 |     print(r2)
14 |     let r3 = &mut data  // Now OK
```

### 10.3 Lifetime Errors

```
Error[E0515]: Cannot return reference to local variable
  --> file.aria:5:5
   |
 3 | fn bad() -> &Int {
   |             ---- return type indicates reference
 4 |     let x = 42
   |         - local variable
 5 |     &x
   |     ^^ reference to local would be dangling
   |
Help: Return owned value instead:
   |
 3 | fn good() -> Int {
 4 |     let x = 42
 5 |     x
   |
Or use a smart pointer:
   |
 3 | fn good() -> Box[Int] {
 4 |     Box.new(42)
```
