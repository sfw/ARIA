# Trait Objects and Dynamic Dispatch Research

## Overview

This document researches trait objects and dynamic dispatch mechanisms across systems programming languages (Rust, Go, Swift) to inform ARIA's design decisions.

---

## 1. Virtual Tables (Vtables) - Rust Implementation

### 1.1 Memory Layout of Trait Objects

Rust uses **fat pointers** for trait objects. A `&dyn Trait` or `Box<dyn Trait>` consists of two pointers:

```
┌─────────────────────────────────────────┐
│           Fat Pointer (16 bytes)        │
├─────────────────┬───────────────────────┤
│   data_ptr      │     vtable_ptr        │
│   (8 bytes)     │     (8 bytes)         │
└────────┬────────┴───────────┬───────────┘
         │                    │
         ▼                    ▼
┌─────────────────┐   ┌──────────────────────────────┐
│  Concrete Data  │   │         Vtable               │
│  (e.g., Cat)    │   ├──────────────────────────────┤
│                 │   │  drop_in_place (destructor)  │
│  name: "Whiskers"│  │  size: 24                    │
│  age: 5         │   │  align: 8                    │
│                 │   │  method1_ptr                 │
└─────────────────┘   │  method2_ptr                 │
                      │  ...                         │
                      └──────────────────────────────┘
```

**Key insight**: Unlike C++, where the vtable pointer is embedded in each object, Rust keeps objects "thin" and carries the vtable pointer alongside the data pointer. This means:
- Objects don't need vtable space if never used polymorphically
- Cross-crate trait implementations work seamlessly
- But trait object pointers are 2x the size of regular pointers

### 1.2 Vtable Structure

The Rust vtable contains a header followed by method pointers:

```rust
// Conceptual vtable layout (compiler-generated)
struct Vtable {
    // Header (metadata)
    drop_in_place: fn(*mut ()),     // Destructor
    size: usize,                     // Size of concrete type
    align: usize,                    // Alignment of concrete type

    // Method pointers (one per trait method)
    methods: [fn(); N],              // Variable length
}
```

**For trait hierarchies** (e.g., `trait D: B + C`), the vtable includes:
1. Metadata (drop, size, align)
2. Supertrait methods (in depth-first post-order)
3. Supertrait vtable pointers (for upcasting)
4. Trait's own methods

### 1.3 Runtime Method Dispatch

```rust
// When calling: trait_obj.method(args)

// 1. Load vtable pointer from fat pointer
let vtable = fat_ptr.vtable;

// 2. Index into vtable at known offset
let method_ptr = vtable[METHOD_OFFSET];

// 3. Call through function pointer with data pointer as self
method_ptr(fat_ptr.data, args);
```

The offset is computed at compile time since trait method order is fixed.

### 1.4 Rust vs C++ Approach Comparison

| Aspect | Rust | C++ |
|--------|------|-----|
| Vtable storage | Fat pointer (external) | Embedded in object |
| Object size | No vtable overhead | +1 pointer per vtable |
| Cast cost | Must construct fat pointer | Zero (pointer cast) |
| Cross-crate traits | Works naturally | N/A |
| Multiple inheritance | Via supertrait vtable ptrs | Multiple vtable ptrs |

---

## 2. Go Interfaces

### 2.1 Interface Internal Structure

Go uses a different representation for interface values. The `iface` struct:

```go
type iface struct {
    tab  *itab          // Interface table pointer
    data unsafe.Pointer // Pointer to concrete value
}
```

For the empty interface `interface{}` (now `any`), Go uses a simpler structure:

```go
type eface struct {
    type_ *_type        // Type metadata only
    data  unsafe.Pointer
}
```

### 2.2 The itab (Interface Table)

```go
type itab struct {
    inter *interfacetype  // Static interface type info
    _type *_type          // Dynamic concrete type info
    hash  uint32          // Copy of _type.hash for switch
    _     [4]byte         // Padding
    fun   [1]uintptr      // Method table (variable size)
}
```

