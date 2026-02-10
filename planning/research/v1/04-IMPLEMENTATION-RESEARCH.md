# Practical Language Implementation Research

## Executive Summary

This document covers practical aspects of implementing a new programming language: compiler backends, incremental compilation, IDE integration, toolchain design, FFI, debugging, and self-hosting strategies.

**Key Finding:** Design for incrementality from day one. Consider using a unified toolchain with a single entry point, and plan self-hosting but don't rush it.

---

## 1. Compiler Backends: LLVM vs Cranelift vs Custom

### LLVM

**Pros:**
- Mature, battle-tested with extensive optimizations
- Supports 15+ target architectures
- Rich ecosystem of tools (sanitizers, profilers, debuggers)
- Excellent for release/production builds

**Cons:**
- Over 20 million lines of code—massive dependency
- Slow compilation times, especially for JIT scenarios
- Complex to integrate and maintain
- LLVM IR is not portable or stable across versions

### Cranelift

**Pros:**
- ~200,000 lines of code (100x smaller than LLVM)
- Compiles an order of magnitude faster for code generation
- Written in Rust with memory safety focus
- Runtime performance only ~2-14% slower than LLVM

**Cons:**
- Fewer target architectures (x86-64, AArch64, RISC-V, s390x)
- Less aggressive optimizations
- IR at lower abstraction level

### Custom Backend

**When to Consider:**
- Zig's approach: self-hosted x86 backend reduced compilation from 22.8s to 275ms
- Enables incremental compilation with in-place binary patching
- Full control over dependency tracking

**Recommendation:** Start with LLVM, add Cranelift for debug builds, consider custom backend once mature.

---

## 2. Incremental Compilation Strategies

### Query-Based Architecture (Salsa/Rust)

- Define compilation as pure queries: `K -> V` mappings
- Results memoized and recomputed only when inputs change
- "Red-green algorithm" tracks valid cached results
- Early cutoff: if result unchanged despite input changes, dependents skip recomputation

### Dependency Tracking

- Build Dependency DAG indexing query relationships
- Use hash sums for API representations to detect changes
- Implement durability levels: stdlib queries change less than user code

### In-Place Binary Patching (Zig's Approach)

- Generate position-independent code
- Use Global Offset Table for all function calls
- Patch binaries directly instead of rebuilding
- Enables sub-millisecond incremental rebuilds

### Implementation Challenges

- Correctly tracking cross-module dependencies
- Cache invalidation complexity
- Memory overhead for dependency storage
- Retrofitting incrementality is harder than designing it in

**Recommendation:** Design for incrementality from day one. Study the Salsa library as a reference.

---

## 3. IDE/LSP Integration

### Language Server Protocol (LSP) Overview

JSON-RPC protocol enabling:
- Auto-complete
- Go to definition
- Find all references
- Hover information
- Diagnostics

### Implementation Strategy

**Capabilities System:**
- Not every server needs all features
- Exchange capabilities during initialization
- Start with essentials: diagnostics, go-to-definition, completion

**Shared Compilation Infrastructure:**
- rust-analyzer demonstrates best practices
- Share parsing and type-checking between compiler and language server
- Use same query-based incremental system
- IDE operations need sub-100ms latency

**Essential Features to Implement First:**
1. `textDocument/didOpen`, `didChange`, `didClose` - document synchronization
2. `textDocument/publishDiagnostics` - error reporting
3. `textDocument/completion` - auto-complete
4. `textDocument/definition` - go-to-definition
5. `textDocument/hover` - type information

---

## 4. Toolchain Design

### Components of a Modern Toolchain

1. **Compiler** - generates executable code
2. **Linker** - links object files
3. **Build System** - orchestrates compilation
4. **Package Manager** - handles dependencies
5. **Formatter** - enforces code style
6. **Debugger** - runtime debugging
7. **Runtime Libraries** - OS interface

### Design Philosophy: Unified vs Modular

