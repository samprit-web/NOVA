//! NOVA Compiler Frontend
//!
//! This crate contains the lexer, parser, AST, HIR, MIR, and diagnostics for the NOVA language.

pub mod ast;
pub mod diagnostics;
pub mod hir;
pub mod lexer;
pub mod mir;
pub mod parser;
pub mod parser_gen;
pub mod codegen;
pub mod optimizer;
pub mod vm;

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use nova_diagnostics::{Diagnostic, DiagnosticCollection, Severity};

/// Compilation session containing all state needed for compilation
pub struct Session {
    pub source_map: SourceMap,
    pub diagnostics: DiagnosticCollection,
    pub config: CompileConfig,
}

/// Configuration for compilation
#[derive(Debug, Clone)]
pub struct CompileConfig {
    pub target: Target,
    pub opt_level: OptLevel,
    pub debug_info: bool,
    pub test_mode: bool,
    pub doc_mode: bool,
    pub edition: Edition,
}

/// Target compilation backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Native,
    Wasm32,
    Bytecode,
    Jvm,
    Dotnet,
    C,
}

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptLevel {
    None = 0,
    Basic = 1,
    Standard = 2,
    Full = 3,
    Size = 4,
}

/// Language edition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edition {
    Nova2024,
    Nova2026,
}

/// Source map for tracking source locations
pub struct SourceMap {
    files: Vec<SourceFile>,
}

pub struct SourceFile {
    pub path: std::path::PathBuf,
    pub content: String,
    pub lines: Vec<usize>, // Byte offset of each line start
}

impl Session {
    pub fn new(config: CompileConfig) -> Self {
        Self {
            source_map: SourceMap::new(),
            diagnostics: DiagnosticCollection::new(),
            config,
        }
    }

    pub fn add_file(&mut self, path: impl AsRef<Path>, content: String) -> FileId {
        self.source_map.add_file(path, content)
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }
}

impl SourceMap {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn add_file(&mut self, path: impl AsRef<Path>, content: String) -> FileId {
        let lines = Self::compute_lines(&content);
        let id = FileId(self.files.len() as u32);
        self.files.push(SourceFile {
            path: path.as_ref().to_path_buf(),
            content,
            lines,
        });
        id
    }

    fn compute_lines(content: &str) -> Vec<usize> {
        let mut lines = vec![0];
        for (i, c) in content.char_indices() {
            if c == '\n' {
                lines.push(i + 1);
            }
        }
        lines
    }

    pub fn get_file(&self, id: FileId) -> Option<&SourceFile> {
        self.files.get(id.0 as usize)
    }

    pub fn span_to_location(&self, span: Span) -> Option<Location> {
        let file = self.get_file(span.file_id)?;
        let line = file.lines.binary_search(&span.start).unwrap_or_else(|i| i - 1);
        let col = span.start - file.lines[line];
        Some(Location {
            file: span.file_id,
            line: line + 1,
            column: col + 1,
        })
    }
}

/// File identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(pub u32);

