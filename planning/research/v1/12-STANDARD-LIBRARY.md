# ARIA Standard Library Architecture

## Design Principles

1. **Batteries Included**: Common tasks should be easy
2. **Zero-Cost Abstractions**: Don't pay for what you don't use
3. **Consistent API**: Familiar patterns throughout
4. **Cross-Platform**: Portable by default
5. **Well-Documented**: Every function documented with examples

---

## 1. Library Structure

```
std/
├── core/           # Fundamental types (no heap, no OS)
│   ├── primitives  # Int, Float, Bool, Char
│   ├── option      # Option[T]
│   ├── result      # Result[T, E]
│   ├── ops         # Operator traits
│   ├── cmp         # Comparison traits
│   ├── convert     # Type conversion traits
│   ├── iter        # Iterator traits
│   └── marker      # Marker traits (Copy, Send, Sync)
│
├── alloc/          # Heap allocation (no OS)
│   ├── string      # String, &str
│   ├── vec         # List[T]
│   ├── boxed       # Box[T]
│   ├── rc          # Rc[T], Arc[T]
│   └── collections # HashMap, HashSet, etc.
│
├── std/            # Full standard library (requires OS)
│   ├── io          # I/O operations
│   ├── fs          # File system
│   ├── net         # Networking
│   ├── env         # Environment
│   ├── process     # Process management
│   ├── thread      # Threading
│   ├── sync        # Synchronization
│   ├── time        # Time and duration
│   └── path        # Path manipulation
│
└── extra/          # Extended library (separate packages)
    ├── regex       # Regular expressions
    ├── json        # JSON parsing
    ├── http        # HTTP client/server
    ├── crypto      # Cryptography
    └── async       # Async runtime
```

---

## 2. Core Types

### 2.1 Option

```aria
pub enum Option[T] {
    Some(T)
    None
}

impl[T] Option[T] {
    // Constructors
    pub fn some(value: T) -> Option[T]
    pub fn none() -> Option[T]

    // Querying
    pub fn is_some(&self) -> Bool
    pub fn is_none(&self) -> Bool
    pub fn contains(&self, value: &T) -> Bool where T: Eq

    // Extracting
    pub fn unwrap(self) -> T
    pub fn unwrap_or(self, default: T) -> T
    pub fn unwrap_or_else(self, f: fn() -> T) -> T
    pub fn unwrap_or_default(self) -> T where T: Default
    pub fn expect(self, msg: &str) -> T

    // Transforming
    pub fn map[U](self, f: fn(T) -> U) -> Option[U]
    pub fn map_or[U](self, default: U, f: fn(T) -> U) -> U
    pub fn and_then[U](self, f: fn(T) -> Option[U]) -> Option[U]
    pub fn filter(self, predicate: fn(&T) -> Bool) -> Option[T]
    pub fn flatten(self) -> Option[T] where T: Option[U]
    pub fn zip[U](self, other: Option[U]) -> Option[(T, U)]

    // Converting
    pub fn ok_or[E](self, err: E) -> Result[T, E]
    pub fn ok_or_else[E](self, f: fn() -> E) -> Result[T, E]
    pub fn iter(&self) -> Iter[&T]
}
```

### 2.2 Result

```aria
pub enum Result[T, E] {
    Ok(T)
    Err(E)
}

impl[T, E] Result[T, E] {
    // Constructors
    pub fn ok(value: T) -> Result[T, E]
    pub fn err(error: E) -> Result[T, E]

    // Querying
    pub fn is_ok(&self) -> Bool
    pub fn is_err(&self) -> Bool

    // Extracting
    pub fn unwrap(self) -> T
    pub fn unwrap_err(self) -> E
    pub fn unwrap_or(self, default: T) -> T
    pub fn unwrap_or_else(self, f: fn(E) -> T) -> T
    pub fn expect(self, msg: &str) -> T
    pub fn expect_err(self, msg: &str) -> E

    // Transforming
    pub fn map[U](self, f: fn(T) -> U) -> Result[U, E]
    pub fn map_err[F](self, f: fn(E) -> F) -> Result[T, F]
    pub fn and_then[U](self, f: fn(T) -> Result[U, E]) -> Result[U, E]
    pub fn or_else[F](self, f: fn(E) -> Result[T, F]) -> Result[T, F]
    pub fn flatten(self) -> Result[T, E] where T: Result[U, E]

    // Converting
    pub fn ok(self) -> Option[T]
    pub fn err(self) -> Option[E]
    pub fn iter(&self) -> Iter[&T]
}
```

