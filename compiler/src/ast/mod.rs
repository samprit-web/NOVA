//! NOVA Abstract Syntax Tree (AST)

use std::sync::Arc;

use crate::{Span, FileId, Type};
use serde::{Deserialize, Serialize};

/// Unique identifier for AST nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

impl NodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Root AST node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

/// Item definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Item {
    Use(UseDecl),
    Module(ModuleDecl),
    ExternCrate(ExternCrateDecl),
    Function(FunctionDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    TypeAlias(TypeAliasDecl),
    Trait(TraitDecl),
    Impl(ImplDecl),
    Const(ConstDecl),
    Static(StaticDecl),
    Macro(MacroDecl),
    MacroRules(MacroRulesDecl),
    Attribute(AttributeItem),
    // NOVA Unique Features
    Effect(EffectDecl),
    EffectHandler(EffectHandlerDecl),
    Actor(ActorDecl),
    Comptime(ComptimeDecl),
    Pipeline(PipelineDecl),
}

/// Use declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseDecl {
    pub tree: UseTree,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UseTree {
    Simple(Path),
    Glob(Path),
    Group(Path, Vec<UseTree>),
    Self_,
    Super,
    Crate,
}

/// Module declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDecl {
    pub visibility: Visibility,
    pub name: Ident,
    pub body: Option<Vec<Item>>, // None = external module
    pub span: Span,
}

/// Extern crate declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternCrateDecl {
    pub name: Ident,
    pub alias: Option<Ident>,
    pub span: Span,
}

/// Function declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub constness: Constness,
    pub asyncness: Asyncness,
    pub unsafety: Unsafety,
    pub name: Ident,
    pub generics: Generics,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub where_clause: Option<WhereClause>,
    pub body: Option<Block>, // None = external/abstract
    pub span: Span,
}

/// Struct declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub generics: Generics,
    pub where_clause: Option<WhereClause>,
    pub kind: StructKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructKind {
    Record(Vec<StructField>),
    Tuple(Vec<TypeExpr>),
    Unit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub ty: TypeExpr,
    pub span: Span,
}

/// Enum declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub generics: Generics,
    pub where_clause: Option<WhereClause>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariant {
    pub attrs: Vec<Attribute>,
    pub name: Ident,
    pub kind: VariantKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariantKind {
    Unit,
    Tuple(Vec<TypeExpr>),
    Record(Vec<StructField>),
}

/// Type alias
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeAliasDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub generics: Generics,
    pub ty: TypeExpr,
    pub span: Span,
}

/// Trait declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub unsafety: Unsafety,
    pub name: Ident,
    pub generics: Generics,
    pub where_clause: Option<WhereClause>,
    pub items: Vec<TraitItem>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraitItem {
    Function(TraitFn),
    Type(TraitType),
    Const(TraitConst),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitFn {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub constness: Constness,
    pub asyncness: Asyncness,
    pub name: Ident,
    pub generics: Generics,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub where_clause: Option<WhereClause>,
    pub body: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitType {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub generics: Generics,
    pub bounds: Option<GenericBounds>,
    pub default: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitConst {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub ty: TypeExpr,
    pub default: Option<Expr>,
    pub span: Span,
}

/// Impl declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub unsafety: Unsafety,
    pub generics: Generics,
    pub trait_ref: Option<TraitRef>,
    pub self_ty: TypeExpr,
    pub where_clause: Option<WhereClause>,
    pub items: Vec<ImplItem>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplItem {
    Function(FunctionDecl),
    Type(TypeAliasDecl),
    Const(ConstDecl),
}

/// Const declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub ty: TypeExpr,
    pub value: Expr,
    pub span: Span,
}

/// Static declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub mutability: Mutability,
    pub name: Ident,
    pub ty: TypeExpr,
    pub value: Expr,
    pub span: Span,
}

/// Macro declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub params: Vec<MacroParam>,
    pub rules: MacroRules,
    pub span: Span,
}

/// Macro rules declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroRulesDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub rules: MacroRules,
    pub span: Span,
}

/// Attribute item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeItem {
    pub attr: Attribute,
    pub span: Span,
}

/// Effect declaration (NOVA unique feature - Algebraic Effects)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub generics: Generics,
    pub operations: Vec<EffectOperation>,
    pub span: Span,
}

/// Effect operation signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectOperation {
    pub attrs: Vec<Attribute>,
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub span: Span,
}

/// Effect handler declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectHandlerDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub effect: Path,
    pub generics: Generics,
    pub handlers: Vec<EffectHandlerArm>,
    pub span: Span,
}

/// Effect handler arm (handles one operation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectHandlerArm {
    pub attrs: Vec<Attribute>,
    pub operation: Ident,
    pub params: Vec<Param>,
    pub body: Block,
    pub span: Span,
}

/// Actor declaration (NOVA unique feature - Actor Model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub generics: Generics,
    pub fields: Vec<StructField>,
    pub methods: Vec<FunctionDecl>,
    pub supervision: Option<SupervisionSpec>,
    pub span: Span,
}

/// Supervision specification for actors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionSpec {
    pub strategy: SupervisionStrategy,
    pub max_restarts: Option<u32>,
    pub time_window: Option<DurationExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupervisionStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
}

/// Duration expression for supervision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationExpr {
    pub value: u64,
    pub unit: TimeUnit,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeUnit {
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
}

/// Comptime declaration (NOVA unique feature - Compile-time execution)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComptimeDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub kind: ComptimeKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComptimeKind {
    /// comptime { ... } block
    Block(Block),
    /// comptime fn name() { ... }
    Function(FunctionDecl),
    /// comptime const NAME = expr
    Const(Ident, TypeExpr, Expr),
    /// comptime static NAME = expr
    Static(Ident, TypeExpr, Expr),
}

