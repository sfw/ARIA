# Async/Concurrency Implementation Research for ARIA

## Executive Summary

This document provides comprehensive research on async/concurrency patterns across modern systems programming languages, with specific recommendations for ARIA's implementation. Key findings:

1. **Async Model**: Rust's lazy futures with state machine compilation provides zero-cost abstractions but requires runtime. ARIA should adopt this model with simplified syntax.

2. **Runtime**: Start with single-threaded event loop (minimal complexity), then add work-stealing multi-threaded runtime. Both should be swappable.

3. **Second-Class References**: ARIA's second-class references significantly simplify concurrency safety - no lifetime issues across await points.

4. **Channels**: Bounded by default, with clear CSP-style semantics. Select/poll for multiple channels essential.

5. **Structured Concurrency**: Non-negotiable. Nurseries/task groups prevent resource leaks and simplify reasoning.

---

## 1. Async/Await Models

### 1.1 Rust's Future Trait and State Machines

Rust transforms async functions into state machines implementing the `Future` trait:

```rust
trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

enum Poll<T> {
    Ready(T),
    Pending,
}
```

**How the Compiler Transforms Async Code:**

1. Each async function becomes a unique anonymous type implementing `Future`
2. Await points become state machine transitions
3. Local variables that live across await points are stored in the state machine struct
4. The `poll` method advances the state machine, returning `Pending` or `Ready`

```rust
// Source
async fn example() -> i32 {
    let x = await compute_x();
    let y = await compute_y();
    x + y
}

// Compiler generates approximately:
enum ExampleState {
    Start,
    WaitingX { },
    WaitingY { x: i32 },
    Complete,
}

impl Future for Example {
    type Output = i32;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<i32> {
        loop {
            match self.state {
                Start => {
                    self.future_x = compute_x();
                    self.state = WaitingX;
                }
                WaitingX => {
                    match self.future_x.poll(cx) {
                        Pending => return Pending,
                        Ready(x) => {
                            self.x = x;
                            self.future_y = compute_y();
                            self.state = WaitingY;
                        }
                    }
                }
                WaitingY { x } => {
                    match self.future_y.poll(cx) {
                        Pending => return Pending,
                        Ready(y) => return Ready(x + y),
                    }
                }
            }
        }
    }
}
```