**Key design**: The itab is **per (interface, concrete type) pair**, not just per type. This means:
- First assignment of type T to interface I generates/caches the itab
- Subsequent assignments reuse the cached itab
- Go runtime maintains a global itab cache

### 2.3 Implicit Interface Satisfaction

Go's interfaces are satisfied **implicitly**:

```go
type Reader interface {
    Read(p []byte) (n int, err error)
}

// File implicitly implements Reader - no declaration needed
type File struct { ... }
func (f *File) Read(p []byte) (n int, err error) { ... }

// This just works:
var r Reader = &File{}
```

**Trade-offs**:
- Pro: Decoupled interface definition from implementation
- Pro: Can add interfaces after the fact
- Con: No compile-time verification that you "meant" to implement
- Con: Method signature typos cause silent non-implementation

### 2.4 Type Assertions and Switches

```go
// Type assertion
file, ok := r.(File)    // ok = false if r doesn't hold File

// Type switch
switch v := r.(type) {
case File:
    // v is File
case *Buffer:
    // v is *Buffer
default:
    // unknown type
}
```

The `hash` field in itab enables fast type switches via hash comparison.

### 2.5 Value Storage Optimization

```go
// Small values (1 word) stored directly in data pointer
var i interface{} = 42  // No heap allocation

// Large values require heap allocation
var i interface{} = LargeStruct{...}  // Heap allocated
```

---

## 3. Swift Protocol Witness Tables

### 3.1 Existential Containers

When you have a value of protocol type, Swift uses an **existential container**:

```
┌────────────────────────────────────────────────────┐
│         Existential Container (40 bytes)           │
├────────────────────────────────────────────────────┤
│  Value Buffer (24 bytes / 3 words)                 │
│  ┌──────────────────────────────────────────────┐  │
│  │  Either: inline value data                   │  │
│  │  Or: pointer to heap-allocated value         │  │
│  └──────────────────────────────────────────────┘  │
├────────────────────────────────────────────────────┤
│  Value Witness Table Pointer (8 bytes)             │
├────────────────────────────────────────────────────┤
│  Protocol Witness Table Pointer (8 bytes)          │
└────────────────────────────────────────────────────┘
```

### 3.2 Value Witness Table (VWT)

The VWT manages the **lifecycle** of values:

```swift
struct ValueWitnessTable {
    // Memory management
    initializeBufferWithCopyOfBuffer: fn
    destroy: fn
    initializeWithCopy: fn
    assignWithCopy: fn
    initializeWithTake: fn
    assignWithTake: fn

    // Metadata
    size: Int
    stride: Int
    flags: UInt32  // Includes alignment, POD status, inline-ability
}
```

**One VWT per type** in the program.

### 3.3 Protocol Witness Table (PWT)

The PWT maps protocol requirements to concrete implementations:

```swift
struct ProtocolWitnessTable {
    // For each protocol requirement:
    requirement1: fn  // Points to concrete implementation
    requirement2: fn
    associatedType1: TypeMetadata
    // etc.
}
```

**One PWT per (type, protocol) pair**.

### 3.4 Inline vs Heap Storage

```swift
// Small value (fits in 3 words) - stored inline
protocol Drawable { func draw() }
struct Point: Drawable { var x, y: Double; func draw() {...} }
var d: Drawable = Point(x: 1, y: 2)  // No heap allocation

// Large value - heap allocated, pointer in buffer
struct LargeShape: Drawable { var data: [Double]; func draw() {...} }
var d: Drawable = LargeShape(...)  // Heap allocated
```

### 3.5 Generics vs Existentials

Swift distinguishes between:

```swift
// Existential (dynamic dispatch, runtime cost)
func draw(shape: Drawable) { shape.draw() }

// Generic (can be specialized, potential static dispatch)
func draw<T: Drawable>(shape: T) { shape.draw() }
```

With `-O` optimization, generic code can be specialized and devirtualized.

---

## 4. Performance Implications

### 4.1 Cost of Virtual Calls

