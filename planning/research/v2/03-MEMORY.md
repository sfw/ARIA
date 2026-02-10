# ARIA v2 Memory Model

> Memory model is unchanged from v1 — the innovation is in syntax, not semantics.

## Core Principle: Second-Class References

References cannot be stored in data structures. This eliminates lifetime annotations.

---

## 1. Ownership Rules

```
# Every value has one owner
x = Str.from "hello"    # x owns the string

# Assignment moves ownership
y = x                    # ownership moves to y
# x is now invalid

# Clone for explicit copy
z = y.clone              # z owns a copy
# y still valid
```

---

## 2. Borrowing Rules

### 2.1 Immutable Borrows

```
# Multiple immutable borrows OK
data = [1, 2, 3]
a = &data
b = &data
print a.len + b.len     # OK

# Function parameter borrows
f sum(nums: &[Int]) -> Int
    nums | fold(0, +)

total = sum &data       # borrows data
```

### 2.2 Mutable Borrows

```
# Only one mutable borrow at a time
data := [1, 2, 3]
a = &mut data
# b = &mut data        # ERROR: already borrowed
a.push 4               # OK

# After a's last use, can borrow again
b = &mut data          # OK now
```

### 2.3 No Mixed Borrows

```
data := [1, 2, 3]
a = &mut data
# b = &data            # ERROR: mutable borrow active
a.push 4
# After a done:
b = &data              # OK
```

---

## 3. Second-Class References

### What's Allowed

```
# References as parameters
f process(data: &[Int]) -> Int
    data.sum

# References as local variables
f compute(data: &Data)
    x = &data.field
    use x

# Return reference derived from input
f first[T](list: &[T]) -> &T
    &list[0]

# Method returning reference to self
i Container[T]
    f get(&self, idx: Int) -> &T
        &self.items[idx]
```

### What's NOT Allowed

```
# Cannot store reference in struct
s Bad
    ref: &Int           # ERROR

# Cannot store in collection
refs: [&Int] = []       # ERROR

# Cannot return reference to local
f bad -> &Int
    x = 42
    &x                  # ERROR: x will be dropped
```

---

## 4. Move Semantics

### 4.1 Default: Move

```
s Data
    value: Str

f take(d: Data)
    print d.value

data = Data("hello")
take data
# data is moved, cannot use
```

### 4.2 Copy Types

```
# Primitives are Copy
x = 42
y = x          # copied
print x        # still valid

# Derive Copy for small types
@derive(Copy, Clone)
s Point
    x: Float
    y: Float

p1 = Point(1.0, 2.0)
p2 = p1        # copied
print p1       # still valid
```

### 4.3 Explicit Move

```
# Force move with 'mv'
f consume(s: Str)
    print s

data = Str.from "hello"
consume mv data       # explicit move
```

---

## 5. Drop and Destructors

```
s FileHandle
    handle: RawHandle

i Drop for FileHandle
    f drop(&mut self)
        close_raw self.handle

f process
    file = FileHandle.open "data.txt"
    # use file...
# file dropped here, close_raw called
```

---

## 6. Smart Pointers

### 6.1 Box (Heap Allocation)

```
# Single owner on heap
boxed = Box.new 42
print *boxed          # dereference
```

### 6.2 Rc (Reference Counting)

```
# Shared ownership, single thread
shared = Rc.new Data("hello")
clone1 = shared.clone
clone2 = shared.clone
# All three access same data
# Dropped when last ref gone
```

### 6.3 Arc (Atomic Reference Counting)

```
# Shared ownership, multi-thread
shared = Arc.new Data("hello")
spawn ||
    print shared.value    # safe across threads
```

### 6.4 Interior Mutability

```
# RefCell: runtime borrow checking
cell = RefCell.new 42
*cell.borrow_mut = 43
print *cell.borrow

# Mutex: thread-safe
mutex = Mutex.new 42
guard = mutex.lock
*guard = 43
```

---

## 7. Arenas

For complex data structures needing internal references:

```
# All allocations share arena lifetime
arena = Arena.new
node1 = arena.alloc Node(1)
node2 = arena.alloc Node(2, next: node1)
# All valid while arena exists

# Typed arena
arena = TypedArena[Node].new
```

---

## 8. Unsafe Escape Hatch

```
un
    # Raw pointer dereference
    ptr: *mut Int = get_ptr
    *ptr = 42

    # Type transmute
    bytes: [u8;4] = transmute 42i32

    # FFI call
    libc.malloc size
```

---

## 9. Thread Safety

### 9.1 Send and Sync

```
# Send: can transfer to another thread
# Sync: can share between threads (&T is Send)

# Most types are Send + Sync
# Exceptions:
Rc[T]          # not Send or Sync
RefCell[T]     # not Sync
*T             # not Send or Sync by default
```

### 9.2 Safe Sharing

```
# Arc + Mutex for shared mutable state
counter = Arc.new Mutex.new 0

spawn ||
    *counter.lock += 1

spawn ||
    *counter.lock += 1

# Wait for completion, print result
```

---

## 10. Memory Safety Guarantees

### What ARIA Prevents

| Issue | Prevention |
|-------|------------|
| Use after free | Ownership ensures values exist |
| Double free | Single owner = single drop |
| Dangling pointer | Second-class references |
| Data race | Send/Sync + borrow rules |
| Buffer overflow | Bounds checking |
| Null dereference | No null, use Option |

### What Requires Unsafe

| Operation | Why Unsafe |
|-----------|------------|
| Raw pointer deref | No borrow checking |
| Transmute | Type safety bypass |
| FFI calls | No Rust guarantees |
| Inline assembly | Direct hardware access |

---

## 11. Error Messages

```
Error[E0382]: use of moved value
  --> src/main.ar:10:5
   |
 8 |     s = Str.from "hello"
   |     - value defined here
 9 |     t = s
   |         - value moved here
10 |     print s
   |           ^ value used after move
   |
Help: consider cloning:
   |
 9 |     t = s.clone
   |          ++++++
```

```
Error[E0502]: cannot borrow as mutable
  --> src/main.ar:12:5
   |
10 |     r1 = &data
   |          ----- immutable borrow here
12 |     r2 = &mut data
   |          ^^^^^^^^^ mutable borrow attempted
13 |     print r1
   |           -- immutable borrow used here
   |
Help: use mutable borrow after immutable is done:
   |
10 |     r1 = &data
11 |     print r1
12 |     r2 = &mut data    # now OK
```
