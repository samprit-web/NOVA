// UI Tests for NOVA Compiler Error Messages

use nova_compiler::{Session, CompileConfig, Target, OptLevel, Edition, compile};

#[test]
fn test_undefined_variable() {
    let source = r#"
fn main() {
    println!("{}", undefined_var);
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
    
    // Check for specific error
    let errors = output.diagnostics.errors();
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.message.contains("undefined") || e.message.contains("not found")));
}

#[test]
fn test_type_mismatch() {
    let source = r#"
fn main() {
    let x: i32 = "hello";
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
    
    let errors = output.diagnostics.errors();
    assert!(errors.iter().any(|e| e.message.contains("type") || e.message.contains("mismatch")));
}

#[test]
fn test_missing_semicolon() {
    let source = r#"
fn main() {
    let x = 42
    println!("{}", x);
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
fn test_unclosed_brace() {
    let source = r#"
fn main() {
    if true {
        println!("hello");
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
fn test_invalid_escape() {
    let source = r#"
fn main() {
    let s = "hello \z world";
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
fn test_unmatched_type_in_match() {
    let source = r#"
fn main() {
    let x = Some(42);
    match x {
        Some(y) => println!("{}", y),
    }
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
    // Should warn about non-exhaustive match
    assert!(output.diagnostics.warnings().len() > 0 || output.diagnostics.errors().len() > 0);
}

#[test]
fn test_use_of_moved_value() {
    let source = r#"
fn main() {
    let x = String::from("hello");
    let y = x;
    println!("{}", x); // Error: use of moved value
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
    
    let errors = output.diagnostics.errors();
    assert!(errors.iter().any(|e| e.message.contains("move") || e.message.contains("moved")));
}

#[test]
fn test_borrow_checker() {
    let source = r#"
fn main() {
    let mut x = 42;
    let r1 = &mut x;
    let r2 = &mut x; // Error: cannot borrow as mutable more than once
    *r1 = 1;
    *r2 = 2;
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
    
    let errors = output.diagnostics.errors();
    assert!(errors.iter().any(|e| e.message.contains("borrow") || e.message.contains("mutable")));
}

#[test]
fn test_lifetime_error() {
    let source = r#"
fn get_str() -> &str {
    let s = String::from("hello");
    &s // Error: returns reference to local variable
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
    
    let errors = output.diagnostics.errors();
    assert!(errors.iter().any(|e| e.message.contains("lifetime") || e.message.contains("borrow") || e.message.contains("local")));
}