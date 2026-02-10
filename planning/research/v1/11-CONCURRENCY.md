# ARIA Concurrency Model

## Design Philosophy

1. **Structured Concurrency**: Spawned tasks complete before scope exit
2. **Safety First**: Data races prevented at compile time
3. **Flexibility**: Multiple paradigms (async/await, channels, actors)
4. **Performance**: Zero-cost abstractions, no runtime overhead when unused
5. **Simplicity**: Sensible defaults, explicit choices

---

## 1. Async/Await

### 1.1 Async Functions

```aria
// Async function returns a Future
async fn fetch_data(url: String) -> Result[Data, Error] {
    let response = await http.get(url)?
    let data = await response.json()?
    ok(data)
}

// Calling async functions
async fn main() {
    let data = await fetch_data("https://api.example.com/data")
    process(data)
}
```

### 1.2 Async Blocks

```aria
// Create future from block
let future = async {
    let x = await compute_x()
    let y = await compute_y()
    x + y
}

// Execute later
let result = await future
```

### 1.3 Concurrent Execution

```aria
// Sequential (slow)
async fn sequential() {
    let a = await fetch_a()  // Wait...
    let b = await fetch_b()  // Then wait...
    (a, b)
}

// Concurrent (fast)
async fn concurrent() {
    let (a, b) = await join(fetch_a(), fetch_b())
    (a, b)
}

// Racing (first wins)
async fn race_example() {
    match await race(fetch_primary(), fetch_backup()) {
        First(data) => data
        Second(data) => data
    }
}

// Select (handle multiple)
async fn select_example() {
    select {
        msg = await channel.recv() => handle_message(msg)
        _ = await timeout(Duration.seconds(5)) => handle_timeout()
        _ = await shutdown.recv() => return
    }
}
```

### 1.4 Cancellation

```aria
// Dropping a future cancels it
async fn with_timeout() -> Result[Data, Error] {
    let future = fetch_data(url)

    match await timeout(Duration.seconds(10), future) {
        Ok(data) => ok(data)
        Err(Elapsed) => err(Error.Timeout)
    }
}

// Cancellation tokens
async fn cancellable(token: CancellationToken) -> Result[Data, Error] {
    loop {
        if token.is_cancelled() {
            return err(Error.Cancelled)
        }
        // Do work...
    }
}
```

---

## 2. Structured Concurrency

### 2.1 Task Scopes

```aria
// All spawned tasks must complete before scope exits
async fn process_items(items: List[Item]) -> Result[List[Output], Error] {
    scope(|s| async {
        let handles = items
            .map(|item| s.spawn(|| process_item(item)))
            .collect()

        // All tasks complete here
        let results = await join_all(handles)
        results.collect()
    })
}
```

### 2.2 Nursery Pattern

```aria
async fn main() {
    // Nursery ensures all children complete
    nursery(|n| async {
        n.spawn(worker_a())
        n.spawn(worker_b())
        n.spawn(worker_c())
        // All workers complete before nursery exits
    })
}

// With error handling
async fn with_errors() -> Result[Unit, Error] {
    nursery(|n| async {
        n.spawn_catch(|| risky_task_a())
        n.spawn_catch(|| risky_task_b())
        // If any task fails, others are cancelled
    })?
}
```

### 2.3 Background Tasks

```aria
// Fire and forget (still structured)
async fn with_background() {
    scope(|s| async {
        // Background task
        s.spawn_background(|| {
            loop {
                cleanup_old_data()
                await sleep(Duration.minutes(5))
            }
        })

        // Main work
        await main_work()

        // Background task cancelled when scope exits
    })
}
```

---

## 3. Channels

### 3.1 Bounded Channels

```aria
// Create bounded channel
let (tx, rx) = channel[Int](capacity: 10)

// Send (blocks if full)
tx.send(42)

// Receive (blocks if empty)
let value = rx.recv()

// Try operations (non-blocking)
match tx.try_send(42) {
    Ok(()) => print("Sent")
    Err(Full(value)) => print("Channel full")
}

match rx.try_recv() {
    Ok(value) => print("Got: {value}")
    Err(Empty) => print("Channel empty")
}
```

### 3.2 Unbounded Channels

```aria
// Create unbounded channel (never blocks on send)
let (tx, rx) = unbounded_channel[Int]()

tx.send(42)  // Always succeeds
```

### 3.3 One-Shot Channels

```aria
// Single value channel
let (tx, rx) = oneshot[Result[Data, Error]]()

spawn(|| async {
    let result = await compute()
    tx.send(result)
})

let result = await rx.recv()
```

### 3.4 Broadcast Channels

```aria
// Multiple receivers
let (tx, _) = broadcast[Event](capacity: 100)

let rx1 = tx.subscribe()
let rx2 = tx.subscribe()

tx.send(Event.Update)  // Both receivers get it
```

