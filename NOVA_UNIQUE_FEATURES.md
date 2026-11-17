# NOVA Unique Features Specification

## Core Differentiators from Rust

### 1. **Effect System Instead of Result/Option Everywhere**
- Algebraic effects as first-class citizens
- No more `Result<T, E>` boilerplate
- Automatic error propagation with `?` but more powerful
- Effect handlers for cross-cutting concerns (logging, transactions, retries)

### 2. **Gradual Type System**
- Start with dynamic typing (like Python)
- Add type annotations incrementally
- Compile-time verification when types are present
- No "fighting the borrow checker" for beginners

### 3. **First-Class Actor Model**
- Actors as language primitives (not library)
- Automatic message serialization
- Distributed actors across machines
- Built-in supervision trees

### 4. **Compile-Time Execution (Comptime)**
- Run any function at compile time
- Generate code, validate configs, compute constants
- No separate `const fn` / `proc_macro` distinction

### 5. **Universal Deployment Targets**
Single compilation → Multiple targets:
- Native (x86_64, ARM64, RISC-V)
- WebAssembly (browser, WASI)
- JVM bytecode
- .NET IL
- Bytecode VM (embedded)
- C transpilation

### 6. **Structured Concurrency Built-In**
- No `async`/`await` color problem
- Scoped tasks with automatic cleanup
- Cancellation propagation
- No `Send`/`Sync` complexity

### 7. **Hot Reload & REPL**
- Sub-second hot reload in development
- Full REPL with type inspection
- Time-travel debugging

### 8. **Syntax: Concise but Explicit**
```nova
// Rust: 8 lines
fn process(items: Vec<Item>) -> Result<Vec<Output>, Error> {
    items.into_iter()
        .map(|item| transform(item))
        .collect()
}

// NOVA: 2 lines
fn process(items: [Item]) -> [Output] {
    items.map(transform)
}
```

### 9. **Built-in Package Manager with Decentralized Registry**
- No central crates.io required
- Git-based dependencies
- Content-addressed storage
- Offline-first

### 10. **Algebraic Data Types with Pattern Matching Exhaustiveness**
- More powerful than Rust enums
- GADTs, refinement types
- Compile-time exhaustiveness checking

---

## Implementation Priority

### Phase 1: Syntax & Type System (High Impact)
1. Effect system design
2. Gradual typing
3. Concise syntax sugar
4. Comptime execution

### Phase 2: Concurrency & Runtime (High Impact)
1. Actor primitives
2. Structured concurrency
2. Hot reload runtime

### Phase 3: Deployment & Tooling (Medium Impact)
1. Multi-target compilation
2. REPL implementation
3. Package manager

### Phase 4: Advanced Features (Differentiation)
1. Dependent types
2. Linear types
3. JIT specialization
4. Distributed actors

---

## Example: NOVA vs Rust

### Error Handling
```rust
// Rust
fn read_config(path: &str) -> Result<Config, Error> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
```

```nova
// NOVA - Effect system
effect Io { fn read(path: str) -> str }
effect Parse { fn parse<T>(s: str) -> T }

fn read_config(path: str) -> Config with Io, Parse {
    let content = Io::read(path)
    Parse::parse(content)
}

// Handler can be swapped: file, network, test mock
```

### Concurrency
```rust
// Rust - complex async
async fn handle_client(stream: TcpStream) -> Result<()> {
    let mut buf = vec![0; 1024];
    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 { break; }
        stream.write_all(&buf[..n]).await?;
    }
    Ok(())
}
```

```nova
// NOVA - Actor model
actor TcpConnection {
    stream: TcpStream
    
    fn handle(mut self) {
        loop {
            match self.stream.read() {
                Ok(data) if data.is_empty() => break,
                Ok(data) => self.stream.write(data).await,
                Err(_) => break,
            }
        }
    }
}

// Spawn: let conn = TcpConnection::new(stream); conn.handle().spawn()
```

### Generics Without Boilerplate
```rust
// Rust
fn map<T, U, F>(items: Vec<T>, f: F) -> Vec<U> 
where F: Fn(T) -> U {
    items.into_iter().map(f).collect()
}
```

```nova
// NOVA - Type inference + effects
fn map<T, U>(items: [T], f: fn(T) -> U) -> [U] {
    items.map(f)
}

// Even simpler with pipelines
fn process(data: [Input]) -> [Output] {
    data.filter(valid).map(transform).sort()
}
```

