# Sprint 25: LLVM Fix + Runtime Expansion + Async Fixes

**Goal:** Fix double-recursion LLVM bug, expand runtime library, and fix async interpreter issues
**Priority:** CRITICAL
**Estimated Effort:** 2-3 days

---

## Part A: Fix Double-Recursion LLVM Segfault

### Root Cause Analysis

The segfault occurs because **user-defined function call results bypass type-safety** that builtin calls use.

For `fib(n-1) + fib(n-2)`:
1. First call `fib(n-1)` stores result in local `_0`
2. Second call `fib(n-2)` stores result in local `_1`
3. BinaryOp loads both locals for addition

**The bug:** In `Terminator::Call` handling (line ~1390), results are stored with:
```rust
self.builder.build_store(*alloca, result)?;
```

But this doesn't check if the alloca type matches the result type. The `store_builtin_result()` function (line ~1061) DOES handle this with re-allocation, but it's only called for builtins.

### Task 25-A.1: Apply store_builtin_result to All Calls

**File:** `src/codegen/llvm.rs`

**Location:** Around line 1390-1397 in `Terminator::Call` handling

**Current Code:**
```rust
if let Some(local) = dest {
    if let Some(result) = call.try_as_basic_value().left() {
        if let Some(alloca) = self.locals.get(&(local.0 as usize)) {
            self.builder.build_store(*alloca, result)?;
        }
    }
}
```

**Fixed Code:**
```rust
if let Some(local) = dest {
    if let Some(result) = call.try_as_basic_value().left() {
        // Use store_builtin_result for type-safe storage (handles re-allocation)
        self.store_builtin_result(result, &Some(*local))?;
    }
}
```

This ensures:
- Type mismatches are detected
- Allocas are re-created with correct types when needed
- Loads later use the correct type

### Verification

```bash
# Create test file
cat > /tmp/fib_test.forma << 'EOF'
f fib(n: Int) -> Int
    if n <= 1 then n else fib(n - 1) + fib(n - 2)

f main() -> Int
    print(fib(10))
    0
EOF

# Compile and run
./target/release/forma build /tmp/fib_test.forma -o /tmp/fib_test
/tmp/fib_test
# Should print: 55
```

---

## Part B: Expand Runtime Library

### Priority Functions to Add

Based on research, these are needed for example programs:

#### Tier 1: Essential (~15 functions)

| Function | Category | Needed For |
|----------|----------|------------|
| `forma_vec_new` | Vector | All |
| `forma_vec_len` | Vector | All |
| `forma_vec_push` | Vector | All |
| `forma_vec_get` | Vector | All |
| `forma_vec_set` | Vector | All |
| `forma_map_new` | Map | comprehensive |
| `forma_map_get` | Map | comprehensive |
| `forma_map_set` | Map | comprehensive |
| `forma_map_len` | Map | comprehensive |
| `forma_args` | CLI | cli_with_db |
| `forma_time_now_ms` | Time | async examples |
| `forma_sleep` | Time | async examples |
| `forma_env_get` | Env | cli_with_db |

### Task 25-B.1: Implement Vector Operations

**File:** `runtime/src/vec.rs` (NEW)

