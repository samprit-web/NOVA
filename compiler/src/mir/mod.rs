//! NOVA Mid-Level Intermediate Representation (MIR)
//!
//! MIR is a control-flow graph representation with:
//! - Explicit control flow (basic blocks)
//! - Typed SSA-like variables (places)
//! - Simplified operations
//! - Ready for optimization and codegen

use std::collections::HashMap;

use crate::{Span, FileId, NodeId, Type, GenericArg, GenericArgs};
use crate::hir::{ItemId, VarId, Ident, Visibility, Constness, Asyncness, Unsafety, Mutability, GenericParam, WhereClause, Lifetime, AttrStyle, Attribute, MetaItem};

/// MIR program
#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<MirItem>,
    pub span: Span,
}

/// MIR items
#[derive(Debug, Clone)]
pub enum MirItem {
    Function(Function),
    Static(Static),
    Const(Const),
    TypeDefinition(TypeDefinition),
}

/// Function definition in MIR
#[derive(Debug, Clone)]
pub struct Function {
    pub id: ItemId,
    pub name: Ident,
    pub visibility: Visibility,
    pub constness: Constness,
    pub asyncness: Asyncness,
    pub unsafety: Unsafety,
    pub generics: Generics,
    pub signature: FnSig,
    pub body: Option<Body>,
    pub span: Span,
}

/// Function signature
#[derive(Debug, Clone)]
pub struct FnSig {
    pub params: Vec<Param>,
    pub return_type: Type,
    pub c_variadic: bool,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub name: Ident,
    pub ty: Type,
    pub mutability: Mutability,
}

/// Generics
#[derive(Debug, Clone)]
pub struct Generics {
    pub params: Vec<GenericParam>,
    pub where_clause: WhereClause,
}

/// Function body (CFG)
#[derive(Debug, Clone)]
pub struct Body {
    pub basic_blocks: Vec<BasicBlock>,
    pub local_decls: Vec<LocalDecl>,
    pub arg_count: usize,
    pub return_place: Place,
    pub span: Span,
}

/// Basic block
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BasicBlockId,
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
    pub span: Span,
}

/// Basic block identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasicBlockId(pub u32);

/// Local variable declaration
#[derive(Debug, Clone)]
pub struct LocalDecl {
    pub name: Option<Ident>,
    pub ty: Type,
    pub mutability: Mutability,
    pub is_arg: bool,
    pub is_return: bool,
    pub span: Span,
}

/// Places (lvalues)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Place {
    Local(LocalId),
    Static(ItemId),
    Field(Box<Place>, FieldIdx),
    Index(Box<Place>, Box<Place>),
    Deref(Box<Place>),
    Downcast(Box<Place>, VariantIdx),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldIdx(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariantIdx(pub u32);

/// Statements
#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    Assign(Place, Rvalue),
    SetDiscriminant(Place, VariantIdx),
    StorageLive(LocalId),
    StorageDead(LocalId),
    Retag(Place, RetagKind),
    Asm(Asm),
    Nop,
    Coverage(CoverageKind),
}

#[derive(Debug, Clone)]
pub enum RetagKind {
    Default,
    Raw,
    FnEntry,
    TwoPhase,
}

/// Rvalues (rvalues)
#[derive(Debug, Clone)]
pub enum Rvalue {
    Use(Operand),
    Copy(Place),
    Clone(Place),
    Ref(Place, Mutability),
    AddressOf(Place),
    Len(Place),
    Cast(CastKind, Operand, Type),
    BinaryOp(BinOp, Operand, Operand),
    UnaryOp(UnOp, Operand),
    Discriminant(Place),
    Aggregate(AggregateKind, Vec<Operand>),
    ShallowInitBox(Operand, Place),
    ThreadLocalRef(ItemId),
    NullaryOp(NullOp, Type),
}

/// Operands
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operand {
    Copy(Place),
    Move(Place),
    Constant(Constant),
}

/// Constants
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Constant {
    pub ty: Type,
    pub kind: ConstantKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstantKind {
    ZeroSized,
    Value(Vec<u8>),
    Unevaluated(Instance, GenericArgs),
}

/// Instances (for const evaluation)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Instance {
    pub def_id: ItemId,
    pub generics: GenericArgs,
}