---

## 3. Collections

### 3.1 List (Dynamic Array)

```aria
pub struct List[T] { ... }

impl[T] List[T] {
    // Constructors
    pub fn new() -> List[T]
    pub fn with_capacity(capacity: Int) -> List[T]
    pub fn from_iter[I: Iterator[Item = T]](iter: I) -> List[T]

    // Capacity
    pub fn len(&self) -> Int
    pub fn is_empty(&self) -> Bool
    pub fn capacity(&self) -> Int
    pub fn reserve(&mut self, additional: Int)
    pub fn shrink_to_fit(&mut self)

    // Accessing
    pub fn get(&self, index: Int) -> Option[&T]
    pub fn get_mut(&mut self, index: Int) -> Option[&mut T]
    pub fn first(&self) -> Option[&T]
    pub fn last(&self) -> Option[&T]

    // Modifying
    pub fn push(&mut self, value: T)
    pub fn pop(&mut self) -> Option[T]
    pub fn insert(&mut self, index: Int, value: T)
    pub fn remove(&mut self, index: Int) -> T
    pub fn clear(&mut self)
    pub fn append(&mut self, other: &mut List[T])
    pub fn extend[I: Iterator[Item = T]](&mut self, iter: I)

    // Searching
    pub fn contains(&self, value: &T) -> Bool where T: Eq
    pub fn position(&self, predicate: fn(&T) -> Bool) -> Option[Int]
    pub fn binary_search(&self, value: &T) -> Result[Int, Int] where T: Ord

    // Iterating
    pub fn iter(&self) -> Iter[&T]
    pub fn iter_mut(&mut self) -> IterMut[&mut T]
    pub fn into_iter(self) -> IntoIter[T]

    // Transforming
    pub fn map[U](&self, f: fn(&T) -> U) -> List[U]
    pub fn filter(&self, predicate: fn(&T) -> Bool) -> List[T] where T: Clone
    pub fn fold[U](&self, init: U, f: fn(U, &T) -> U) -> U

    // Sorting
    pub fn sort(&mut self) where T: Ord
    pub fn sort_by(&mut self, compare: fn(&T, &T) -> Ordering)
    pub fn reverse(&mut self)
}

// Index operator
impl[T] Index[Int] for List[T] {
    type Output = T
    fn index(&self, index: Int) -> &T
}
```

### 3.2 Map (HashMap)

```aria
pub struct Map[K, V] where K: Hash + Eq { ... }

impl[K, V] Map[K, V] where K: Hash + Eq {
    // Constructors
    pub fn new() -> Map[K, V]
    pub fn with_capacity(capacity: Int) -> Map[K, V]

    // Capacity
    pub fn len(&self) -> Int
    pub fn is_empty(&self) -> Bool
    pub fn capacity(&self) -> Int

    // Accessing
    pub fn get(&self, key: &K) -> Option[&V]
    pub fn get_mut(&mut self, key: &K) -> Option[&mut V]
    pub fn contains_key(&self, key: &K) -> Bool
    pub fn keys(&self) -> Keys[K]
    pub fn values(&self) -> Values[V]

    // Modifying
    pub fn insert(&mut self, key: K, value: V) -> Option[V]
    pub fn remove(&mut self, key: &K) -> Option[V]
    pub fn clear(&mut self)
    pub fn entry(&mut self, key: K) -> Entry[K, V]

    // Iterating
    pub fn iter(&self) -> Iter[(K, V)]
    pub fn iter_mut(&mut self) -> IterMut[(K, &mut V)]
}

// Entry API for efficient insert-or-update
pub enum Entry[K, V] {
    Occupied(OccupiedEntry[K, V])
    Vacant(VacantEntry[K, V])
}

impl[K, V] Entry[K, V] {
    pub fn or_insert(self, default: V) -> &mut V
    pub fn or_insert_with(self, f: fn() -> V) -> &mut V
    pub fn and_modify(self, f: fn(&mut V)) -> Entry[K, V]
}
```