```rust
use std::os::raw::c_void;
use std::ptr;
use std::alloc::{alloc, dealloc, realloc, Layout};

/// FORMA Vector representation
#[repr(C)]
pub struct FormaVec {
    data: *mut u8,
    len: usize,
    cap: usize,
    elem_size: usize,
}

#[no_mangle]
pub extern "C" fn forma_vec_new(elem_size: usize) -> *mut FormaVec {
    let vec = Box::new(FormaVec {
        data: ptr::null_mut(),
        len: 0,
        cap: 0,
        elem_size,
    });
    Box::into_raw(vec)
}

#[no_mangle]
pub extern "C" fn forma_vec_len(v: *const FormaVec) -> i64 {
    if v.is_null() { return 0; }
    unsafe { (*v).len as i64 }
}

#[no_mangle]
pub extern "C" fn forma_vec_push(v: *mut FormaVec, elem: *const u8) {
    if v.is_null() || elem.is_null() { return; }
    unsafe {
        let vec = &mut *v;
        if vec.len == vec.cap {
            let new_cap = if vec.cap == 0 { 4 } else { vec.cap * 2 };
            let new_layout = Layout::from_size_align_unchecked(new_cap * vec.elem_size, 8);
            let new_data = if vec.data.is_null() {
                alloc(new_layout)
            } else {
                let old_layout = Layout::from_size_align_unchecked(vec.cap * vec.elem_size, 8);
                realloc(vec.data, old_layout, new_cap * vec.elem_size)
            };
            vec.data = new_data;
            vec.cap = new_cap;
        }
        ptr::copy_nonoverlapping(elem, vec.data.add(vec.len * vec.elem_size), vec.elem_size);
        vec.len += 1;
    }
}

#[no_mangle]
pub extern "C" fn forma_vec_get(v: *const FormaVec, idx: i64) -> *const u8 {
    if v.is_null() { return ptr::null(); }
    unsafe {
        let vec = &*v;
        if idx < 0 || idx as usize >= vec.len {
            return ptr::null();
        }
        vec.data.add(idx as usize * vec.elem_size)
    }
}

#[no_mangle]
pub extern "C" fn forma_vec_set(v: *mut FormaVec, idx: i64, elem: *const u8) {
    if v.is_null() || elem.is_null() { return; }
    unsafe {
        let vec = &mut *v;
        if idx >= 0 && (idx as usize) < vec.len {
            ptr::copy_nonoverlapping(elem, vec.data.add(idx as usize * vec.elem_size), vec.elem_size);
        }
    }
}

#[no_mangle]
pub extern "C" fn forma_vec_free(v: *mut FormaVec) {
    if v.is_null() { return; }
    unsafe {
        let vec = Box::from_raw(v);
        if !vec.data.is_null() && vec.cap > 0 {
            let layout = Layout::from_size_align_unchecked(vec.cap * vec.elem_size, 8);
            dealloc(vec.data, layout);
        }
    }
}
```

### Task 25-B.2: Implement Map Operations

**File:** `runtime/src/map.rs` (NEW)

```rust
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Simple string->string map for runtime
pub struct FormaMap {
    inner: HashMap<String, String>,
}

#[no_mangle]
pub extern "C" fn forma_map_new() -> *mut FormaMap {
    Box::into_raw(Box::new(FormaMap { inner: HashMap::new() }))
}

#[no_mangle]
pub extern "C" fn forma_map_len(m: *const FormaMap) -> i64 {
    if m.is_null() { return 0; }
    unsafe { (*m).inner.len() as i64 }
}

#[no_mangle]
pub extern "C" fn forma_map_get(m: *const FormaMap, key: *const c_char) -> *mut c_char {
    if m.is_null() || key.is_null() { return std::ptr::null_mut(); }
    unsafe {
        let k = CStr::from_ptr(key).to_str().unwrap_or("");
        match (*m).inner.get(k) {
            Some(v) => CString::new(v.as_str()).unwrap().into_raw(),
            None => std::ptr::null_mut(),
        }
    }
}

#[no_mangle]
pub extern "C" fn forma_map_set(m: *mut FormaMap, key: *const c_char, value: *const c_char) {
    if m.is_null() || key.is_null() || value.is_null() { return; }
    unsafe {
        let k = CStr::from_ptr(key).to_str().unwrap_or("").to_string();
        let v = CStr::from_ptr(value).to_str().unwrap_or("").to_string();
        (*m).inner.insert(k, v);
    }
}

#[no_mangle]
pub extern "C" fn forma_map_contains(m: *const FormaMap, key: *const c_char) -> bool {
    if m.is_null() || key.is_null() { return false; }
    unsafe {
        let k = CStr::from_ptr(key).to_str().unwrap_or("");
        (*m).inner.contains_key(k)
    }
}

#[no_mangle]
pub extern "C" fn forma_map_remove(m: *mut FormaMap, key: *const c_char) {
    if m.is_null() || key.is_null() { return; }
    unsafe {
        let k = CStr::from_ptr(key).to_str().unwrap_or("");
        (*m).inner.remove(k);
    }
}

#[no_mangle]
pub extern "C" fn forma_map_free(m: *mut FormaMap) {
    if !m.is_null() {
        unsafe { let _ = Box::from_raw(m); }
    }
}
```

### Task 25-B.3: Implement Time Functions

**File:** `runtime/src/time.rs` (NEW)

