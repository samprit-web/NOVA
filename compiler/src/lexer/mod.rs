//! NOVA Lexer using Pest parser generator

use std::sync::Arc;

use anyhow::Result;
use pest::Parser;
use pest::iterators::Pair;

use crate::{Session, Span, FileId, Diagnostic, Severity};
use crate::parser_gen::NovaParser;
use crate::parser_gen::Rule;

/// Token representation
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// Token kinds
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Async,
    Await,
    Break,
    Const,
    Continue,
    Crate,
    Else,
    Enum,
    Extern,
    False,
    Fn,
    For,
    If,
    Impl,
    In,
    Let,
    Loop,
    Match,
    Mod,
    Move,
    Mut,
    Pub,
    Ref,
    Return,
    SelfValue,
    SelfType,
    Static,
    Struct,
    Super,
    Trait,
    True,
    Type,
    Unsafe,
    Use,
    Where,
    While,
    Yield,
    Try,
    Catch,
    Throw,
    Spawn,
    Actor,
    Channel,
    Select,
    Defer,
    Drop,
    Box,
    Dyn,
    Abstract,
    Final,
    Override,
    Virtual,
    Sealed,
    Open,
    Inline,
    NoInline,
    ConstEval,
    Comptime,
    Effect,
    Handle,
    Resume,
    Linear,
    Unique,
    Shared,
    Atomic,
    Volatile,
    Unaligned,
    Packed,
    Transparent,
    Repr,
    Align,
    Cfg,
    CfgAttr,
    Derive,
    Macro,
    MacroRules,
    Include,
    IncludeStr,
    IncludeBytes,
    Concat,
    Stringify,
    File,
    Line,
    Column,
    ModulePath,
    CompileError,
    CompileWarning,
    Todo,
    Unimplemented,
    Unreachable,
    Panic,
    Assert,
    AssertEq,
    AssertNe,
    DebugAssert,
    DebugAssertEq,
    DebugAssertNe,

    // Literals
    Integer(String),
    Float(String),
    Char(char),
    String(String),
    RawString(String),
    ByteString(Vec<u8>),
    RawByteString(Vec<u8>),
    Bool(bool),
    Unit,

    // Identifiers
    Ident(String),
    RawIdent(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Amp,
    Pipe,
    Caret,
    Shl,
    Shr,
    EqEq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    AndAnd,
    OrOr,
    Not,
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    PercentAssign,
    AmpAssign,
    PipeAssign,
    CaretAssign,
    ShlAssign,
    ShrAssign,
    Dot,
    DotDot,
    DotDotDot,
    Arrow,
    FatArrow,
    LeftArrow,
    ColonColon,
    Colon,
    Semi,
    Comma,
    Question,
    PipeRight,
    PipeLeft,
    At,
    Hash,
    Dollar,
    Tilde,
    Backtick,

    // Punctuation
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    // Special
    Eof,
}

impl TokenKind {
    pub fn is_keyword(&self) -> bool {
        matches!(self,
            TokenKind::Async | TokenKind::Await | TokenKind::Break | TokenKind::Const |
            TokenKind::Continue | TokenKind::Crate | TokenKind::Else | TokenKind::Enum |
            TokenKind::Extern | TokenKind::False | TokenKind::Fn | TokenKind::For |
            TokenKind::If | TokenKind::Impl | TokenKind::In | TokenKind::Let |
            TokenKind::Loop | TokenKind::Match | TokenKind::Mod | TokenKind::Move |
            TokenKind::Mut | TokenKind::Pub | TokenKind::Ref | TokenKind::Return |
            TokenKind::SelfValue | TokenKind::SelfType | TokenKind::Static |
            TokenKind::Struct | TokenKind::Super | TokenKind::Trait | TokenKind::True |
            TokenKind::Type | TokenKind::Unsafe | TokenKind::Use | TokenKind::Where |
            TokenKind::While | TokenKind::Yield | TokenKind::Try | TokenKind::Catch |
            TokenKind::Throw | TokenKind::Spawn | TokenKind::Actor | TokenKind::Channel |
            TokenKind::Select | TokenKind::Defer | TokenKind::Drop | TokenKind::Box |
            TokenKind::Dyn | TokenKind::Abstract | TokenKind::Final | TokenKind::Override |
            TokenKind::Virtual | TokenKind::Sealed | TokenKind::Open | TokenKind::Inline |
            TokenKind::NoInline | TokenKind::ConstEval | TokenKind::Comptime |
            TokenKind::Effect | TokenKind::Handle | TokenKind::Resume | TokenKind::Linear |
            TokenKind::Unique | TokenKind::Shared | TokenKind::Atomic | TokenKind::Volatile |
            TokenKind::Unaligned | TokenKind::Packed | TokenKind::Transparent |
            TokenKind::Repr | TokenKind::Align | TokenKind::Cfg | TokenKind::CfgAttr |
            TokenKind::Derive | TokenKind::Macro | TokenKind::MacroRules |
            TokenKind::Include | TokenKind::IncludeStr | TokenKind::IncludeBytes |
            TokenKind::Concat | TokenKind::Stringify | TokenKind::File | TokenKind::Line |
            TokenKind::Column | TokenKind::ModulePath | TokenKind::CompileError |
            TokenKind::CompileWarning | TokenKind::Todo | TokenKind::Unimplemented |
            TokenKind::Unreachable | TokenKind::Panic | TokenKind::Assert |
            TokenKind::AssertEq | TokenKind::AssertNe | TokenKind::DebugAssert |
            TokenKind::DebugAssertEq | TokenKind::DebugAssertNe
        )
    }
}

