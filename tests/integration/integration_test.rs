// Integration tests for NOVA Compiler Pipeline

use nova_compiler::{Session, CompileConfig, Target, OptLevel, Edition, compile};
use nova_compiler::codegen::codegen;

#[test]
fn test_compile_simple_program() {
    let source = r#"
fn main() {
    let x = 42;
    println!("Hello, {}!", x);
}
"#;
    
    let config = CompileConfig {
        target: Target::Bytecode,
        opt_level: OptLevel::None,
        debug_info: true,
        test_mode: false,
        doc_mode: false,
        edition: Edition::Nova2026,
    };
    
    let result = compile(source, config);
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(!output.diagnostics.has_errors());
}

#[test]
fn test_compile_with_struct() {
    let source = r#"
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

fn main() {
    let p = Point { x: 3.0, y: 4.0 };
    println!("Magnitude: {}", p.magnitude());
}
"#;
    
    let config = CompileConfig {
        target: Target::Bytecode,
        opt_level: OptLevel::Standard,
        debug_info: true,
        test_mode: false,
        doc_mode: false,
        edition: Edition::Nova2026,
    };
    
    let result = compile(source, config);
    assert!(result.is_ok());
}

#[test]
fn test_compile_with_generics() {
    let source = r#"
fn identity<T>(x: T) -> T {
    x
}

fn main() {
    let x = identity(42);
    let y = identity("hello");
    println!("{} {}", x, y);
}
"#;
    
    let config = CompileConfig {
        target: Target::Bytecode,
        opt_level: OptLevel::Standard,
        debug_info: true,
        test_mode: false,
        doc_mode: false,
        edition: Edition::Nova2026,
    };
    
    let result = compile(source, config);
    assert!(result.is_ok());
}

#[test]
fn test_compile_async() {
    let source = r#"
async fn fetch() -> String {
    "data".to_string()
}

async fn main() {
    let data = fetch().await;
    println!("{}", data);
}
"#;
    
    let config = CompileConfig {
        target: Target::Bytecode,
        opt_level: OptLevel::Standard,
        debug_info: true,
        test_mode: false,
        doc_mode: false,
        edition: Edition::Nova2026,
    };
    
    let result = compile(source, config);
    assert!(result.is_ok());
}

#[test]
fn test_compile_with_error() {
    let source = r#"
fn main() {
    let x: i32 = "not a number";
}
"#;
    
    let config = CompileConfig {
        target: Target::Bytecode,
        opt_level: OptLevel::None,
        debug_info: true,
        test_mode: false,
        doc_mode: false,
        edition: Edition::Nova2026,
    };
    
    let result = compile(source, config);
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(output.diagnostics.has_errors());
}

#[test]
fn test_compile_wasm_target() {
    let source = r#"
fn main() {
    println!("Hello, WASM!");
}
"#;
    
    let config = CompileConfig {
        target: Target::Wasm32,
        opt_level: OptLevel::Standard,
        debug_info: false,
        test_mode: false,
        doc_mode: false,
        edition: Edition::Nova2026,
    };
    
    let result = compile(source, config);
    assert!(result.is_ok());
}

#[test]
fn test_compile_native_target() {
    let source = r#"
fn main() {
    println!("Hello, Native!");
}
"#;
    
    let config = CompileConfig {
        target: Target::Native,
        opt_level: OptLevel::Standard,
        debug_info: false,
        test_mode: false,
        doc_mode: false,
        edition: Edition::Nova2026,
    };
    
    let result = compile(source, config);
    assert!(result.is_ok());
}