```rust
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::thread;

#[no_mangle]
pub extern "C" fn forma_time_now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn forma_time_now_us() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn forma_sleep_ms(ms: i64) {
    if ms > 0 {
        thread::sleep(Duration::from_millis(ms as u64));
    }
}

#[no_mangle]
pub extern "C" fn forma_sleep_secs(secs: i64) {
    if secs > 0 {
        thread::sleep(Duration::from_secs(secs as u64));
    }
}
```

### Task 25-B.4: Implement CLI Args

**File:** `runtime/src/env.rs` (NEW)

```rust
use std::ffi::CString;
use std::os::raw::c_char;
use std::env;

/// Get command line arguments as a null-terminated array of strings
#[no_mangle]
pub extern "C" fn forma_args_count() -> i64 {
    env::args().count() as i64
}

#[no_mangle]
pub extern "C" fn forma_args_get(idx: i64) -> *mut c_char {
    env::args()
        .nth(idx as usize)
        .and_then(|s| CString::new(s).ok())
        .map(|c| c.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn forma_env_get(name: *const c_char) -> *mut c_char {
    if name.is_null() { return std::ptr::null_mut(); }
    unsafe {
        let key = std::ffi::CStr::from_ptr(name).to_str().unwrap_or("");
        env::var(key)
            .ok()
            .and_then(|v| CString::new(v).ok())
            .map(|c| c.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }
}
```

### Task 25-B.5: Update lib.rs

**File:** `runtime/src/lib.rs`

```rust
pub mod memory;
pub mod string;
pub mod io;
pub mod math;
pub mod panic;
pub mod vec;    // NEW
pub mod map;    // NEW
pub mod time;   // NEW
pub mod env;    // NEW

pub use memory::*;
pub use string::*;
pub use io::*;
pub use math::*;
pub use panic::*;
pub use vec::*;
pub use map::*;
pub use time::*;
pub use env::*;
```

### Task 25-B.6: Update LLVM Codegen Mappings

**File:** `src/codegen/llvm.rs`

Add to `runtime_function_name()`:

```rust
// Vector operations
"vec_new" => Some("forma_vec_new"),
"vec_len" => Some("forma_vec_len"),
"vec_push" => Some("forma_vec_push"),
"vec_get" => Some("forma_vec_get"),
"vec_set" => Some("forma_vec_set"),

// Map operations
"map_new" => Some("forma_map_new"),
"map_len" => Some("forma_map_len"),
"map_get" => Some("forma_map_get"),
"map_set" => Some("forma_map_set"),
"map_contains" => Some("forma_map_contains"),

// Time operations
"time_now_ms" => Some("forma_time_now_ms"),
"time_ms" => Some("forma_time_now_ms"),
"sleep" => Some("forma_sleep_ms"),
"time_sleep" => Some("forma_sleep_ms"),

// CLI/Env
"args" => Some("forma_args_get"),
"env_get" => Some("forma_env_get"),
```

---

## Part C: Fix Async Runtime Issues

### Task 25-C.1: Shared Tokio Runtime

**File:** `src/mir/interp.rs`

**Problem:** Each spawn creates new runtime (line 382-386)

**Solution:** Use a global shared runtime

```rust
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::runtime::Runtime;

static GLOBAL_RUNTIME: Lazy<Arc<Runtime>> = Lazy::new(|| {
    Arc::new(
        Runtime::new().expect("Failed to create Tokio runtime")
    )
});

impl Interpreter {
    pub fn new_for_task(program: Arc<Program>) -> Result<Self, InterpError> {
        Ok(Self {
            program,
            runtime: GLOBAL_RUNTIME.clone(),  // Use shared runtime
            // ... rest of fields
        })
    }
}
```

**Also update Cargo.toml:**
```toml
[dependencies]
once_cell = "1.19"
```

### Task 25-C.2: Propagate Spawned Task Errors

**File:** `src/mir/interp.rs`

**Problem:** Errors become Value::Unit silently (lines 1031-1043)

**Location:** Find the spawn handling and fix error propagation

**Current:**
```rust
match task_interp.execute(&func) {
    Ok(val) => val,
    Err(e) => {
        eprintln!("Spawned task error: {}", e.message);
        Value::Enum {
            type_name: "Result".to_string(),
            variant: "Err".to_string(),
            fields: vec![Value::Str(e.message)],
        }
    }
}
```