/// Lex source code into tokens
pub fn lex(session: &mut Session, file_id: FileId, source: &str) -> Result<Vec<Token>> {
    let pairs = NovaParser::parse(Rule::program, source)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let mut tokens = Vec::new();
    for pair in pairs {
        tokens.extend(lex_pair(session, file_id, pair)?);
    }

    // Add EOF token
    let last_span = tokens.last().map(|t| t.span).unwrap_or(Span::new(file_id, 0, 0));
    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span::new(file_id, last_span.end, last_span.end),
    });

    Ok(tokens)
}

fn lex_pair(session: &mut Session, file_id: FileId, pair: Pair<Rule>) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();

    match pair.as_rule() {
        Rule::program => {
            for inner in pair.into_inner() {
                tokens.extend(lex_pair(session, file_id, inner)?);
            }
        }
        Rule::keyword => {
            tokens.push(token_from_keyword(file_id, pair.as_span(), pair.as_str()));
        }
        Rule::ident | Rule::raw_identifier => {
            tokens.push(Token {
                kind: TokenKind::Ident(pair.as_str().to_string()),
                span: span_from_pest(file_id, pair.as_span()),
            });
        }
        Rule::integer => {
            tokens.push(Token {
                kind: TokenKind::Integer(pair.as_str().to_string()),
                span: span_from_pest(file_id, pair.as_span()),
            });
        }
        Rule::float => {
            tokens.push(Token {
                kind: TokenKind::Float(pair.as_str().to_string()),
                span: span_from_pest(file_id, pair.as_span()),
            });
        }
        Rule::char => {
            let ch = parse_char(pair.as_str());
            tokens.push(Token {
                kind: TokenKind::Char(ch),
                span: span_from_pest(file_id, pair.as_span()),
            });
        }
        Rule::string | Rule::raw_string => {
            let s = parse_string(pair.as_str(), pair.as_rule() == Rule::raw_string);
            tokens.push(Token {
                kind: if pair.as_rule() == Rule::raw_string {
                    TokenKind::RawString(s)
                } else {
                    TokenKind::String(s)
                },
                span: span_from_pest(file_id, pair.as_span()),
            });
        }
        Rule::byte_string | Rule::raw_byte_string => {
            let bytes = parse_byte_string(pair.as_str(), pair.as_rule() == Rule::raw_byte_string);
            tokens.push(Token {
                kind: if pair.as_rule() == Rule::raw_byte_string {
                    TokenKind::RawByteString(bytes)
                } else {
                    TokenKind::ByteString(bytes)
                },
                span: span_from_pest(file_id, pair.as_span()),
            });
        }
        Rule::bool => {
            tokens.push(Token {
                kind: TokenKind::Bool(pair.as_str() == "true"),
                span: span_from_pest(file_id, pair.as_span()),
            });
        }
        Rule::unit => {
            tokens.push(Token {
                kind: TokenKind::Unit,
                span: span_from_pest(file_id, pair.as_span()),
            });
        }
        Rule::operator => {
            tokens.push(token_from_operator(file_id, pair.as_span(), pair.as_str()));
        }
        Rule::punct => {
            tokens.push(token_from_punct(file_id, pair.as_span(), pair.as_str()));
        }
        _ => {
            // Recurse into children for other rules
            for inner in pair.into_inner() {
                tokens.extend(lex_pair(session, file_id, inner)?);
            }
        }
    }

    Ok(tokens)
}

fn span_from_pest(file_id: FileId, span: pest::Span) -> Span {
    Span::new(file_id, span.start(), span.end())
}