### 3.3 Set (HashSet)

```aria
pub struct Set[T] where T: Hash + Eq { ... }

impl[T] Set[T] where T: Hash + Eq {
    // Constructors
    pub fn new() -> Set[T]
    pub fn from_iter[I: Iterator[Item = T]](iter: I) -> Set[T]

    // Capacity
    pub fn len(&self) -> Int
    pub fn is_empty(&self) -> Bool

    // Accessing
    pub fn contains(&self, value: &T) -> Bool
    pub fn get(&self, value: &T) -> Option[&T]

    // Modifying
    pub fn insert(&mut self, value: T) -> Bool
    pub fn remove(&mut self, value: &T) -> Bool
    pub fn clear(&mut self)

    // Set operations
    pub fn union(&self, other: &Set[T]) -> Set[T] where T: Clone
    pub fn intersection(&self, other: &Set[T]) -> Set[T] where T: Clone
    pub fn difference(&self, other: &Set[T]) -> Set[T] where T: Clone
    pub fn symmetric_difference(&self, other: &Set[T]) -> Set[T] where T: Clone
    pub fn is_subset(&self, other: &Set[T]) -> Bool
    pub fn is_superset(&self, other: &Set[T]) -> Bool
    pub fn is_disjoint(&self, other: &Set[T]) -> Bool

    // Iterating
    pub fn iter(&self) -> Iter[&T]
}
```

---

## 4. String Operations

### 4.1 String

```aria
pub struct String { ... }

impl String {
    // Constructors
    pub fn new() -> String
    pub fn from(s: &str) -> String
    pub fn with_capacity(capacity: Int) -> String

    // Capacity
    pub fn len(&self) -> Int
    pub fn is_empty(&self) -> Bool
    pub fn capacity(&self) -> Int

    // Accessing
    pub fn as_str(&self) -> &str
    pub fn chars(&self) -> Chars
    pub fn bytes(&self) -> Bytes
    pub fn lines(&self) -> Lines

    // Modifying
    pub fn push(&mut self, c: Char)
    pub fn push_str(&mut self, s: &str)
    pub fn insert(&mut self, index: Int, c: Char)
    pub fn insert_str(&mut self, index: Int, s: &str)
    pub fn clear(&mut self)
    pub fn truncate(&mut self, len: Int)

    // Searching
    pub fn contains(&self, pattern: &str) -> Bool
    pub fn starts_with(&self, prefix: &str) -> Bool
    pub fn ends_with(&self, suffix: &str) -> Bool
    pub fn find(&self, pattern: &str) -> Option[Int]
    pub fn rfind(&self, pattern: &str) -> Option[Int]

    // Transforming
    pub fn to_lowercase(&self) -> String
    pub fn to_uppercase(&self) -> String
    pub fn trim(&self) -> &str
    pub fn trim_start(&self) -> &str
    pub fn trim_end(&self) -> &str
    pub fn replace(&self, from: &str, to: &str) -> String
    pub fn split(&self, pattern: &str) -> Split
    pub fn splitn(&self, n: Int, pattern: &str) -> SplitN

    // Parsing
    pub fn parse[T: FromStr](&self) -> Result[T, T.Err]
}
```

### 4.2 String Formatting