**Improved:** This is actually okay - it wraps errors in Result::Err. The issue is the `.unwrap_or(Value::Unit)` after await. Fix that:

```rust
// Change from:
}).await.unwrap_or(Value::Unit)

// To:
}).await.unwrap_or_else(|e| {
    Value::Enum {
        type_name: "Result".to_string(),
        variant: "Err".to_string(),
        fields: vec![Value::Str(format!("Task join error: {:?}", e))],
    }
})
```

### Task 25-C.3: Clean Up await_any Tasks

**File:** `src/mir/interp.rs`

**Problem:** Remaining tasks dropped without cleanup (lines 2670-2691)

**Fix:** Explicitly abort remaining tasks

```rust
// After select_all returns
let (result, _completed_idx, remaining) = self.runtime.block_on(async {
    futures::future::select_all(handles).await
});

// Abort remaining tasks to prevent resource leaks
for handle in remaining {
    handle.abort();
}

// Return the first completed result
let value = result.map_err(|e| InterpError { ... })?;
Ok(Some(value))
```

### Task 25-C.4: Thread-Safe Environment Variables

**File:** `src/mir/interp.rs`

**Problem:** env_set/env_remove use unsafe std::env (lines 3429-3447)

**Solution:** Use thread-safe wrapper

```rust
use parking_lot::RwLock;
use std::collections::HashMap;

// Add to Interpreter struct:
pub struct Interpreter {
    // ... existing fields ...
    /// Thread-safe environment variable storage
    env_vars: Arc<RwLock<HashMap<String, String>>>,
}

// Modify env_set builtin:
"env_set" => {
    validate_args!(args, 2, "env_set");
    let name = match &args[0] { Value::Str(s) => s.clone(), _ => return Err(...) };
    let value = match &args[1] { Value::Str(s) => s.clone(), _ => return Err(...) };

    // Thread-safe write
    self.env_vars.write().insert(name, value);
    Ok(Some(Value::Unit))
}

// Modify env_get builtin:
"env_get" => {
    validate_args!(args, 1, "env_get");
    let name = match &args[0] { Value::Str(s) => s.clone(), _ => return Err(...) };

    // Check local env first, then fall back to system env
    let result = self.env_vars.read().get(&name).cloned()
        .or_else(|| std::env::var(&name).ok());

    match result {
        Some(v) => Ok(Some(Value::Str(v))),
        None => Ok(Some(Value::Option { value: None, inner_type: "Str".to_string() })),
    }
}

// Modify env_remove:
"env_remove" => {
    validate_args!(args, 1, "env_remove");
    let name = match &args[0] { Value::Str(s) => s.clone(), _ => return Err(...) };

    self.env_vars.write().remove(&name);
    Ok(Some(Value::Unit))
}
```

**Also update Cargo.toml:**
```toml
[dependencies]
parking_lot = "0.12"
```

---

## Verification Checklist

### Part A: LLVM Double-Recursion
- [ ] `fib(10)` compiles and runs, prints 55
- [ ] `fib(20)` works without segfault
- [ ] Other double-recursion patterns work

### Part B: Runtime Expansion
- [ ] `forma_vec_*` functions work
- [ ] `forma_map_*` functions work
- [ ] `forma_time_now_ms` returns valid timestamp
- [ ] `forma_args_get` returns command line args
- [ ] Runtime library builds without warnings

### Part C: Async Fixes
- [ ] Only one Tokio runtime created (check with debug logs)
- [ ] Spawned task errors propagate as Result::Err
- [ ] await_any aborts uncompleted tasks
- [ ] Concurrent env_set/env_get doesn't crash

### Full Integration
```bash
cargo test
cargo build --release

# Test LLVM fib
./target/release/forma build examples/fibonacci.forma -o fib && ./fib

# Test interpreter async
./target/release/forma run examples/async_parallel.forma
```

---

## Summary

| Part | Tasks | Est. Time |
|------|-------|-----------|
| A: LLVM Fix | 1 task | 1 hour |
| B: Runtime | 6 tasks | 4-6 hours |
| C: Async | 4 tasks | 3-4 hours |
| **Total** | **11 tasks** | **8-11 hours** |

---

*"Fix it once, fix it right."*