/// Pipeline declaration (NOVA unique feature - Pipeline operators)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub generics: Generics,
    pub stages: Vec<PipelineStage>,
    pub span: Span,
}

/// Pipeline stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub attrs: Vec<Attribute>,
    pub kind: PipelineStageKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineStageKind {
    /// data.filter(...)
    Filter(Expr),
    /// data.map(...)
    Map(Expr),
    /// data.flat_map(...)
    FlatMap(Expr),
    /// data.fold(init, ...)
    Fold(Expr, Expr),
    /// data.reduce(...)
    Reduce(Expr),
    /// data.collect()
    Collect(Option<TypeExpr>),
    /// data.take(n)
    Take(Expr),
    /// data.skip(n)
    Skip(Expr),
    /// Custom stage
    Custom(Ident, Vec<Expr>),
}

/// Common types

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Private,
    Public,
    Crate,
    Super,
    Self_,
    In(Path),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Constness {
    Const,
    NotConst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Asyncness {
    Async,
    NotAsync,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unsafety {
    Unsafe,
    NotUnsafe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mutability {
    Mutable,
    Immutable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ident {
    pub name: String,
    pub raw: bool,
    pub span: Span,
}

impl Ident {
    pub fn new(name: String, span: Span) -> Self {
        Self { name, raw: false, span }
    }

    pub fn raw(name: String, span: Span) -> Self {
        Self { name, raw: true, span }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generics {
    pub params: Vec<GenericParam>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericParam {
    pub attrs: Vec<Attribute>,
    pub name: Ident,
    pub kind: GenericParamKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenericParamKind {
    Type { bounds: Option<GenericBounds>, default: Option<TypeExpr> },
    Const { ty: TypeExpr, default: Option<Expr> },
    Lifetime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericBounds {
    pub bounds: Vec<GenericBound>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericBound {
    pub trait_ref: TraitRef,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhereClause {
    pub predicates: Vec<WherePredicate>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WherePredicate {
    pub ty: TypeExpr,
    pub bounds: GenericBounds,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub attrs: Vec<Attribute>,
    pub mutability: Mutability,
    pub name: Ident,
    pub ty: TypeExpr,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitRef {
    pub path: Path,
    pub generics: Option<GenericArgs>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericArgs {
    pub args: Vec<GenericArg>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenericArg {
    Type(TypeExpr),
    Const(Expr),
    Lifetime(Lifetime),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lifetime {
    pub name: Ident,
    pub span: Span,
}

/// Type expressions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeExpr {
    pub kind: TypeExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeExprKind {
    Tuple(Vec<TypeExpr>),
    Array(Box<TypeExpr>, Box<Expr>), // [T; N]
    Slice(Box<TypeExpr>),
    Ref(Mutability, Box<TypeExpr>),
    Box(Box<TypeExpr>),
    Fn(Vec<TypeExpr>, Box<TypeExpr>),
    TraitObject(GenericBounds),
    ImplTrait(GenericBounds),
    Path(Path),
    Paren(Box<TypeExpr>),
    Never,
    Infer, // _
    Macro(MacroInvocation),
}

/// Expressions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
    pub attrs: Vec<Attribute>,
    pub node_id: NodeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExprKind {
    // Literals
    Literal(Literal),
    // Paths
    Path(Path),
    // Blocks
    Block(Block),
    // Control flow
    If(Box<Expr>, Block, Option<Box<Expr>>), // condition, then, else
    Match(Box<Expr>, Vec<MatchArm>),
    Loop(Block, Option<Ident>), // body, label
    While(Box<Expr>, Block, Option<Ident>),
    For(Pattern, Box<Expr>, Block, Option<Ident>),
    Try(Box<Expr>, Vec<CatchClause>, Option<Block>),
    // Jumps
    Return(Option<Box<Expr>>),
    Break(Option<Ident>, Option<Box<Expr>>),
    Continue(Option<Ident>),
    Yield(Option<Box<Expr>>),
    // Assignments
    Assign(Box<Expr>, AssignOp, Box<Expr>),
    // Binary ops
    Binary(Box<Expr>, BinOp, Box<Expr>),
    // Unary ops
    Unary(UnOp, Box<Expr>),
    // Postfix
    Call(Box<Expr>, Vec<Expr>),
    MethodCall(Box<Expr>, Ident, Option<GenericArgs>, Vec<Expr>),
    Field(Box<Expr>, Ident),
    Index(Box<Expr>, Box<Expr>),
    TryOperator(Box<Expr>),
    Range(Box<Expr>, RangeEnd, Box<Expr>),
    PathSegment(Box<Expr>, Ident, Option<GenericArgs>),
    Await(Box<Expr>),
    Cast(Box<Expr>, TypeExpr),
    // Struct/Enum construction
    Struct(Path, Vec<FieldExpr>),
    Tuple(Vec<Expr>),
    Array(Vec<Expr>),
    // Closures
    Closure(ClosureExpr),
    // Async
    Async(Block),
    // Macros
    Macro(MacroInvocation),
    // Paren
    Paren(Box<Expr>),
    // NOVA Unique Features - Effect System
    EffectPerform(Path, Vec<Expr>),                    // perform Effect::operation(args)
    EffectHandle(Box<Expr>, Vec<EffectHandlerArm>),    // handle expr with { ... }
    // NOVA Unique Features - Actor Model
    ActorSpawn(Box<Expr>),                             // actor_expr.spawn()
    ActorSend(Box<Expr>, Box<Expr>),                   // actor_ref.send(msg)
    ActorCall(Box<Expr>, Ident, Vec<Expr>),            // actor_ref.method(args)
    ActorSupervise(Box<Expr>, SupervisionSpec),        // supervise(actor, spec)
    // NOVA Unique Features - Comptime
    Comptime(Box<Expr>),                               // comptime { expr }
    ComptimeEval(Box<Expr>),                           // @comptime(expr)
    // NOVA Unique Features - Pipeline Operators
    Pipeline(Box<Expr>, Vec<PipelineStage>),           // data |> stage1 |> stage2
    PipelineStageExpr(PipelineStageKind, Vec<Expr>),   // .filter(...), .map(...), etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Literal {
    pub kind: LiteralKind,
    pub suffix: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiteralKind {
    Integer(String),
    Float(String),
    Char(char),
    String(String),
    RawString(String),
    ByteString(Vec<u8>),
    RawByteString(Vec<u8>),
    Bool(bool),
    Unit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub segments: Vec<PathSegment>,
    pub span: Span,
    pub global: bool, // starts with ::
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSegment {
    pub ident: Ident,
    pub generics: Option<GenericArgs>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub expr: Option<Box<Expr>>,
    pub span: Span,
    pub labels: Vec<Ident>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatchClause {
    pub pattern: Pattern,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    Wildcard(Span),
    Literal(Literal),
    Binding(Ident, Option<Box<Pattern>>), // name, @ subpattern
    Tuple(Vec<Pattern>),
    Struct(Path, Vec<FieldPattern>),
    Enum(Path, Ident, Option<Vec<Pattern>>), // path, variant, fields
    Slice(Vec<Pattern>, Option<Box<Pattern>>), // prefix, ..rest
    Or(Vec<Pattern>),
    Ref(Mutability, Box<Pattern>),
    Range(Box<Expr>, RangeEnd, Box<Expr>),
    Guard(Box<Pattern>, Box<Expr>),
    Macro(MacroInvocation),
    // NOVA Unique Features
    EffectBind(Ident, Path, Ident, Vec<Pattern>),  // op args => handler
    ActorPattern(Path, Ident, Option<Vec<Pattern>>), // Actor::Variant(fields)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldPattern {
    pub name: Ident,
    pub pattern: Option<Pattern>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RangeEnd {
    Exclusive, // ..
    Inclusive, // ..=
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldExpr {
    pub name: Ident,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    ShlAssign,
    ShrAssign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
    // Type
    As,
    Is,
    In,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnOp {
    Neg,
    Not,
    BitNot,
    Deref,
    Ref,
    RefMut,
    Box,
    Try,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureExpr {
    pub params: Vec<ClosureParam>,
    pub return_type: Option<TypeExpr>,
    pub body: Block,
    pub asyncness: Asyncness,
    pub movability: Movability,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureParam {
    pub attrs: Vec<Attribute>,
    pub name: Ident,
    pub ty: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Movability {
    Move,
    Copy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroInvocation {
    pub path: Path,
    pub bang: bool, // ! vs ?
    pub args: MacroArgs,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroArgs {
    Paren(Vec<TokenTree>),
    Bracket(Vec<TokenTree>),
    Brace(Vec<TokenTree>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenTree {
    Ident(Ident),
    Literal(Literal),
    Punct(char),
    Group(Delimiter, Vec<TokenTree>),
    Macro(MacroInvocation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Delimiter {
    Paren,
    Bracket,
    Brace,
    None,
}

/// Statements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
    pub attrs: Vec<Attribute>,
    pub node_id: NodeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StmtKind {
    Let(LetStmt),
    Item(Item),
    Expr(Expr),
    Macro(MacroInvocation),
    Empty,
    // NOVA Unique Features
    EffectHandle(Box<Expr>, Vec<EffectHandlerArm>),    // handle expr with { ... }
    ActorSpawn(Box<Expr>),                             // spawn actor
    Comptime(Block),                                   // comptime { ... }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LetStmt {
    pub mutability: Mutability,
    pub pattern: Pattern,
    pub ty: Option<TypeExpr>,
    pub initializer: Option<Expr>,
    pub span: Span,
}

/// Attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub style: AttrStyle,
    pub path: Path,
    pub args: Option<Vec<MetaItem>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttrStyle {
    Outer,  // #[attr]
    Inner,  // #![attr]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetaItem {
    Path(Path),
    Literal(Literal),
    NameValue(Path, Expr),
    List(Path, Vec<MetaItem>),
}

/// Macro definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroParam {
    pub name: Ident,
    pub fragment: MacroFragmentSpec,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroFragmentSpec {
    Expr,
    Stmt,
    Item,
    Type,
    Pat,
    Path,
    Ty,
    Ident,
    Literal,
    Block,
    Meta,
    Tt,
    Vis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroRules {
    pub rules: Vec<MacroRule>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroRule {
    pub pattern: Vec<TokenTree>,
    pub body: Vec<TokenTree>,
    pub span: Span,
}

/// Visitor pattern for AST traversal
pub trait AstVisitor {
    fn visit_program(&mut self, program: &Program) {
        walk_program(self, program);
    }

    fn visit_item(&mut self, item: &Item) {
        walk_item(self, item);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        walk_expr(self, expr);
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        walk_stmt(self, stmt);
    }

    fn visit_pattern(&mut self, pattern: &Pattern) {
        walk_pattern(self, pattern);
    }

    fn visit_type(&mut self, ty: &TypeExpr) {
        walk_type(self, ty);
    }
}

pub fn walk_program<V: AstVisitor>(visitor: &mut V, program: &Program) {
    for item in &program.items {
        visitor.visit_item(item);
    }
}

pub fn walk_item<V: AstVisitor>(visitor: &mut V, item: &Item) {
    match item {
        Item::Use(u) => walk_use(visitor, u),
        Item::Module(m) => walk_module(visitor, m),
        Item::Function(f) => walk_function(visitor, f),
        Item::Struct(s) => walk_struct(visitor, s),
        Item::Enum(e) => walk_enum(visitor, e),
        Item::TypeAlias(t) => walk_type_alias(visitor, t),
        Item::Trait(t) => walk_trait(visitor, t),
        Item::Impl(i) => walk_impl(visitor, i),
        Item::Const(c) => walk_const(visitor, c),
        Item::Static(s) => walk_static(visitor, s),
        Item::Macro(m) => walk_macro(visitor, m),
        Item::MacroRules(m) => walk_macro_rules(visitor, m),
        Item::Attribute(a) => walk_attribute(visitor, a),
        _ => {}
    }
}

pub fn walk_use<V: AstVisitor>(visitor: &mut V, use_decl: &UseDecl) {
    walk_use_tree(visitor, &use_decl.tree);
}

pub fn walk_use_tree<V: AstVisitor>(visitor: &mut V, tree: &UseTree) {
    match tree {
        UseTree::Simple(path) | UseTree::Glob(path) => walk_path(visitor, path),
        UseTree::Group(path, trees) => {
            walk_path(visitor, path);
            for t in trees {
                walk_use_tree(visitor, t);
            }
        }
        _ => {}
    }
}

pub fn walk_module<V: AstVisitor>(visitor: &mut V, module: &ModuleDecl) {
    if let Some(body) = &module.body {
        for item in body {
            visitor.visit_item(item);
        }
    }
}

pub fn walk_function<V: AstVisitor>(visitor: &mut V, func: &FunctionDecl) {
    walk_generics(visitor, &func.generics);
    for param in &func.params {
        walk_param(visitor, param);
    }
    if let Some(ret) = &func.return_type {
        visitor.visit_type(ret);
    }
    if let Some(where_clause) = &func.where_clause {
        walk_where_clause(visitor, where_clause);
    }
    if let Some(body) = &func.body {
        visitor.visit_expr(&Expr {
            kind: ExprKind::Block(body.clone()),
            span: body.span,
            attrs: Vec::new(),
            node_id: NodeId(0),
        });
    }
}

pub fn walk_generics<V: AstVisitor>(visitor: &mut V, generics: &Generics) {
    for param in &generics.params {
        match &param.kind {
            GenericParamKind::Type { bounds, default } => {
                if let Some(b) = bounds {
                    walk_generic_bounds(visitor, b);
                }
                if let Some(d) = default {
                    visitor.visit_type(d);
                }
            }
            GenericParamKind::Const { ty, default } => {
                visitor.visit_type(ty);
                if let Some(d) = default {
                    visitor.visit_expr(d);
                }
            }
            GenericParamKind::Lifetime => {}
        }
    }
}

pub fn walk_generic_bounds<V: AstVisitor>(visitor: &mut V, bounds: &GenericBounds) {
    for bound in &bounds.bounds {
        walk_trait_ref(visitor, &bound.trait_ref);
    }
}

pub fn walk_trait_ref<V: AstVisitor>(visitor: &mut V, trait_ref: &TraitRef) {
    walk_path(visitor, &trait_ref.path);
    if let Some(args) = &trait_ref.generics {
        walk_generic_args(visitor, args);
    }
}

pub fn walk_generic_args<V: AstVisitor>(visitor: &mut V, args: &GenericArgs) {
    for arg in &args.args {
        match arg {
            GenericArg::Type(ty) => visitor.visit_type(ty),
            GenericArg::Const(expr) => visitor.visit_expr(expr),
            GenericArg::Lifetime(_) => {}
        }
    }
}

pub fn walk_where_clause<V: AstVisitor>(visitor: &mut V, where_clause: &WhereClause) {
    for pred in &where_clause.predicates {
        visitor.visit_type(&pred.ty);
        walk_generic_bounds(visitor, &pred.bounds);
    }
}

pub fn walk_param<V: AstVisitor>(visitor: &mut V, param: &Param) {
    visitor.visit_type(&param.ty);
}

pub fn walk_struct<V: AstVisitor>(visitor: &mut V, struct_decl: &StructDecl) {
    walk_generics(visitor, &struct_decl.generics);
    if let Some(where_clause) = &struct_decl.where_clause {
        walk_where_clause(visitor, where_clause);
    }
    match &struct_decl.kind {
        StructKind::Record(fields) => {
            for field in fields {
                visitor.visit_type(&field.ty);
            }
        }
        StructKind::Tuple(types) => {
            for ty in types {
                visitor.visit_type(ty);
            }
        }
        StructKind::Unit => {}
    }
}

pub fn walk_enum<V: AstVisitor>(visitor: &mut V, enum_decl: &EnumDecl) {
    walk_generics(visitor, &enum_decl.generics);
    if let Some(where_clause) = &enum_decl.where_clause {
        walk_where_clause(visitor, where_clause);
    }
    for variant in &enum_decl.variants {
        match &variant.kind {
            VariantKind::Unit => {}
            VariantKind::Tuple(types) => {
                for ty in types {
                    visitor.visit_type(ty);
                }
            }
            VariantKind::Record(fields) => {
                for field in fields {
                    visitor.visit_type(&field.ty);
                }
            }
        }
    }
}

pub fn walk_type_alias<V: AstVisitor>(visitor: &mut V, alias: &TypeAliasDecl) {
    walk_generics(visitor, &alias.generics);
    visitor.visit_type(&alias.ty);
}

pub fn walk_trait<V: AstVisitor>(visitor: &mut V, trait_decl: &TraitDecl) {
    walk_generics(visitor, &trait_decl.generics);
    if let Some(where_clause) = &trait_decl.where_clause {
        walk_where_clause(visitor, where_clause);
    }
    for item in &trait_decl.items {
        match item {
            TraitItem::Function(f) => walk_trait_fn(visitor, f),
            TraitItem::Type(t) => walk_trait_type(visitor, t),
            TraitItem::Const(c) => walk_trait_const(visitor, c),
        }
    }
}

pub fn walk_trait_fn<V: AstVisitor>(visitor: &mut V, fn_item: &TraitFn) {
    walk_generics(visitor, &fn_item.generics);
    for param in &fn_item.params {
        walk_param(visitor, param);
    }
    if let Some(ret) = &fn_item.return_type {
        visitor.visit_type(ret);
    }
    if let Some(where_clause) = &fn_item.where_clause {
        walk_where_clause(visitor, where_clause);
    }
    if let Some(body) = &fn_item.body {
        visitor.visit_expr(&Expr {
            kind: ExprKind::Block(body.clone()),
            span: body.span,
            attrs: Vec::new(),
            node_id: NodeId(0),
        });
    }
}

pub fn walk_trait_type<V: AstVisitor>(visitor: &mut V, type_item: &TraitType) {
    walk_generics(visitor, &type_item.generics);
    if let Some(bounds) = &type_item.bounds {
        walk_generic_bounds(visitor, bounds);
    }
    if let Some(default) = &type_item.default {
        visitor.visit_type(default);
    }
}

pub fn walk_trait_const<V: AstVisitor>(visitor: &mut V, const_item: &TraitConst) {
    visitor.visit_type(&const_item.ty);
    if let Some(default) = &const_item.default {
        visitor.visit_expr(default);
    }
}

pub fn walk_impl<V: AstVisitor>(visitor: &mut V, impl_decl: &ImplDecl) {
    walk_generics(visitor, &impl_decl.generics);
    if let Some(trait_ref) = &impl_decl.trait_ref {
        walk_trait_ref(visitor, trait_ref);
    }
    visitor.visit_type(&impl_decl.self_ty);
    if let Some(where_clause) = &impl_decl.where_clause {
        walk_where_clause(visitor, where_clause);
    }
    for item in &impl_decl.items {
        match item {
            ImplItem::Function(f) => walk_function(visitor, f),
            ImplItem::Type(t) => walk_type_alias(visitor, t),
            ImplItem::Const(c) => walk_const(visitor, c),
        }
    }
}

pub fn walk_const<V: AstVisitor>(visitor: &mut V, const_decl: &ConstDecl) {
    visitor.visit_type(&const_decl.ty);
    visitor.visit_expr(&const_decl.value);
}

pub fn walk_static<V: AstVisitor>(visitor: &mut V, static_decl: &StaticDecl) {
    visitor.visit_type(&static_decl.ty);
    visitor.visit_expr(&static_decl.value);
}

pub fn walk_macro<V: AstVisitor>(visitor: &mut V, macro_decl: &MacroDecl) {
    for param in &macro_decl.params {
        // Fragment specs don't contain AST nodes
    }
    walk_macro_rules(visitor, &macro_decl.rules);
}

pub fn walk_macro_rules<V: AstVisitor>(visitor: &mut V, rules: &MacroRules) {
    for rule in &rules.rules {
        // Token trees don't contain AST nodes in this simplified version
    }
}

pub fn walk_attribute<V: AstVisitor>(visitor: &mut V, attr: &AttributeItem) {
    walk_path(visitor, &attr.attr.path);
    if let Some(args) = &attr.attr.args {
        for arg in args {
            walk_meta_item(visitor, arg);
        }
    }
}

pub fn walk_meta_item<V: AstVisitor>(visitor: &mut V, item: &MetaItem) {
    match item {
        MetaItem::Path(p) => walk_path(visitor, p),
        MetaItem::Literal(_) => {}
        MetaItem::NameValue(p, e) => {
            walk_path(visitor, p);
            visitor.visit_expr(e);
        }
        MetaItem::List(p, items) => {
            walk_path(visitor, p);
            for item in items {
                walk_meta_item(visitor, item);
            }
        }
    }
}

pub fn walk_path<V: AstVisitor>(visitor: &mut V, path: &Path) {
    for segment in &path.segments {
        if let Some(args) = &segment.generics {
            walk_generic_args(visitor, args);
        }
    }
}

pub fn walk_expr<V: AstVisitor>(visitor: &mut V, expr: &Expr) {
    match &expr.kind {
        ExprKind::Literal(_) => {}
        ExprKind::Path(path) => walk_path(visitor, path),
        ExprKind::Block(block) => walk_block(visitor, block),
        ExprKind::If(cond, then_branch, else_branch) => {
            visitor.visit_expr(cond);
            walk_block(visitor, then_branch);
            if let Some(else_) = else_branch {
                visitor.visit_expr(else_);
            }
        }
        ExprKind::Match(scrutinee, arms) => {
            visitor.visit_expr(scrutinee);
            for arm in arms {
                walk_pattern(visitor, &arm.pattern);
                if let Some(guard) = &arm.guard {
                    visitor.visit_expr(guard);
                }
                visitor.visit_expr(&arm.body);
            }
        }
        ExprKind::Loop(body, _) => walk_block(visitor, body),
        ExprKind::While(cond, body, _) => {
            visitor.visit_expr(cond);
            walk_block(visitor, body);
        }
        ExprKind::For(pattern, iter, body, _) => {
            walk_pattern(visitor, pattern);
            visitor.visit_expr(iter);
            walk_block(visitor, body);
        }
        ExprKind::Try(body, catches, finally) => {
            walk_block(visitor, body);
            for catch in catches {
                walk_pattern(visitor, &catch.pattern);
                walk_block(visitor, &catch.body);
            }
            if let Some(fin) = finally {
                walk_block(visitor, fin);
            }
        }
        ExprKind::Return(e) => {
            if let Some(e) = e {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Break(_, e) => {
            if let Some(e) = e {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Yield(e) => {
            if let Some(e) = e {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Assign(lhs, _, rhs) => {
            visitor.visit_expr(lhs);
            visitor.visit_expr(rhs);
        }
        ExprKind::Binary(lhs, _, rhs) => {
            visitor.visit_expr(lhs);
            visitor.visit_expr(rhs);
        }
        ExprKind::Unary(_, expr) => visitor.visit_expr(expr),
        ExprKind::Call(func, args) => {
            visitor.visit_expr(func);
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        ExprKind::MethodCall(receiver, _, args, _) => {
            visitor.visit_expr(receiver);
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        ExprKind::Field(expr, _) => visitor.visit_expr(expr),
        ExprKind::Index(expr, index) => {
            visitor.visit_expr(expr);
            visitor.visit_expr(index);
        }
        ExprKind::TryOperator(expr) => visitor.visit_expr(expr),
        ExprKind::Range(start, _, end) => {
            visitor.visit_expr(start);
            visitor.visit_expr(end);
        }
        ExprKind::PathSegment(expr, _, args) => {
            visitor.visit_expr(expr);
            if let Some(args) = args {
                walk_generic_args(visitor, args);
            }
        }
        ExprKind::Await(expr) => visitor.visit_expr(expr),
        ExprKind::Cast(expr, ty) => {
            visitor.visit_expr(expr);
            visitor.visit_type(ty);
        }
        ExprKind::Struct(path, fields) => {
            walk_path(visitor, path);
            for field in fields {
                visitor.visit_expr(&field.value);
            }
        }
        ExprKind::Tuple(exprs) | ExprKind::Array(exprs) => {
            for e in exprs {
                visitor.visit_expr(e);
            }
        }
        ExprKind::Closure(closure) => walk_closure(visitor, closure),
        ExprKind::Async(block) => walk_block(visitor, block),
        ExprKind::Macro(macro_inv) => walk_macro_invocation(visitor, macro_inv),
        ExprKind::Paren(expr) => visitor.visit_expr(expr),
    }
}

pub fn walk_block<V: AstVisitor>(visitor: &mut V, block: &Block) {
    for stmt in &block.stmts {
        visitor.visit_stmt(stmt);
    }
    if let Some(expr) = &block.expr {
        visitor.visit_expr(expr);
    }
}

pub fn walk_stmt<V: AstVisitor>(visitor: &mut V, stmt: &Stmt) {
    match &stmt.kind {
        StmtKind::Let(let_stmt) => walk_let_stmt(visitor, let_stmt),
        StmtKind::Item(item) => visitor.visit_item(item),
        StmtKind::Expr(expr) => visitor.visit_expr(expr),
        StmtKind::Macro(macro_inv) => walk_macro_invocation(visitor, macro_inv),
        StmtKind::Empty => {}
    }
}

pub fn walk_let_stmt<V: AstVisitor>(visitor: &mut V, let_stmt: &LetStmt) {
    walk_pattern(visitor, &let_stmt.pattern);
    if let Some(ty) = &let_stmt.ty {
        visitor.visit_type(ty);
    }
    if let Some(init) = &let_stmt.initializer {
        visitor.visit_expr(init);
    }
}

pub fn walk_pattern<V: AstVisitor>(visitor: &mut V, pattern: &Pattern) {
    match pattern {
        Pattern::Wildcard(_) => {}
        Pattern::Literal(lit) => {} // Literals don't have sub-AST
        Pattern::Binding(_, sub) => {
            if let Some(sub) = sub {
                walk_pattern(visitor, sub);
            }
        }
        Pattern::Tuple(patterns) => {
            for p in patterns {
                walk_pattern(visitor, p);
            }
        }
        Pattern::Struct(path, fields) => {
            walk_path(visitor, path);
            for field in fields {
                if let Some(p) = &field.pattern {
                    walk_pattern(visitor, p);
                }
            }
        }
        Pattern::Enum(path, _, patterns) => {
            walk_path(visitor, path);
            if let Some(patterns) = patterns {
                for p in patterns {
                    walk_pattern(visitor, p);
                }
            }
        }
        Pattern::Slice(prefix, rest) => {
            for p in prefix {
                walk_pattern(visitor, p);
            }
            if let Some(rest) = rest {
                walk_pattern(visitor, rest);
            }
        }
        Pattern::Or(patterns) => {
            for p in patterns {
                walk_pattern(visitor, p);
            }
        }
        Pattern::Ref(_, p) => walk_pattern(visitor, p),
        Pattern::Range(start, _, end) => {
            // These are expressions, not patterns in our AST
            // In a full implementation, they'd be Expr
        }
        Pattern::Guard(p, e) => {
            walk_pattern(visitor, p);
            visitor.visit_expr(e);
        }
        Pattern::Macro(macro_inv) => walk_macro_invocation(visitor, macro_inv),
    }
}

pub fn walk_type<V: AstVisitor>(visitor: &mut V, ty: &TypeExpr) {
    match &ty.kind {
        TypeExprKind::Tuple(types) => {
            for t in types {
                visitor.visit_type(t);
            }
        }
        TypeExprKind::Array(elem, len) => {
            visitor.visit_type(elem);
            visitor.visit_expr(len);
        }
        TypeExprKind::Slice(elem) => visitor.visit_type(elem),
        TypeExprKind::Ref(_, elem) => visitor.visit_type(elem),
        TypeExprKind::Box(elem) => visitor.visit_type(elem),
        TypeExprKind::Fn(params, ret) => {
            for p in params {
                visitor.visit_type(p);
            }
            visitor.visit_type(ret);
        }
        TypeExprKind::TraitObject(bounds) | TypeExprKind::ImplTrait(bounds) => {
            walk_generic_bounds(visitor, bounds);
        }
        TypeExprKind::Path(path) => walk_path(visitor, path),
        TypeExprKind::Paren(inner) => visitor.visit_type(inner),
        TypeExprKind::Never | TypeExprKind::Infer => {}
        TypeExprKind::Macro(macro_inv) => walk_macro_invocation(visitor, macro_inv),
    }
}

pub fn walk_closure<V: AstVisitor>(visitor: &mut V, closure: &ClosureExpr) {
    for param in &closure.params {
        if let Some(ty) = &param.ty {
            visitor.visit_type(ty);
        }
    }
    if let Some(ret) = &closure.return_type {
        visitor.visit_type(ret);
    }
    walk_block(visitor, &closure.body);
}

pub fn walk_macro_invocation<V: AstVisitor>(visitor: &mut V, macro_inv: &MacroInvocation) {
    walk_path(visitor, &macro_inv.path);
    // Token trees not visited in this simplified version
}

/// Pretty printer for AST
pub mod pretty {
    use super::*;
    use std::fmt;

    pub fn print_program(program: &Program) -> String {
        let mut printer = Printer::new();
        printer.print_program(program);
        printer.output
    }

    struct Printer {
        output: String,
        indent: usize,
    }

    impl Printer {
        fn new() -> Self {
            Self { output: String::new(), indent: 0 }
        }

        fn print_program(&mut self, program: &Program) {
            for item in &program.items {
                self.print_item(item);
                self.output.push('\n');
            }
        }

        fn print_item(&mut self, item: &Item) {
            self.indent_str();
            match item {
                Item::Function(f) => self.print_function(f),
                Item::Struct(s) => self.print_struct(s),
                Item::Enum(e) => self.print_enum(e),
                Item::Use(u) => self.output.push_str(&format!("use {};", u.tree)),
                Item::Module(m) => {
                    self.output.push_str(&format!("mod {};", m.name.name));
                }
                _ => self.output.push_str(&format!("{:?}", item)),
            }
        }

        fn print_function(&mut self, func: &FunctionDecl) {
            let vis = match func.visibility {
                Visibility::Public => "pub ",
                _ => "",
            };
            let async_kw = if matches!(func.asyncness, Asyncness::Async) { "async " } else { "" };
            let const_kw = if matches!(func.constness, Constness::Const) { "const " } else { "" };
            let unsafe_kw = if matches!(func.unsafety, Unsafety::Unsafe) { "unsafe " } else { "" };

            self.output.push_str(&format!(
                "{}{}{}{}fn {}{}",
                vis, unsafe_kw, async_kw, const_kw, func.name.name,
                if func.generics.params.is_empty() {
                    String::new()
                } else {
                    format!("<{}>", func.generics.params.iter().map(|p| p.name.name.clone()).collect::<Vec<_>>().join(", "))
                }
            ));

            self.output.push('(');
            for (i, param) in func.params.iter().enumerate() {
                if i > 0 { self.output.push_str(", "); }
                let mut_kw = if matches!(param.mutability, Mutability::Mutable) { "mut " } else { "" };
                self.output.push_str(&format!("{}{}: {}", mut_kw, param.name.name, param.ty));
            }
            self.output.push(')');

            if let Some(ret) = &func.return_type {
                self.output.push_str(&format!(" -> {}", ret));
            }

            if let Some(body) = &func.body {
                self.output.push_str(" {\n");
                self.indent += 1;
                for stmt in &body.stmts {
                    self.print_stmt(stmt);
                }
                if let Some(expr) = &body.expr {
                    self.indent_str();
                    self.output.push_str(&format!("{};", expr));
                }
                self.indent -= 1;
                self.indent_str();
                self.output.push('}');
            } else {
                self.output.push(';');
            }
        }

        fn print_struct(&mut self, struct_decl: &StructDecl) {
            let vis = match struct_decl.visibility {
                Visibility::Public => "pub ",
                _ => "",
            };
            self.output.push_str(&format!("{}struct {}", vis, struct_decl.name.name));
            if !struct_decl.generics.params.is_empty() {
                self.output.push_str(&format!("<{}>", struct_decl.generics.params.iter().map(|p| p.name.name.clone()).collect::<Vec<_>>().join(", ")));
            }

            match &struct_decl.kind {
                StructKind::Record(fields) => {
                    self.output.push_str(" {\n");
                    self.indent += 1;
                    for field in fields {
                        self.indent_str();
                        let fvis = match field.visibility {
                            Visibility::Public => "pub ",
                            _ => "",
                        };
                        self.output.push_str(&format!("{}{}: {},", fvis, field.name.name, field.ty));
                        self.output.push('\n');
                    }
                    self.indent -= 1;
                    self.indent_str();
                    self.output.push('}');
                }
                StructKind::Tuple(types) => {
                    self.output.push_str(&format!("({});", types.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(", ")));
                }
                StructKind::Unit => {
                    self.output.push(';');
                }
            }
        }

        fn print_enum(&mut self, enum_decl: &EnumDecl) {
            let vis = match enum_decl.visibility {
                Visibility::Public => "pub ",
                _ => "",
            };
            self.output.push_str(&format!("{}enum {} {{\n", vis, enum_decl.name.name));
            self.indent += 1;
            for variant in &enum_decl.variants {
                self.indent_str();
                self.output.push_str(&variant.name.name);
                match &variant.kind {
                    VariantKind::Unit => {}
                    VariantKind::Tuple(types) => {
                        self.output.push_str(&format!("({})", types.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(", ")));
                    }
                    VariantKind::Record(fields) => {
                        self.output.push_str(" {\n");
                        self.indent += 1;
                        for field in fields {
                            self.indent_str();
                            let fvis = match field.visibility {
                                Visibility::Public => "pub ",
                                _ => "",
                            };
                            self.output.push_str(&format!("{}{}: {},", fvis, field.name.name, field.ty));
                            self.output.push('\n');
                        }
                        self.indent -= 1;
                        self.indent_str();
                        self.output.push('}');
                    }
                }
                self.output.push(',');
                self.output.push('\n');
            }
            self.indent -= 1;
            self.indent_str();
            self.output.push('}');
        }

        fn print_stmt(&mut self, stmt: &Stmt) {
            self.indent_str();
            match &stmt.kind {
                StmtKind::Let(let_stmt) => {
                    let mut_kw = if matches!(let_stmt.mutability, Mutability::Mutable) { "mut " } else { "" };
                    self.output.push_str(&format!("let {}{}: {} = {};", mut_kw, let_stmt.pattern, let_stmt.ty.as_ref().map(|t| format!("{}", t)).unwrap_or("_".to_string()), let_stmt.initializer.as_ref().map(|e| format!("{}", e)).unwrap_or("".to_string())));
                }
                StmtKind::Expr(expr) => {
                    self.output.push_str(&format!("{};", expr));
                }
                _ => self.output.push_str(&format!("{:?}", stmt)),
            }
            self.output.push('\n');
        }

        fn indent_str(&mut self) {
            for _ in 0..self.indent {
                self.output.push_str("    ");
            }
        }
    }

    impl fmt::Display for Pattern {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Pattern::Wildcard(_) => write!(f, "_"),
                Pattern::Binding(name, sub) => {
                    if let Some(sub) = sub {
                        write!(f, "{} @ {}", name.name, sub)
                    } else {
                        write!(f, "{}", name.name)
                    }
                }
                Pattern::Tuple(patterns) => {
                    write!(f, "({})", patterns.iter().map(|p| format!("{}", p)).collect::<Vec<_>>().join(", "))
                }
                Pattern::Struct(path, fields) => {
                    write!(f, "{} {{ {} }}", path, fields.iter().map(|f| format!("{}", f)).collect::<Vec<_>>().join(", "))
                }
                _ => write!(f, "{:?}", self),
            }
        }
    }

    impl fmt::Display for Expr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match &self.kind {
                ExprKind::Literal(lit) => write!(f, "{}", lit),
                ExprKind::Path(path) => write!(f, "{}", path),
                ExprKind::Block(block) => write!(f, "{{ ... }}"),
                ExprKind::Call(func, args) => write!(f, "{}", format!("{}({})", func, args.iter().map(|a| format!("{}", a)).collect::<Vec<_>>().join(", "))),
                ExprKind::Binary(lhs, op, rhs) => write!(f, "{} {:?} {}", lhs, op, rhs),
                ExprKind::Unary(op, expr) => write!(f, "{:?}{}", op, expr),
                ExprKind::Struct(path, fields) => write!(f, "{} {{ {} }}", path, fields.iter().map(|f| format!("{}", f)).collect::<Vec<_>>().join(", ")),
                _ => write!(f, "{:?}", self.kind),
            }
        }
    }

    impl fmt::Display for Literal {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match &self.kind {
                LiteralKind::Integer(s) => write!(f, "{}", s),
                LiteralKind::Float(s) => write!(f, "{}", s),
                LiteralKind::Char(c) => write!(f, "'{}'", c.escape_debug()),
                LiteralKind::String(s) => write!(f, "\"{}\"", s.escape_debug()),
                LiteralKind::Bool(b) => write!(f, "{}", b),
                LiteralKind::Unit => write!(f, "()"),
                _ => write!(f, "{:?}", self.kind),
            }
        }
    }

    impl fmt::Display for Path {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.global {
                write!(f, "::")?;
            }
            write!(f, "{}", self.segments.iter().map(|s| format!("{}", s)).collect::<Vec<_>>().join("::"))
        }
    }

    impl fmt::Display for PathSegment {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.ident.name)?;
            if let Some(args) = &self.generics {
                write!(f, "<{}>", args.args.iter().map(|a| format!("{}", a)).collect::<Vec<_>>().join(", "))?;
            }
            Ok(())
        }
    }

    impl fmt::Display for GenericArg {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                GenericArg::Type(t) => write!(f, "{}", t),
                GenericArg::Const(e) => write!(f, "{}", e),
                GenericArg::Lifetime(l) => write!(f, "'{}", l.name.name),
            }
        }
    }

    impl fmt::Display for TypeExpr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match &self.kind {
                TypeExprKind::Path(path) => write!(f, "{}", path),
                TypeExprKind::Ref(mutability, ty) => write!(f, "&{} {}", if *mutability == Mutability::Mutable { "mut " } else { "" }, ty),
                TypeExprKind::Box(ty) => write!(f, "Box<{}>", ty),
                TypeExprKind::Tuple(types) => write!(f, "({})", types.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(", ")),
                TypeExprKind::Array(elem, len) => write!(f, "[{}; {}]", elem, len),
                TypeExprKind::Slice(elem) => write!(f, "[{}]", elem),
                TypeExprKind::Fn(params, ret) => write!(f, "fn({}) -> {}", params.iter().map(|p| format!("{}", p)).collect::<Vec<_>>().join(", "), ret),
                TypeExprKind::Never => write!(f, "!"),
                TypeExprKind::Infer => write!(f, "_"),
                _ => write!(f, "{:?}", self.kind),
            }
        }
    }
}