```aria
// String interpolation
let name = "World"
let greeting = "Hello, {name}!"

// Format expressions
let pi = 3.14159
let formatted = "Pi is {pi:.2}"  // "Pi is 3.14"

// Format specifiers
"{value}"         // Default
"{value:?}"       // Debug format
"{value:width}"   // Minimum width
"{value:>width}"  // Right align
"{value:<width}"  // Left align
"{value:^width}"  // Center
"{value:0width}"  // Zero-pad
"{value:.prec}"   // Precision (floats)
"{value:x}"       // Hexadecimal
"{value:b}"       // Binary
"{value:o}"       // Octal
"{value:e}"       // Scientific

// Format macro
let s = format("Name: {}, Age: {}", name, age)
```

---

## 5. I/O Operations

### 5.1 Traits

```aria
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result[Int, IoError]

    fn read_exact(&mut self, buf: &mut [u8]) -> Result[Unit, IoError]
    fn read_to_end(&mut self, buf: &mut List[u8]) -> Result[Int, IoError]
    fn read_to_string(&mut self, buf: &mut String) -> Result[Int, IoError]
    fn bytes(&mut self) -> Bytes
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result[Int, IoError]
    fn flush(&mut self) -> Result[Unit, IoError]

    fn write_all(&mut self, buf: &[u8]) -> Result[Unit, IoError]
    fn write_fmt(&mut self, args: FormatArgs) -> Result[Unit, IoError]
}

pub trait BufRead: Read {
    fn fill_buf(&mut self) -> Result[&[u8], IoError]
    fn consume(&mut self, amt: Int)

    fn read_line(&mut self, buf: &mut String) -> Result[Int, IoError]
    fn lines(&mut self) -> Lines
}
```

### 5.2 Standard I/O

```aria
// Standard streams
pub fn stdin() -> Stdin
pub fn stdout() -> Stdout
pub fn stderr() -> Stderr

// Convenience functions
pub fn print(s: &str)
pub fn println(s: &str)
pub fn eprint(s: &str)
pub fn eprintln(s: &str)

// Reading
let line = stdin().read_line()?
for line in stdin().lines() {
    process(line?)
}
```

---

## 6. File System

### 6.1 File Operations

```aria
pub struct File { ... }

impl File {
    // Opening
    pub fn open(path: &Path) -> Result[File, IoError]
    pub fn create(path: &Path) -> Result[File, IoError]

    pub fn options() -> OpenOptions

    // Metadata
    pub fn metadata(&self) -> Result[Metadata, IoError]
    pub fn set_permissions(&self, perm: Permissions) -> Result[Unit, IoError]
}

pub struct OpenOptions { ... }

impl OpenOptions {
    pub fn new() -> OpenOptions
    pub fn read(&mut self, read: Bool) -> &mut Self
    pub fn write(&mut self, write: Bool) -> &mut Self
    pub fn append(&mut self, append: Bool) -> &mut Self
    pub fn truncate(&mut self, truncate: Bool) -> &mut Self
    pub fn create(&mut self, create: Bool) -> &mut Self
    pub fn create_new(&mut self, create_new: Bool) -> &mut Self
    pub fn open(&self, path: &Path) -> Result[File, IoError]
}

impl Read for File { ... }
impl Write for File { ... }
```

### 6.2 Convenience Functions

```aria
pub mod fs {
    // Reading
    pub fn read(path: &Path) -> Result[List[u8], IoError]
    pub fn read_string(path: &Path) -> Result[String, IoError]

    // Writing
    pub fn write(path: &Path, contents: &[u8]) -> Result[Unit, IoError]
    pub fn write_string(path: &Path, contents: &str) -> Result[Unit, IoError]

    // Directory operations
    pub fn create_dir(path: &Path) -> Result[Unit, IoError]
    pub fn create_dir_all(path: &Path) -> Result[Unit, IoError]
    pub fn remove_dir(path: &Path) -> Result[Unit, IoError]
    pub fn remove_dir_all(path: &Path) -> Result[Unit, IoError]
    pub fn read_dir(path: &Path) -> Result[ReadDir, IoError]

    // File operations
    pub fn copy(from: &Path, to: &Path) -> Result[Int, IoError]
    pub fn rename(from: &Path, to: &Path) -> Result[Unit, IoError]
    pub fn remove(path: &Path) -> Result[Unit, IoError]

    // Metadata
    pub fn metadata(path: &Path) -> Result[Metadata, IoError]
    pub fn exists(path: &Path) -> Bool
    pub fn is_file(path: &Path) -> Bool
    pub fn is_dir(path: &Path) -> Bool
}
```

