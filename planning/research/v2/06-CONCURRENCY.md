# ARIA v2 Concurrency

## Design: Compact Async Syntax

---

## 1. Async Functions

### 1.1 Declaration

```
# Async function
as f fetch(url: Str) -> Data!
    resp = aw http.get url?
    aw resp.json

# Sync equivalent would be
f fetch_sync(url: Str) -> Data!
    resp = http.get_blocking url?
    resp.json_blocking
```

### 1.2 Await

```
# Basic await
data = aw fetch url

# Chain with ?
data = aw fetch url?

# Await in pipeline
result = aw (fetch url? | parse | validate)
```

### 1.3 Async Blocks

```
# Create future
future = as
    x = aw compute_x
    y = aw compute_y
    x + y

# Execute later
result = aw future
```

---

## 2. Concurrent Execution

### 2.1 Join (Parallel)

```
# Run concurrently, wait for all
(a, b) = aw join(fetch url1, fetch url2)

# Multiple
(a, b, c, d) = aw join(
    fetch url1
    fetch url2
    fetch url3
    fetch url4
)

# Join collection
results = aw join_all(urls | map fetch)
```

### 2.2 Race (First Wins)

```
# First to complete wins
result = aw race(fetch primary, fetch backup)

# With handling
m aw race(fetch primary, fetch backup)
    First data -> data
    Second data -> data
```

### 2.3 Select

```
# Handle first ready
aw select
    msg = channel.recv -> handle msg
    _ = timeout 5.secs -> handle_timeout
    _ = shutdown.recv -> ret
```

---

## 3. Structured Concurrency

### 3.1 Scope

```
# All tasks complete before scope exits
as f process_all(items: [Item]) -> [Output]!
    scope |s| as
        handles = items | map |item| s.spawn || process item
        aw join_all handles | collect

# Tasks cannot outlive scope
```

### 3.2 Nursery

```
# Nursery manages child tasks
as f main
    nursery |n| as
        n.spawn worker_a
        n.spawn worker_b
        n.spawn worker_c
        # All complete before exit

# With error handling
as f with_errors -> ()!
    nursery |n| as
        n.spawn_catch || risky_a
        n.spawn_catch || risky_b
        # If any fails, others cancelled
```

### 3.3 Background Tasks

```
as f with_background
    scope |s| as
        # Background task
        s.spawn_bg || lp
            cleanup
            aw sleep 5.mins

        # Main work
        aw main_work

        # Background cancelled on exit
```

---

## 4. Channels

### 4.1 Bounded Channel

```
# Create
(tx, rx) = channel[Int](cap: 10)

# Send (blocks if full)
tx.send 42

# Receive (blocks if empty)
value = rx.recv

# Try operations
m tx.try_send 42
    Ok () -> print "sent"
    Err Full v -> print "full"

m rx.try_recv
    Ok v -> print "got {v}"
    Err Empty -> print "empty"
```

### 4.2 Unbounded Channel

```
(tx, rx) = unbounded[Int]()
tx.send 42    # never blocks
```

### 4.3 Oneshot

```
(tx, rx) = oneshot[Data!]()

spawn || as
    result = aw compute
    tx.send result

data = aw rx.recv
```

### 4.4 Broadcast

```
(tx, _) = broadcast[Event](cap: 100)

rx1 = tx.subscribe
rx2 = tx.subscribe

tx.send Event.Update    # both receive
```

---

## 5. Spawn and Tasks

### 5.1 Spawn Task

```
# Spawn async task
handle = spawn || as
    aw do_work
    42

# Wait for result
result = aw handle.join

# Named task
handle = spawn_named "worker" || as
    aw worker_loop
```

### 5.2 Task Handle

```
# Check status
if handle.is_finished
    result = handle.try_join

# Cancel
handle.cancel

# Detach
handle.detach
```

---

## 6. Synchronization

### 6.1 Mutex

```
counter = Arc.new Mutex.new 0

handles = 0..10 | map |_|
    c = counter.clone
    spawn || as
        *aw c.lock += 1

aw join_all handles
print *counter.lock    # 10
```

### 6.2 RwLock

```
data = Arc.new RwLock.new Data.new

# Read (concurrent OK)
guard = aw data.read
print guard.value

# Write (exclusive)
mut_guard = aw data.write
mut_guard.value = new_value
```

### 6.3 Semaphore

```
sem = Arc.new Semaphore.new(permits: 3)

as f limited(sem: &Semaphore)
    permit = aw sem.acquire
    # Only 3 concurrent
    aw do_work
    # permit released on drop
```

### 6.4 Barrier

```
barrier = Arc.new Barrier.new(parties: 3)

for i in 0..3
    b = barrier.clone
    spawn || as
        do_phase_1
        aw b.wait     # all wait here
        do_phase_2
```

---

## 7. Parallel Iterators

```
# Parallel map
results = items | par_map process | collect

# Parallel filter
evens = numbers | par_filter (% 2 == 0) | collect

# Parallel reduce
sum = numbers | par_reduce(0, +)

# Parallel for_each
items | par_for_each process

# Parallel sort
sorted = items | par_sort
```

---

## 8. Timeouts and Cancellation

### 8.1 Timeout

```
as f with_timeout -> Data!
    m aw timeout(10.secs, fetch url)
        Ok data -> ok data
        Err Elapsed -> err Timeout
```

### 8.2 Cancellation Token

```
as f cancellable(token: CancelToken) -> Data!
    lp
        if token.is_cancelled
            ret err Cancelled
        # do work...
```

---

## 9. Runtime

### 9.1 Configuration

```
# Default multi-threaded
f main
    runtime.block_on as_main

# Single-threaded
f main
    runtime.single_thread.block_on as_main

# Custom
f main
    rt = Runtime.builder
        | worker_threads 4
        | max_blocking 512
        | enable_io
        | enable_time
        | build

    rt.block_on as_main
```

### 9.2 Blocking Operations

```
# Run blocking in async context
as f read_file(path: Str) -> Str!
    spawn_blocking || fs.read path

# DON'T block async runtime
as f bad
    std.thread.sleep 1.secs    # BAD

as f good
    aw sleep 1.secs            # Good
```

---

## 10. Complete Example

```
us std.net.{TcpListener, TcpStream}
us std.io.{BufReader, BufWriter}

s Server
    addr: Str
    handler: as Request -> Response

as f serve(server: Server) -> ()!
    listener = TcpListener.bind server.addr?

    print "Listening on {server.addr}"

    # Accept connections
    nursery |n| as
        lp
            (stream, addr) = aw listener.accept?
            print "Connection from {addr}"

            handler = server.handler
            n.spawn || handle_conn(stream, handler)

as f handle_conn(stream: TcpStream, handler: as Request -> Response)
    reader = BufReader.new &stream
    writer = BufWriter.new &stream

    lp
        # Read request with timeout
        request = m aw timeout(30.secs, read_request &reader)
            Ok req -> req
            Err _ -> br

        # Handle with timeout
        response = m aw timeout(60.secs, handler request)
            Ok resp -> resp
            Err _ -> Response.timeout

        # Write response
        aw write_response(&writer, response)?
        aw writer.flush?

f main
    server = Server(
        addr: "127.0.0.1:8080"
        handler: || as |req|
            m req.path
                "/" -> Response.text "Hello!"
                "/api" -> aw handle_api req
                _ -> Response.not_found
    )

    runtime.block_on || serve server
```