**Key Properties:**
- Allocation-free composition (futures can be nested without heap allocation)
- Lazy execution (futures don't run until polled)
- Cancellation by dropping (no explicit cancellation needed)
- Pin required for self-referential state machines

**Pinning Requirement:**
When local variables that are references need to live across await points, the future becomes self-referential. `Pin<&mut Self>` ensures the future cannot be moved after first poll.

### 1.2 JavaScript/C# Style Async (Eager Model)

**JavaScript Promises:**
```javascript
async function example() {
    const x = await computeX();  // Already running
    const y = await computeY();
    return x + y;
}
```

**Key Differences from Rust:**
- **Eager execution**: Async functions start running immediately when called
- **Heap allocated**: Promises always live on the heap
- **No cancellation by default**: Need explicit `AbortController`
- **Automatic unwrapping**: Promises automatically flatten nested promises

**C# Tasks:**
```csharp
async Task<int> Example() {
    var x = await ComputeX();  // May run on thread pool
    var y = await ComputeY();
    return x + y;
}
```

**C# Properties:**
- Compiler generates state machines (like Rust)
- Dynamic dispatch and heap allocation required
- Built-in `CancellationToken` support
- Can create unstarted Tasks

### 1.3 Comparison Table

| Feature | Rust | JavaScript | C# |
|---------|------|------------|-----|
| Execution | Lazy | Eager | Eager |
| Allocation | Stack possible | Always heap | Always heap |
| Cancellation | Drop | AbortController | CancellationToken |
| Runtime Required | Yes | Built-in (V8/Node) | Built-in (CLR) |
| State Machine | Compile-time | Runtime | Compile-time |
| Zero-cost | Yes | No | No |

### 1.4 Recommendation for ARIA

**Adopt Rust's lazy model with simplified syntax:**

```aria
# ARIA async function
as f fetch(url: Str) -> Data!
    resp = aw http.get url?
    aw resp.json

# Equivalent transformation (conceptual)
# The compiler generates a Future[Data!] type with poll method
```

**Benefits for ARIA:**
1. Zero-cost abstraction aligns with systems programming goals
2. Drop-based cancellation integrates with ownership model
3. Second-class references eliminate Pin complexity (see Section 5)
4. Lazy execution gives caller control over when/where to run

---

## 2. Runtime Requirements

### 2.1 Tokio (Rust) - Work-Stealing Multi-Threaded

**Architecture:**
- Fixed thread pool with worker threads (default: one per CPU core)
- Each worker has a local run queue
- Global run queue for overflow
- Work-stealing between workers when queues are imbalanced

**Key Components:**

1. **Local Run Queues**: LIFO for cache locality, fixed size (256 tasks)
2. **Global Run Queue**: Mutex-protected, checked periodically (every ~61 polls)
3. **LIFO Slot**: Single-task optimization for message passing patterns
4. **Work Stealing**: Idle workers steal half of another worker's queue

**Scheduling Algorithm:**
```
loop {
    // 1. Check LIFO slot
    if let Some(task) = lifo_slot.take() { return task; }

    // 2. Check local queue
    if let Some(task) = local_queue.pop() { return task; }

    // 3. Periodically check global queue (1/61 probability)
    if should_check_global() {
        if let Some(task) = global_queue.pop() { return task; }
    }

    // 4. Try to steal from other workers
    for other in workers.shuffle() {
        if let Some(tasks) = other.local_queue.steal_half() {
            return tasks.pop();
        }
    }

    // 5. Park thread until woken
    park();
}
```

**I/O Integration:**
- Uses epoll (Linux), kqueue (macOS), IOCP (Windows)
- I/O driver runs on dedicated thread or worker threads
- Reactor wakes tasks when I/O is ready

### 2.2 Go's Goroutine Scheduler (M:N Threading)

**GMP Model:**
- **G (Goroutine)**: Lightweight user-space thread
- **M (Machine)**: OS thread
- **P (Processor)**: Logical processor, scheduling context

**Key Properties:**
- G runs on M, but needs P to execute
- P count = GOMAXPROCS (default: CPU count)
- M count can exceed P count (for blocking syscalls)

**Scheduling:**
```
for {
    // 1. Check local run queue (1/61: check global first)
    if g := runqget(p); g != nil { return g }

    // 2. Check global run queue
    if g := globrunqget(); g != nil { return g }

    // 3. Check netpoll (non-blocking)
    if g := netpoll(0); g != nil { return g }

    // 4. Steal from other P's local queues
    if g := runqsteal(random_p); g != nil { return g }

    // 5. Check global queue again, then netpoll (blocking)
    stopm()
}
```

**System Call Handling:**
- When G blocks on syscall, M releases P
- Another M can pick up P and continue running other Gs
- `sysmon` thread monitors long-running syscalls

**Preemption (since Go 1.14):**
- Signal-based preemption every ~10ms
- Prevents long-running goroutines from starving others

### 2.3 Single-Threaded Event Loop (Node.js/libuv Style)

**Architecture:**
```
┌─────────────────────────────────────────────────────┐
│                    Event Loop                        │
│  ┌─────────────────────────────────────────────────┐│
│  │ while (hasEvents()) {                           ││
│  │     events = poll(epoll/kqueue/IOCP);           ││
│  │     for (event in events) {                     ││
│  │         callback = getCallback(event);          ││
│  │         callback();                             ││
│  │     }                                           ││
│  │     runTimers();                                ││
│  │     runNextTickQueue();                         ││
│  │ }                                               ││
│  └─────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────┐
│              Thread Pool (libuv)                     │
│  For blocking operations:                            │
│  - File I/O                                          │
│  - DNS lookups                                       │
│  - CPU-intensive work (spawn_blocking)              │
└─────────────────────────────────────────────────────┘
```

**Phases (Node.js):**
1. **Timers**: Execute setTimeout/setInterval callbacks
2. **Pending callbacks**: Execute I/O callbacks deferred to next iteration
3. **Idle/Prepare**: Internal use
4. **Poll**: Retrieve new I/O events; execute I/O callbacks
5. **Check**: setImmediate callbacks
6. **Close callbacks**: socket.on('close'), etc.

**Advantages:**
- Simple mental model (no data races in user code)
- No synchronization needed for shared state
- Predictable execution order

**Disadvantages:**
- Cannot utilize multiple cores for JavaScript
- Long-running tasks block the entire loop
- Need worker threads or child processes for parallelism

### 2.4 Minimum Viable Async Runtime

A minimal runtime needs:

```
┌────────────────────────────────────────────────────────────┐
│                  Minimum Viable Runtime                     │
├────────────────────────────────────────────────────────────┤
│ 1. Task Queue                                               │
│    - Store pending tasks (futures)                          │
│    - VecDeque<Task> or channel-based                        │
├────────────────────────────────────────────────────────────┤
│ 2. Executor                                                 │
│    - Pop task from queue                                    │
│    - Call poll() on task                                    │
│    - If Pending: task will be re-queued by waker            │
│    - If Ready: task complete                                │
├────────────────────────────────────────────────────────────┤
│ 3. Waker                                                    │
│    - Callback to re-queue task when event occurs            │
│    - Created per-task, passed to poll()                     │
│    - Called by I/O reactor or timer                         │
├────────────────────────────────────────────────────────────┤
│ 4. Reactor (optional but needed for I/O)                    │
│    - Wraps epoll/kqueue/IOCP                                │
│    - Registers interest in I/O events                       │
│    - Wakes tasks when events occur                          │
└────────────────────────────────────────────────────────────┘
```

**Minimal Implementation (~200 lines):**

```rust
// Simplified executor
struct Executor {
    queue: VecDeque<Task>,
}

impl Executor {
    fn run(&mut self) {
        while let Some(task) = self.queue.pop_front() {
            let waker = task.create_waker();
            let mut cx = Context::from_waker(&waker);

            match task.future.poll(&mut cx) {
                Poll::Ready(_) => { /* task complete */ }
                Poll::Pending => { /* waker will re-queue */ }
            }
        }
    }
}

// Waker re-queues the task
impl Waker {
    fn wake(self) {
        EXECUTOR.lock().queue.push_back(self.task);
    }
}
```

### 2.5 Recommendation for ARIA

**Phase 1: Single-Threaded Runtime**
- Simplest to implement and debug
- No synchronization complexity
- Perfect for learning/prototyping
- Good for I/O-bound workloads

```aria
# Single-threaded runtime (default for Phase 1)
f main
    runtime.block_on as_main

as f as_main
    data = aw fetch url
    print data
```

**Phase 2: Work-Stealing Multi-Threaded**
- Add when parallelism needed
- Follow Tokio's architecture
- Swappable via runtime configuration

```aria
# Multi-threaded runtime
f main
    rt = Runtime.builder
        | worker_threads 4
        | build
    rt.block_on as_main
```

---

## 3. Channels and Communication

### 3.1 Go Channels (CSP Model)

**Communicating Sequential Processes (CSP):**
- Processes communicate by sending messages through channels
- No shared memory between processes
- Synchronization through channel operations

**Channel Types in Go:**
```go
// Unbuffered (synchronous rendezvous)
ch := make(chan int)      // Sender blocks until receiver ready

// Buffered (asynchronous up to capacity)
ch := make(chan int, 10)  // Sender blocks only when full

// Note: Go has no unbounded channels (intentional)
```

**Select Statement:**
```go
select {
case msg := <-ch1:
    handle(msg)
case ch2 <- value:
    // sent
case <-time.After(5 * time.Second):
    // timeout
default:
    // non-blocking
}
```

### 3.2 Rust's mpsc and crossbeam Channels

**std::sync::mpsc (Multi-Producer Single-Consumer):**
```rust
// Unbounded (actually bounded by memory)
let (tx, rx) = mpsc::channel();

// Bounded (sync_channel)
let (tx, rx) = mpsc::sync_channel(10);
```

**crossbeam-channel (Multi-Producer Multi-Consumer):**
```rust
// Bounded
let (tx, rx) = crossbeam_channel::bounded(10);

// Unbounded
let (tx, rx) = crossbeam_channel::unbounded();

// Zero-capacity (rendezvous)
let (tx, rx) = crossbeam_channel::bounded(0);

// Special channels
let rx = crossbeam_channel::after(Duration::from_secs(5));  // Timer
let rx = crossbeam_channel::tick(Duration::from_secs(1));   // Periodic
let rx = crossbeam_channel::never();                         // Never ready
```

**Select Macro (crossbeam):**
```rust
select! {
    recv(rx1) -> msg => handle(msg),
    send(tx, value) -> res => { /* sent */ },
    recv(timeout_rx) -> _ => { /* timeout */ },
    default => { /* non-blocking */ },
}

// Dynamic selection
let mut sel = Select::new();
let idx1 = sel.recv(&rx1);
let idx2 = sel.recv(&rx2);

let oper = sel.select();
match oper.index() {
    i if i == idx1 => { let msg = oper.recv(&rx1); }
    i if i == idx2 => { let msg = oper.recv(&rx2); }
    _ => unreachable!(),
}
```

### 3.3 Bounded vs Unbounded Channels

**Bounded Channels:**
- Fixed capacity buffer
- Backpressure: sender blocks when full
- Memory bounded
- Better for flow control

**Unbounded Channels:**
- No capacity limit
- Sender never blocks (except for memory)
- Can cause unbounded memory growth
- Useful when producer/consumer rates unknown

**Recommendation:**
- Default to bounded channels
- Require explicit opt-in for unbounded
- Provide clear warnings about memory implications

### 3.4 Select/Poll on Multiple Channels

**Essential for:**
- Multiplexing multiple sources
- Implementing timeouts
- Handling shutdown signals
- Actor message loops

**Implementation Approaches:**

1. **Macro-based (Rust crossbeam):**
   - Zero-allocation for static channel sets
   - Compile-time channel list

2. **Dynamic selection (crossbeam Select):**
   - Runtime channel list
   - Needed for dynamic actor systems

3. **Async select (tokio):**
   ```rust
   tokio::select! {
       msg = rx1.recv() => handle(msg),
       _ = shutdown.recv() => return,
       _ = tokio::time::sleep(Duration::from_secs(5)) => timeout(),
   }
   ```

### 3.5 Channel Design for ARIA

```aria
# Bounded channel (default, recommended)
(tx, rx) = channel[Int](cap: 10)

# Unbounded (explicit, with warning in docs)
(tx, rx) = unbounded[Int]()

# Zero-capacity (rendezvous)
(tx, rx) = channel[Int](cap: 0)

# Oneshot (single value)
(tx, rx) = oneshot[Data]()

# Broadcast (multiple receivers)
(tx, _) = broadcast[Event](cap: 100)
rx1 = tx.subscribe
rx2 = tx.subscribe

# Select
aw select
    msg = rx1.recv -> handle msg
    _ = rx2.recv -> handle_other
    _ = timeout 5.secs -> handle_timeout
    _ = shutdown.recv -> ret

# Dynamic select
sel = Select.new
sel.add rx1
sel.add rx2
(idx, msg) = aw sel.recv
```

---

## 4. Structured Concurrency

### 4.1 Why Unstructured Concurrency is Dangerous

From Nathaniel J. Smith's seminal analysis:

**The Problem with "Go Statement":**
```python
# Unstructured (dangerous)
async def process():
    asyncio.create_task(background_work())  # Fire and forget
    return result  # Background task may outlive this function

# Issues:
# 1. Background task can outlive its logical scope
# 2. Errors in background task are silently lost
# 3. Resources (files, connections) may not be cleaned up
# 4. Cancellation doesn't propagate
# 5. Can't reason about what's running
```

**Analogy to goto:**
- `spawn()` without structure is like `goto` for control flow
- Creates invisible control flow that crosses function boundaries
- Makes reasoning about program behavior impossible

### 4.2 Task Groups and Cancellation

**Swift Task Groups:**
```swift
// All tasks must complete before group exits
await withTaskGroup(of: Data.self) { group in
    for url in urls {
        group.addTask { await fetch(url) }
    }

    for await data in group {
        process(data)
    }
}
// Guaranteed: all tasks complete here

// Cancellation propagates automatically
func fetchAll() async throws {
    try await withThrowingTaskGroup(of: Data.self) { group in
        group.addTask { try await fetchA() }
        group.addTask { try await fetchB() }
        // If fetchA throws, fetchB is automatically cancelled
    }
}
```

**Swift async let:**
```swift
// Concurrent binding
async let a = fetchA()
async let b = fetchB()

// Both run concurrently, await at use site
let result = await a + await b
// Guaranteed: both complete before continuing
```

### 4.3 Kotlin's Coroutine Scopes

**CoroutineScope Hierarchy:**
```kotlin
// Scope defines lifetime of coroutines
coroutineScope {
    launch { workA() }  // Child of scope
    launch { workB() }  // Child of scope
    // Scope waits for all children
}
// Guaranteed: workA and workB complete here

// SupervisorScope: failures don't propagate
supervisorScope {
    launch { riskyWork() }  // If this fails...
    launch { otherWork() }  // ...this continues
}
```

**Job Hierarchy:**
```
ParentJob
├── ChildJob1
│   ├── GrandchildJob1
│   └── GrandchildJob2
└── ChildJob2
    └── GrandchildJob3

// Cancelling ParentJob cancels entire tree
// Child failure (without supervisor) cancels siblings and parent
```

### 4.4 Trio's Nurseries (Python)

**Nursery Pattern:**
```python
async with trio.open_nursery() as nursery:
    nursery.start_soon(task_a)
    nursery.start_soon(task_b)
    nursery.start_soon(task_c)
# All three tasks guaranteed to complete/cancel before exit

# If any task raises exception:
# 1. Other tasks are cancelled
# 2. Nursery waits for all to finish/cancel
# 3. Exception propagates to parent
```

**Benefits:**
1. **No orphan tasks**: Tasks cannot outlive their nursery
2. **Error propagation**: Exceptions bubble up naturally
3. **Resource safety**: `with` blocks work correctly
4. **Cancellation propagation**: Automatic and consistent

### 4.5 Recommendation for ARIA

**Structured concurrency as the default:**

```aria
# Nursery pattern (recommended)
as f process_all(items: [Item]) -> [Output]!
    nursery |n| as
        handles = items | map |item| n.spawn || process item
        aw join_all handles | collect

# Scope pattern
as f with_timeout -> Data!
    scope |s| as
        # Background cleanup (structured)
        s.spawn_bg cleanup_loop

        # Main work with timeout
        m aw timeout(10.secs, main_work)
            Ok data -> data
            Err _ -> err Timeout
        # Background cancelled on exit

# async let equivalent
as f concurrent -> (A, B)
    # Both start immediately, await implicitly at tuple creation
    a = as fetch_a
    b = as fetch_b
    (aw a, aw b)
```

**Escape hatch for rare cases:**
```aria
# Detached task (requires explicit acknowledgment)
as f with_detached
    handle = spawn_detached || background_daemon
    # handle.detach_acknowledged must be called
    # Compiler warning if not
```

---

## 5. Memory Safety in Concurrency

### 5.1 Send and Sync Traits (Rust)

**Definitions:**
- `Send`: Safe to transfer ownership to another thread
- `Sync`: Safe to share references between threads (`&T` is `Send`)

**Automatic Derivation:**
```rust
// Automatically Send + Sync if all fields are
struct SafeData {
    x: i32,      // i32 is Send + Sync
    s: String,   // String is Send + Sync
}

// Not Send because Rc is not Send
struct NotSend {
    rc: Rc<i32>,  // Reference counted, not atomic
}

// Not Sync because RefCell is not Sync
struct NotSync {
    cell: RefCell<i32>,  // Interior mutability without synchronization
}
```

**Why Rc is not Send:**
- Reference count updates are not atomic
- Two threads incrementing count = data race
- Use `Arc` (atomic reference count) instead

**Why RefCell is not Sync:**
- Borrow checking at runtime, not thread-safe
- Two threads could both get `borrow_mut()` = undefined behavior
- Use `Mutex` or `RwLock` instead

### 5.2 Data Races vs Race Conditions

**Data Race (undefined behavior in C/C++/Rust):**
```rust
// Definition: Two threads accessing same memory location where:
// 1. At least one access is a write
// 2. Accesses are not synchronized

static mut COUNTER: i32 = 0;

// Thread 1
unsafe { COUNTER += 1; }

// Thread 2
unsafe { COUNTER += 1; }

// Result: Undefined behavior - could be 1, 2, or corrupted
```

**Race Condition (logic error, not undefined behavior):**
```rust
// Definition: Program correctness depends on timing

fn transfer(from: &Mutex<i32>, to: &Mutex<i32>, amount: i32) {
    // Race condition: check-then-act
    if *from.lock() >= amount {  // Check
        // Another thread could modify here!
        *from.lock() -= amount;   // Act
        *to.lock() += amount;
    }
}

// No data race (mutex used), but logic bug
// Could overdraw if two transfers happen concurrently
```

**Key Distinction:**
- Data races: Type system can prevent (Rust does)
- Race conditions: Logic errors, need correct algorithms

### 5.3 How Second-Class References Help Concurrency

**The Problem with First-Class References:**

In Rust, references can be stored in structs, leading to lifetime complexity in async:

```rust
// Rust: This doesn't compile without complex lifetime annotations
struct Task<'a> {
    data: &'a str,  // Reference stored in struct
}

async fn process(task: Task<'_>) {
    // Complex: lifetime must be valid across await points
    let result = fetch().await;
    println!("{}", task.data);  // Is data still valid?
}
```

**Second-Class References Solve This:**

```aria
# ARIA: References cannot be stored in structs
s Task
    data: &Str        # ERROR: Cannot store reference

# Instead, own the data or use Arc
s Task
    data: Str         # OK: Owns the data
    # or
    data: Arc[Str]    # OK: Shared ownership

# References only exist as:
# 1. Function parameters
# 2. Local variables
# 3. Return values (derived from input references)
```

**Benefits for Concurrency:**

1. **No self-referential futures:**
   - State machine never contains references to itself
   - No need for `Pin`
   - Simpler implementation

2. **Spawn is always safe:**
   ```aria
   # Always safe - task owns all its data
   as f spawn_safe(data: Data)
       spawn || as
           process data    # data is moved into task

   # Sharing requires explicit Arc
   as f spawn_shared(data: Arc[Data])
       spawn || as
           process data.clone    # Arc clone is cheap
   ```

3. **No lifetime issues across await:**
   ```aria
   # This is always safe in ARIA
   as f example(data: &Data)
       x = aw fetch
       use data           # data reference still valid
       # (compiler ensures function doesn't return before data used)
   ```

4. **Send/Sync simpler:**
   - No lifetime parameters in traits
   - Types are Send/Sync based on fields only
   - No complex variance rules

### 5.4 Recommendation for ARIA

**Leverage second-class references for simpler concurrency:**

```aria
# Thread safety traits (similar to Rust but simpler)
tr Send
    # Can be transferred to another thread

tr Sync
    # Can be shared between threads (&T is Send)

# Auto-derived based on fields
s SafeData                    # Send + Sync (all fields are)
    x: Int
    name: Str

# Explicitly not Send/Sync
s NotSend                     # Not Send (Rc is not Send)
    shared: Rc[Int]

# Interior mutability requires synchronization
s SharedCounter               # Sync (Mutex provides synchronization)
    count: Arc[Mutex[Int]]
```

---

## 6. ARIA-Specific Design Recommendations

### 6.1 Simplest Useful Async Model

**Phase 1 Goals:**
- Simple to understand
- Simple to implement
- Useful for I/O-bound programs

**Core Primitives:**

```aria
# 1. Async function declaration
as f fetch(url: Str) -> Data!
    resp = aw http.get url?
    aw resp.json

# 2. Await expression
data = aw fetch url

# 3. Structured spawn (nursery)
nursery |n| as
    n.spawn || task_a
    n.spawn || task_b

# 4. Join (parallel await)
(a, b) = aw join(task_a, task_b)

# 5. Basic channel
(tx, rx) = channel[Int](cap: 10)
tx.send 42
val = aw rx.recv
```

**That's it for Phase 1.** No select, no race, no complex primitives.

### 6.2 Leveraging Second-Class References

**Simplifications:**

1. **No Pin:**
   ```aria
   # Futures are always movable in ARIA
   # because they can't contain self-references
   future = as || compute x
   other_future = future        # Move is always safe
   ```

2. **No lifetime annotations in async:**
   ```aria
   # Input references automatically valid for function body
   as f process(data: &[Int]) -> Int
       total = 0
       for x in data
           subtotal = aw compute x
           total += subtotal
       total
   # No lifetime complexity
   ```

3. **Spawn requires ownership or Arc:**
   ```aria
   # Clear ownership semantics
   as f example(owned: Data, shared: Arc[Data])
       # Move ownership to task
       spawn || process owned

       # Share via Arc
       s = shared.clone
       spawn || use_shared s
   ```

### 6.3 Single-Threaded Runtime (Phase 1)

```aria
# Minimal runtime implementation
s Runtime
    queue: VecDeque[Task]
    reactor: Reactor

i Runtime
    f new -> Self
        Runtime(
            queue: VecDeque.new
            reactor: Reactor.new
        )

    f spawn(&mut self, future: Future[T]) -> Handle[T]
        task = Task.new future
        handle = task.handle
        self.queue.push_back task
        handle

    f block_on[T](&mut self, future: Future[T]) -> T
        handle = self.spawn future

        lp
            m self.queue.pop_front
                Some task ->
                    waker = task.waker
                    cx = Context.new &waker
                    m task.poll &mut cx
                        Ready result -> ret result
                        Pending -> ()  # Waker will re-queue
                None ->
                    # No ready tasks, poll reactor
                    self.reactor.poll
```

### 6.4 Channel Design

**Bounded by default with clear semantics:**

```aria
# Core channel types
(tx, rx) = channel[T](cap: N)    # Bounded, blocks when full
(tx, rx) = unbounded[T]()        # Unbounded (use carefully)
(tx, rx) = oneshot[T]()          # Single value
(tx, _) = broadcast[T](cap: N)   # Multiple receivers

# Operations
tx.send value                     # Async, blocks if full
val = rx.recv                     # Async, blocks if empty
m tx.try_send value               # Non-blocking
    Ok () -> ...
    Err Full v -> ...
m rx.try_recv                     # Non-blocking
    Ok v -> ...
    Err Empty -> ...

# Select (Phase 2)
aw select
    msg = rx1.recv -> handle msg
    _ = rx2.recv -> other
    _ = timeout 5.secs -> timeout_handler
```

---

## 7. Implementation Roadmap

### Phase 1: Minimal Viable Async (MVP)

**Duration:** 2-3 months

**Components:**
1. Future trait and poll mechanism
2. Async function transformation (state machine)
3. Single-threaded executor
4. Basic waker implementation
5. Nursery for structured concurrency
6. Bounded channel (single implementation)

**Deliverables:**
```aria
# Should be able to write:
as f main
    nursery |n| as
        (tx, rx) = channel[Int](cap: 10)

        n.spawn || producer tx
        n.spawn || consumer rx

as f producer(tx: Sender[Int])
    for i in 0..100
        tx.send i
        aw sleep 10.ms

as f consumer(rx: Receiver[Int])
    lp
        m aw rx.recv
            Some i -> print i
            None -> br
```

### Phase 2: I/O Integration

**Duration:** 2-3 months

**Components:**
1. Reactor (epoll/kqueue/IOCP abstraction)
2. Async TCP/UDP sockets
3. Async file I/O (via thread pool)
4. Timeout and timer support
5. Select on multiple channels

**Deliverables:**
```aria
as f serve(addr: Str) -> ()!
    listener = aw TcpListener.bind addr?

    nursery |n| as
        lp
            (stream, peer) = aw listener.accept?
            n.spawn || handle stream
```

### Phase 3: Multi-Threaded Runtime

**Duration:** 3-4 months

**Components:**
1. Work-stealing scheduler
2. Thread-safe task queues
3. Thread pool for blocking ops
4. Runtime configuration API
5. Send/Sync trait enforcement

**Deliverables:**
```aria
f main
    rt = Runtime.builder
        | worker_threads 4
        | max_blocking 64
        | build

    rt.block_on serve "0.0.0.0:8080"
```

### Phase 4: Advanced Features

**Duration:** 3-4 months

**Components:**
1. Parallel iterators
2. Broadcast channels
3. Actor framework (optional)
4. Tracing/debugging support
5. Performance optimization

**Deliverables:**
```aria
# Parallel processing
results = items | par_map process | collect

# Actors (optional)
actor Counter
    state count: Int = 0

    msg Increment
    msg GetCount -> Int

    f handle(Increment)
        self.count += 1

    f handle(GetCount) -> Int
        self.count
```

---

## 8. Summary

### Key Design Decisions for ARIA

1. **Lazy Futures (like Rust)**
   - Zero-cost abstraction
   - Drop-based cancellation
   - Caller controls execution

2. **Second-Class References Simplify Everything**
   - No Pin needed
   - No lifetime annotations in async
   - Clear ownership for spawn

3. **Structured Concurrency Required**
   - Nurseries as primary spawn mechanism
   - Tasks cannot outlive their scope
   - Errors propagate naturally

4. **Bounded Channels by Default**
   - Backpressure built-in
   - Memory bounded
   - Explicit opt-in for unbounded

5. **Start Single-Threaded**
   - Simpler implementation
   - Easier debugging
   - Add multi-threading later

### Sources

- [Rust Async Book - Futures and Syntax](https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html)
- [Rust Async Book - The Future Trait](https://rust-lang.github.io/async-book/02_execution/02_future.html)
- [Tokio Scheduler Design](https://tokio.rs/blog/2019-10-scheduler)
- [Go Scheduler Design](https://www.ardanlabs.com/blog/2018/08/scheduling-in-go-part2.html)
- [Node.js Event Loop](https://nodejs.org/en/learn/asynchronous-work/event-loop-timers-and-nexttick)
- [Crossbeam Channel Documentation](https://docs.rs/crossbeam-channel)
- [Notes on Structured Concurrency](https://vorpus.org/blog/notes-on-structured-concurrency-or-go-statement-considered-harmful/)
- [Swift Structured Concurrency Proposal](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0304-structured-concurrency.md)
- [Kotlin Coroutine Cancellation](https://kotlinlang.org/docs/cancellation-and-timeouts.html)
- [Second-Class References](https://borretti.me/article/second-class-references)
- [Rust Send and Sync](https://doc.rust-lang.org/nomicon/send-and-sync.html)
- [Data Races vs Race Conditions](https://blog.regehr.org/archives/490)