---

## 7. Networking

### 7.1 TCP

```aria
pub struct TcpListener { ... }
pub struct TcpStream { ... }

impl TcpListener {
    pub fn bind(addr: &str) -> Result[TcpListener, IoError]
    pub fn accept(&self) -> Result[(TcpStream, SocketAddr), IoError]
    pub fn incoming(&self) -> Incoming
    pub fn local_addr(&self) -> Result[SocketAddr, IoError]
}

impl TcpStream {
    pub fn connect(addr: &str) -> Result[TcpStream, IoError]
    pub fn peer_addr(&self) -> Result[SocketAddr, IoError]
    pub fn local_addr(&self) -> Result[SocketAddr, IoError]
    pub fn shutdown(&self, how: Shutdown) -> Result[Unit, IoError]
    pub fn set_read_timeout(&self, dur: Option[Duration]) -> Result[Unit, IoError]
    pub fn set_write_timeout(&self, dur: Option[Duration]) -> Result[Unit, IoError]
}

impl Read for TcpStream { ... }
impl Write for TcpStream { ... }
```

### 7.2 UDP

```aria
pub struct UdpSocket { ... }

impl UdpSocket {
    pub fn bind(addr: &str) -> Result[UdpSocket, IoError]
    pub fn connect(&self, addr: &str) -> Result[Unit, IoError]
    pub fn send(&self, buf: &[u8]) -> Result[Int, IoError]
    pub fn send_to(&self, buf: &[u8], addr: &str) -> Result[Int, IoError]
    pub fn recv(&self, buf: &mut [u8]) -> Result[Int, IoError]
    pub fn recv_from(&self, buf: &mut [u8]) -> Result[(Int, SocketAddr), IoError]
}
```

---

## 8. Time

### 8.1 Duration

```aria
pub struct Duration { ... }

impl Duration {
    // Constructors
    pub fn from_secs(secs: u64) -> Duration
    pub fn from_millis(millis: u64) -> Duration
    pub fn from_micros(micros: u64) -> Duration
    pub fn from_nanos(nanos: u64) -> Duration

    // Convenience constructors
    pub fn seconds(secs: u64) -> Duration
    pub fn minutes(mins: u64) -> Duration
    pub fn hours(hours: u64) -> Duration
    pub fn days(days: u64) -> Duration

    // Getters
    pub fn as_secs(&self) -> u64
    pub fn as_millis(&self) -> u128
    pub fn as_micros(&self) -> u128
    pub fn as_nanos(&self) -> u128

    // Arithmetic
    pub fn checked_add(&self, rhs: Duration) -> Option[Duration]
    pub fn checked_sub(&self, rhs: Duration) -> Option[Duration]
    pub fn checked_mul(&self, rhs: u32) -> Option[Duration]
    pub fn checked_div(&self, rhs: u32) -> Option[Duration]
}
```

### 8.2 Instant (Monotonic Time)

```aria
pub struct Instant { ... }

impl Instant {
    pub fn now() -> Instant
    pub fn elapsed(&self) -> Duration
    pub fn duration_since(&self, earlier: Instant) -> Duration
    pub fn checked_duration_since(&self, earlier: Instant) -> Option[Duration]
    pub fn checked_add(&self, duration: Duration) -> Option[Instant]
    pub fn checked_sub(&self, duration: Duration) -> Option[Instant]
}

// Usage
let start = Instant.now()
do_work()
let elapsed = start.elapsed()
print("Took {elapsed.as_millis()}ms")
```

