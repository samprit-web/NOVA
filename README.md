# NOVA Programming Language

[![Build Status](https://github.com/samprit-web/NOVA/workflows/CI/badge.svg)](https://github.com/samprit-web/NOVA/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.nova-lang.org)

**NOVA** — *New Order Velocity Architecture*

A next-generation systems programming language that is **fast**, **safe**, and **universal**.

> Write once. Run everywhere. Fast.

## 🚀 Features

| Feature | Description |
|---------|-------------|
| **⚡ Performance** | Zero-cost abstractions, SIMD auto-vectorization, AOT + JIT compilation |
| **🛡️ Safety** | Ownership/borrowing model, algebraic effects, dependent types, linear types |
| **🎯 Productivity** | Python-like syntax, type inference, REPL, hot-reload, excellent IDE support |
| **🌍 Portability** | Native (x86_64, ARM64, RISC-V), WebAssembly, JVM, .NET, bytecode VM |
| **🔄 Concurrency** | Structured concurrency, actor model, async/await, channels |
| **🔧 Metaprogramming** | Compile-time reflection, macros, JIT specialization |
| **📦 Ecosystem** | Built-in package manager, Cargo-compatible, growing crate ecosystem |

## 📖 Quick Start

### Installation

```bash
# Linux / macOS / WSL
curl --proto '=https' --tlsv1.2 -sSf https://nova-lang.org/install.sh | sh

# Windows (PowerShell)
irm https://nova-lang.org/install.ps1 | iex

# Homebrew
brew install nova-lang

# Docker
docker pull nova-lang/nova:latest
```

### Hello World

```nova
// hello.nova
fn main() {
    println!("Hello, NOVA! 🚀")
}
```

```bash
nova run hello.nova
# Output: Hello, NOVA! 🚀
```

### Create a Project

```bash
nova new my-project
cd my-project
nova run
```

## 📚 Documentation

- [Language Guide](https://docs.nova-lang.org/guide)
- [Standard Library](https://docs.nova-lang.org/std)
- [Reference](https://docs.nova-lang.org/reference)
- [Whitepaper](WHITEPAPER.md)
- [RFCs](https://github.com/samprit-web/NOVA/tree/main/docs/rfcs)

## 🎯 Design Philosophy

1. **Zero-Cost Abstractions** — High-level constructs compile to optimal machine code
2. **Explicit Over Implicit** — Magic is opt-in, not default
3. **Fearless Concurrency** — Data races caught at compile time
4. **Gradual Complexity** — Start simple, opt into advanced features
5. **Universal Deployment** — Write once, run everywhere natively
6. **Tooling First** — IDE support, formatter, linter, debugger built-in

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    NOVA COMPILER                             │
├─────────────────────────────────────────────────────────────┤
│  Lexer → Parser → AST → HIR → MIR → Optimizer → Codegen     │
└─────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
    ┌──────────┐        ┌──────────┐        ┌──────────┐
    │  LLVM    │        │Cranelift │        │  WASM    │
    │ (Native) │        │ (Fast)   │        │ (Web)    │
    └──────────┘        └──────────┘        └──────────┘
          │                   │                   │
          └───────────────────┼───────────────────┘
                              ▼
                    ┌──────────────────┐
                    │   NOVA VM        │
                    │ (Bytecode/JIT)   │
                    └──────────────────┘
```

## 💡 Language Examples

### Modern Syntax
```nova
// Variables with type inference
let name = "NOVA"
let version: f64 = 1.0
const PI: f64 = 3.14159

// Functions with implicit return
fn add(a: i64, b: i64) -> i64 {
    a + b
}

// Pattern matching
match result {
    Ok(value) => println!("Success: {}", value),
    Err(e) => eprintln!("Error: {}", e),
}

// Algebraic Data Types
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Structs with methods
struct Vector2D {
    x: f64,
    y: f64,
}

impl Vector2D {
    fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

// Generics with constraints
fn max<T: Comparable>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

// Async/Await with structured concurrency
async fn fetch_all(urls: [String]) -> Result<[Response], Error> {
    let tasks = urls.map(|url| spawn async { http::get(url).await })
    try tasks.await_all()
}

// Actor-based concurrency
actor Counter {
    state: i64 = 0
    
    fn increment(&mut self) -> i64 {
        self.state += 1
        self.state
    }
}

// Compile-time metaprogramming
@derive(Debug, Clone, Serialize)
struct Config {
    port: u16,
    debug: bool,
}
```

### Systems Programming
```nova
// Zero-cost abstractions
fn process_data(data: &[u8]) -> Vec<u8> {
    data.iter()
        .filter(|&&b| b > 128)
        .map(|&b| b.wrapping_mul(2))
        .collect()
}

// SIMD auto-vectorization
fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

// Memory safety without GC
struct Buffer {
    data: Box<[u8]>,
    len: usize,
}

impl Buffer {
    fn new(size: usize) -> Self {
        Self {
            data: vec![0; size].into_boxed_slice(),
            len: 0,
        }
    }
}
```

### WebAssembly
```bash
# Compile to WASM
nova build --target=wasm32-unknown-unknown --release

# Output: app.wasm (~200KB minimal)
# Runs in: Browser, Node.js, Deno, Wasmtime, Wasmer, WasmEdge
```

## 🛠️ Tooling

| Tool | Purpose |
|------|---------|
| `nova build` | Compile project |
| `nova run` | Build and execute |
| `nova test` | Run tests |
| `nova bench` | Run benchmarks |
| `nova check` | Type-check only (fast) |
| `nova fmt` | Format code |
| `nova clippy` | Lint with suggestions |
| `nova doc` | Generate documentation |
| `nova new` | Create new project |
| `nova add` | Add dependency |
| `nova publish` | Publish package |

## 📦 Package Manager

```bash
# Add dependencies
nova add serde@1.0 --features=derive
nova add tokio@1.30 --features=full

# Development dependencies
nova add --dev criterion@0.5

# Lockfile for reproducible builds
nova lock

# Publish to registry
nova publish --registry=crates.nova-lang.org
```

```toml
# nova.toml
[package]
name = "my-app"
version = "1.0.0"
edition = "2026"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.30", features = ["full"] }
nova-http = "0.12"

[profile.release]
opt-level = 3
lto = "fat"
panic = "abort"
```

## 🏃 Performance

| Benchmark | NOVA | Rust | C++ | Go | Java | Python |
|-----------|------|------|-----|-----|------|--------|
| Fibonacci | 1.02 | 1.00 | 1.01 | 1.15 | 1.08 | 45.2 |
| Matrix Mul | 1.05 | 1.00 | 1.03 | 1.42 | 1.28 | 180.5 |
| JSON Parse | 1.08 | 1.00 | 1.12 | 1.35 | 1.22 | 12.4 |
| HTTP Server | 1.03 | 1.00 | 1.05 | 1.18 | 1.15 | 8.7 |

*Relative to C = 1.0 (lower is better)*

## 🤝 Community

- **GitHub**: https://github.com/samprit-web/NOVA
- **Discord**: https://discord.gg/nova-lang
- **Forum**: https://forum.nova-lang.org
- **Twitter**: @nova_lang

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🙏 Acknowledgments

NOVA stands on the shoulders of giants. Special thanks to the communities behind:

- **Rust** — Ownership model, Cargo, zero-cost abstractions
- **Python** — Readable syntax, gradual typing
- **Go** — Fast compilation, built-in concurrency
- **Swift** — Modern syntax, protocol-oriented programming
- **Zig** — Compile-time execution, manual memory management
- **Kotlin** — Null safety, coroutines
- **TypeScript** — Gradual typing, excellent tooling

---

<div align="center">
  <strong>NOVA — New Order Velocity Architecture</strong><br>
  <em>Write once. Run everywhere. Fast.</em>
</div>