**Unified Approach (Cargo, Go toolchain):**
- Single tool provides compilation, testing, dependency management
- Consistent user experience
- Easier to develop and maintain

**Modular Approach (C/C++ ecosystem):**
- Flexibility to swap components
- Works with existing tools
- More complex integration

### Package Manager Considerations

- Semantic versioning for dependency resolution
- Lock files for reproducible builds
- Registry/repository infrastructure
- Build caching and artifact sharing
- Cross-platform support

### Formatter Design

- Should be opinionated (like `gofmt`)—one canonical style
- Fast enough to run on save
- Integrate with LSP for format-on-save
- Consider automatic import organization

**Recommendation:** Build a unified toolchain driver from the start.

---

## 5. FFI Design for C Interop

### Core Challenges

1. **ABI Compatibility** - C has a stable ABI
2. **Memory Management** - Coordinating GC and manual memory
3. **Type Mapping** - Converting between type systems
4. **String Handling** - C uses null-terminated strings
5. **Calling Conventions** - Platform-specific (cdecl, stdcall, etc.)

### Design Patterns

**Explicit Foreign Declarations:**
```
extern "C" fn malloc(size: usize) -> *void;
```

**Safe Wrappers:**
- Encapsulate unsafe FFI code within safe abstractions
- Validate inputs/outputs at boundaries
- Convert foreign error codes to native error types

**Automated Binding Generation:**
- Parse C headers to generate bindings
- Tools like SWIG, bindgen (Rust), ffigen (Dart)

### Memory Safety Considerations

- Clear ownership semantics at FFI boundaries
- Handle lifetime mismatches
- Provide utilities for C string conversion
- Consider arena allocators for FFI-heavy code

**Recommendation:** Design FFI to be explicit. Make boundary crossing visible.

---

## 6. Debug Information and Tooling

### DWARF Format

Standard debugging format for ELF binaries:
- **DIE (Debugging Information Entry)** - basic entity with tags and attributes
- **Key sections:** `.debug_info`, `.debug_line`, `.debug_frame`
- Tracks source locations, variable types, scope information

### Implementation Requirements

**Line Number Information:**
- Map machine addresses to source file/line numbers
- Implement DWARF line number state machine
- Store in `.debug_line` section

**Type Information:**
- Describe all types in your language
- Map to DWARF base types or define custom structures
- Enable debuggers to display variable values

**Variable Location:**
- Track where variables live (registers, stack, memory)
- Handle variables that move during execution
- Support optimized code scenarios

### Practical Steps

1. Generate DWARF info with `-g` equivalent flag
2. Use `llvm-dwarfdump` to verify output
3. Test with GDB/LLDB early and often
4. Consider libdwarf for DWARF generation

---

## 7. Compile Time vs Runtime Trade-offs

### The Tradeoff Triangle

Cannot optimize all three simultaneously:
- **Runtime performance** - how fast generated code runs
- **Compile time** - how fast the compiler runs
- **Binary size** - how large the executable is

### Compile-Time Techniques

- **Constant Folding:** `3 + 4` becomes `7`
- **Dead Code Elimination:** Remove code with no effect
- **Monomorphization:** Generate specialized code for each type (better runtime, larger binaries, longer compile)

### Runtime Techniques

- **JIT Compilation:** Defer optimization to runtime with profiling
- **Dynamic Dispatch:** Smaller code, slower runtime due to indirect calls

### Practical Guidance

- Offer optimization levels (debug vs release)
- Debug: minimal optimization, fast compilation
- Release: aggressive optimization, accept slower compilation
- Consider profile-guided optimization (PGO) for maximum performance

---

## 8. Fast Compilation Techniques

### Go's Approach

1. **No cyclic dependencies** - enables DAG-based parallel compilation
2. **Efficient imports** - imported package contains all transitive information
3. **No symbol table** - language parseable without one
4. **Simple compiler** - fewer optimization passes
5. **Unused imports are errors** - no wasted compilation