fn token_from_keyword(file_id: FileId, span: pest::Span, text: &str) -> Token {
    let kind = match text {
        "async" => TokenKind::Async,
        "await" => TokenKind::Await,
        "break" => TokenKind::Break,
        "const" => TokenKind::Const,
        "continue" => TokenKind::Continue,
        "crate" => TokenKind::Crate,
        "else" => TokenKind::Else,
        "enum" => TokenKind::Enum,
        "extern" => TokenKind::Extern,
        "false" => TokenKind::False,
        "fn" => TokenKind::Fn,
        "for" => TokenKind::For,
        "if" => TokenKind::If,
        "impl" => TokenKind::Impl,
        "in" => TokenKind::In,
        "let" => TokenKind::Let,
        "loop" => TokenKind::Loop,
        "match" => TokenKind::Match,
        "mod" => TokenKind::Mod,
        "move" => TokenKind::Move,
        "mut" => TokenKind::Mut,
        "pub" => TokenKind::Pub,
        "ref" => TokenKind::Ref,
        "return" => TokenKind::Return,
        "self" => TokenKind::SelfValue,
        "Self" => TokenKind::SelfType,
        "static" => TokenKind::Static,
        "struct" => TokenKind::Struct,
        "super" => TokenKind::Super,
        "trait" => TokenKind::Trait,
        "true" => TokenKind::True,
        "type" => TokenKind::Type,
        "unsafe" => TokenKind::Unsafe,
        "use" => TokenKind::Use,
        "where" => TokenKind::Where,
        "while" => TokenKind::While,
        "yield" => TokenKind::Yield,
        "try" => TokenKind::Try,
        "catch" => TokenKind::Catch,
        "throw" => TokenKind::Throw,
        "spawn" => TokenKind::Spawn,
        "actor" => TokenKind::Actor,
        "channel" => TokenKind::Channel,
        "select" => TokenKind::Select,
        "defer" => TokenKind::Defer,
        "drop" => TokenKind::Drop,
        "box" => TokenKind::Box,
        "dyn" => TokenKind::Dyn,
        "abstract" => TokenKind::Abstract,
        "final" => TokenKind::Final,
        "override" => TokenKind::Override,
        "virtual" => TokenKind::Virtual,
        "sealed" => TokenKind::Sealed,
        "open" => TokenKind::Open,
        "inline" => TokenKind::Inline,
        "noinline" => TokenKind::NoInline,
        "const_eval" => TokenKind::ConstEval,
        "comptime" => TokenKind::Comptime,
        "effect" => TokenKind::Effect,
        "handle" => TokenKind::Handle,
        "resume" => TokenKind::Resume,
        "linear" => TokenKind::Linear,
        "unique" => TokenKind::Unique,
        "shared" => TokenKind::Shared,
        "atomic" => TokenKind::Atomic,
        "volatile" => TokenKind::Volatile,
        "unaligned" => TokenKind::Unaligned,
        "packed" => TokenKind::Packed,
        "transparent" => TokenKind::Transparent,
        "repr" => TokenKind::Repr,
        "align" => TokenKind::Align,
        "cfg" => TokenKind::Cfg,
        "cfg_attr" => TokenKind::CfgAttr,
        "derive" => TokenKind::Derive,
        "macro" => TokenKind::Macro,
        "macro_rules" => TokenKind::MacroRules,
        "include" => TokenKind::Include,
        "include_str" => TokenKind::IncludeStr,
        "include_bytes" => TokenKind::IncludeBytes,
        "concat" => TokenKind::Concat,
        "stringify" => TokenKind::Stringify,
        "file" => TokenKind::File,
        "line" => TokenKind::Line,
        "column" => TokenKind::Column,
        "module_path" => TokenKind::ModulePath,
        "compile_error" => TokenKind::CompileError,
        "compile_warning" => TokenKind::CompileWarning,
        "todo" => TokenKind::Todo,
        "unimplemented" => TokenKind::Unimplemented,
        "unreachable" => TokenKind::Unreachable,
        "panic" => TokenKind::Panic,
        "assert" => TokenKind::Assert,
        "assert_eq" => TokenKind::AssertEq,
        "assert_ne" => TokenKind::AssertNe,
        "debug_assert" => TokenKind::DebugAssert,
        "debug_assert_eq" => TokenKind::DebugAssertEq,
        "debug_assert_ne" => TokenKind::DebugAssertNe,
        _ => TokenKind::Ident(text.to_string()),
    };

    Token {
        kind,
        span: span_from_pest(file_id, span),
    }
}

