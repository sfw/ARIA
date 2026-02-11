---
name: forma
description: >
  FORMA programming language reference. Use when writing FORMA (.forma) code,
  working on the FORMA compiler, or answering questions about FORMA syntax,
  builtins, contracts, or CLI usage.
---

# FORMA Language Skill

FORMA is an indentation-based, AI-optimized systems programming language. It uses
short keywords, design-by-contract, capability-based sandboxing, and a rich stdlib
of 316+ builtins.

For the complete builtin reference, read [docs/ai-reference.md](../../docs/ai-reference.md).

## Core Syntax

- Indentation-based blocks (like Python), no braces needed
- Comments: `# single line`
- Variables: `x := value` (mutable by default), `x: Type = value` (annotated)
- Last expression is implicit return value
- Generics use `[T]` not `<T>`
- F-strings: `f"hello {name}, {x + 1}"`

## Keywords (Short Forms)

| Short | Meaning     | Short | Meaning    |
|-------|-------------|-------|------------|
| `f`   | function    | `m`   | match      |
| `s`   | struct      | `wh`  | while      |
| `e`   | enum        | `lp`  | loop       |
| `t`   | trait        | `br`  | break      |
| `i`   | impl        | `ct`  | continue   |
| `us`  | use/import  | `ret` | return     |
| `md`  | module      | `as`  | async      |
| `pub` | public      | `aw`  | await      |
| `mut` | mutable     | `sp`  | spawn      |
| `ref` | reference   | `un`  | unsafe     |

Literals: `true`/`T`, `false`/`F`, `none`/`N`, `Some`, `Ok`/`ok`, `Err`/`err`

## Types

Primitives: `Int`, `Float`, `Bool`, `Char`, `Str`, `()`, `!`
Sized ints: `i8` `i16` `i32` `i64` `i128` `u8` `u16` `u32` `u64` `u128` `isize` `usize`
Collections: `[T]` list, `[T; N]` fixed, `{K: V}` map, `{T}` set, `(A, B)` tuple
Special: `T?` Option, `T!E` Result, `&T` shared ref, `&mut T` mutable ref
Async: `Task[T]`, `Future[T]`, `Sender[T]`, `Receiver[T]`, `Mutex[T]`

## Function

```forma
f name(a: Int, b: Int) -> Int
    a + b

f single_expr(a: Int, b: Int) -> Int = a + b

as f async_fn() -> Str!Str
    aw some_future()?

f with_ref(ref data: [Int]) -> Int          # shared ref param
f with_mut(ref mut data: [Int]) -> Unit     # mutable ref param
```

## Struct, Enum, Trait, Impl

```forma
s Point
    x: Float
    y: Float

s Pair(Int, Int)                             # tuple struct
s Unit                                       # unit struct

e Color = Red | Green | Blue
e Option[T] = Some(T) | None

t Printable
    f to_string(&self) -> Str

i Point
    f distance(&self) -> Float
        sqrt(self.x * self.x + self.y * self.y)

i Printable for Point
    f to_string(&self) -> Str
        f"{self.x}, {self.y}"
```

## Control Flow

```forma
if cond then expr else expr                  # inline expression form
if cond
    block
else if cond
    block
else
    block

m value                                      # match
    Pattern -> expr
    Variant(x) -> expr
    Point { x, y } -> x + y                 # struct destructure
    _ if guard -> expr                       # guard
    _ -> default

wh condition                                 # while
    body

for x in collection                          # for-in
    body

lp                                           # infinite loop
    if done then br

'outer: for x in xs                          # labeled loop
    for y in ys
        if x == y then br 'outer
```

## Error Handling

```forma
# Propagate with ? (Result only)
f load(path: Str) -> Str!Str
    content := file_read(path)?
    Ok(content)

# Default with ?? (Option only)
name := env_get("USER") ?? "unknown"

# Match on Result/Option
m result
    Ok(v) -> use(v)
    Err(e) -> handle(e)
```

## Contracts (Design by Contract)

```forma
@pre(n >= 0)
@post(result > 0, "must be positive")
f factorial(n: Int) -> Int
    if n <= 1 then 1 else n * factorial(n - 1)

@post(old(balance) + delta == result)
f deposit(balance: Int, delta: Int) -> Int
    balance + delta

@pre(values.len() > 0)
@post(forall i in 0..result.len()-1: result[i] <= result[i+1])
@post(permutation(values, result))
f sort(values: [Int]) -> [Int]
    sort_ints(values)
```

Named patterns: `@nonempty(x)`, `@sorted(x)`, `@unique(x)`, `@permutation(a,b)`,
`@nonnegative(x)`, `@positive(x)`, `@bounded(x,lo,hi)`, `@unchanged(x)`, `@pure`

## Functional Operations

```forma
doubled := map([1, 2, 3], |x: Int| x * 2)           # [2, 4, 6]
evens := filter([1, 2, 3, 4], |x: Int| x % 2 == 0)  # [2, 4]
total := reduce([1, 2, 3], 0, |acc: Int, x: Int| acc + x)  # 6
any([1, 2, 3], |x: Int| x > 2)                       # true
all([1, 2, 3], |x: Int| x > 0)                       # true
vec_sort([3, 1, 2])                                   # [1, 2, 3]
```