### 8.3 DateTime

```aria
pub struct DateTime { ... }

impl DateTime {
    // Current time
    pub fn now() -> DateTime
    pub fn now_utc() -> DateTime

    // Constructors
    pub fn from_timestamp(secs: i64, nanos: u32) -> DateTime
    pub fn parse(s: &str, fmt: &str) -> Result[DateTime, ParseError]

    // Components
    pub fn year(&self) -> Int
    pub fn month(&self) -> Int
    pub fn day(&self) -> Int
    pub fn hour(&self) -> Int
    pub fn minute(&self) -> Int
    pub fn second(&self) -> Int

    // Formatting
    pub fn format(&self, fmt: &str) -> String

    // Arithmetic
    pub fn add(&self, duration: Duration) -> DateTime
    pub fn sub(&self, duration: Duration) -> DateTime
    pub fn duration_since(&self, earlier: DateTime) -> Duration
}
```

---

## 9. Path Manipulation

```aria
pub struct Path { ... }
pub struct PathBuf { ... }

impl Path {
    // Creation
    pub fn new(s: &str) -> &Path

    // Components
    pub fn file_name(&self) -> Option[&str]
    pub fn file_stem(&self) -> Option[&str]
    pub fn extension(&self) -> Option[&str]
    pub fn parent(&self) -> Option[&Path]
    pub fn components(&self) -> Components

    // Queries
    pub fn exists(&self) -> Bool
    pub fn is_file(&self) -> Bool
    pub fn is_dir(&self) -> Bool
    pub fn is_absolute(&self) -> Bool
    pub fn is_relative(&self) -> Bool

    // Manipulation
    pub fn join(&self, path: &Path) -> PathBuf
    pub fn with_extension(&self, ext: &str) -> PathBuf
    pub fn with_file_name(&self, name: &str) -> PathBuf

    // Conversion
    pub fn to_str(&self) -> Option[&str]
    pub fn to_string_lossy(&self) -> String
    pub fn to_path_buf(&self) -> PathBuf
}

impl PathBuf {
    pub fn new() -> PathBuf
    pub fn from(s: &str) -> PathBuf
    pub fn push(&mut self, path: &Path)
    pub fn pop(&mut self) -> Bool
    pub fn set_extension(&mut self, ext: &str) -> Bool
    pub fn set_file_name(&mut self, name: &str)
}
```

---

## 10. Environment

```aria
pub mod env {
    // Environment variables
    pub fn var(key: &str) -> Result[String, VarError]
    pub fn var_os(key: &str) -> Option[String]
    pub fn set_var(key: &str, value: &str)
    pub fn remove_var(key: &str)
    pub fn vars() -> Vars

    // Current directory
    pub fn current_dir() -> Result[PathBuf, IoError]
    pub fn set_current_dir(path: &Path) -> Result[Unit, IoError]

    // Executable path
    pub fn current_exe() -> Result[PathBuf, IoError]

    // Command line arguments
    pub fn args() -> Args
    pub fn args_os() -> ArgsOs
}

// Usage
let home = env.var("HOME")?
let args: List[String] = env.args().collect()
```

---

## 11. Prelude

The prelude is automatically imported into every module:

```aria
// Prelude contents
pub use core.option.Option
pub use core.option.Option.{Some, None}
pub use core.result.Result
pub use core.result.Result.{Ok, Err}

pub use core.clone.Clone
pub use core.copy.Copy
pub use core.default.Default
pub use core.cmp.{Eq, Ord, PartialEq, PartialOrd}
pub use core.convert.{From, Into}
pub use core.iter.{Iterator, IntoIterator}
pub use core.ops.{Drop, Fn, FnMut, FnOnce}

pub use alloc.string.String
pub use alloc.vec.List
pub use alloc.boxed.Box

pub use std.print
pub use std.println
```