**Direct call**: ~1 cycle (inlined) to ~3-5 cycles (call instruction)

**Virtual call overhead**:
1. Load vtable pointer: 1 memory access
2. Load method pointer from vtable: 1 memory access
3. Indirect call: branch predictor penalty if mispredicted (~15-20 cycles)

**Total**: ~5-10 cycles best case, ~25-30 cycles with misprediction

### 4.2 Monomorphization vs Dynamic Dispatch

| Aspect | Monomorphization | Dynamic Dispatch |
|--------|------------------|------------------|
| Binary size | Larger (code duplication) | Smaller |
| Compile time | Longer | Shorter |
| Runtime performance | Faster (inlining possible) | Indirect call overhead |
| Flexibility | Types known at compile time | True runtime polymorphism |
| Separate compilation | Limited | Full support |

### 4.3 When to Use Each

**Static dispatch (generics/monomorphization)**:
- Hot loops where performance matters
- When types are known at compile time
- Small number of instantiations

**Dynamic dispatch (trait objects)**:
- Heterogeneous collections (`Vec<Box<dyn Draw>>`)
- Plugin systems / runtime extensibility
- Reducing binary bloat
- Recursive types (trait objects break infinite size)

### 4.4 Devirtualization Optimizations

Compilers can often eliminate virtual calls when they can prove the concrete type:

```rust
// Compiler knows exact type - can devirtualize
let x: Box<dyn Display> = Box::new(42i32);
println!("{}", x);  // Can be devirtualized if optimizer sees construction

// Cannot devirtualize - type unknown
fn print_it(x: &dyn Display) {
    println!("{}", x);  // Must use vtable
}
```

**Devirtualization techniques**:
1. **Local type analysis**: Track types through control flow
2. **Class hierarchy analysis**: If only one implementation exists, inline it
3. **Speculative devirtualization**: Inline common case, fall back to vtable
4. **Profile-guided optimization**: Specialize for observed types

---

## 5. Object Safety (Dyn Safety)

### 5.1 Why Not All Traits Can Be Trait Objects

A trait is **dyn-safe** (object-safe) only if:

1. **No `Self` in return position** (except in receiver):
   ```rust
   trait Clone {
       fn clone(&self) -> Self;  // NOT dyn-safe!
   }
   ```
   Problem: Return type size unknown at compile time.

2. **No generic methods**:
   ```rust
   trait Converter {
       fn convert<T>(&self, t: T);  // NOT dyn-safe!
   }
   ```
   Problem: Would need infinite vtable entries (one per T).

3. **No `Sized` bound on `Self`**:
   ```rust
   trait Foo: Sized { }  // NOT dyn-safe!
   ```
   Problem: `dyn Foo` is unsized by definition.

4. **No associated types without bounds** (partially):
   ```rust
   trait Iterator {
       type Item;
       fn next(&mut self) -> Option<Self::Item>;
   }
   // Can be dyn-safe if Item is specified: dyn Iterator<Item = i32>
   ```

### 5.2 Self Type Issues

```rust
trait Clonable {
    fn clone(&self) -> Self;      // Self is the concrete type
}

// Why this fails with dyn:
fn clone_it(x: &dyn Clonable) -> ??? {
    x.clone()  // What's the return type? We don't know Self!
}
```

The vtable contains methods, but the compiler needs to know:
- Stack space for return value
- How to handle the returned value

### 5.3 Workarounds

**1. Use `where Self: Sized` to opt out methods**:
```rust
trait MyTrait {
    fn safe_method(&self);

    // This method not available on dyn MyTrait
    fn clone(&self) -> Self where Self: Sized;
}
```

**2. Return `Box<dyn Trait>` instead of `Self`**:
```rust
trait Clonable {
    fn clone_boxed(&self) -> Box<dyn Clonable>;
}
```

**3. Associated type with bounds**:
```rust
// dyn Iterator<Item = i32> specifies the associated type
fn sum(iter: &mut dyn Iterator<Item = i32>) -> i32 { ... }
```

---

## 6. ARIA Design Recommendations