**Export Data:**
- Compiler writes type info, IR for inlining, escape analysis
- Downstream packages read only the summary

### Zig's Approach

**Self-Hosted Custom Backend:**
- Bypasses LLVM for debug builds
- x86 backend: 22.8s → 275ms for hello world
- For compiler itself: 75s → 20s

**Incremental Compilation:**
- In-place binary patching
- Position-independent code with Global Offset Table
- Full control over dependency tracking

**Memory Efficiency:**
- 3x reduction vs bootstrap compiler
- Can build on 32-bit systems

### Common Patterns

1. Avoid header/include mechanisms that re-parse code
2. Use module systems with precompiled interface files
3. Parallelize at function/module level
4. Minimize inter-procedural analysis in debug builds
5. Cache intermediate representations

---

## 9. WebAssembly as a Compilation Target

### Design Goals

- **Portability** - same ISA for every machine
- **Stability** - binary format doesn't change
- **Small encoding** - compact for network transmission
- **Memory safety** - sandboxed execution
- **Near-native performance** - leverages common hardware

### Why Not LLVM IR?

LLVM IR is unsuitable as a portable target:
- IR is architecture-specific
- IR changes between LLVM versions
- Optimized for compiler development, not portability

### Implementation Considerations

**Stack Machine Model:**
- Wasm uses stack-based VM
- Different from register-based IRs

**Memory Model:**
- Linear memory with bounds checking
- No direct memory access outside sandbox
- Wasm 3.0: 64-bit address space, multiple memories

**Garbage Collection:**
- Wasm 3.0 added GC support
- Enables efficient compilation for managed languages

### Languages Targeting Wasm

- **Native support:** Rust, C/C++, Go, Zig
- **Via compilation:** C# (Blazor), Python (Pyodide), AssemblyScript

**Recommendation:** WebAssembly is excellent as secondary target. Use existing backends.

---

## 10. Self-Hosting Strategies

### Bootstrap Strategies

**Strategy 1: Write in Another Language First**
- Implement initial compiler in existing language (Rust was first in OCaml)
- Rewrite in target language once functional
- Compile new compiler with old one

**Strategy 2: Subset Bootstrap**
- Write compiler in subset of your language
- Use existing compiler for that subset
- Gradually expand features

**Strategy 3: Cross-Compilation Chain**
- Use cross-compilers to generate initial native code
- Build up capability iteratively

### Multi-Stage Bootstrap Process

- **Stage 0:** Prepare environment for bootstrap compiler
- **Stage 1:** Bootstrap compiles itself (basic executable)
- **Stage 2:** Full compiler from bootstrap (can compile new features but doesn't use them)
- **Stage 3:** Full compiler compiled by Stage 2 (uses all features, production quality)

### Verification

- Compare Stage 2 and Stage 3 outputs
- If different, bug exists in bootstrap or full compiler
- Regression testing ensures compiler can compile itself

### Benefits of Self-Hosting

1. **Dogfooding** - non-trivial test of the language
2. **Single language** - developers only need target language
3. **Improvements compound** - better backend improves both programs and compiler

### Practical Timeline

1. Start with bootstrap compiler in stable language (Rust, Go, OCaml)
2. Implement core language features
3. Begin rewrite once parsing and type-checking work
4. Achieve self-hosting before 1.0 release
5. Retire bootstrap compiler after self-hosted version is stable

---

## Summary Recommendations

1. **Start with LLVM**, add Cranelift for debug builds
2. **Design for incrementality from day one** using query-based architecture
3. **Build LSP support early** - share infrastructure with compiler
4. **Create unified toolchain** with single entry point
5. **Design explicit FFI** with safe wrappers
6. **Generate DWARF debug info** and test with debuggers
7. **Offer multiple optimization levels**
8. **Study Go and Zig** for fast compilation techniques
9. **WebAssembly is good secondary target**
10. **Plan self-hosting bootstrap** but prioritize stability