---

## Unique Standard Library Design

### 1. **Pipeline Operators**
```nova
data
    .filter(|x| x > 0)
    .map(|x| x * 2)
    .collect()
```

### 2. **Unified Collections**
```nova
// Single interface for all sequences
trait Sequence<T> {
    fn map<U>(self, f: fn(T) -> U) -> Self::Mapped<U>
    fn filter(self, f: fn(T) -> bool) -> Self
    fn fold<U>(self, init: U, f: fn(U, T) -> U) -> U
}

// Implemented for: Array, List, Set, Map, Stream, Channel, AsyncStream
```

### 3. **Built-in Serialization**
```nova
@serialize
struct User {
    id: u64,
    name: str,
    email: str,
}

// Automatically supports: JSON, TOML, YAML, MessagePack, Bincode, CBOR
let json = user.to_json()
let user = User::from_json(json)
```

### 4. **Testing as First-Class**
```nova
fn add(a: i32, b: i32) -> i32 { a + b }

test add {
    assert_eq(add(2, 3), 5)
    assert_eq(add(-1, 1), 0)
    property(a: i32, b: i32) { add(a, b) == add(b, a) }
}
```

---

## Compiler Architecture for Uniqueness

### 1. **Multi-Stage Compilation**
```
Source → AST → HIR (with effects) → MIR → 
    ├─ Native (LLVM/Cranelift)
    ├─ WASM
    ├─ JVM bytecode
    ├─ .NET IL
    ├─ C transpile
    └─ Bytecode VM
```

### 2. **Incremental Compilation with Hot Reload**
- Module-level incremental compilation
- Function-level hot swap in running process
- State preservation across reloads

### 3. **JIT Specialization**
- Runtime profiling → specialized machine code
- Polymorphic inline caching
- Trace-based compilation

### 4. **Time-Travel Debugging**
- Record execution trace
- Step backward/forward
- Inspect any historical state

---

## Package Manager: `nova pkg`

### Decentralized, Git-Based
```toml
# nova.toml
[package]
name = "my-app"
version = "1.0.0"

[dependencies]
# Git URLs with content addressing
utils = { git = "https://github.com/user/utils.git", rev = "abc123" }
# Local path
core = { path = "../core" }
# Registry (optional)
serde = "1.0"
```

### Features
- **Content-addressed**: Immutable packages, no version conflicts
- **Offline-first**: Full local cache
- **Parallel resolution**: Sub-second dependency resolution
- **Build isolation**: Hermetic builds
- **Cross-compilation**: `nova build --target wasm32`

---

## Language Server Protocol (LSP) Features

### Beyond Standard LSP
- **Type-driven completions**: Suggest based on expected type
- **Effect inference**: Show required effects inline
- **Performance hints**: Show allocation hotspots
- **Concurrency visualization**: Actor message flows
- **Time-travel debugging**: Step through history
- **Hot reload status**: Live indicator in editor

---

## Benchmarks Target

| Metric | Target |
|--------|--------|
| Compile time (10k LOC) | < 2s incremental |
| Hot reload latency | < 100ms |
| Binary size (hello world) | < 500KB |
| Startup time | < 5ms |
| Memory overhead (vs C) | < 10% |
| Throughput (web server) | > 1M req/s |
| SIMD auto-vectorization | > 80% loops |

---

## Migration Path from Rust

### Compatibility Layer
```nova
// Import Rust crates directly
extern crate serde;
extern crate tokio;

// Use Rust types
type RustString = rust::std::string::String;
```

### Automated Translation Tool
```bash
nova translate --from-rust src/ --to-nova src/
```

---

## Success Metrics

1. **Developer Experience**: 50% less code than Rust for same task
2. **Learning Curve**: Productive in 1 day (vs 2-4 weeks for Rust)
3. **Performance**: Within 5% of hand-optimized C
4. **Adoption**: 100k developers in 2 years
5. **Ecosystem**: 5000 packages in registry

---

## Conclusion

NOVA is not "Rust with better syntax" — it's a **fundamentally different approach** to systems programming:

1. **Effects over monads** for error handling
2. **Actors over async/await** for concurrency
3. **Comptime over macros** for metaprogramming
4. **Gradual typing over mandatory types** for accessibility
5. **Universal targets over platform-specific** for deployment

This makes NOVA the first language that is truly **accessible to beginners** yet **powerful enough for experts** — the best of both worlds.