fn token_from_operator(file_id: FileId, span: pest::Span, text: &str) -> Token {
    let kind = match text {
        "+" => TokenKind::Plus,
        "-" => TokenKind::Minus,
        "*" => TokenKind::Star,
        "/" => TokenKind::Slash,
        "%" => TokenKind::Percent,
        "&" => TokenKind::Amp,
        "|" => TokenKind::Pipe,
        "^" => TokenKind::Caret,
        "<<" => TokenKind::Shl,
        ">>" => TokenKind::Shr,
        "==" => TokenKind::EqEq,
        "!=" => TokenKind::Ne,
        "<" => TokenKind::Lt,
        ">" => TokenKind::Gt,
        "<=" => TokenKind::Le,
        ">=" => TokenKind::Ge,
        "&&" => TokenKind::AndAnd,
        "||" => TokenKind::OrOr,
        "!" => TokenKind::Not,
        "=" => TokenKind::Assign,
        "+=" => TokenKind::PlusAssign,
        "-=" => TokenKind::MinusAssign,
        "*=" => TokenKind::StarAssign,
        "/=" => TokenKind::SlashAssign,
        "%=" => TokenKind::PercentAssign,
        "&=" => TokenKind::AmpAssign,
        "|=" => TokenKind::PipeAssign,
        "^=" => TokenKind::CaretAssign,
        "<<=" => TokenKind::ShlAssign,
        ">>=" => TokenKind::ShrAssign,
        "." => TokenKind::Dot,
        ".." => TokenKind::DotDot,
        "..." => TokenKind::DotDotDot,
        "->" => TokenKind::Arrow,
        "=>" => TokenKind::FatArrow,
        "<-" => TokenKind::LeftArrow,
        "::" => TokenKind::ColonColon,
        ":" => TokenKind::Colon,
        ";" => TokenKind::Semi,
        "," => TokenKind::Comma,
        "?" => TokenKind::Question,
        "|>" => TokenKind::PipeRight,
        "<|" => TokenKind::PipeLeft,
        "@" => TokenKind::At,
        "#" => TokenKind::Hash,
        "$" => TokenKind::Dollar,
        "~" => TokenKind::Tilde,
        "`" => TokenKind::Backtick,
        _ => TokenKind::Ident(text.to_string()),
    };

    Token {
        kind,
        span: span_from_pest(file_id, span),
    }
}

fn token_from_punct(file_id: FileId, span: pest::Span, text: &str) -> Token {
    let kind = match text {
        "(" => TokenKind::LParen,
        ")" => TokenKind::RParen,
        "[" => TokenKind::LBracket,
        "]" => TokenKind::RBracket,
        "{" => TokenKind::LBrace,
        "}" => TokenKind::RBrace,
        "|" => TokenKind::Pipe,
        _ => TokenKind::Ident(text.to_string()),
    };

    Token {
        kind,
        span: span_from_pest(file_id, span),
    }
}

fn parse_char(s: &str) -> char {
    // Remove quotes
    let content = &s[1..s.len()-1];
    if content.starts_with('\\') {
        parse_escape(content)
    } else {
        content.chars().next().unwrap()
    }
}

fn parse_string(s: &str, raw: bool) -> String {
    if raw {
        // Raw string: r#"...""#
        let start = s.find('"').unwrap() + 1;
        let end = s.rfind('"').unwrap();
        s[start..end].to_string()
    } else {
        // Regular string with escapes
        let content = &s[1..s.len()-1];
        unescape_string(content)
    }
}

fn parse_byte_string(s: &str, raw: bool) -> Vec<u8> {
    if raw {
        let start = s.find('"').unwrap() + 1;
        let end = s.rfind('"').unwrap();
        s[start..end].as_bytes().to_vec()
    } else {
        let content = &s[2..s.len()-1]; // b"..."
        unescape_bytes(content)
    }
}

fn parse_escape(s: &str) -> char {
    match &s[1..] {
        "n" => '\n',
        "r" => '\r',
        "t" => '\t',
        "\\" => '\\',
        "\"" => '"',
        "'" => '\'',
        "0" => '\0',
        s if s.starts_with('x') => {
            u8::from_str_radix(&s[1..3], 16).unwrap() as char
        }
        s if s.starts_with('u') && s.ends_with('}') => {
            let hex = &s[2..s.len()-1];
            char::from_u32(u32::from_str_radix(hex, 16).unwrap()).unwrap()
        }
        _ => '?',
    }
}

fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                match next {
                    'n' => { chars.next(); result.push('\n'); }
                    'r' => { chars.next(); result.push('\r'); }
                    't' => { chars.next(); result.push('\t'); }
                    '\\' => { chars.next(); result.push('\\'); }
                    '"' => { chars.next(); result.push('"'); }
                    '\'' => { chars.next(); result.push('\''); }
                    '0' => { chars.next(); result.push('\0'); }
                    'x' => {
                        chars.next(); // consume 'x'
                        let hex: String = chars.by_ref().take(2).collect();
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                            result.push(byte as char);
                        }
                    }
                    'u' => {
                        chars.next(); // consume 'u'
                        if chars.next() == Some('{') {
                            let hex: String = chars.by_ref().take_while(|&c| c != '}').collect();
                            chars.next(); // consume '}'
                            if let Ok(code) = u32::from_str_radix(&hex, 16) {
                                if let Some(ch) = char::from_u32(code) {
                                    result.push(ch);
                                }
                            }
                        }
                    }
                    _ => result.push('\\'),
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn unescape_bytes(s: &str) -> Vec<u8> {
    let mut result = Vec::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                match next {
                    'n' => { chars.next(); result.push(b'\n'); }
                    'r' => { chars.next(); result.push(b'\r'); }
                    't' => { chars.next(); result.push(b'\t'); }
                    '\\' => { chars.next(); result.push(b'\\'); }
                    '"' => { chars.next(); result.push(b'"'); }
                    '\'' => { chars.next(); result.push(b'\''); }
                    '0' => { chars.next(); result.push(b'\0'); }
                    'x' => {
                        chars.next();
                        let hex: String = chars.by_ref().take(2).collect();
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                            result.push(byte);
                        }
                    }
                    _ => result.push(b'\\'),
                }
            } else {
                result.push(b'\\');
            }
        } else {
            result.push(c as u8);
        }
    }

    result
}

/// Token stream for parser
pub struct TokenStream {
    tokens: Vec<Token>,
    pos: usize,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub fn peek(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.pos + n)
    }

    pub fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        self.pos += 1;
        token
    }

    pub fn expect(&mut self, kind: TokenKind) -> Result<&Token, String> {
        if let Some(token) = self.current() {
            if std::mem::discriminant(&token.kind) == std::mem::discriminant(&kind) {
                return Ok(self.advance().unwrap());
            }
        }
        Err(format!("Expected {:?}, found {:?}", kind, self.current().map(|t| &t.kind)))
    }

    pub fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || matches!(self.current(), Some(t) if matches!(t.kind, TokenKind::Eof))
    }

    pub fn span(&self) -> Span {
        self.current().map(|t| t.span).unwrap_or(Span::new(FileId(0), 0, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_simple() {
        let mut session = Session::new(crate::CompileConfig {
            target: crate::Target::Native,
            opt_level: crate::OptLevel::None,
            debug_info: false,
            test_mode: false,
            doc_mode: false,
            edition: crate::Edition::Nova2026,
        });

        let source = r#"
fn main() {
    let x = 42;
    println!("Hello, {}!", x);
}
"#;

        let tokens = lex(&mut session, FileId(0), source).unwrap();
        assert!(!tokens.is_empty());
        assert!(matches!(tokens.last().unwrap().kind, TokenKind::Eof));
    }

    #[test]
    fn test_lex_keywords() {
        let mut session = Session::new(crate::CompileConfig {
            target: crate::Target::Native,
            opt_level: crate::OptLevel::None,
            debug_info: false,
            test_mode: false,
            doc_mode: false,
            edition: crate::Edition::Nova2026,
        });

        let source = "fn let mut const async await match if else loop for while break continue return";
        let tokens = lex(&mut session, FileId(0), source).unwrap();

        let keywords: Vec<_> = tokens.iter()
            .filter(|t| t.kind.is_keyword())
            .map(|t| &t.kind)
            .collect();

        assert_eq!(keywords.len(), 18);
    }

    #[test]
    fn test_lex_literals() {
        let mut session = Session::new(crate::CompileConfig {
            target: crate::Target::Native,
            opt_level: crate::OptLevel::None,
            debug_info: false,
            test_mode: false,
            doc_mode: false,
            edition: crate::Edition::Nova2026,
        });

        let source = r#"42 3.14 'a' "hello" true false ()"#;
        let tokens = lex(&mut session, FileId(0), source).unwrap();

        let literals: Vec<_> = tokens.iter()
            .filter(|t| matches!(t.kind, TokenKind::Integer(_) | TokenKind::Float(_) | TokenKind::Char(_) | TokenKind::String(_) | TokenKind::Bool(_) | TokenKind::Unit))
            .collect();

        assert_eq!(literals.len(), 8);
    }
}