/// Source code span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub file_id: FileId,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(file_id: FileId, start: usize, end: usize) -> Self {
        Self { file_id, start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        assert_eq!(self.file_id, other.file_id);
        Span {
            file_id: self.file_id,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// Source location (line/column)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub file: FileId,
    pub line: usize,
    pub column: usize,
}

/// Compilation entry point
pub fn compile(source: &str, config: CompileConfig) -> Result<CompilationOutput> {
    let mut session = Session::new(config);
    let file_id = session.add_file("<input>", source.to_string());

    // Lex
    let tokens = crate::lexer::lex(&mut session, file_id, source)?;

    // Parse
    let ast = crate::parser::parse(&mut session, file_id, &tokens)?;

    // Lower to HIR
    let hir = crate::hir::lower(&mut session, ast)?;

    // Type check and lower to MIR
    let mir = crate::mir::lower(&mut session, hir)?;

    // Optimize
    let mir = crate::optimizer::optimize(&mut session, mir)?;

    // Codegen
    let output = crate::codegen::codegen(&mut session, mir)?;

    Ok(output)
}

/// Compilation output artifact
pub struct CompilationOutput {
    pub artifact: Artifact,
    pub diagnostics: DiagnosticCollection,
}

/// Compilation artifact
pub enum Artifact {
    Native(Vec<u8>),           // Native binary
    Wasm(Vec<u8>),             // WebAssembly module
    Bytecode(BytecodeModule),  // NOVA bytecode
    LlvmIr(String),            // LLVM IR
    Assembly(String),          // Assembly output
    Object(Vec<u8>),           // Object file
}

/// NOVA bytecode module
pub struct BytecodeModule {
    pub version: u16,
    pub functions: Vec<BytecodeFunction>,
    pub constants: ConstantPool,
    pub types: TypeTable,
    pub globals: Vec<GlobalVar>,
    pub debug_info: Option<DebugInfo>,
}

pub struct BytecodeFunction {
    pub name: String,
    pub params: Vec<Type>,
    pub return_type: Type,
    pub locals: Vec<Type>,
    pub instructions: Vec<Instruction>,
    pub exception_handlers: Vec<ExceptionHandler>,
}

pub struct ConstantPool {
    pub integers: Vec<i64>,
    pub floats: Vec<f64>,
    pub strings: Vec<String>,
    pub bytes: Vec<Vec<u8>>,
}

pub struct TypeTable {
    pub types: Vec<Type>,
}

pub struct GlobalVar {
    pub name: String,
    pub ty: Type,
    pub init: Option<Constant>,
    pub mutable: bool,
}

pub struct DebugInfo {
    pub source_files: Vec<String>,
    pub line_info: Vec<LineInfo>,
    pub scopes: Vec<ScopeInfo>,
}

pub struct LineInfo {
    pub instruction_offset: usize,
    pub file_id: usize,
    pub line: u32,
    pub column: u32,
}

pub struct ScopeInfo {
    pub function: usize,
    pub start_instruction: usize,
    pub end_instruction: usize,
    pub variables: Vec<VariableInfo>,
}

pub struct VariableInfo {
    pub name: String,
    pub ty: Type,
    pub location: VariableLocation,
}

pub enum VariableLocation {
    Local(u32),
    Upvalue(u32),
    Global(u32),
}

pub struct ExceptionHandler {
    pub try_start: usize,
    pub try_end: usize,
    pub catch_start: usize,
    pub catch_type: Option<Type>,
}

/// NOVA bytecode instructions
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // Stack manipulation
    PushConst(u32),        // Push constant from pool
    PushLocal(u32),        // Push local variable
    PushGlobal(u32),       // Push global variable
    Pop,                   // Pop and discard
    Dup,                   // Duplicate top
    Swap,                  // Swap top two

    // Arithmetic
    Add, Sub, Mul, Div, Rem,
    Neg, Not,
    And, Or, Xor,
    Shl, Shr, Rotl, Rotr,

    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,

    // Control flow
    Jump(u32),             // Unconditional jump
    JumpIf(u32),           // Jump if true
    JumpIfNot(u32),        // Jump if false
    Call(u32),             // Call function by index
    CallIndirect(u32),     // Call function pointer
    Return,                // Return from function
    TailCall(u32),         // Tail call

    // Memory
    Load(u32),             // Load from local
    Store(u32),            // Store to local
    LoadGlobal(u32),       // Load from global
    StoreGlobal(u32),      // Store to global
    Alloc(u32),            // Allocate object (type index)
    Free,                  // Free object

    // References
    Ref,                   // Create reference
    Deref,                 // Dereference
    Borrow,                // Borrow immutable
    BorrowMut,             // Borrow mutable

    // Struct/Enum
    MakeStruct(u32),       // Create struct (type index)
    MakeEnum(u32, u32),    // Create enum variant (type index, variant index)
    GetField(u32),         // Get struct field (field index)
    SetField(u32),         // Set struct field
    GetVariant,            // Get enum variant tag
    MatchEnum(u32),        // Match enum (jump table index)

    // Arrays/Slices
    NewArray(u32),         // New array (type index)
    NewSlice(u32),         // New slice
    ArrayLen,              // Get array/slice length
    ArrayGet,              // Array get (index on stack)
    ArraySet,              // Array set (index, value on stack)

    // Async
    Await,                 // Await future
    Yield,                 // Yield execution
    Spawn,                 // Spawn task

    // Exception handling
    Try(u32),              // Start try block (handler index)
    EndTry,                // End try block
    Throw,                 // Throw exception
    Rethrow,               // Rethrow current exception

    // Intrinsics
    Intrinsic(u32),        // Call intrinsic (index)

    // Debug
    Nop,                   // No operation
    Line(u32),             // Line number for debugging
}

/// NOVA types (simplified for bytecode)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Unit,
    Bool,
    Char,
    I8, I16, I32, I64, I128, ISize,
    U8, U16, U32, U64, U128, USize,
    F16, F32, F64, F128,
    Never,
    Ptr(Box<Type>),           // *const T / *mut T
    Ref(Box<Type>, Mutability), // &T / &mut T
    Box(Box<Type>),           // Box<T>
    Rc(Box<Type>),            // Rc<T>
    Arc(Box<Type>),           // Arc<T>
    Fn(Vec<Type>, Box<Type>), // fn(...) -> T
    Tuple(Vec<Type>),
    Array(Box<Type>, u32),    // [T; N]
    Slice(Box<Type>),         // [T]
    Struct(u32),              // Struct type index
    Enum(u32),                // Enum type index
    TraitObject(Vec<TraitBound>), // dyn Trait
    TypeParam(u32),           // Generic parameter
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mutability {
    Immutable,
    Mutable,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitBound {
    pub trait_id: u32,
    pub params: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Unit,
    Bool(bool),
    Char(char),
    Integer(i128),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<Constant>),
    Struct(Vec<Constant>),
    Enum(u32, Vec<Constant>), // variant index, fields
}

/// Main compiler API
pub struct Compiler {
    session: Session,
}

impl Compiler {
    pub fn new(config: CompileConfig) -> Self {
        Self {
            session: Session::new(config),
        }
    }

    pub fn compile_file(&mut self, path: impl AsRef<Path>) -> Result<CompilationOutput> {
        let content = std::fs::read_to_string(&path)?;
        let file_id = self.session.add_file(path, content.clone());
        self.compile_source(file_id, &content)
    }

    pub fn compile_source(&mut self, file_id: FileId, source: &str) -> Result<CompilationOutput> {
        // Lex
        let tokens = crate::lexer::lex(&mut self.session, file_id, source)?;

        // Parse
        let ast = crate::parser::parse(&mut self.session, file_id, &tokens)?;

        // Lower to HIR
        let hir = crate::hir::lower(&mut self.session, ast)?;

        // Type check and lower to MIR
        let mir = crate::mir::lower(&mut self.session, hir)?;

        // Optimize
        let mir = crate::optimizer::optimize(&mut self.session, mir)?;

        // Codegen
        let output = crate::codegen::codegen(&mut self.session, mir)?;

        Ok(output)
    }

    pub fn diagnostics(&self) -> &DiagnosticCollection {
        &self.session.diagnostics
    }

    pub fn has_errors(&self) -> bool {
        self.session.has_errors()
    }
}

// Re-export key types
pub use ast::*;
pub use diagnostics::*;
pub use hir::*;
pub use mir::*;
pub use lexer::*;
pub use parser::*;