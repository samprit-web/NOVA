// Unit tests for NOVA Compiler

use nova_compiler::{Session, CompileConfig, Target, OptLevel, Edition, lex, parse};
use nova_compiler::ast::*;

fn create_session() -> Session {
    Session::new(CompileConfig {
        target: Target::Native,
        opt_level: OptLevel::None,
        debug_info: false,
        test_mode: true,
        doc_mode: false,
        edition: Edition::Nova2026,
    })
}

#[test]
fn test_lexer_simple() {
    let mut session = create_session();
    let source = r#"
fn main() {
    let x = 42;
    println!("Hello, {}!", x);
}
"#;
    let file_id = session.add_file("test.nova", source.to_string());
    let tokens = lex(&mut session, file_id, source).unwrap();
    
    assert!(!tokens.is_empty());
    assert!(matches!(tokens.last().unwrap().kind, nova_compiler::TokenKind::Eof));
}

#[test]
fn test_lexer_keywords() {
    let mut session = create_session();
    let source = "fn let mut const async await match if else loop for while break continue return";
    let file_id = session.add_file("test.nova", source.to_string());
    let tokens = lex(&mut session, file_id, source).unwrap();
    
    let keywords: Vec<_> = tokens.iter()
        .filter(|t| matches!(t.kind, nova_compiler::TokenKind::Fn | 
            nova_compiler::TokenKind::Let | 
            nova_compiler::TokenKind::Mut | 
            nova_compiler::TokenKind::Const | 
            nova_compiler::TokenKind::Async | 
            nova_compiler::TokenKind::Await | 
            nova_compiler::TokenKind::Match | 
            nova_compiler::TokenKind::If | 
            nova_compiler::TokenKind::Else | 
            nova_compiler::TokenKind::Loop | 
            nova_compiler::TokenKind::For | 
            nova_compiler::TokenKind::While | 
            nova_compiler::TokenKind::Break | 
            nova_compiler::TokenKind::Continue | 
            nova_compiler::TokenKind::Return))
        .collect();
    
    assert_eq!(keywords.len(), 18);
}

#[test]
fn test_lexer_literals() {
    let mut session = create_session();
    let source = r#"42 3.14 'a' "hello" true false ()"#;
    let file_id = session.add_file("test.nova", source.to_string());
    let tokens = lex(&mut session, file_id, source).unwrap();
    
    let literals: Vec<_> = tokens.iter()
        .filter(|t| matches!(t.kind, 
            nova_compiler::TokenKind::Integer(_) | 
            nova_compiler::TokenKind::Float(_) | 
            nova_compiler::TokenKind::Char(_) | 
            nova_compiler::TokenKind::String(_) | 
            nova_compiler::TokenKind::Bool(_) | 
            nova_compiler::TokenKind::Unit))
        .collect();
    
    assert_eq!(literals.len(), 8);
}

#[test]
fn test_parser_simple_function() {
    let mut session = create_session();
    let source = r#"
fn main() {
    let x = 42;
    println!("Hello, {}!", x);
}
"#;
    let file_id = session.add_file("test.nova", source.to_string());
    let tokens = lex(&mut session, file_id, source).unwrap();
    let program = parse(&mut session, file_id, &tokens).unwrap();
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        Item::Function(f) => {
            assert_eq!(f.name.name, "main");
            assert_eq!(f.params.len(), 0);
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parser_struct() {
    let mut session = create_session();
    let source = r#"
struct Point {
    x: f64,
    y: f64,
}
"#;
    let file_id = session.add_file("test.nova", source.to_string());
    let tokens = lex(&mut session, file_id, source).unwrap();
    let program = parse(&mut session, file_id, &tokens).unwrap();
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        Item::Struct(s) => {
            assert_eq!(s.name.name, "Point");
            match &s.kind {
                StructKind::Record(fields) => {
                    assert_eq!(fields.len(), 2);
                    assert_eq!(fields[0].name.name, "x");
                    assert_eq!(fields[1].name.name, "y");
                }
                _ => panic!("Expected record struct"),
            }
        }
        _ => panic!("Expected struct"),
    }
}

#[test]
fn test_parser_enum() {
    let mut session = create_session();
    let source = r#"
enum Option<T> {
    Some(T),
    None,
}
"#;
    let file_id = session.add_file("test.nova", source.to_string());
    let tokens = lex(&mut session, file_id, source).unwrap();
    let program = parse(&mut session, file_id, &tokens).unwrap();
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        Item::Enum(e) => {
            assert_eq!(e.name.name, "Option");
            assert_eq!(e.generics.params.len(), 1);
            assert_eq!(e.variants.len(), 2);
        }
        _ => panic!("Expected enum"),
    }
}

#[test]
fn test_parser_generics() {
    let mut session = create_session();
    let source = r#"
fn map<T, U>(vec: Vec<T>, f: fn(T) -> U) -> Vec<U> {
    vec.iter().map(f).collect()
}
"#;
    let file_id = session.add_file("test.nova", source.to_string());
    let tokens = lex(&mut session, file_id, source).unwrap();
    let program = parse(&mut session, file_id, &tokens).unwrap();
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        Item::Function(f) => {
            assert_eq!(f.generics.params.len(), 2);
            assert_eq!(f.params.len(), 2);
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parser_async() {
    let mut session = create_session();
    let source = r#"
async fn fetch_data() -> Result<String, Error> {
    let response = http::get("https://api.example.com").await?;
    response.text().await
}
"#;
    let file_id = session.add_file("test.nova", source.to_string());
    let tokens = lex(&mut session, file_id, source).unwrap();
    let program = parse(&mut session, file_id, &tokens).unwrap();
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        Item::Function(f) => {
            assert!(matches!(f.asyncness, Asyncness::Async));
        }
        _ => panic!("Expected async function"),
    }
}

#[test]
fn test_parser_actor() {
    let mut session = create_session();
    let source = r#"
actor Counter {
    count: i64 = 0
    
    fn increment(&mut self) -> i64 {
        self.count += 1;
        self.count
    }
}
"#;
    let file_id = session.add_file("test.nova", source.to_string());
    let tokens = lex(&mut session, file_id, source).unwrap();
    let program = parse(&mut session, file_id, &tokens).unwrap();
    
    assert_eq!(program.items.len(), 1);
    // Actor parsing would be implemented
}

#[test]
fn test_parser_macros() {
    let mut session = create_session();
    let source = r#"
macro_rules! vec {
    () => { Vec::new() };
    ($($x:expr),+ $(,)?) => {{
        let mut v = Vec::with_capacity(0 $(+ 1)*);
        $(v.push($x);)+
        v
    }};
}
"#;
    let file_id = session.add_file("test.nova", source.to_string());
    let tokens = lex(&mut session, file_id, source).unwrap();
    let program = parse(&mut session, file_id, &tokens).unwrap();
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        Item::MacroRules(m) => {
            assert_eq!(m.name.name, "vec");
        }
        _ => panic!("Expected macro_rules"),
    }
}