/// Terminators
#[derive(Debug, Clone)]
pub enum Terminator {
    Goto(BasicBlockId),
    SwitchInt(Operand, SwitchTargets),
    Return,
    Unreachable,
    Drop(Place, BasicBlockId, Option<BasicBlockId>),
    DropAndReplace(Place, Place, BasicBlockId, Option<BasicBlockId>),
    Call(Call),
    Assert(Operand, AssertKind, BasicBlockId),
    Yield(Place, BasicBlockId),
    CoroutineDrop(Place),
    FalseEdge(Vec<BasicBlockId>),
    FalseUnwind(Vec<BasicBlockId>),
    InlineAsm(InlineAsm),
}

#[derive(Debug, Clone)]
pub struct SwitchTargets {
    pub values: Vec<u128>,
    pub targets: Vec<BasicBlockId>,
    pub otherwise: BasicBlockId,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub func: Operand,
    pub args: Vec<Operand>,
    pub destination: Place,
    pub target: Option<BasicBlockId>,
    pub unwind: Option<BasicBlockId>,
    pub call_source: CallSource,
    pub fn_span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallSource {
    Normal,
    OverloadedOperator,
    Misc,
}

#[derive(Debug, Clone)]
pub enum AssertKind {
    BoundsCheck { len: Place, index: Place },
    Overflow { op: BinOp, left: Operand, right: Operand },
    DivisionByZero,
    RemainderByZero,
    NullPointerDeref,
    MisalignedPointerDeref,
    AlignmentCheck { align: u64 },
    SubsliceOutOfBounds { len: Place, start: Place, end: Place },
    Panic,
}

/// Inline assembly
#[derive(Debug, Clone)]
pub struct InlineAsm {
    pub template: Vec<AsmTemplatePiece>,
    pub operands: Vec<AsmOperand>,
    pub options: AsmOptions,
    pub line_spans: Vec<Span>,
}

#[derive(Debug, Clone)]
pub enum AsmTemplatePiece {
    String(String),
    Placeholder(usize),
}

#[derive(Debug, Clone)]
pub struct AsmOperand {
    pub kind: AsmOperandKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsmOperandKind {
    In(Operand),
    Out(Place, bool), // place, late
    InOut(Place, Operand),
    Const(Constant),
    Sym(ItemId),
    Label(BasicBlockId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsmOptions {
    pub pure: bool,
    pub nomem: bool,
    pub readonly: bool,
    pub preserves_flags: bool,
    pub noreturn: bool,
    pub nostack: bool,
    pub att_syntax: bool,
}

/// Coverage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageKind {
    Counter,
    Expression,
}

/// Static item
#[derive(Debug, Clone)]
pub struct Static {
    pub id: ItemId,
    pub name: Ident,
    pub visibility: Visibility,
    pub mutability: Mutability,
    pub ty: Type,
    pub value: Constant,
    pub span: Span,
}

/// Const item
#[derive(Debug, Clone)]
pub struct Const {
    pub id: ItemId,
    pub name: Ident,
    pub visibility: Visibility,
    pub ty: Type,
    pub value: Constant,
    pub span: Span,
}

/// Type definition
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub id: ItemId,
    pub name: Ident,
    pub generics: Generics,
    pub kind: TypeDefKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypeDefKind {
    Struct(Vec<Field>),
    Enum(Vec<Variant>),
    Union(Vec<Field>),
    Alias(Type),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: Ident,
    pub ty: Type,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub name: Ident,
    pub fields: Vec<Field>,
    pub discriminant: Option<Constant>,
    pub span: Span,
}

/// Binary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    BitAnd, BitOr, BitXor,
    Shl, Shr,
    Eq, Ne, Lt, Le, Gt, Ge,
    Offset,
}

/// Unary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Not,
    Neg,
}

/// Cast kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastKind {
    Misc,
    ReifyFnPointer,
    FnPtrToPtr,
    PtrToPtr,
    PtrToFnPtr,
    IntToInt,
    FloatToFloat,
    FloatToInt,
    IntToFloat,
    PtrToInt,
    IntToPtr,
}

/// Null operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NullOp {
    SizeOf,
    AlignOf,
    OffsetOf(Vec<FieldIdx>),
    UbChecks,
}