### 6.1 Simplest Vtable Design That Works

For ARIA's initial implementation, recommend a **Go-style approach**:

```
┌─────────────────────────────────────────────┐
│     Interface Value (trait object)          │
├─────────────────────┬───────────────────────┤
│   data_ptr (8)      │   itable_ptr (8)      │
└─────────┬───────────┴───────────┬───────────┘
          │                       │
          ▼                       ▼
    ┌───────────┐         ┌─────────────────────┐
    │  Concrete │         │      ITable         │
    │   Value   │         ├─────────────────────┤
    └───────────┘         │  type_info          │
                          │  drop_fn            │
                          │  size               │
                          │  align              │
                          │  method_0           │
                          │  method_1           │
                          │  ...                │
                          └─────────────────────┘
```

**Rationale**:
- 16 bytes total (same as Rust)
- Type info enables type assertions/switches
- Drop function handles cleanup
- Methods at fixed offsets for fast dispatch

### 6.2 Interaction with Second-Class References

ARIA's second-class references create interesting constraints:

**Can work**:
```aria
# Trait object by value (owned)
t Drawable
    f draw(&self)

f render(shapes: [Box[dyn Drawable]])
    for shape in shapes
        shape.draw

# Trait object by reference (borrowed)
f render_ref(shape: &dyn Drawable)
    shape.draw
```

**Cannot work (would require storing references)**:
```aria
# This violates second-class references
s ShapeCache
    items: [&dyn Drawable]  # ERROR: can't store references

# Workaround: use owned trait objects
s ShapeCache
    items: [Box[dyn Drawable]]  # OK
```

**Design principle**: Trait objects are compatible with second-class references because:
1. `Box<dyn Trait>` is owned, not a reference
2. `&dyn Trait` follows normal borrow rules
3. The fat pointer's internal vtable pointer is an implementation detail

### 6.3 Go-Style Interfaces (Simpler Alternative)

Consider starting with Go-style interfaces, deferring full trait objects:

```aria
# Interface definition (like Go)
interface Drawable
    draw(&self)

# Implicit satisfaction (any type with matching methods)
s Circle
    radius: Float

i Circle
    f draw(&self)
        println "Circle r={self.radius}"

# Usage - interface value
shapes: [Drawable] = []      # Box-like semantics implicit
shapes.push Circle(5.0)
shapes.push Rect(3.0, 4.0)

for shape in shapes
    shape.draw
```

**Advantages**:
- Simpler mental model
- No explicit `dyn` keyword needed
- Implicit satisfaction like Go
- Defer complex trait object questions

**Disadvantages**:
- Less explicit about costs
- Harder to optimize (implicit boxing)
- Different from Rust patterns

### 6.4 Hybrid Approach (Recommended)

```aria
# Define traits with 't' (static dispatch by default)
t Drawable
    f draw(&self)

# Use 'dyn' explicitly for dynamic dispatch
f render(shapes: [Box[dyn Drawable]])
    for shape in shapes
        shape.draw

# Generics for static dispatch
f render_static[T: Drawable](shapes: [T])
    for shape in shapes
        shape.draw  # Monomorphized
```

**This preserves**:
- Explicit control over dispatch
- Compatibility with Rust patterns
- Optimization opportunities
- Clear cost model

### 6.5 Standard Library Requirements

For a useful stdlib, these traits need to work as trait objects:

**Essential dyn-safe traits**:

```aria
# Iterator (with specified Item)
t Iterator
    type Item
    f next(&mut self) -> Self.Item?

# dyn Iterator[Item = Int] works

# Read/Write for I/O
t Read
    f read(&mut self, buf: &mut [u8]) -> Int!

t Write
    f write(&mut self, buf: &[u8]) -> Int!

# Error trait
t Error: Display
    f source(&self) -> &dyn Error?
    f description(&self) -> &str

# Display/Debug for formatting
t Display
    f fmt(&self, f: &mut Formatter) -> ()!

t Debug
    f fmt(&self, f: &mut Formatter) -> ()!
```

