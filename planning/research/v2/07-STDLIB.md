# ARIA v2 Standard Library

## Design: Compact Method Names, Pipeline-Friendly

---

## 1. Collections

### 1.1 List ([T])

```
# Creation
items: [Int] = []
items = [1, 2, 3]
items = List.new
items = List.with_cap 100

# Query
items.len
items.empty
items.cap

# Access
items[0]              # panics if out of bounds
items.get 0           # returns T?
items.first           # T?
items.last            # T?

# Modify
items.push 4
items.pop             # T?
items.insert(0, x)
items.remove 0        # T
items.clear
items.extend other

# Search
items.contains &x
items.pos |x| x > 5   # Int? (position)
items.find |x| x > 5  # T?

# Transform (pipeline-friendly)
items | map f
items | filter p
items | fold(init, f)
items | flat_map f
items | take n
items | skip n
items | rev
items | sort
items | sort_by f
items | dedup
items | chunks n
items | zip other
items | enum          # [(Int, T)]

# Collect
iter | collect        # to [T]
```

### 1.2 Map ({K:V})

```
# Creation
scores: {Str:Int} = {}
scores = {"alice": 100, "bob": 95}
scores = Map.new

# Query
scores.len
scores.empty
scores.contains_key &k

# Access
scores["alice"]       # panics if missing
scores.get "alice"    # V?

# Modify
scores.insert("charlie", 90)  # V? (old value)
scores.remove "alice"         # V?
scores.clear

# Entry API
scores.entry("alice")
    | or_insert 0
    | and_modify |v| *v += 10

# Iterate
scores | keys         # [K]
scores | values       # [V]
scores | iter         # [(K, V)]
```

### 1.3 Set ({T})

```
# Creation
ids: {Int} = {}
ids = {1, 2, 3}
ids = Set.new

# Query
ids.len
ids.empty
ids.contains &x

# Modify
ids.insert x          # B (was new?)
ids.remove &x         # B (existed?)
ids.clear

# Set operations
a | union b           # {T}
a | intersect b       # {T}
a | diff b            # {T}
a | sym_diff b        # {T}
a.is_subset &b
a.is_superset &b
a.is_disjoint &b
```

---

## 2. Strings

### 2.1 Str

```
# Creation
s = "hello"
s = Str.new
s = Str.with_cap 100

# Query
s.len
s.empty
s.cap

# Access
s.chars              # iterator
s.bytes              # iterator
s.lines              # iterator

# Modify
s.push 'c'
s.push_str "world"
s.insert(0, 'H')
s.clear
s.truncate 5

# Search
s.contains "ell"
s.starts_with "he"
s.ends_with "lo"
s.find "ll"          # Int?
s.rfind "l"          # Int?

# Transform
s | lower
s | upper
s | trim
s | trim_start
s | trim_end
s | replace("a", "b")
s | split " "        # [Str]
s | splitn(2, " ")

# Parse
s.parse[Int]         # Int!
s.parse[Float]       # Float!
```

### 2.2 Formatting

```
# Interpolation
name = "World"
msg = "Hello, {name}!"

# Expressions
"Sum: {a + b}"
"Pi: {pi:.2}"        # 2 decimal places

# Format specifiers
"{x}"                # default
"{x:?}"              # debug
"{x:10}"             # width 10
"{x:>10}"            # right align
"{x:<10}"            # left align
"{x:^10}"            # center
"{x:010}"            # zero pad
"{x:.3}"             # 3 decimal places
"{x:x}"              # hex
"{x:b}"              # binary
```

---

## 3. I/O

### 3.1 Traits

```
t Read
    f read(&mut self, buf: &mut [u8]) -> Int!
    f read_exact(&mut self, buf: &mut [u8]) -> ()!
    f read_all(&mut self, buf: &mut [u8]) -> Int!
    f read_str(&mut self, buf: &mut Str) -> Int!

t Write
    f write(&mut self, buf: &[u8]) -> Int!
    f write_all(&mut self, buf: &[u8]) -> ()!
    f flush(&mut self) -> ()!

t BufRead: Read
    f read_line(&mut self, buf: &mut Str) -> Int!
    f lines(&mut self) -> Lines
```

### 3.2 Standard I/O

```
# Output
print "Hello"
println "Hello"
eprint "Error"
eprintln "Error"

# Input
line = stdin.read_line?
for line in stdin.lines
    process line?
```

---

## 4. File System

### 4.1 File