/// Aggregate kinds
#[derive(Debug, Clone)]
pub enum AggregateKind {
    Array(Type),
    Tuple,
    Adt(ItemId, VariantIdx, GenericArgs, Vec<Type>),
    Closure(ItemId, GenericArgs),
    Coroutine(ItemId, GenericArgs),
    CoroutineClosure(ItemId, GenericArgs),
}

/// Asm statement
#[derive(Debug, Clone)]
pub struct Asm {
    pub template: Vec<AsmTemplatePiece>,
    pub operands: Vec<AsmOperand>,
    pub options: AsmOptions,
    pub line_spans: Vec<Span>,
}

/// Lowering from HIR to MIR
pub mod lower {
    use super::*;
    use crate::hir;
    use crate::{Session, Diagnostic, Severity};

    pub fn lower(session: &mut Session, hir: hir::Program) -> Result<Program, Vec<Diagnostic>> {
        let mut lowerer = Lowerer::new(session);
        lowerer.lower_program(hir)
    }

    struct Lowerer<'a> {
        session: &'a mut Session,
        next_item_id: u32,
        next_bb_id: u32,
        next_local_id: u32,
        current_function: Option<Function>,
        current_body: Option<Body>,
        local_map: HashMap<hir::VarId, LocalId>,
    }

    impl<'a> Lowerer<'a> {
        fn new(session: &'a mut Session) -> Self {
            Self {
                session,
                next_item_id: 1,
                next_bb_id: 1,
                next_local_id: 1,
                current_function: None,
                current_body: None,
                local_map: HashMap::new(),
            }
        }

        fn next_item_id(&mut self) -> ItemId {
            let id = ItemId(self.next_item_id);
            self.next_item_id += 1;
            id
        }

        fn next_bb_id(&mut self) -> BasicBlockId {
            let id = BasicBlockId(self.next_bb_id);
            self.next_bb_id += 1;
            id
        }

        fn next_local_id(&mut self) -> LocalId {
            let id = LocalId(self.next_local_id);
            self.next_local_id += 1;
            id
        }

        fn lower_program(&mut self, hir: hir::Program) -> Result<Program, Vec<Diagnostic>> {
            let mut items = Vec::new();
            for item in hir.items {
                items.extend(self.lower_item(item)?);
            }
            Ok(Program { items, span: hir.span })
        }

        fn lower_item(&mut self, item: hir::Item) -> Result<Vec<MirItem>, Vec<Diagnostic>> {
            match item {
                hir::Item::Function(f) => Ok(vec![MirItem::Function(self.lower_function(f)?)]),
                hir::Item::Struct(s) => Ok(vec![MirItem::TypeDefinition(self.lower_struct(s)?)]),
                hir::Item::Enum(e) => Ok(vec![MirItem::TypeDefinition(self.lower_enum(e)?)]),
                hir::Item::TypeAlias(t) => Ok(vec![MirItem::TypeDefinition(self.lower_type_alias(t)?)]),
                hir::Item::Const(c) => Ok(vec![MirItem::Const(self.lower_const(c)?)]),
                hir::Item::Static(s) => Ok(vec![MirItem::Static(self.lower_static(s)?)]),
                _ => {
                    self.session.diagnostics.push(Diagnostic::new(
                        Severity::Error,
                        format!("Unsupported item in MIR lowering: {:?}", item),
                        self.current_span(),
                    ));
                    Ok(vec![])
                }
            }
        }

        fn current_span(&self) -> Span {
            Span::new(FileId(0), 0, 0)
        }

        fn lower_function(&mut self, f: hir::Function) -> Result<Function, Vec<Diagnostic>> {
            self.current_function = Some(Function {
                id: f.id,
                name: f.name,
                visibility: f.visibility,
                constness: f.constness,
                asyncness: f.asyncness,
                unsafety: f.unsafety,
                generics: self.lower_generics(f.generics),
                signature: FnSig {
                    params: f.params.iter().map(|p| Param {
                        name: p.name.clone(),
                        ty: p.ty.clone(),
                        mutability: p.mutability,
                    }).collect(),
                    return_type: f.return_type.clone(),
                    c_variadic: false,
                },
                body: None,
                span: f.span,
            });

            if let Some(body) = f.body {
                let mir_body = self.lower_body(body)?;
                self.current_function.as_mut().unwrap().body = Some(mir_body);
            }

            Ok(self.current_function.take().unwrap())
        }

        fn lower_body(&mut self, body: hir::Body) -> Result<Body, Vec<Diagnostic>> {
            self.local_map.clear();
            let mut local_decls = Vec::new();
            let mut arg_count = 0;

            // Create local declarations for parameters
            for param in &body.params {
                let local_id = self.next_local_id();
                self.local_map.insert(param.id, local_id);
                local_decls.push(LocalDecl {
                    name: Some(param.name.clone()),
                    ty: param.ty.clone(),
                    mutability: param.mutability,
                    is_arg: true,
                    is_return: false,
                    span: param.span,
                });
                arg_count += 1;
            }

            // Return place
            let return_place = Place::Local(self.next_local_id());
            local_decls.push(LocalDecl {
                name: Some(Ident::new("_return".to_string(), self.current_span())),
                ty: self.current_function.as_ref().unwrap().signature.return_type.clone(),
                mutability: Mutability::Mutable,
                is_arg: false,
                is_return: true,
                span: body.span,
            });

            // Lower statements to basic blocks
            let mut basic_blocks = Vec::new();
            let start_bb = self.next_bb_id();
            let mut current_bb = BasicBlock {
                id: start_bb,
                statements: Vec::new(),
                terminator: Terminator::Goto(self.next_bb_id()), // Will be fixed
                span: body.span,
            };

            for stmt in body.statements {
                self.lower_statement(stmt, &mut current_bb, &mut local_decls)?;
            }

            // Handle return expression
            if let Some(ret_expr) = body.return_expr {
                let ret_place = return_place.clone();
                let rvalue = self.lower_expr_as_rvalue(ret_expr, &mut current_bb, &mut local_decls)?;
                current_bb.statements.push(Statement {
                    kind: StatementKind::Assign(ret_place, rvalue),
                    span: current_bb.span,
                });
            }

            current_bb.terminator = Terminator::Return;
            basic_blocks.push(current_bb);

            Ok(Body {
                basic_blocks,
                local_decls,
                arg_count,
                return_place,
                span: body.span,
            })
        }

        fn lower_statement(&mut self, stmt: hir::Statement, bb: &mut BasicBlock, local_decls: &mut Vec<LocalDecl>) -> Result<(), Vec<Diagnostic>> {
            match stmt.kind {
                hir::StmtKind::Let(let_stmt) => {
                    let local_id = self.next_local_id();
                    self.local_map.insert(let_stmt.id, local_id);
                    local_decls.push(LocalDecl {
                        name: None, // Pattern binding names handled separately
                        ty: let_stmt.ty.unwrap_or(Type::Tuple(Vec::new())),
                        mutability: Mutability::Immutable, // Determined by pattern
                        is_arg: false,
                        is_return: false,
                        span: let_stmt.span,
                    });

                    if let Some(init) = let_stmt.initializer {
                        let place = Place::Local(local_id);
                        let rvalue = self.lower_expr_as_rvalue(init, bb, local_decls)?;
                        bb.statements.push(Statement {
                            kind: StatementKind::Assign(place, rvalue),
                            span: init.span,
                        });
                    }
                }
                hir::StmtKind::Expr(expr) => {
                    self.lower_expr_as_rvalue(expr, bb, local_decls)?;
                }
                hir::StmtKind::Item(_) => {
                    // Items not lowered to MIR statements
                }
            }
            Ok(())
        }

        fn lower_expr_as_rvalue(&mut self, expr: hir::Expr, bb: &mut BasicBlock, local_decls: &mut Vec<LocalDecl>) -> Result<Rvalue, Vec<Diagnostic>> {
            Ok(match expr.kind {
                hir::ExprKind::Literal(lit) => Rvalue::Use(Operand::Constant(Constant {
                    ty: lit.ty,
                    kind: ConstantKind::Value(self.literal_to_bytes(lit)?),
                })),
                hir::ExprKind::Var(var_id) => {
                    let local_id = *self.local_map.get(&var_id).unwrap();
                    Rvalue::Use(Operand::Copy(Place::Local(local_id)))
                }
                hir::ExprKind::Binary(lhs, op, rhs) => {
                    let lhs_op = self.lower_expr_as_operand(*lhs, bb, local_decls)?;
                    let rhs_op = self.lower_expr_as_operand(*rhs, bb, local_decls)?;
                    Rvalue::BinaryOp(self.lower_binop(op), lhs_op, rhs_op)
                }
                hir::ExprKind::Unary(op, expr) => {
                    let opnd = self.lower_expr_as_operand(*expr, bb, local_decls)?;
                    Rvalue::UnaryOp(self.lower_unop(op), opnd)
                }
                hir::ExprKind::Call(func, args) => {
                    let func_op = self.lower_expr_as_operand(*func, bb, local_decls)?;
                    let arg_ops = args.into_iter().map(|a| self.lower_expr_as_operand(a, bb, local_decls)).collect::<Result<_, _>>()?;
                    let dest = Place::Local(self.next_local_id());
                    bb.terminator = Terminator::Call(Call {
                        func: func_op,
                        args: arg_ops,
                        destination: dest.clone(),
                        target: Some(bb.id),
                        unwind: None,
                        call_source: CallSource::Normal,
                        fn_span: expr.span,
                    });
                    Rvalue::Use(Operand::Move(dest))
                }
                hir::ExprKind::MethodCall(receiver, ident, generics, args) => {
                    // Lower as UFCS call
                    let recv_op = self.lower_expr_as_operand(*receiver, bb, local_decls)?;
                    let mut all_args = vec![recv_op];
                    all_args.extend(args.into_iter().map(|a| self.lower_expr_as_operand(a, bb, local_decls)).collect::<Result<_, _>>()?);
                    let func_op = Operand::Constant(Constant {
                        ty: Type::Fn(all_args.iter().map(|o| o.ty().clone()).collect(), Box::new(Type::Tuple(Vec::new()))),
                        kind: ConstantKind::Unevaluated(Instance { def_id: ItemId(0), generics: GenericArgs { args: Vec::new(), span: expr.span } }, GenericArgs { args: Vec::new(), span: expr.span }),
                    });
                    let dest = Place::Local(self.next_local_id());
                    bb.terminator = Terminator::Call(Call {
                        func: func_op,
                        args: all_args,
                        destination: dest.clone(),
                        target: Some(bb.id),
                        unwind: None,
                        call_source: CallSource::OverloadedOperator,
                        fn_span: expr.span,
                    });
                    Rvalue::Use(Operand::Move(dest))
                }
                hir::ExprKind::Field(expr, ident) => {
                    let place = self.lower_expr_as_place(*expr, bb, local_decls)?;
                    // Find field index
                    let field_idx = FieldIdx(0); // Simplified
                    Rvalue::Use(Operand::Copy(Place::Field(Box::new(place), field_idx)))
                }
                hir::ExprKind::Index(expr, index) => {
                    let place = self.lower_expr_as_place(*expr, bb, local_decls)?;
                    let index_op = self.lower_expr_as_operand(*index, bb, local_decls)?;
                    Rvalue::Use(Operand::Copy(Place::Index(Box::new(place), Box::new(Place::Local(LocalId(0)))))) // Simplified
                }
                hir::ExprKind::Cast(expr, ty) => {
                    let op = self.lower_expr_as_operand(*expr, bb, local_decls)?;
                    Rvalue::Cast(CastKind::Misc, op, ty)
                }
                hir::ExprKind::Struct(path, fields) => {
                    let ops = fields.into_iter().map(|f| self.lower_expr_as_operand(f.value, bb, local_decls)).collect::<Result<_, _>>()?;
                    Rvalue::Aggregate(AggregateKind::Adt(ItemId(0), VariantIdx(0), GenericArgs { args: Vec::new(), span: expr.span }, Vec::new()), ops)
                }
                hir::ExprKind::Tuple(exprs) => {
                    let ops = exprs.into_iter().map(|e| self.lower_expr_as_operand(e, bb, local_decls)).collect::<Result<_, _>>()?;
                    Rvalue::Aggregate(AggregateKind::Tuple, ops)
                }
                hir::ExprKind::Array(exprs) => {
                    let ops = exprs.into_iter().map(|e| self.lower_expr_as_operand(e, bb, local_decls)).collect::<Result<_, _>>()?;
                    Rvalue::Aggregate(AggregateKind::Array(Type::Tuple(Vec::new())), ops)
                }
                hir::ExprKind::Ref(expr, mutability) => {
                    let place = self.lower_expr_as_place(*expr, bb, local_decls)?;
                    Rvalue::Ref(place, mutability)
                }
                hir::ExprKind::Deref(expr) => {
                    let place = self.lower_expr_as_place(*expr, bb, local_decls)?;
                    Rvalue::Use(Operand::Copy(Place::Deref(Box::new(place))))
                }
                hir::ExprKind::TryOperator(expr) => {
                    let op = self.lower_expr_as_operand(*expr, bb, local_decls)?;
                    // Try operator becomes a call to try trait
                    Rvalue::Use(op)
                }
                hir::ExprKind::Block(block) => {
                    // Block expression - evaluate last expression
                    let mut last_rvalue = Rvalue::Use(Operand::Constant(Constant { ty: Type::Tuple(Vec::new()), kind: ConstantKind::ZeroSized }));
                    for stmt in block.statements {
                        if let hir::StmtKind::Expr(e) = stmt.kind {
                            last_rvalue = self.lower_expr_as_rvalue(e, bb, local_decls)?;
                        }
                    }
                    if let Some(ret) = block.return_expr {
                        last_rvalue = self.lower_expr_as_rvalue(*ret, bb, local_decls)?;
                    }
                    last_rvalue
                }
                _ => Rvalue::Use(Operand::Constant(Constant { ty: Type::Tuple(Vec::new()), kind: ConstantKind::ZeroSized })),
            })
        }

        fn lower_expr_as_operand(&mut self, expr: hir::Expr, bb: &mut BasicBlock, local_decls: &mut Vec<LocalDecl>) -> Result<Operand, Vec<Diagnostic>> {
            Ok(match expr.kind {
                hir::ExprKind::Literal(lit) => Operand::Constant(Constant {
                    ty: lit.ty,
                    kind: ConstantKind::Value(self.literal_to_bytes(lit)?),
                }),
                hir::ExprKind::Var(var_id) => {
                    let local_id = *self.local_map.get(&var_id).unwrap();
                    Operand::Copy(Place::Local(local_id))
                }
                _ => {
                    // For complex expressions, evaluate to a temporary
                    let temp = Place::Local(self.next_local_id());
                    let rvalue = self.lower_expr_as_rvalue(expr, bb, local_decls)?;
                    bb.statements.push(Statement {
                        kind: StatementKind::Assign(temp.clone(), rvalue),
                        span: expr.span,
                    });
                    Operand::Move(temp)
                }
            })
        }

        fn lower_expr_as_place(&mut self, expr: hir::Expr, bb: &mut BasicBlock, local_decls: &mut Vec<LocalDecl>) -> Result<Place, Vec<Diagnostic>> {
            Ok(match expr.kind {
                hir::ExprKind::Var(var_id) => {
                    let local_id = *self.local_map.get(&var_id).unwrap();
                    Place::Local(local_id)
                }
                hir::ExprKind::Field(base, ident) => {
                    let base_place = self.lower_expr_as_place(*base, bb, local_decls)?;
                    Place::Field(Box::new(base_place), FieldIdx(0)) // Simplified
                }
                hir::ExprKind::Index(base, index) => {
                    let base_place = self.lower_expr_as_place(*base, bb, local_decls)?;
                    let index_op = self.lower_expr_as_operand(*index, bb, local_decls)?;
                    Place::Index(Box::new(base_place), Box::new(Place::Local(LocalId(0)))) // Simplified
                }
                hir::ExprKind::Deref(base) => {
                    let base_place = self.lower_expr_as_place(*base, bb, local_decls)?;
                    Place::Deref(Box::new(base_place))
                }
                _ => {
                    // Evaluate to temporary and return place
                    let temp = Place::Local(self.next_local_id());
                    let rvalue = self.lower_expr_as_rvalue(expr, bb, local_decls)?;
                    bb.statements.push(Statement {
                        kind: StatementKind::Assign(temp.clone(), rvalue),
                        span: expr.span,
                    });
                    temp
                }
            })
        }

        fn literal_to_bytes(&self, lit: hir::Literal) -> Result<Vec<u8>, Vec<Diagnostic>> {
            Ok(match lit.kind {
                hir::LiteralKind::Integer(i) => i.to_le_bytes().to_vec(),
                hir::LiteralKind::Float(f) => f.to_le_bytes().to_vec(),
                hir::LiteralKind::Char(c) => (c as u32).to_le_bytes().to_vec(),
                hir::LiteralKind::String(s) => s.into_bytes(),
                hir::LiteralKind::Bool(b) => vec![b as u8],
                hir::LiteralKind::Unit => Vec::new(),
            })
        }

        fn lower_binop(&self, op: hir::BinOp) -> BinOp {
            match op {
                hir::BinOp::Add => BinOp::Add,
                hir::BinOp::Sub => BinOp::Sub,
                hir::BinOp::Mul => BinOp::Mul,
                hir::BinOp::Div => BinOp::Div,
                hir::BinOp::Rem => BinOp::Rem,
                hir::BinOp::BitAnd => BinOp::BitAnd,
                hir::BinOp::BitOr => BinOp::BitOr,
                hir::BinOp::BitXor => BinOp::BitXor,
                hir::BinOp::Shl => BinOp::Shl,
                hir::BinOp::Shr => BinOp::Shr,
                hir::BinOp::Eq => BinOp::Eq,
                hir::BinOp::Ne => BinOp::Ne,
                hir::BinOp::Lt => BinOp::Lt,
                hir::BinOp::Le => BinOp::Le,
                hir::BinOp::Gt => BinOp::Gt,
                hir::BinOp::Ge => BinOp::Ge,
                hir::BinOp::And => BinOp::Eq, // Logical ops lowered differently
                hir::BinOp::Or => BinOp::Eq,
                hir::BinOp::As => BinOp::Eq,
                hir::BinOp::Is => BinOp::Eq,
                hir::BinOp::In => BinOp::Eq,
            }
        }

        fn lower_unop(&self, op: hir::UnOp) -> UnOp {
            match op {
                hir::UnOp::Not => UnOp::Not,
                hir::UnOp::Neg => UnOp::Neg,
                _ => UnOp::Not,
            }
        }

        fn lower_generics(&mut self, g: hir::Generics) -> Generics {
            Generics {
                params: g.params.into_iter().map(|p| GenericParam {
                    attrs: p.attrs,
                    name: p.name,
                    kind: match p.kind {
                        hir::GenericParamKind::Type { bounds, default } => hir::GenericParamKind::Type {
                            bounds: bounds.map(|b| self.lower_generic_bounds(b)),
                            default: default.map(|d| self.lower_type(d)),
                        },
                        hir::GenericParamKind::Const { ty, default } => hir::GenericParamKind::Const {
                            ty: self.lower_type(ty),
                            default: default.map(|d| self.lower_expr(d)).ok().flatten(),
                        },
                        hir::GenericParamKind::Lifetime => hir::GenericParamKind::Lifetime,
                    },
                    span: p.span,
                }).collect(),
                where_clause: self.lower_where_clause(g.where_clause),
            }
        }

        fn lower_generic_bounds(&mut self, b: hir::GenericBounds) -> hir::GenericBounds {
            hir::GenericBounds {
                bounds: b.bounds.into_iter().map(|bound| hir::GenericBound {
                    trait_ref: self.lower_trait_ref(bound.trait_ref),
                    span: bound.span,
                }).collect(),
                span: b.span,
            }
        }

        fn lower_trait_ref(&mut self, t: hir::TraitRef) -> hir::TraitRef {
            hir::TraitRef {
                path: self.lower_path(t.path).unwrap(),
                generics: t.generics.map(|g| self.lower_generic_args(g)),
                span: t.span,
            }
        }

        fn lower_where_clause(&mut self, w: Option<hir::WhereClause>) -> WhereClause {
            w.map(|wc| WhereClause {
                predicates: wc.predicates.into_iter().map(|p| hir::WherePredicate {
                    ty: self.lower_type(p.ty),
                    bounds: self.lower_generic_bounds(p.bounds),
                    span: p.span,
                }).collect(),
                span: wc.span,
            }).unwrap_or(WhereClause { predicates: Vec::new(), span: self.current_span() })
        }

        fn lower_generic_args(&mut self, g: hir::GenericArgs) -> GenericArgs {
            GenericArgs {
                args: g.args.into_iter().map(|a| match a {
                    hir::GenericArg::Type(t) => GenericArg::Type(self.lower_type(t)),
                    hir::GenericArg::Const(e) => GenericArg::Const(self.lower_expr(e).ok().unwrap()),
                    hir::GenericArg::Lifetime(l) => GenericArg::Lifetime(Lifetime { name: l.name, span: l.span }),
                }).collect(),
                span: g.span,
            }
        }

        fn lower_type(&mut self, t: hir::Type) -> Type {
            // Types are the same in HIR and MIR
            t
        }

        fn lower_expr(&mut self, e: hir::Expr) -> Result<hir::Expr, Vec<Diagnostic>> {
            // This is a placeholder - actual lowering happens in lower_expr_as_rvalue
            Ok(e)
        }

        fn lower_path(&mut self, p: hir::Path) -> Result<hir::Path, Vec<Diagnostic>> {
            Ok(p)
        }

        fn lower_struct(&mut self, s: hir::Struct) -> Result<TypeDefinition, Vec<Diagnostic>> {
            let kind = match s.kind {
                hir::StructKind::Record(fields) => TypeDefKind::Struct(fields.into_iter().map(|f| Field {
                    name: f.name,
                    ty: f.ty,
                    visibility: f.visibility,
                    span: f.span,
                }).collect()),
                hir::StructKind::Tuple(types) => TypeDefKind::Struct(types.into_iter().enumerate().map(|(i, ty)| Field {
                    name: Ident::new(format!("_{}", i), self.current_span()),
                    ty,
                    visibility: Visibility::Private,
                    span: self.current_span(),
                }).collect()),
                hir::StructKind::Unit => TypeDefKind::Struct(Vec::new()),
            };
            Ok(TypeDefinition {
                id: s.id,
                name: s.name,
                generics: self.lower_generics(s.generics),
                kind,
                span: s.span,
            })
        }

        fn lower_enum(&mut self, e: hir::Enum) -> Result<TypeDefinition, Vec<Diagnostic>> {
            let variants = e.variants.into_iter().map(|v| Variant {
                name: v.name,
                fields: match v.kind {
                    hir::VariantKind::Unit => Vec::new(),
                    hir::VariantKind::Tuple(types) => types.into_iter().enumerate().map(|(i, ty)| Field {
                        name: Ident::new(format!("_{}", i), self.current_span()),
                        ty,
                        visibility: Visibility::Private,
                        span: self.current_span(),
                    }).collect(),
                    hir::VariantKind::Record(fields) => fields.into_iter().map(|f| Field {
                        name: f.name,
                        ty: f.ty,
                        visibility: f.visibility,
                        span: f.span,
                    }).collect(),
                },
                discriminant: None,
                span: v.span,
            }).collect();
            Ok(TypeDefinition {
                id: e.id,
                name: e.name,
                generics: self.lower_generics(e.generics),
                kind: TypeDefKind::Enum(variants),
                span: e.span,
            })
        }

        fn lower_type_alias(&mut self, t: hir::TypeAlias) -> TypeDefinition {
            TypeDefinition {
                id: t.id,
                name: t.name,
                generics: self.lower_generics(t.generics),
                kind: TypeDefKind::Alias(t.ty),
                span: t.span,
            }
        }

        fn lower_const(&mut self, c: hir::Const) -> Const {
            Const {
                id: c.id,
                name: c.name,
                visibility: c.visibility,
                ty: c.ty,
                value: Constant {
                    ty: c.ty,
                    kind: ConstantKind::Value(Vec::new()), // Filled by const evaluator
                },
                span: c.span,
            }
        }

        fn lower_static(&mut self, s: hir::Static) -> Static {
            Static {
                id: s.id,
                name: s.name,
                visibility: s.visibility,
                mutability: s.mutability,
                ty: s.ty,
                value: Constant {
                    ty: s.ty,
                    kind: ConstantKind::Value(Vec::new()),
                },
                span: s.span,
            }
        }
    }
}