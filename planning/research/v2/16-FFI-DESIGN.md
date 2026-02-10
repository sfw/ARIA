# ARIA v2 Foreign Function Interface (FFI) Design

> Comprehensive research and implementation recommendations for FFI in ARIA

---

## Table of Contents

1. [C ABI Compatibility](#1-c-abi-compatibility)
2. [Memory Safety Across FFI](#2-memory-safety-across-ffi)
3. [Binding Generation](#3-binding-generation)
4. [Callbacks and Function Pointers](#4-callbacks-and-function-pointers)
5. [ARIA-Specific Design](#5-aria-specific-design)
6. [Built-in Runtime Functions vs FFI](#6-built-in-runtime-functions-vs-ffi)
7. [Implementation Recommendations](#7-implementation-recommendations)

---

## 1. C ABI Compatibility

### 1.1 How Languages Handle C Calling Conventions

#### Rust Approach

```rust
// Declare external C function
extern "C" {
    fn printf(format: *const c_char, ...) -> c_int;
    fn malloc(size: size_t) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

// Export function with C ABI
#[no_mangle]
pub extern "C" fn my_function(x: c_int) -> c_int {
    x + 1
}

// Struct with C-compatible layout
#[repr(C)]
pub struct Point {
    x: f64,
    y: f64,
}
```

**Key Features:**
- `extern "C"` block declares foreign functions
- `#[no_mangle]` prevents symbol name mangling
- `#[repr(C)]` ensures C-compatible struct layout
- `extern "C"` on function exports with C ABI

#### Go (cgo) Approach

```go
package main

/*
#include <stdio.h>
#include <stdlib.h>

// Inline C code
int add(int a, int b) {
    return a + b;
}
*/
import "C"
import "unsafe"

func main() {
    // Call C function
    result := C.add(C.int(1), C.int(2))

    // String conversion
    cstr := C.CString("hello")
    defer C.free(unsafe.Pointer(cstr))
    C.printf(C.CString("%s\n"), cstr)
}

//export GoCallback
func GoCallback(x C.int) C.int {
    return x * 2
}
```

**Key Features:**
- Magic `"C"` pseudo-package
- C code in preamble comments above `import "C"`
- Type conversion: `C.int`, `C.CString`, etc.
- `//export` directive for callbacks

#### Zig Approach

```zig
const std = @import("std");
const c = @cImport({
    @cInclude("stdio.h");
    @cInclude("stdlib.h");
});

// Zig function with C calling convention
pub export fn my_function(x: c_int) callconv(.C) c_int {
    return x + 1;
}

// Extern struct (C-compatible layout)
const Point = extern struct {
    x: f64,
    y: f64,
};

pub fn main() void {
    // Direct C function call
    _ = c.printf("Hello from Zig\n");

    // Allocation via C
    const ptr = c.malloc(100);
    defer c.free(ptr);
}
```

**Key Features:**
- `@cImport` and `@cInclude` for C headers
- `callconv(.C)` for C calling convention
- `extern struct` for C-compatible layout
- Direct symbol access without wrappers

### 1.2 C Type Mappings

| C Type | Rust | Go | Zig | ARIA (Proposed) |
|--------|------|-----|-----|-----------------|
| `char` | `c_char` | `C.char` | `c_char` | `c.Char` |
| `int` | `c_int` | `C.int` | `c_int` | `c.Int` |
| `unsigned int` | `c_uint` | `C.uint` | `c_uint` | `c.UInt` |
| `long` | `c_long` | `C.long` | `c_long` | `c.Long` |
| `size_t` | `size_t` | `C.size_t` | `usize` | `c.Size` |
| `void*` | `*mut c_void` | `unsafe.Pointer` | `*anyopaque` | `c.Ptr` |
| `char*` | `*const c_char` | `*C.char` | `[*:0]const u8` | `c.CStr` |

### 1.3 Platform-Specific Calling Conventions

#### x86 (32-bit)

| Convention | Used By | Stack Cleanup | Arguments |
|------------|---------|---------------|-----------|
| `cdecl` | Default (Unix/GCC) | Caller | Stack (right-to-left) |
| `stdcall` | Win32 API | Callee | Stack (right-to-left) |
| `fastcall` | Performance | Callee | ECX, EDX, then stack |

#### x86-64

| Platform | Convention | Integer Args | Float Args |
|----------|------------|--------------|------------|
| Unix/Linux | System V ABI | RDI, RSI, RDX, RCX, R8, R9 | XMM0-7 |
| Windows | Microsoft x64 | RCX, RDX, R8, R9 | XMM0-3 |

**ARIA Recommendation:**

```
# Default: platform C ABI
ex "C" f malloc(size: c.Size) -> c.Ptr

# Windows-specific (when needed)
ex "stdcall" f MessageBoxA(
    hwnd: c.Ptr
    text: c.CStr
    caption: c.CStr
    type: c.UInt
) -> c.Int

# System-adaptive (Windows uses stdcall, Unix uses C)
ex "system" f SomeSystemCall()
```

---

## 2. Memory Safety Across FFI

### 2.1 The `unsafe` Block Model (Rust-style)

```rust
// Safe Rust cannot do these things
// Unsafe Rust can:
unsafe {
    // 1. Dereference raw pointers
    let ptr: *const i32 = &x;
    let value = *ptr;

    // 2. Call unsafe functions
    let mem = malloc(100);

    // 3. Access mutable statics
    GLOBAL_COUNTER += 1;

    // 4. Implement unsafe traits
    // 5. Access union fields
}
```

### 2.2 What Can vs Cannot Be Checked

| Aspect | Can Be Checked | Must Be Trusted |
|--------|----------------|-----------------|
| Rust type signatures | Yes | - |
| Pointer validity | No | C code must provide valid pointers |
| Buffer sizes | Partial | C must respect declared sizes |
| Aliasing rules | No | C can violate Rust's aliasing |
| Thread safety | No | Must manually ensure Send/Sync |
| Lifetime validity | No | C doesn't track lifetimes |
| Null pointers | Partial | Can check, but C may pass null |

### 2.3 Common FFI Pitfalls

#### Dangling Pointers

```rust
// WRONG: Local goes out of scope
fn get_pointer() -> *const i32 {
    let local = 42;
    &local as *const i32  // Dangling!
}

// CORRECT: Use heap or caller-provided buffer
fn get_pointer_safe() -> *mut i32 {
    Box::into_raw(Box::new(42))  // Caller must free
}
```

#### Lifetime Issues

```rust
// WRONG: Returning reference to borrowed data
extern "C" fn bad_api(data: *const Data) -> *const Inner {
    unsafe {
        &(*data).inner  // May outlive data!
    }
}

// CORRECT: Copy or ref-count
extern "C" fn good_api(data: *const Data) -> Inner {
    unsafe { (*data).inner.clone() }
}
```

#### Use-After-Free

```
# C code:
# char* str = get_string();
# free(str);
# printf("%s", str);  // Use after free!

# ARIA must ensure:
un
    str = c.get_string
    defer c.free str    # Guaranteed cleanup
    process str         # Safe within block
```

### 2.4 ARIA's Unsafe Block Design

```
# ARIA's unsafe block
un
    # Raw pointer operations
    ptr: *mut Int = get_ptr
    *ptr = 42

    # Type transmute
    bytes: [u8; 4] = transmute value

    # FFI calls
    result = c.some_function arg

# Unsafe function declaration
un f dangerous_operation(ptr: *mut c.Void)
    # Body is implicitly unsafe

# Safe wrapper pattern
f safe_wrapper(data: &[u8]) -> Int!
    un
        ptr = data.as_ptr
        len = data.len
        result = c.process_bytes(ptr, len)
        if result < 0
            ret err Error.from_c result
        ok result
```

---

## 3. Binding Generation

### 3.1 How bindgen Works (Rust)

**Process:**
1. Parse C/C++ headers using libclang
2. Extract type definitions, function signatures, macros
3. Generate Rust FFI declarations
4. Handle platform-specific variations

**Example:**

```c
// input.h
typedef struct {
    int x;
    int y;
} Point;

int calculate_distance(const Point* a, const Point* b);
```

```rust
// Generated bindings
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Point {
    pub x: ::std::os::raw::c_int,
    pub y: ::std::os::raw::c_int,
}

extern "C" {
    pub fn calculate_distance(
        a: *const Point,
        b: *const Point
    ) -> ::std::os::raw::c_int;
}
```

**build.rs integration:**

```rust
use bindgen;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    bindings.write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");
}
```

### 3.2 How cgo Works (Go)

**Process:**
1. Parse preamble C code/headers
2. Generate C wrapper functions for Go->C calls
3. Generate Go stubs for C->Go callbacks
4. Compile C and Go code separately
5. Link together

**Generated artifacts:**
- `_cgo_gotypes.go` - Go type definitions
- `_cgo_export.h` - C headers for exported Go functions
- `.cgo1.go` - Rewritten Go source
- `.cgo2.c` - C stub implementations

### 3.3 ARIA Binding Generation Proposal

**Option A: Built-in @cImport (Zig-style)**

```
# Direct C header import
c = @cImport
    @cInclude "stdio.h"
    @cInclude "mylib.h"

# Use directly
f main
    c.printf "Hello %s\n" name
```

**Option B: aria-bindgen Tool (Rust-style)**

```bash
# Generate bindings
aria-bindgen --header mylib.h --output bindings.ar

# In build.ar
bind_c "wrapper.h"
    # Configuration
    allowlist_function "^my_.*"
    blocklist_type "InternalStruct"
```

**Option C: Inline Declaration (Manual)**

```
# Manual FFI declarations
mod c_bindings
    ex "C"
        f malloc(size: c.Size) -> c.Ptr
        f free(ptr: c.Ptr)
        f memcpy(dst: c.Ptr, src: c.Ptr, n: c.Size) -> c.Ptr

    @repr(C)
    s Point
        x: c.Int
        y: c.Int

    ex "C"
        f calculate_distance(a: *Point, b: *Point) -> c.Int
```

**Recommendation:** Implement all three progressively:
1. Phase 1: Manual inline declarations
2. Phase 2: aria-bindgen tool
3. Phase 3: Built-in @cImport (if bootstrapping to self-hosted)

---

## 4. Callbacks and Function Pointers

### 4.1 Passing ARIA Functions to C

**Problem:** C expects plain function pointers, but ARIA closures capture state.

#### Non-Capturing Functions (Simple Case)

```
# Non-capturing function can be directly used
f compare(a: *c.Void, b: *c.Void) -> c.Int
    ua = *(a as *Int)
    ub = *(b as *Int)
    if ua < ub then -1
    elif ua > ub then 1
    else 0

# Pass to C
ex "C" f qsort(
    base: c.Ptr
    num: c.Size
    size: c.Size
    cmp: f(*c.Void, *c.Void) -> c.Int
)

f sort_ints(arr: &mut [Int])
    un
        c.qsort(
            arr.as_mut_ptr as c.Ptr
            arr.len
            size_of[Int]
            compare
        )
```

#### Closures with Captures (Trampoline Pattern)

```
# C callback signature with user_data
# typedef void (*callback_t)(void* user_data, int value);
# void register_callback(callback_t cb, void* user_data);

# ARIA implementation
s CallbackContext[F]
    closure: F

# Trampoline function
un f trampoline[F: Fn(Int)](ctx: c.Ptr, value: c.Int)
    un
        closure = &*(ctx as *CallbackContext[F])
        (closure.closure)(value as Int)

# Safe wrapper
f with_callback[F: Fn(Int)](callback: F, work: f())
    ctx = Box.new CallbackContext { closure: callback }
    un
        ptr = Box.into_raw ctx
        c.register_callback(trampoline[F], ptr as c.Ptr)
        work()
        c.unregister_callback()
        _ = Box.from_raw ptr    # Cleanup

# Usage
with_callback |x|
    print "Got: {x}"
do
    c.do_work()
```

### 4.2 Closure Capture Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| Dangling captures | Closure outlives captured data | Use `'static` or Arc |
| Mutable aliasing | Multiple mutable captures | Use Mutex/RefCell |
| Thread unsafety | Captured data not Send | Require Send bound |
| Memory leaks | Forgotten cleanup | RAII wrappers |

```
# WRONG: Dangling reference
f bad_example
    local = 42
    register_callback || print local
    # local goes out of scope!

# CORRECT: Move ownership
f good_example
    data = Arc.new 42
    register_callback mv ||
        print *data    # data is moved into closure
```

### 4.3 Thread Safety for Callbacks

```
# Thread-safe callback registration
s SafeCallback[F: Fn() + Send + Sync]
    inner: Arc[Mutex[Option[F]]]

i SafeCallback[F]
    f new(callback: F) -> Self
        Self { inner: Arc.new Mutex.new Some callback }

    f as_c_callback(&self) -> (f(), c.Ptr)
        (Self.trampoline, Arc.into_raw self.inner.clone as c.Ptr)

    un f trampoline(ctx: c.Ptr)
        un
            arc = Arc.from_raw ctx as *Mutex[Option[F]]
            if cb =? *arc.lock
                cb()
            Arc.into_raw arc    # Don't drop

# Usage ensures Send + Sync
callback = SafeCallback.new ||
    # This closure must be Send + Sync
    print "Called from any thread!"
```

---

## 5. ARIA-Specific Design

### 5.1 Minimum FFI for File I/O and Syscalls

**Essential syscalls for a minimal runtime:**

| Category | Syscalls | Purpose |
|----------|----------|---------|
| File I/O | open, read, write, close, lseek | Basic file operations |
| Memory | mmap, munmap, mprotect | Memory management |
| Process | exit, getpid, fork, exec | Process control |
| Time | clock_gettime | Timing |
| Directory | opendir, readdir, mkdir, rmdir | Directory operations |
| Network | socket, bind, listen, accept, connect | Networking |

**ARIA Runtime Core:**

```
# Minimal unsafe runtime (runtime/sys.ar)
mod sys
    # File descriptors
    type Fd = c.Int

    ex "C"
        # File I/O
        f open(path: c.CStr, flags: c.Int, mode: c.Int) -> Fd
        f read(fd: Fd, buf: c.Ptr, count: c.Size) -> c.SSize
        f write(fd: Fd, buf: c.Ptr, count: c.Size) -> c.SSize
        f close(fd: Fd) -> c.Int
        f lseek(fd: Fd, offset: c.Off, whence: c.Int) -> c.Off

        # Memory
        f mmap(
            addr: c.Ptr
            len: c.Size
            prot: c.Int
            flags: c.Int
            fd: Fd
            offset: c.Off
        ) -> c.Ptr
        f munmap(addr: c.Ptr, len: c.Size) -> c.Int

        # Process
        f exit(status: c.Int) -> !
        f getpid() -> c.Int

    # Safe wrappers
    f sys_read(fd: Fd, buf: &mut [u8]) -> Int!
        un
            result = read(fd, buf.as_mut_ptr as c.Ptr, buf.len)
            if result < 0
                ret err Error.last_os_error
            ok result as Int
```

### 5.2 Maintaining Memory Safety with FFI

**Strategy: Layered Safety**

```
Layer 4: [Safe ARIA API]      - User code uses this
         |
Layer 3: [Safe Wrappers]      - Validates inputs, manages lifetimes
         |
Layer 2: [Unsafe FFI Layer]   - Direct C calls in `un` blocks
         |
Layer 1: [C Runtime/Syscalls] - Operating system
```

**Example Implementation:**

```
# Layer 2: Unsafe FFI (internal)
mod internal
    ex "C" f read(fd: c.Int, buf: c.Ptr, count: c.Size) -> c.SSize

# Layer 3: Safe wrapper (internal)
mod safe_sys
    f read(fd: RawFd, buf: &mut [u8]) -> Int!
        if buf.empty
            ret ok 0
        un
            result = internal.read(fd, buf.as_mut_ptr as c.Ptr, buf.len)
            if result < 0
                ret err IoError.last_os
            ok result as Int

# Layer 4: High-level API (public)
pub s File
    fd: RawFd

pub i File
    pub f read(&mut self, buf: &mut [u8]) -> Int!
        safe_sys.read(self.fd, buf)
```

### 5.3 Restricted FFI: Trusted stdlib Only

**Option 1: Capability-Based FFI**

```
# FFI capability marker trait
t FfiCapability

# Only runtime has this capability
mod runtime
    pub s RuntimeFfiCap: FfiCapability

    # Private constructor - only runtime can create
    f new() -> RuntimeFfiCap
        RuntimeFfiCap {}

# FFI requires capability
f call_c_function[C: FfiCapability](cap: &C, ...)
    un
        c.function(...)

# User code cannot call C directly
# Must go through stdlib which has capability
```

**Option 2: Module-Level Restriction**

```
# In aria.toml
[package]
name = "my-app"

[ffi]
# Only these modules can use FFI
allowed = ["std.*", "runtime.*"]
# User code gets compilation error for `ex` or `un`
```

**Option 3: Compile-Time Flag**

```bash
# Compile with restricted FFI
aria build --ffi-mode=stdlib-only

# Or in aria.toml
[build]
ffi-mode = "stdlib-only"  # "open" | "stdlib-only" | "none"
```

**Recommendation:** Combine approaches:
1. Default: `stdlib-only` mode
2. `#[allow_ffi]` attribute for vetted libraries
3. Explicit `--allow-ffi` flag for builds needing it

```
# Requires --allow-ffi or #[allow_ffi]
#[allow_ffi]
mod my_c_bindings
    ex "C" f custom_function()

# Always allowed (in std)
us std.fs    # Uses FFI internally but safe API
```

---

## 6. Built-in Runtime Functions vs FFI

### 6.1 Operations That MUST Be Built-in

| Operation | Why Built-in? |
|-----------|---------------|
| Memory allocation | Need before FFI setup |
| Panic/abort | Core safety mechanism |
| Type introspection | Compiler-generated |
| Integer overflow checks | Performance-critical |
| Bounds checking | Must be inlined |
| Reference counting ops | Atomic, performance-critical |
| Stack unwinding | Platform-specific, compiler-integrated |

```
# Built-in intrinsics (compiler magic)
@intrinsic f size_of[T]() -> Int
@intrinsic f align_of[T]() -> Int
@intrinsic f type_name[T]() -> Str
@intrinsic f transmute[T, U](value: T) -> U

# Memory intrinsics
@intrinsic f alloc(size: Int, align: Int) -> *mut u8
@intrinsic f dealloc(ptr: *mut u8, size: Int, align: Int)
@intrinsic f copy_nonoverlapping[T](src: *T, dst: *mut T, count: Int)

# Atomic intrinsics
@intrinsic f atomic_load[T](ptr: *T) -> T
@intrinsic f atomic_store[T](ptr: *mut T, val: T)
@intrinsic f atomic_cas[T](ptr: *mut T, old: T, new: T) -> (T, B)
@intrinsic f atomic_add[T](ptr: *mut T, val: T) -> T
```

### 6.2 Operations That CAN Be in ARIA

| Operation | Implementation |
|-----------|----------------|
| String formatting | Pure ARIA |
| Collections (Vec, Map) | ARIA + alloc intrinsic |
| Iterators | Pure ARIA |
| Option/Result | Pure ARIA (enums) |
| Math functions | ARIA or FFI to libm |
| Parsing | Pure ARIA |
| Serialization | Pure ARIA |

```
# String formatting in pure ARIA
t Display
    f fmt(&self, f: &mut Formatter) -> FmtResult

i Display for Int
    f fmt(&self, f: &mut Formatter) -> FmtResult
        # Pure ARIA implementation
        if *self == 0
            ret f.write_str "0"

        buf: [u8; 20] := []
        n := self.abs
        idx := 19

        while n > 0
            buf[idx] = '0' + (n % 10) as u8
            n /= 10
            idx -= 1

        if *self < 0
            buf[idx] = '-'
            idx -= 1

        f.write_bytes &buf[idx + 1..]

# Vec implementation uses intrinsics
s Vec[T]
    ptr: *mut T
    len: Int
    cap: Int

i Vec[T]
    f new() -> Self
        Vec { ptr: 0 as *mut T, len: 0, cap: 0 }

    f push(&mut self, item: T)
        if self.len == self.cap
            self.grow()
        un
            ptr = self.ptr.offset self.len
            copy_nonoverlapping(&item, ptr, 1)
            forget item    # Don't drop, we moved it
        self.len += 1

    f grow(&mut self)
        new_cap = if self.cap == 0 then 4 else self.cap * 2
        new_size = new_cap * size_of[T]

        new_ptr = un
            if self.cap == 0
                alloc(new_size, align_of[T]) as *mut T
            else
                realloc(self.ptr as *mut u8, new_size) as *mut T

        self.ptr = new_ptr
        self.cap = new_cap
```

### 6.3 Hybrid Approach: Intrinsics + Optional FFI

```
# Math: ARIA implementation with FFI fallback
mod math
    # Pure ARIA (slower but portable)
    f sqrt_aria(x: Float) -> Float
        # Newton-Raphson implementation
        if x < 0.0
            ret Float.nan
        if x == 0.0
            ret 0.0

        guess := x / 2.0
        for _ in 0..50
            guess = (guess + x / guess) / 2.0
        guess

    # FFI to libm (faster)
    ex "C" f sqrt_c(x: Float) -> Float = "sqrt"

    # Public API uses best available
    #[cfg(feature = "libm")]
    pub f sqrt(x: Float) -> Float
        un { sqrt_c x }

    #[cfg(not(feature = "libm"))]
    pub f sqrt(x: Float) -> Float
        sqrt_aria x
```

---

## 7. Implementation Recommendations

### 7.1 Phase 1: Bootstrap (Minimal FFI)

**Goal:** Get ARIA self-hosting with minimal unsafe surface.

```
# Core FFI module (runtime/core_ffi.ar)
# This is the ONLY file with direct FFI in bootstrap

mod core_ffi
    # Memory
    ex "C"
        f malloc(size: c.Size) -> c.Ptr
        f realloc(ptr: c.Ptr, size: c.Size) -> c.Ptr
        f free(ptr: c.Ptr)
        f memcpy(dst: c.Ptr, src: c.Ptr, n: c.Size) -> c.Ptr
        f memset(s: c.Ptr, c: c.Int, n: c.Size) -> c.Ptr

    # I/O
    ex "C"
        f open(path: c.CStr, flags: c.Int) -> c.Int
        f read(fd: c.Int, buf: c.Ptr, count: c.Size) -> c.SSize
        f write(fd: c.Int, buf: c.Ptr, count: c.Size) -> c.SSize
        f close(fd: c.Int) -> c.Int

    # Process
    ex "C"
        f exit(status: c.Int) -> !
        f abort() -> !

    # Time (for profiling)
    ex "C"
        f clock_gettime(clk: c.Int, tp: *c.Timespec) -> c.Int
```

**Safe stdlib built on core_ffi:**

```
# std/alloc.ar
mod alloc
    us runtime.core_ffi as ffi

    pub f alloc(size: Int) -> *mut u8
        un
            ptr = ffi.malloc size as c.Size
            if ptr == null
                panic "allocation failed"
            ptr as *mut u8

    pub f dealloc(ptr: *mut u8)
        un { ffi.free ptr as c.Ptr }

# std/io.ar
mod io
    us runtime.core_ffi as ffi

    pub s File
        fd: c.Int

    pub i File
        pub f open(path: &Path) -> File!
            cpath = path.to_cstring
            un
                fd = ffi.open cpath.as_ptr flags
                if fd < 0
                    ret err IoError.last
                ok File { fd }

        pub f read(&self, buf: &mut [u8]) -> Int!
            un
                result = ffi.read self.fd buf.as_mut_ptr buf.len
                if result < 0
                    ret err IoError.last
                ok result as Int
```

### 7.2 Phase 2: Binding Generator

```
# aria-bindgen tool design

# Input: C header
// mylib.h
typedef struct {
    int x, y;
} Point;

int process_point(const Point* p);
void set_callback(void (*cb)(int));

# Generated output: bindings.ar
# AUTO-GENERATED by aria-bindgen - DO NOT EDIT

mod mylib_bindings
    @repr(C)
    pub s Point
        pub x: c.Int
        pub y: c.Int

    # Callback type
    pub type Callback = f(c.Int)

    ex "C"
        pub f process_point(p: *Point) -> c.Int
        pub f set_callback(cb: Callback)
```

### 7.3 Phase 3: Full FFI System

**Complete FFI specification:**

```
# FFI Declaration Syntax
ex "<abi>" [f|s|type] name ...

# ABI options
"C"        # Default C ABI
"stdcall"  # Windows stdcall
"fastcall" # Fastcall convention
"system"   # Platform's system call convention
"rust"     # Rust ABI (for Rust interop)

# Struct layout control
@repr(C)           # C-compatible layout
@repr(packed)      # No padding
@repr(C, packed)   # C layout, no padding
@repr(align(N))    # Minimum alignment

# Example complete FFI module
mod sdl
    @repr(C)
    pub s SDL_Rect
        x: c.Int
        y: c.Int
        w: c.Int
        h: c.Int

    @repr(C)
    pub s SDL_Event
        type_: c.UInt
        # ... union fields via separate structs

    ex "C"
        pub f SDL_Init(flags: c.UInt) -> c.Int
        pub f SDL_Quit()
        pub f SDL_CreateWindow(
            title: c.CStr
            x: c.Int
            y: c.Int
            w: c.Int
            h: c.Int
            flags: c.UInt
        ) -> *SDL_Window
        pub f SDL_PollEvent(event: *mut SDL_Event) -> c.Int

    # Opaque types
    pub s SDL_Window
        _private: ()

    # Safe wrapper
    pub s Window
        raw: *SDL_Window

    pub i Window
        pub f new(title: &Str, w: Int, h: Int) -> Window!
            cstr = title.to_cstring
            un
                ptr = SDL_CreateWindow(
                    cstr.as_ptr
                    0x2FFF0000  # SDL_WINDOWPOS_CENTERED
                    0x2FFF0000
                    w as c.Int
                    h as c.Int
                    0
                )
                if ptr == null
                    ret err SdlError.last
                ok Window { raw: ptr }

    pub i Drop for Window
        f drop(&mut self)
            un { SDL_DestroyWindow self.raw }
```

### 7.4 Safety Recommendations Summary

| Recommendation | Implementation |
|----------------|----------------|
| Default to stdlib-only FFI | Compile flag + module restriction |
| Require `un` blocks | All FFI calls must be in unsafe |
| Safe wrapper pattern | stdlib wraps all raw FFI |
| Automatic bounds checking | Even in unsafe, check what's checkable |
| Clear error propagation | FFI errors -> ARIA Result |
| RAII for resources | Drop trait for cleanup |
| No implicit conversions | Explicit casts for C types |
| Audit tooling | `aria audit --ffi` to list all FFI uses |

---

## References

- [Rust FFI - The Rustonomicon](https://doc.rust-lang.org/nomicon/ffi.html)
- [cgo command - Go Packages](https://pkg.go.dev/cmd/cgo)
- [Zig C Interop - zig.guide](https://zig.guide/working-with-c/)
- [bindgen - Rust FFI bindings generator](https://rust-lang.github.io/rust-bindgen/)
- [x86 calling conventions - Wikipedia](https://en.wikipedia.org/wiki/X86_calling_conventions)
- [Effective Rust - FFI](https://www.effective-rust.com/ffi.html)
- [Rust Closures in FFI - Michael Bryan](https://adventures.michaelfbryan.com/posts/rust-closures-in-ffi/)
- [SandCell: Sandboxing Rust](https://arxiv.org/html/2509.24032v1)
- [Inko FFI Challenges](https://inko-lang.org/news/the-challenge-of-building-a-foreign-function-interface/)