```
# Open
file = File.open path?
file = File.create path?

file = File.opts
    | read T
    | write T
    | append T
    | create T
    | open path?

# Read/Write
content = file.read?
file.write data?
file.flush?

# Metadata
meta = file.metadata?
file.set_perms perms?
```

### 4.2 fs Module

```
# Read
data = fs.read path?           # [u8]
text = fs.read_str path?       # Str

# Write
fs.write(path, data)?
fs.write_str(path, text)?

# Directory
fs.create_dir path?
fs.create_dir_all path?
fs.remove_dir path?
fs.remove_dir_all path?
entries = fs.read_dir path?    # [DirEntry]

# File operations
fs.copy(from, to)?
fs.rename(from, to)?
fs.remove path?

# Query
meta = fs.metadata path?
fs.exists path                 # B
fs.is_file path                # B
fs.is_dir path                 # B
```

---

## 5. Path

```
# Create
p = Path.new "/home/user"
p = "/home/user".as_path

# Components
p.file_name          # Str?
p.file_stem          # Str?
p.ext                # Str?
p.parent             # Path?
p.components         # iterator

# Query
p.exists
p.is_file
p.is_dir
p.is_abs
p.is_rel

# Manipulation
p.join "subdir"
p.with_ext "txt"
p.with_name "new.txt"

# Convert
p.to_str             # Str?
p.to_string          # Str (lossy)
```

---

## 6. Time

### 6.1 Duration

```
# Create
d = Duration.secs 60
d = Duration.millis 1000
d = Duration.micros 1000000
d = Duration.nanos 1000000000

# Shorthand
d = 60.secs
d = 1000.millis
d = 5.mins
d = 2.hours

# Query
d.as_secs
d.as_millis
d.as_micros
d.as_nanos

# Arithmetic
d1 + d2
d1 - d2
d * 2
d / 2
```

### 6.2 Instant

```
# Monotonic clock
start = Instant.now
do_work
elapsed = start.elapsed

# Comparison
if start.elapsed > 5.secs
    timeout
```

### 6.3 DateTime

```
# Current
now = DateTime.now
utc = DateTime.now_utc

# Parse/Format
dt = DateTime.parse("2025-01-22", "%Y-%m-%d")?
s = dt.format "%Y-%m-%d %H:%M:%S"

# Components
dt.year
dt.month
dt.day
dt.hour
dt.minute
dt.second

# Arithmetic
dt + 1.days
dt - 2.hours
dt1.since dt2          # Duration
```

---

## 7. Networking

### 7.1 TCP

```
# Server
listener = TcpListener.bind "127.0.0.1:8080"?
for (stream, addr) in listener.incoming
    handle stream?

# Client
stream = TcpStream.connect "127.0.0.1:8080"?
stream.write b"Hello"?
response = stream.read?
```

### 7.2 UDP

```
socket = UdpSocket.bind "127.0.0.1:8080"?
socket.send_to(data, "127.0.0.1:9000")?
(data, addr) = socket.recv_from?
```

---

## 8. JSON (Extended Library)

```
us std.json

# Parse
data: JsonValue = json.parse text?
user: User = json.parse text?

# Generate
text = json.to_string &data
text = json.to_string_pretty &data

# Access
data["key"]
data[0]
data.get "key"       # JsonValue?
data.as_str          # Str?
data.as_int          # Int?
data.as_bool         # B?
data.as_array        # [JsonValue]?
data.as_object       # {Str:JsonValue}?

# Derive for custom types
@derive(Serialize, Deserialize)
s User
    name: Str
    age: Int
```

---

## 9. Environment

```
us std.env

# Variables
value = env.var "HOME"?
value = env.var_or("PORT", "8080")
env.set_var("KEY", "value")
env.remove_var "KEY"
for (k, v) in env.vars
    print "{k}={v}"

# Directory
cwd = env.current_dir?
env.set_current_dir path?

# Executable
exe = env.current_exe?

# Arguments
args = env.args             # [Str]
```

---

## 10. Prelude

Automatically imported into every module:

```
# Types
Int, Float, Str, B, Char
[T], {K:V}, {T}
T?, T!, T!E

# Option/Result
Some, N, none
Ok, Err, ok, err

# Traits
Clone, Copy, Default
Eq, Ord
Display, Debug
From, Into
Iterator

# Functions
print, println
eprint, eprintln

# Smart pointers
Box, Rc, Arc

# Collections
List, Map, Set
```