### 3.5 Channel Patterns

```aria
// Fan-out
async fn fan_out[T, R](
    input: Receiver[T],
    workers: Int,
    f: fn(T) -> R
) -> Receiver[R] {
    let (tx, rx) = channel[R](workers * 2)

    for _ in 0..workers {
        let tx = tx.clone()
        let input = input.clone()
        spawn(|| async {
            while let Some(item) = await input.recv() {
                tx.send(f(item))
            }
        })
    }

    rx
}

// Pipeline
async fn pipeline() {
    let numbers = produce_numbers()
    let doubled = numbers |> map_channel(|n| n * 2)
    let filtered = doubled |> filter_channel(|n| n > 10)

    while let Some(n) = await filtered.recv() {
        print(n)
    }
}
```

---

## 4. Synchronization Primitives

### 4.1 Mutex

```aria
// Mutual exclusion
let counter = Arc.new(Mutex.new(0))

let handles = (0..10).map(|_| {
    let counter = counter.clone()
    spawn(|| async {
        let mut guard = await counter.lock()
        *guard += 1
    })
}).collect()

await join_all(handles)
print(*counter.lock())  // 10
```

### 4.2 RwLock

```aria
// Multiple readers, single writer
let data = Arc.new(RwLock.new(Data.new()))

// Read (concurrent)
let read_guard = await data.read()
print(read_guard.value)

// Write (exclusive)
let mut write_guard = await data.write()
write_guard.value = new_value
```

### 4.3 Semaphore

```aria
// Limit concurrent access
let semaphore = Arc.new(Semaphore.new(permits: 3))

async fn limited_operation(sem: &Semaphore) {
    let permit = await sem.acquire()
    // Only 3 concurrent operations
    await do_work()
    // Permit released when dropped
}
```

### 4.4 Barrier

```aria
// Wait for all parties
let barrier = Arc.new(Barrier.new(parties: 3))

for i in 0..3 {
    let b = barrier.clone()
    spawn(|| async {
        do_phase_1()
        await b.wait()  // All wait here
        do_phase_2()
    })
}
```

### 4.5 Condition Variables

```aria
let pair = Arc.new((Mutex.new(false), Condvar.new()))

// Waiter
spawn(|| async {
    let (lock, cvar) = &*pair
    let mut ready = await lock.lock()
    while !*ready {
        ready = await cvar.wait(ready)
    }
    print("Ready!")
})

// Notifier
spawn(|| async {
    let (lock, cvar) = &*pair
    *await lock.lock() = true
    cvar.notify_one()
})
```

---

## 5. Thread Safety

### 5.1 Send and Sync Traits

```aria
// Send: Can be transferred to another thread
// Sync: Can be shared between threads (&T is Send)

// Most types are Send + Sync automatically
struct SafeData {
    value: Int      // Int is Send + Sync
    name: String    // String is Send + Sync
}

// Explicitly not Send/Sync
struct NotSend {
    rc: Rc[Int]     // Rc is not Send or Sync
}

// Explicitly not Sync
struct NotSync {
    cell: Cell[Int]  // Cell is Send but not Sync
}
```

### 5.2 Thread-Local Storage

```aria
thread_local! {
    static CACHE: RefCell[Cache] = RefCell.new(Cache.new())
}

fn use_cache() {
    CACHE.with(|cache| {
        let mut c = cache.borrow_mut()
        c.insert("key", "value")
    })
}
```

### 5.3 Atomic Types

```aria
// Lock-free atomic operations
let counter = AtomicInt.new(0)

// Atomic operations
counter.fetch_add(1, Ordering.SeqCst)
counter.store(42, Ordering.Release)
let value = counter.load(Ordering.Acquire)

// Compare and swap
let result = counter.compare_exchange(
    current: 42,
    new: 43,
    success: Ordering.SeqCst,
    failure: Ordering.Relaxed
)

// Available atomic types
AtomicBool
AtomicInt, AtomicI32, AtomicI64
AtomicUInt, AtomicU32, AtomicU64
AtomicPtr[T]
```

---

## 6. Spawn and Tasks

### 6.1 Spawning Tasks

```aria
// Spawn async task
let handle = spawn(|| async {
    await do_work()
    42
})

// Wait for result
let result = await handle.join()

// Spawn with name (for debugging)
let handle = spawn_named("worker", || async {
    await worker_loop()
})
```

### 6.2 Task Handles

```aria
// Check if complete
if handle.is_finished() {
    let result = handle.try_join()
}

// Cancel task
handle.cancel()

// Detach (fire and forget, but still structured)
handle.detach()
```

### 6.3 Task Local Storage

```aria
task_local! {
    static REQUEST_ID: String
}

async fn handle_request(id: String) {
    REQUEST_ID.scope(id, || async {
        // All code in this scope can access REQUEST_ID
        log("Processing {REQUEST_ID.get()}")
        await process()
    })
}
```

