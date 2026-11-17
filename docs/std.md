# NOVA Standard Library Reference

Complete API documentation for the NOVA standard library (`nova-std`).

## Table of Contents

1. [Core Types](#core-types)
2. [Collections](#collections)
3. [Strings](#strings)
4. [Option and Result](#option-and-result)
5. [I/O](#io)
6. [Concurrency Primitives](#concurrency-primitives)
7. [Time](#time)
8. [Math](#math)
9. [File System](#file-system)
10. [Networking](#networking)
11. [Serialization](#serialization)
12. [Testing](#testing)

---

## Core Types

### Primitive Types

| Type | Description |
|------|-------------|
| `i8, i16, i32, i64, i128, isize` | Signed integers |
| `u8, u16, u32, u64, u128, usize` | Unsigned integers |
| `f32, f64` | IEEE 754 floats |
| `bool` | Boolean (true/false) |
| `char` | Unicode scalar value |
| `&str` | String slice |
| `()` | Unit type |

### Smart Pointers

```nova
use std::boxed::Box
use std::rc::Rc
use std::sync::Arc

// Box - heap allocation
let boxed = Box::new(42)

// Rc - reference counted (single-threaded)
let rc = Rc::new(vec![1, 2, 3])
let rc2 = Rc::clone(&rc)

// Arc - atomic reference counted (thread-safe)
let arc = Arc::new(Mutex::new(0))
let arc2 = Arc::clone(&arc)
```

### Cell Types (Interior Mutability)

```nova
use std::cell::{Cell, RefCell, OnceCell}

// Cell - copy types
let cell = Cell::new(0)
cell.set(1)
let val = cell.get()

// RefCell - runtime borrow checking
let ref_cell = RefCell::new(vec![1, 2, 3])
ref_cell.borrow_mut().push(4)

// OnceCell - write-once
let once = OnceCell::new()
once.set("value").unwrap()
let val = once.get().unwrap()
```

---

## Collections

### Vector (`Vec<T>`)

```nova
use std::vec::Vec

// Creation
let mut v = Vec::new()
let v = vec![1, 2, 3]
let v = vec![0; 100]  // 100 zeros

// Access
v[0]           // panics if out of bounds
v.get(0)       // Option<&T>
v.first()      // Option<&T>
v.last()       // Option<&T>

// Modification
v.push(4)
v.pop()        // Option<T>
v.insert(0, 0)
v.remove(0)
v.clear()

// Iteration
for x in &v { ... }
for x in &mut v { ... }
for x in v { ... }  // consumes

// Transformations
v.map(|x| x * 2)
v.filter(|x| *x > 2)
v.fold(0, |a, b| a + b)
```

### HashMap (`HashMap<K, V>`)

```nova
use std::collections::HashMap

let mut map = HashMap::new()
map.insert("key", 42)
map.get(&"key")           // Option<&V>
map.contains_key(&"key")  // bool
map.remove(&"key")

// Entry API
map.entry("key").or_insert(0)
map.entry("key").or_insert_with(|| compute())
map.entry("key").and_modify(|v| *v += 1).or_insert(1)
```

### HashSet (`HashSet<T>`)

```nova
use std::collections::HashSet

let mut set = HashSet::new()
set.insert(1)
set.contains(&1)
set.remove(&1)

// Set operations
let a: HashSet<_> = [1, 2, 3].into_iter().collect()
let b: HashSet<_> = [3, 4, 5].into_iter().collect()

a.union(&b).cloned().collect()           // {1, 2, 3, 4, 5}
a.intersection(&b).cloned().collect()    // {3}
a.difference(&b).cloned().collect()      // {1, 2}
a.symmetric_difference(&b).cloned().collect() // {1, 2, 4, 5}
```

### BTreeMap / BTreeSet (Ordered)

```nova
use std::collections::{BTreeMap, BTreeSet}

let mut map = BTreeMap::new()
map.insert("c", 3)
map.insert("a", 1)
map.insert("b", 2)
// Iterates in order: a, b, c

// Range queries
for (k, v) in map.range("a".."c") { ... }
```

### VecDeque (Double-ended Queue)

```nova
use std::collections::VecDeque

let mut deque = VecDeque::new()
deque.push_front(1)
deque.push_back(2)
deque.pop_front()
deque.pop_back()
```

---

## Strings

### String Types

```nova
// Owned string
let mut s = String::new()
let s = String::from("hello")
let s = "hello".to_string()

// String slice (borrowed)
let slice: &str = &s
let slice = "hello"
```

### String Operations

```nova
let mut s = String::from("hello")

// Capacity
s.capacity()
s.reserve(10)
s.shrink_to_fit()

// Modification
s.push('!')
s.push_str(" world")
s.insert(5, ',')
s.remove(5)
s.clear()

// Formatting
format!("{} {}", "hello", "world")
format!("{:?}", vec![1, 2, 3])

// Splitting
"a,b,c".split(',').collect::<Vec<_>>()  // ["a", "b", "c"]
"hello".chars().collect::<Vec<_>>()     // ['h', 'e', 'l', 'l', 'o']
"hello".bytes().collect::<Vec<_>>()     // [104, 101, 108, 108, 111]

// Trimming
"  hello  ".trim()      // "hello"
"hello\n".trim_end()    // "hello"

// Case
"HELLO".to_lowercase()  // "hello"
"hello".to_uppercase()  // "HELLO"

// Replacement
"hello world".replace("world", "NOVA")
```

### String Search

```nova
"hello".contains("ell")     // true
"hello".starts_with("he")   // true
"hello".ends_with("lo")     // true
"hello".find("el")          // Some(1)
"hello".rfind("l")          // Some(3)
```

---

## Option and Result

### Option<T>

```nova
use std::option::Option::{Some, None}

let some = Some(42)
let none: Option<i32> = None

// Methods
some.is_some()    // true
some.is_none()    // false
some.unwrap()     // 42 (panics if None)
some.unwrap_or(0) // 42
none.unwrap_or(0) // 0
some.expect("msg") // 42

// Transformations
some.map(|x| x * 2)              // Some(84)
some.and_then(|x| Some(x + 1))   // Some(43)
some.filter(|x| *x > 10)         // Some(42)
some.or(Some(100))               // Some(42)
none.or(Some(100))               // Some(100)
```

### Result<T, E>

```nova
use std::result::Result::{Ok, Err}

let ok: Result<i32, &str> = Ok(42)
let err: Result<i32, &str> = Err("error")

// Methods
ok.is_ok()     // true
ok.is_err()    // false
ok.unwrap()    // 42
err.unwrap_err() // "error"
ok.unwrap_or(0) // 42
err.unwrap_or(0) // 0

// Transformations
ok.map(|x| x * 2)           // Ok(84)
ok.map_err(|e| e.len())     // Ok(42)
err.map_err(|e| e.len())    // Err(5)
ok.and_then(|x| Ok(x + 1))  // Ok(43)
err.or_else(|_| Ok(0))      // Ok(0)

// ? operator
fn read() -> Result<String, std::io::Error> {
    let mut s = String::new()
    std::io::stdin().read_to_string(&mut s)?
    Ok(s)
}
```

### Conversion Between Option and Result

```nova
let opt = Some(42)
let res: Result<i32, &str> = opt.ok_or("none")

let res = Ok(42)
let opt = res.ok()

let res = Err("error")
let opt = res.ok()  // None
```

---

## I/O

### Standard Input/Output

```nova
use std::io::{self, Read, Write, BufRead, BufReader, BufWriter}

// Stdout
println!("Hello")
print!("No newline")
io::stdout().flush()?

// Stdin
let mut input = String::new()
io::stdin().read_line(&mut input)?

// Buffered reading
let reader = BufReader::new(io::stdin())
for line in reader.lines() {
    let line = line?
    println!("Got: {}", line)
}

// Buffered writing
let mut writer = BufWriter::new(io::stdout())
writer.write_all(b"Hello")?
writer.flush()?
```

### File I/O

```nova
use std::fs::File
use std::io::{Read, Write, BufReader, BufWriter}

// Reading
let mut file = File::open("file.txt")?
let mut contents = String::new()
file.read_to_string(&mut contents)?

// Or use convenience functions
let contents = std::fs::read_to_string("file.txt")?
let bytes = std::fs::read("file.txt")?

// Writing
let mut file = File::create("output.txt")?
file.write_all(b"Hello, NOVA!")?

// Or use convenience
std::fs::write("output.txt", "Hello, NOVA!")?

// Buffered
let file = File::open("large.txt")?
let reader = BufReader::new(file)
for line in reader.lines() { ... }
```

### Paths

```nova
use std::path::{Path, PathBuf}

let path = Path::new("foo/bar.txt")
path.exists()
path.is_file()
path.is_dir()
path.extension()      // Some("txt")
path.file_name()      // Some("bar.txt")
path.parent()         // Some("foo")

// PathBuf for building paths
let mut path = PathBuf::new()
path.push("foo")
path.push("bar.txt")
path.set_extension("rs")
```

---

## Concurrency Primitives

### Threads

```nova
use std::thread
use std::time::Duration

thread::spawn(|| {
    println!("Hello from thread!")
}).join()

// Scoped threads (can borrow from environment)
thread::scope(|s| {
    s.spawn(|| println!("Scoped!"))
})
```

### Synchronization

```nova
use std::sync::{Mutex, RwLock, Condvar, Barrier, OnceLock, LazyLock}
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering}

// Mutex
let mutex = Mutex::new(0)
let mut guard = mutex.lock().unwrap()
*guard += 1

// RwLock
let rwlock = RwLock::new(vec![1, 2, 3])
let read = rwlock.read().unwrap()
let mut write = rwlock.write().unwrap()

// Atomic
let counter = AtomicUsize::new(0)
counter.fetch_add(1, Ordering::Relaxed)
counter.load(Ordering::Relaxed)

// Barrier
let barrier = Barrier::new(3)
barrier.wait()

// OnceLock / LazyLock
static CONFIG: OnceLock<String> = OnceLock::new()
CONFIG.get_or_init(|| "config".to_string())
```

### Channels

```nova
use std::sync::mpsc

let (tx, rx) = mpsc::channel()
tx.send(42)?
let val = rx.recv()?

// Sync channel (bounded)
let (tx, rx) = mpsc::sync_channel(10)
```

### Thread-local Storage

```nova
use std::thread_local

thread_local! {
    static COUNTER: RefCell<u32> = RefCell::new(0)
}

COUNTER.with(|c| *c.borrow_mut() += 1)
```

---

## Time

```nova
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH}

// Duration
let dur = Duration::from_secs(1)
let dur = Duration::from_millis(100)
let dur = Duration::from_nanos(1_000_000_000)

// Instant (monotonic clock)
let start = Instant::now()
// ... do work ...
let elapsed = start.elapsed()

// System time
let now = SystemTime::now()
let since_epoch = now.duration_since(UNIX_EPOCH)?
let timestamp = since_epoch.as_secs()
```

---

## Math

### Basic Operations

```nova
use std::f64::consts::PI

// Constants
PI, E, SQRT_2, FRAC_PI_2, etc.

// Basic
let abs = (-5.0f64).abs()
let min = f64::min(1.0, 2.0)
let max = f64::max(1.0, 2.0)
let clamped = 5.0.clamp(0.0, 10.0)

// Powers and roots
let sqrt = 16.0.sqrt()
let pow = 2.0f64.powi(10)
let cbrt = 27.0.cbrt()
let hypot = 3.0.hypot(4.0)  // 5.0

// Trigonometry
let sin = PI.sin()
let cos = PI.cos()
let tan = (PI/4.0).tan()
let asin = 0.5.asin()

// Exponential and logarithmic
let exp = 1.0.exp()
let ln = 2.0f64.ln()
let log2 = 8.0.log2()
let log10 = 100.0.log10()

// Rounding
let floor = 3.7.floor()    // 3.0
let ceil = 3.2.ceil()      // 4.0
let round = 3.5.round()    // 4.0
let trunc = 3.9.trunc()    // 3.0
let fract = 3.5.fract()    // 0.5

// Integer math
let div = 10 / 3      // 3
let rem = 10 % 3      // 1
let div_euclid = 10.div_euclid(3)  // 3
let rem_euclid = 10.rem_euclid(3)  // 1
```

### Random Numbers

```nova
use std::rand::{Rng, thread_rng}

let mut rng = thread_rng()
let n: i32 = rng.gen()
let n: i32 = rng.gen_range(0..10)
let f: f64 = rng.gen()
let bool = rng.gen_bool(0.5)
let choice = rng.choose(&[1, 2, 3])
```

---

## File System

### Directory Operations

```nova
use std::fs
use std::path::Path

// Create
fs::create_dir("dir")?
fs::create_dir_all("a/b/c")?

// Read
let entries = fs::read_dir(".")?
for entry in entries {
    let entry = entry?
    println!("{:?}", entry.path())
}

// Copy/Move/Remove
fs::copy("src.txt", "dst.txt")?
fs::rename("old.txt", "new.txt")?
fs::remove_file("file.txt")?
fs::remove_dir("dir")?
fs::remove_dir_all("dir")?

// Metadata
let meta = fs::metadata("file.txt")?
meta.is_file()
meta.is_dir()
meta.len()
meta.modified()?
```

### Temp Files

```nova
use std::fs
use std::io::Write

let mut temp = fs::NamedTempFile::new()?
temp.write_all(b"data")?
temp.flush()?
let path = temp.path()
```

---

## Networking

### TCP

```nova
use std::net::{TcpListener, TcpStream, SocketAddr}
use std::io::{Read, Write}

// Server
let listener = TcpListener::bind("127.0.0.1:8080")?
for stream in listener.incoming() {
    let mut stream = stream?
    let mut buf = [0; 1024]
    let n = stream.read(&mut buf)?
    stream.write_all(&buf[..n])?
}

// Client
let mut stream = TcpStream::connect("127.0.0.1:8080")?
stream.write_all(b"Hello")?
```

### UDP

```nova
use std::net::UdpSocket

let socket = UdpSocket::bind("127.0.0.1:0")?
socket.send_to(b"hello", "127.0.0.1:8080")?
let mut buf = [0; 1024]
let (n, addr) = socket.recv_from(&mut buf)?
```

---

## Serialization

### JSON (serde_json)

```nova
use serde::{Serialize, Deserialize}

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

let user = User { id: 1, name: "Alice".into(), email: "alice@example.com".into() }

// To JSON string
let json = serde_json::to_string(&user)?
// {"id":1,"name":"Alice","email":"alice@example.com"}

// Pretty
let pretty = serde_json::to_string_pretty(&user)?

// From JSON
let user: User = serde_json::from_str(&json)?
```

### Other Formats

```nova
// TOML
let toml = toml::to_string(&user)?
let user: User = toml::from_str(&toml)?

// YAML
let yaml = serde_yaml::to_string(&user)?
let user: User = serde_yaml::from_str(&yaml)?

// MessagePack
let msgpack = rmp_serde::to_vec(&user)?
let user: User = rmp_serde::from_slice(&msgpack)?

// Bincode
let bincode = bincode::serialize(&user)?
let user: User = bincode::deserialize(&bincode)?
```

---

## Testing

### Unit Tests

```nova
#[cfg(test)]
mod tests {
    use super::*

    #[test]
    fn test_addition() {
        assert_eq!(2 + 2, 4)
    }

    #[test]
    fn test_panic() {
        assert!(true)
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_divide_by_zero() {
        let _ = 10 / 0
    }
}
```

### Integration Tests

```nova
// tests/integration_test.nova
use my_crate::*;

#[test]
fn test_full_workflow() {
    let result = process("input")
    assert_eq!(result, "expected")
}
```

### Benchmarks

```nova
// benches/bench.nova
use nova_bench::Bencher;

#[bench]
fn bench_hashmap(b: &mut Bencher) {
    b.iter(|| {
        let mut map = HashMap::new()
        for i in 0..1000 { map.insert(i, i) }
    })
}
```

### Running Tests

```bash
nova test              # Run all tests
nova test --lib        # Unit tests only
nova test --tests      # Integration tests only
nova bench             # Run benchmarks
nova test --doc        # Run doc tests
```

---

## Error Handling Patterns

### Common Patterns

```nova
// Early return with ?
fn process() -> Result<(), Error> {
    let data = read_file("input.txt")?
    let parsed = parse(data)?
    write_file("output.txt", parsed)?
    Ok(())
}

// Multiple error types
fn complex() -> Result<String, Box<dyn std::error::Error>> {
    let a = step1()?
    let b = step2(a)?
    Ok(b)
}

// Collecting results
let results: Result<Vec<_>, _> = items.iter().map(process).collect()

// Partitioning
let (oks, errs): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok)
```