## Closures

Closures require typed parameters:
```forma
|x: Int, y: Int| x + y
|x: Int| -> Int x * 2
```

## Async

```forma
as f work() -> Int
    result := aw some_future()
    result

task := sp work()
value := aw task
results := aw await_all(tasks)
```

## Modules & Imports

```forma
us module.path
us module.{A, B}
us std.io                    # stdlib module

md name
    pub f helper() -> Int = 0
```

## Reference Parameters

```forma
f sum(ref arr: [Int]) -> Int         # read-only reference
    total := 0
    for x in arr
        total := total + x
    total

f sort(ref mut arr: [Int]) -> Unit   # mutable reference

# Calling with ref
total := sum(ref data)
sort(ref mut data)
```

## Main Function

```forma
f main()
    # program logic here
    print("hello")

f main() -> Int                      # explicit exit code
    if ok then 0 else 1
```

## CLI Commands

```bash
forma run <file>                        # run program
forma run <file> --allow-all            # all capabilities (untrusted: use least-privilege)
forma run <file> --allow-read --allow-network  # specific capabilities
forma run <file> --no-check-contracts   # disable contract checking
forma run <file> --no-optimize          # disable MIR optimization
forma check <file>                      # type check only
forma check <file> --error-format json  # JSON diagnostic output
forma explain <file> --format json      # contract explanations
forma verify <path> --report            # verify contracts with examples
forma build <file>                      # build native binary (LLVM)
forma fmt <file>                        # format source code
forma grammar --format ebnf|json        # export language grammar
forma new <name>                        # create new project
forma init                              # init project in current dir
forma repl                              # interactive REPL
forma lsp                               # start LSP server
```

Capability flags: `--allow-read`, `--allow-write`, `--allow-network`, `--allow-exec`,
`--allow-env`, `--allow-unsafe`, `--allow-all`

## Key Builtins (Summary)

Full list in [docs/ai-reference.md](../../docs/ai-reference.md).

**I/O:** `print(v)` `eprintln(v)` `str(v)` `debug(v)`
**Math:** `abs(n)` `sqrt(x)` `pow(b,e)` `sin(x)` `cos(x)` `log(x)` `floor(x)` `ceil(x)` `round(x)`
**String:** `str_len(s)` `str_contains(s,sub)` `str_split(s,d)` `str_trim(s)` `str_slice(s,i,j)` `str_replace(s,old,new)` `str_to_int(s)`
**Collection:** `len(c)` `vec_push(v,x)` `vec_pop(v)` `vec_get(v,i)` `map_get(m,k)` `map_insert(m,k,v)` `map_keys(m)`
**Functional:** `map(arr,fn)` `filter(arr,fn)` `reduce(arr,init,fn)` `any(arr,fn)` `all(arr,fn)`
**Option/Result:** `unwrap(v)` `unwrap_or(v,d)` `is_some(v)` `is_none(v)` `is_ok(v)` `is_err(v)` `map_opt(opt,fn)`
**File:** `file_read(p)` `file_write(p,c)` `file_exists(p)` `dir_list(p)`
**JSON:** `json_parse(s)` `json_stringify(v)` `json_get(o,k)` `json_object()` `json_set(o,k,v)`
**HTTP:** `http_get(url)` `http_post(url,body)` `http_serve(port,handler)` `http_response(code,body)`
**DB:** `db_open(path)` `db_execute(db,sql)` `db_query(db,sql)` `db_close(db)`
**Async:** `channel_new()` `channel_send(s,v)` `channel_recv(r)` `mutex_new(v)` `sleep_async(s)` `await_all(tasks)`
**Random:** `random()` `random_int(min,max)` `shuffle(list)`
**Time:** `time_now()` `time_sleep(s)` `time_format(ts,fmt)`
**Regex:** `regex_match(pat,s)` `regex_find_all(pat,s)` `regex_replace(pat,s,r)`

## Stdlib Modules

```forma
us std.core    # clamp, gcd, lcm
us std.io      # file_read_lines, file_write_lines, puts
us std.string  # str_join, str_replace_first, str_index_of
us std.vec     # int_vec_index_of, int_vec_sum, int_vec_max
us std.iter    # range, enumerate
us std.map     # map_get_or, map_update
```

## Common Patterns

### HTTP Server
```forma
f handler(req: HttpRequest) -> HttpResponse
    m req.path
        "/" -> http_response(200, "OK")
        "/json" -> http_json_response(200, json_object())
        _ -> http_response(404, "Not Found")

f main()
    http_serve(8080, handler)!
```

### Database
```forma
f main()
    db := db_open("app.db")!
    db_execute(db, "CREATE TABLE t (id INTEGER, name TEXT)")!
    rows := db_query(db, "SELECT * FROM t")!
    for row in rows
        print(row_get_str(row, 1))
    db_close(db)
```

### Option Chaining
```forma
map_opt(Some(5), |x: Int| x * 2)                                    # Some(10)
and_then(Some(5), |x: Int| if x > 0 then Some(x * 10) else None)   # Some(50)
flatten(Some(Some(42)))                                              # Some(42)
```
