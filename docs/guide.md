# NOVA Language Guide

A comprehensive guide to learning and using the NOVA programming language.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic Syntax](#basic-syntax)
3. [Variables and Types](#variables-and-types)
4. [Control Flow](#control-flow)
5. [Functions](#functions)
6. [Structs, Enums, and Patterns](#structs-enums-and-patterns)
7. [Error Handling](#error-handling)
8. [Concurrency](#concurrency)
9. [Async/Await](#asyncawait)
10. [Modules and Packages](#modules-and-packages)
11. [Generics and Traits](#generics-and-traits)
12. [Advanced Features](#advanced-features)

---

## Getting Started

### Installation

```bash
# Build from source (works everywhere)
git clone https://github.com/samprit-web/NOVA.git
cd NOVA
cargo build --workspace --release

# Run NOVA
./target/release/nova --version
```

### Your First Program

```nova
// hello.nova
fn main() {
    println!("Hello, NOVA! 🚀")
}
```

```bash
./target/release/nova run hello.nova
# Output: Hello, NOVA! 🚀
```

### Creating a Project

```bash
./target/release/nova new my-project
cd my-project
./target/release/nova run
```

---

## Basic Syntax

### Comments

```nova
// Single line comment

/*
  Multi-line
  comment
*/

/// Documentation comment
/// for public APIs
```

### Basic Types

```nova
// Integers
let a: i8 = 127
let b: i32 = 42
let c: i64 = 1000000
let d: u32 = 42u32  // unsigned

// Floats
let pi: f32 = 3.14
let e: f64 = 2.718281828

// Boolean
let yes = true
let no = false

// Character
let ch = '🚀'

// String
let name = "NOVA"
let raw = r#"Raw string with "quotes""#
```

---

## Variables and Types

### Immutable by Default

```nova
let x = 10      // immutable
let mut y = 20  // mutable
y = 30          // OK
// x = 20       // Error: cannot assign twice
```

### Type Inference

```nova
let a = 42          // i32 (default integer)
let b = 3.14        // f64 (default float)
let c = "hello"     // &str
let d = true        // bool
let e = 'c'         // char
```

### Explicit Types

```nova
let x: i64 = 100
let y: f32 = 2.5
let z: &str = "explicit"
```

### Constants

```nova
const PI: f64 = 3.141592653589793
const MAX_SIZE: usize = 1024
const APP_NAME: &str = "MyApp"
```

### Shadowing

```nova
let x = 5
let x = x + 1      // shadows previous x
let x = "hello"    // can change type!
```

---

## Control Flow

### If Expressions

```nova
// If as expression (returns value)
let result = if x > 0 { "positive" } else { "non-positive" }

// Traditional if/else
if x > 0 {
    println!("Positive")
} else if x < 0 {
    println!("Negative")
} else {
    println!("Zero")
}

// Pattern matching with if let
if let Some(value) = optional {
    println!("Got: {}", value)
}
```

### Loops

```nova
// Infinite loop
loop {
    if condition { break }
}

// While loop
while x < 10 {
    x += 1
}

// For loops over ranges
for i in 1..=10 {      // inclusive 1 to 10
    println!("{}", i)
}

for i in 0..5 {        // exclusive 0 to 4
    println!("{}", i)
}

// For over collections
let items = vec![1, 2, 3]
for item in &items {
    println!("{}", item)
}

// With enumerate
for (i, item) in items.iter().enumerate() {
    println!("{}: {}", i, item)
}
```

### Break and Continue

```nova
for i in 1..=100 {
    if i == 5 { continue }   // skip 5
    if i == 10 { break }     // stop at 10
    println!("{}", i)
}
```

---

## Functions

### Basic Functions

```nova
fn add(a: i32, b: i32) -> i32 {
    a + b  // implicit return (no semicolon)
}

fn greet(name: &str) {
    println!("Hello, {}!", name)
}
```

### Multiple Returns (Tuples)

```nova
fn min_max(numbers: &[i32]) -> (i32, i32) {
    let mut min = numbers[0]
    let mut max = numbers[0]
    for &n in numbers {
        if n < min { min = n }
        if n > max { max = n }
    }
    (min, max)
}
```

### Generic Functions

```nova
fn identity<T>(value: T) -> T {
    value
}

fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
```

### Closures

```nova
let add_one = |x: i32| x + 1
let add = |a: i32, b: i32| a + b

// Capturing environment
let factor = 10
let scale = |x: i32| x * factor

// Move closure
let data = vec![1, 2, 3]
let consume = move || println!("{:?}", data)
```

---

## Structs, Enums, and Patterns

### Structs

```nova
struct Point {
    x: f64,
    y: f64,
}

struct Color(u8, u8, u8);  // tuple struct
struct Unit;               // unit struct

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
    
    fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x
        let dy = self.y - other.y
        (dx * dx + dy * dy).sqrt()
    }
}
```

### Enums

```nova
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
    Point,
}
```

### Pattern Matching

```nova
match shape {
    Shape::Circle(r) => println!("Circle radius: {}", r),
    Shape::Rectangle(w, h) => println!("Rect: {}x{}", w, h),
    Shape::Point => println!("Point"),
}

match optional {
    Some(x) => println!("Got: {}", x),
    None => println!("Nothing"),
}

// Destructuring
let Point { x, y } = point
let (a, b, c) = tuple
let Color(r, g, b) = color
```

---

## Error Handling

### Option Type

```nova
fn find_user(id: u64) -> Option<User> {
    if id == 0 { None } else { Some(User::new(id)) }
}

// Safe handling
let user = find_user(42)
match user {
    Some(u) => println!("Found: {}", u.name),
    None => println!("Not found"),
}

// Combinators
let name = user.map(|u| u.name).unwrap_or("Unknown")
```

### Result Type

```nova
fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

// ? operator for propagation
fn process(path: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let number = content.trim().parse::<i32>()?;
    Ok(number)
}
```

### Custom Errors

```nova
#[derive(Debug)]
enum AppError {
    NotFound,
    InvalidInput(String),
    Io(std::io::Error),
}

impl std::fmt::Display for AppError { ... }
impl std::error::Error for AppError { ... }
```

---

## Concurrency

### Threads

```nova
use std::thread
use std::time::Duration

let handle = thread::spawn(|| {
    for i in 1..=5 {
        println!("Thread: {}", i)
        thread::sleep(Duration::from_millis(100))
    }
})

handle.join().unwrap()
```

### Mutex and Arc

```nova
use std::sync::{Arc, Mutex}

let counter = Arc::new(Mutex::new(0))
let mut handles = vec![]

for _ in 0..10 {
    let counter = Arc::clone(&counter)
    handles.push(thread::spawn(move || {
        let mut num = counter.lock().unwrap()
        *num += 1
    }))
}

for h in handles { h.join().unwrap() }
println!("Result: {}", *counter.lock().unwrap())
```

### Channels

```nova
use std::sync::mpsc

let (tx, rx) = mpsc::channel()

thread::spawn(move || {
    tx.send(42).unwrap()
})

println!("Received: {}", rx.recv().unwrap())
```

---

## Async/Await

### Basic Async

```nova
async fn fetch_data() -> String {
    // Simulated async work
    std::task::sleep(std::time::Duration::from_millis(100)).await
    "data".to_string()
}

fn main() {
    let result = std::task::block_on(async {
        fetch_data().await
    })
    println!("{}", result)
}
```

### Concurrent Execution

```nova
async fn fetch_all() {
    // Sequential
    let a = fetch(1).await
    let b = fetch(2).await
    
    // Concurrent with join!
    let (c, d) = std::task::join(fetch(3), fetch(4)).await
}
```

### Timeout

```nova
let result = std::task::block_on(async {
    std::task::timeout(Duration::from_millis(100), slow_operation()).await
})
```

---

## Modules and Packages

### Module System

```nova
// Inline module
mod math {
    pub fn add(a: i32, b: i32) -> i32 { a + b }
    pub mod constants {
        pub const PI: f64 = 3.14159
    }
}

// Using modules
use math::add
use math::constants::PI

// File-based modules
// mod utils;  // loads utils.nova or utils/mod.nova
```

### Package Manifest (nova.toml)

```toml
[package]
name = "my-app"
version = "1.0.0"
edition = "2026"

[dependencies]
serde = "1.0"
tokio = "1.0"
```

---

## Generics and Traits

### Generic Structs

```nova
struct Container<T> {
    value: T,
}

impl<T> Container<T> {
    fn new(value: T) -> Self {
        Container { value }
    }
}
```

### Traits

```nova
trait Drawable {
    fn draw(&self) -> String;
    fn area(&self) -> f64;
}

struct Circle { radius: f64 }
struct Rectangle { width: f64, height: f64 }

impl Drawable for Circle {
    fn draw(&self) -> String { format!("Circle(r={})", self.radius) }
    fn area(&self) -> f64 { std::f64::consts::PI * self.radius * self.radius }
}

impl Drawable for Rectangle {
    fn draw(&self) -> String { format!("Rect({}x{})", self.width, self.height) }
    fn area(&self) -> f64 { self.width * self.height }
}
```

### Trait Objects

```nova
fn draw_shape(shape: &dyn Drawable) {
    println!("{}", shape.draw())
}
```

---

## Advanced Features

### Macros

```nova
macro_rules! vec_of {
    ($($x:expr),*) => { vec![$($x),*] };
}

let v = vec_of![1, 2, 3, 4, 5]
```

### SIMD

```nova
use std::simd::f32x4

let a = f32x4::from_array([1.0, 2.0, 3.0, 4.0])
let b = f32x4::from_array([5.0, 6.0, 7.0, 8.0])
let c = a + b  // vectorized!
```

### FFI (C Interop)

```nova
extern "C" {
    fn abs(input: i32) -> i32;
}

let result = unsafe { abs(-42) }  // 42
```

### Actor Model

```nova
actor Counter {
    count: i64 = 0
    fn increment(&mut self) -> i64 { self.count += 1; self.count }
}

let counter = Counter::new()
let val = counter.increment().await
```

---

## Next Steps

- Read the [Standard Library Documentation](std.md)
- Check the [Language Reference](reference.md)
- Explore [Examples](../examples/comprehensive/)
- Join the [Community](https://github.com/samprit-web/NOVA/discussions)