**Non-dyn-safe by design**:

```aria
# Clone returns Self
t Clone
    f clone(&self) -> Self

# Copy is a marker
t Copy: Clone

# From/Into are generic
t From[T]
    f from(t: T) -> Self

# Sized is a marker for statically-sized types
t Sized
```

### 6.6 Data Structure Layouts Summary

#### Fat Pointer (Trait Object Reference)
```
┌─────────────────────────────────────┐
│  &dyn Trait / &mut dyn Trait        │
├─────────────────┬───────────────────┤
│  data: *T       │  vtable: *Vtable  │
│  (8 bytes)      │  (8 bytes)        │
└─────────────────┴───────────────────┘
```

#### Box<dyn Trait>
```
┌─────────────────────────────────────┐
│  Box[dyn Trait]                     │
├─────────────────┬───────────────────┤
│  data: *T       │  vtable: *Vtable  │
│  (8 bytes)      │  (8 bytes)        │
└─────────────────┴───────────────────┘
(owns the data, will drop)
```

#### Vtable Layout
```
┌─────────────────────────────────────┐
│  Vtable for T implementing Trait    │
├─────────────────────────────────────┤
│  drop_in_place: fn(*mut T)          │  // Destructor
│  size: usize                        │  // sizeof(T)
│  align: usize                       │  // alignof(T)
│  type_id: TypeId                    │  // For type assertions
├─────────────────────────────────────┤
│  method_0: fn(*const T, ...) -> R   │  // First trait method
│  method_1: fn(*const T, ...) -> R   │  // Second trait method
│  ...                                │
└─────────────────────────────────────┘
```

#### Interface Value (Go-style, if adopted)
```
┌─────────────────────────────────────┐
│  interface Trait                    │
├─────────────────┬───────────────────┤
│  data: *T       │  itable: *ITable  │
│  (8 bytes)      │  (8 bytes)        │
└─────────────────┴───────────────────┘

┌─────────────────────────────────────┐
│  ITable                             │
├─────────────────────────────────────┤
│  interface_type: *InterfaceInfo     │
│  concrete_type: *TypeInfo           │
│  hash: u32                          │
│  methods: [fn; N]                   │
└─────────────────────────────────────┘
```

---

## 7. Implementation Roadmap for ARIA

### Phase 1: Static Traits Only
- Implement traits with generics (monomorphization)
- No `dyn` keyword yet
- Full type inference for trait bounds

### Phase 2: Simple Trait Objects
- Add `dyn Trait` syntax
- Implement fat pointers and vtables
- Object safety checking (basic rules)
- `Box[dyn Trait]`, `&dyn Trait`, `&mut dyn Trait`

### Phase 3: Advanced Features
- Trait upcasting (`dyn SubTrait` to `dyn SuperTrait`)
- Multiple trait bounds (`dyn Trait1 + Trait2`)
- Associated type specification (`dyn Iterator[Item = Int]`)

### Phase 4: Optimizations
- Devirtualization passes
- Vtable deduplication
- Inline small values (Swift-style optimization)

---

## References

- [Rust Trait Object Layout](https://neugierig.org/software/blog/2025/03/trait-object-layout.html)
- [Rust Vtable Layout RFC](https://rust-lang.github.io/dyn-upcasting-coercion-initiative/design-discussions/vtable-layout.html)
- [Go Data Structures: Interfaces](https://research.swtch.com/interfaces)
- [Go Internals: Interfaces](https://cmc.gitbook.io/go-internals/chapter-ii-interfaces)
- [Understanding Swift Performance (WWDC 2016)](https://developer.apple.com/videos/play/wwdc2016/416/)
- [Rust Object Safety RFC](https://rust-lang.github.io/rfcs/0255-object-safety.html)
- [Devirtualization in LLVM](https://blog.llvm.org/2017/03/devirtualization-in-llvm-and-clang.html)
- [Virtual Method Tables (Wikipedia)](https://en.wikipedia.org/wiki/Virtual_method_table)