---

## 7. Parallel Iterators

### 7.1 Parallel Map

```aria
// Sequential
let results = items.map(|x| expensive_computation(x)).collect()

// Parallel
let results = items.par_map(|x| expensive_computation(x)).collect()

// With thread pool
let pool = ThreadPool.new(threads: 4)
let results = pool.map(items, |x| expensive_computation(x))
```

### 7.2 Parallel Operations

```aria
// Parallel filter
let evens = numbers.par_filter(|n| n % 2 == 0).collect()

// Parallel reduce
let sum = numbers.par_reduce(0, |a, b| a + b)

// Parallel for_each
items.par_for_each(|item| {
    process(item)
})

// Parallel sort
let sorted = items.par_sort()
let sorted = items.par_sort_by(|a, b| b.cmp(a))
```

---

## 8. Runtime Configuration

### 8.1 Executor Selection

```aria
// Default multi-threaded runtime
fn main() {
    runtime.block_on(async_main())
}

// Single-threaded runtime
fn main() {
    runtime.single_thread().block_on(async_main())
}

// Custom configuration
fn main() {
    let rt = Runtime.builder()
        .worker_threads(4)
        .max_blocking_threads(512)
        .enable_io()
        .enable_time()
        .build()

    rt.block_on(async_main())
}
```

### 8.2 Blocking Operations

```aria
// Run blocking code in async context
async fn read_file(path: String) -> Result[String, Error] {
    // Move to blocking thread pool
    spawn_blocking(|| {
        fs.read_string(&path)
    })
}

// Don't block the async runtime!
async fn bad() {
    std.thread.sleep(Duration.seconds(1))  // BAD: blocks runtime
}

async fn good() {
    await sleep(Duration.seconds(1))  // Good: yields to runtime
}
```

---

## 9. Actors (Optional Pattern)

### 9.1 Actor Definition

```aria
// Actor with state
actor Counter {
    state count: Int = 0

    // Messages
    msg Increment
    msg Decrement
    msg GetCount -> Int

    // Message handlers
    fn handle(msg: Increment) {
        self.count += 1
    }

    fn handle(msg: Decrement) {
        self.count -= 1
    }

    fn handle(msg: GetCount) -> Int {
        self.count
    }
}

// Using actors
async fn main() {
    let counter = Counter.spawn()

    counter.send(Counter.Increment)
    counter.send(Counter.Increment)

    let count = await counter.ask(Counter.GetCount)
    print("Count: {count}")
}
```

### 9.2 Supervision

```aria
// Supervisor restarts failed actors
supervisor MyApp {
    strategy: OneForOne  // Restart only failed child

    children {
        DatabaseActor.new(config.db_url)
        CacheActor.new(config.cache_size)
        WebActor.new(config.port)
    }
}

async fn main() {
    let app = MyApp.spawn()
    await app.wait()
}
```

---

## 10. Best Practices

### 10.1 Avoid Common Pitfalls

```aria
// DON'T: Hold lock across await
async fn bad() {
    let mut guard = await mutex.lock()
    await some_io()  // Lock held during I/O!
    *guard += 1
}

// DO: Minimize lock scope
async fn good() {
    let value = *await mutex.lock()
    let result = await some_io()
    *await mutex.lock() += result
}

// DON'T: Block in async context
async fn bad() {
    std.thread.sleep(Duration.seconds(1))
}

// DO: Use async sleep
async fn good() {
    await sleep(Duration.seconds(1))
}

// DON'T: Ignore cancellation
async fn bad(token: CancellationToken) {
    for item in huge_list {
        process(item)  // Never checks cancellation
    }
}

// DO: Check cancellation periodically
async fn good(token: CancellationToken) {
    for item in huge_list {
        token.check()?  // Returns Err if cancelled
        process(item)
    }
}
```

### 10.2 Performance Guidelines

```aria
// Batch small messages
async fn good() {
    let batch = Vec.with_capacity(100)
    for msg in messages {
        batch.push(msg)
        if batch.len() >= 100 {
            channel.send(batch)
            batch = Vec.with_capacity(100)
        }
    }
}

// Use appropriate buffer sizes
let (tx, rx) = channel[Message](capacity: 1000)  // Tune for workload

// Prefer ownership over Arc when possible
async fn better(data: Data) {  // Owns data
    spawn(move || process(data))
}
```

### 10.3 Testing Async Code

```aria
#[test]
async fn test_async_function() {
    let result = await fetch_data()
    assert_eq(result, expected)
}

#[test]
async fn test_timeout() {
    let result = await timeout(
        Duration.millis(100),
        slow_operation()
    )
    assert(result.is_err())
}

#[test]
async fn test_concurrent() {
    let results = await join(op_a(), op_b(), op_c())
    assert_eq(results, (a, b, c))
}
```
