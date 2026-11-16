//! NOVA Code Generation
//!
//! Supports multiple backends:
//! - LLVM (native)
//! - Cranelift (native, WASM)
//! - NOVA Bytecode (VM)
//! - WASM (direct)

use std::collections::HashMap;

use crate::{Session, Span, Type};
use crate::mir::{Program, MirItem, Function, Body, BasicBlock, Statement, Terminator, Place, LocalDecl, Rvalue, Operand, Constant, BinOp, UnOp, CastKind, AggregateKind, Call, SwitchTargets, AssertKind, BasicBlockId, LocalId, FieldIdx, ItemId, Param, FnSig, Generics, StatementKind, TypeDefinition, TypeDefKind, Field, Variant};

/// Codegen backend trait
pub trait CodegenBackend {
    fn codegen_program(&mut self, program: &Program) -> Result<CodegenOutput, CodegenError>;
    fn target(&self) -> Target;
}

/// Target compilation backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    NativeX86_64,
    NativeAArch64,
    NativeRiscV64,
    Wasm32,
    Bytecode,
    Jvm,
    Dotnet,
    C,
}

/// Codegen output artifact
pub enum CodegenOutput {
    Native(NativeArtifact),
    Wasm(WasmArtifact),
    Bytecode(BytecodeArtifact),
    LlvmIr(String),
    Assembly(String),
}

/// Native binary artifact
pub struct NativeArtifact {
    pub bytes: Vec<u8>,
    pub debug_info: Option<Vec<u8>>,
}

/// WebAssembly artifact
pub struct WasmArtifact {
    pub module: Vec<u8>,
    pub debug_info: Option<Vec<u8>>,
}

/// NOVA bytecode artifact
pub struct BytecodeArtifact {
    pub module: crate::BytecodeModule,
}

/// Codegen errors
#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    #[error("Unsupported target: {0}")]
    UnsupportedTarget(String),
    #[error("Codegen failed: {0}")]
    Failed(String),
    #[error("Link failed: {0}")]
    LinkFailed(String),
    #[error("Invalid MIR: {0}")]
    InvalidMir(String),
}

/// Codegen context
pub struct CodegenCx<'a, B: CodegenBackend> {
    backend: &'a mut B,
    session: &'a mut Session,
    type_cache: HashMap<Type, B::Type>,
    function_cache: HashMap<ItemId, B::Function>,
    basic_block_cache: HashMap<BasicBlockId, B::BasicBlock>,
    local_cache: HashMap<LocalId, B::Value>,
    string_cache: HashMap<String, B::Value>,
}

impl<'a, B: CodegenBackend> CodegenCx<'a, B> {
    pub fn new(backend: &'a mut B, session: &'a mut Session) -> Self {
        Self {
            backend,
            session,
            type_cache: HashMap::new(),
            function_cache: HashMap::new(),
            basic_block_cache: HashMap::new(),
            local_cache: HashMap::new(),
            string_cache: HashMap::new(),
        }
    }

    pub fn codegen_program(&mut self, program: &Program) -> Result<CodegenOutput, CodegenError> {
        self.backend.codegen_program(program)
    }
}

/// LLVM backend
#[cfg(feature = "codegen")]
pub mod llvm {
    use super::*;
    use inkwell::{
        context::Context,
        module::Module,
        builder::Builder,
        types::{BasicType, BasicTypeEnum, FunctionType},
        values::{FunctionValue, PointerValue, BasicValueEnum, BasicMetadataValueEnum},
        AddressSpace,
    };

    pub struct LlvmBackend {
        context: Context,
        module: Module,
        builder: Builder,
        target_triple: String,
    }

    impl LlvmBackend {
        pub fn new(target: Target) -> Result<Self, CodegenError> {
            let context = Context::create();
            let module = context.create_module("nova_module");
            let builder = context.create_builder();

            let target_triple = match target {
                Target::NativeX86_64 => "x86_64-unknown-linux-gnu".to_string(),
                Target::NativeAArch64 => "aarch64-unknown-linux-gnu".to_string(),
                Target::NativeRiscV64 => "riscv64-unknown-linux-gnu".to_string(),
                _ => return Err(CodegenError::UnsupportedTarget("LLVM backend only supports native targets".to_string())),
            };

            module.set_triple(&target_triple);

            Ok(Self { context, module, builder, target_triple })
        }

        fn type_of(&self, ty: &Type) -> BasicTypeEnum {
            match ty {
                Type::Unit => self.context.void_type().into(),
                Type::Bool => self.context.bool_type().into(),
                Type::I8 => self.context.i8_type().into(),
                Type::I16 => self.context.i16_type().into(),
                Type::I32 => self.context.i32_type().into(),
                Type::I64 => self.context.i64_type().into(),
                Type::I128 => self.context.i128_type().into(),
                Type::ISize => self.context.ptr_sized_int_type(&self.module.get_target_data().unwrap(), None).into(),
                Type::U8 => self.context.i8_type().into(),
                Type::U16 => self.context.i16_type().into(),
                Type::U32 => self.context.i32_type().into(),
                Type::U64 => self.context.i64_type().into(),
                Type::U128 => self.context.i128_type().into(),
                Type::USize => self.context.ptr_sized_int_type(&self.module.get_target_data().unwrap(), None).into(),
                Type::F16 => self.context.f16_type().into(),
                Type::F32 => self.context.f32_type().into(),
                Type::F64 => self.context.f64_type().into(),
                Type::F128 => self.context.f128_type().into(),
                Type::Never => self.context.void_type().into(),
                Type::Ptr(inner) => {
                    let inner_ty = self.type_of(inner);
                    inner_ty.ptr_type(AddressSpace::default()).into()
                }
                Type::Ref(inner, _) => {
                    let inner_ty = self.type_of(inner);
                    inner_ty.ptr_type(AddressSpace::default()).into()
                }
                Type::Box(inner) => {
                    let inner_ty = self.type_of(inner);
                    inner_ty.ptr_type(AddressSpace::default()).into()
                }
                Type::Fn(params, ret) => {
                    let param_types: Vec<_> = params.iter().map(|p| self.type_of(p)).collect();
                    let ret_type = self.type_of(ret);
                    let fn_type = ret_type.fn_type(&param_types.iter().map(|t| t.into()).collect::<Vec<_>>(), false);
                    fn_type.ptr_type(AddressSpace::default()).into()
                }
                _ => self.context.i8_type().into(), // Fallback
            }
        }
    }

    impl CodegenBackend for LlvmBackend {
        fn codegen_program(&mut self, program: &Program) -> Result<CodegenOutput, CodegenError> {
            // Codegen each function
            for item in &program.items {
                if let MirItem::Function(func) = item {
                    self.codegen_function(func)?;
                }
            }

            // Verify module
            if let Err(e) = self.module.verify() {
                return Err(CodegenError::Failed(format!("Module verification failed: {}", e)));
            }

            // Emit object file or assembly
            let output = self.module.print_to_string().to_string();
            Ok(CodegenOutput::LlvmIr(output))
        }

        fn target(&self) -> Target {
            Target::NativeX86_64 // Simplified
        }
    }

    impl LlvmBackend {
        fn codegen_function(&mut self, func: &Function) -> Result<FunctionValue, CodegenError> {
            let fn_type = self.function_type(&func.signature);
            let fn_value = self.module.add_function(&func.name.name, fn_type, None);

            // Set linkage/visibility
            match func.visibility {
                Visibility::Public => {}
                _ => fn_value.set_linkage(inkwell::module::Linkage::Internal),
            }

            if func.unsafety == Unsafety::Unsafe {
                // No special handling needed for LLVM
            }

            // Create entry block
            let entry = self.context.append_basic_block(fn_value, "entry");
            self.builder.position_at_end(entry);

            // Codegen body
            if let Some(body) = &func.body {
                self.codegen_body(fn_value, body)?;
            }

            Ok(fn_value)
        }

        fn function_type(&self, sig: &FnSig) -> FunctionType {
            let param_types: Vec<_> = sig.params.iter().map(|p| self.type_of(&p.ty).into()).collect();
            let ret_type = self.type_of(&sig.return_type);
            ret_type.fn_type(&param_types, sig.c_variadic)
        }

        fn codegen_body(&mut self, fn_value: FunctionValue, body: &Body) -> Result<(), CodegenError> {
            // Allocate locals
            for (i, local) in body.local_decls.iter().enumerate() {
                let alloca = self.builder.build_alloca(self.type_of(&local.ty), &format!("local_{}", i));
                // Store in local cache
            }

            // Codegen basic blocks
            for bb in &body.basic_blocks {
                self.codegen_basic_block(fn_value, bb)?;
            }

            Ok(())
        }

        fn codegen_basic_block(&mut self, fn_value: FunctionValue, bb: &BasicBlock) -> Result<(), CodegenError> {
            // Create or get LLVM basic block
            let llvm_bb = self.context.append_basic_block(fn_value, &format!("bb{}", bb.id.0));
            self.builder.position_at_end(llvm_bb);

            // Codegen statements
            for stmt in &bb.statements {
                self.codegen_statement(stmt)?;
            }

            // Codegen terminator
            self.codegen_terminator(&bb.terminator)?;

            Ok(())
        }

        fn codegen_statement(&mut self, stmt: &Statement) -> Result<(), CodegenError> {
            match &stmt.kind {
                StatementKind::Assign(place, rvalue) => {
                    let dest = self.place_to_value(place)?;
                    let value = self.rvalue_to_value(rvalue)?;
                    self.builder.build_store(dest, value)?;
                }
                StatementKind::StorageLive(local) => {
                    // Already handled by alloca
                }
                StatementKind::StorageDead(_local) => {
                    // Lifetime end - no action needed for LLVM
                }
                _ => {}
            }
            Ok(())
        }

        fn rvalue_to_value(&mut self, rvalue: &Rvalue) -> Result<BasicValueEnum, CodegenError> {
            Ok(match rvalue {
                Rvalue::Use(operand) => self.operand_to_value(operand)?,
                Rvalue::Copy(place) | Rvalue::Move(place) => {
                    let ptr = self.place_to_value(place)?;
                    self.builder.build_load(self.type_of(&place.ty()), ptr, "load")?
                }
                Rvalue::Ref(place, _) => self.place_to_value(place)?,
                Rvalue::AddressOf(place) => self.place_to_value(place)?,
                Rvalue::Len(place) => {
                    // Array/slice length
                    let ptr = self.place_to_value(place)?;
                    // Simplified - would need actual type info
                    self.context.i64_type().const_zero().into()
                }
                Rvalue::Cast(kind, operand, ty) => {
                    let value = self.operand_to_value(operand)?;
                    let dest_ty = self.type_of(ty);
                    self.cast_value(value, *kind, dest_ty)?
                }
                Rvalue::BinaryOp(op, lhs, rhs) => {
                    let lhs_val = self.operand_to_value(lhs)?;
                    let rhs_val = self.operand_to_value(rhs)?;
                    self.binary_op(*op, lhs_val, rhs_val)?
                }
                Rvalue::UnaryOp(op, operand) => {
                    let val = self.operand_to_value(operand)?;
                    self.unary_op(*op, val)?
                }
                Rvalue::Aggregate(kind, operands) => {
                    self.aggregate_value(kind, operands)?
                }
                Rvalue::Constant(constant) => self.constant_to_value(constant)?,
                _ => self.context.i8_type().const_zero().into(),
            })
        }

        fn operand_to_value(&mut self, operand: &Operand) -> Result<BasicValueEnum, CodegenError> {
            Ok(match operand {
                Operand::Copy(place) | Operand::Move(place) => {
                    let ptr = self.place_to_value(place)?;
                    self.builder.build_load(self.type_of(&place.ty()), ptr, "load")?
                }
                Operand::Constant(constant) => self.constant_to_value(constant)?,
            })
        }

        fn constant_to_value(&mut self, constant: &Constant) -> Result<BasicValueEnum, CodegenError> {
            match &constant.kind {
                ConstantKind::ZeroSized => Ok(self.context.void_type().const_nil().into()),
                ConstantKind::Value(bytes) => {
                    // Interpret bytes based on type
                    Ok(self.context.i8_type().const_zero().into())
                }
                ConstantKind::Unevaluated(_, _) => {
                    // Would need const evaluator
                    Ok(self.context.i8_type().const_zero().into())
                }
            }
        }

        fn place_to_value(&mut self, place: &Place) -> Result<PointerValue, CodegenError> {
            // Simplified - would need proper place resolution
            Ok(self.builder.build_alloca(self.context.i8_type(), "temp")?)
        }

        fn cast_value(&mut self, value: BasicValueEnum, kind: CastKind, dest_ty: BasicTypeEnum) -> Result<BasicValueEnum, CodegenError> {
            // Simplified casting
            Ok(value)
        }

        fn binary_op(&mut self, op: BinOp, lhs: BasicValueEnum, rhs: BasicValueEnum) -> Result<BasicValueEnum, CodegenError> {
            // Simplified binary ops
            Ok(lhs)
        }

        fn unary_op(&mut self, op: UnOp, val: BasicValueEnum) -> Result<BasicValueEnum, CodegenError> {
            Ok(val)
        }

        fn aggregate_value(&mut self, kind: &AggregateKind, operands: &[Operand]) -> Result<BasicValueEnum, CodegenError> {
            Ok(self.context.i8_type().const_zero().into())
        }

        fn codegen_terminator(&mut self, terminator: &Terminator) -> Result<(), CodegenError> {
            match terminator {
                Terminator::Goto(target) => {
                    // Branch to target block
                }
                Terminator::SwitchInt(_, _) => {}
                Terminator::Return => {
                    self.builder.build_return(None)?;
                }
                Terminator::Unreachable => {
                    self.builder.build_unreachable()?;
                }
                _ => {}
            }
            Ok(())
        }
    }
}

/// Cranelift backend
#[cfg(feature = "codegen")]
pub mod cranelift {
    use super::*;

    pub struct CraneliftBackend {
        // Cranelift context
    }

    impl CraneliftBackend {
        pub fn new(_target: Target) -> Result<Self, CodegenError> {
            Ok(Self {})
        }
    }

    impl CodegenBackend for CraneliftBackend {
        fn codegen_program(&mut self, _program: &Program) -> Result<CodegenOutput, CodegenError> {
            Err(CodegenError::Failed("Cranelift backend not implemented".to_string()))
        }

        fn target(&self) -> Target {
            Target::NativeX86_64
        }
    }
}

/// Bytecode backend
pub mod bytecode {
    use super::*;
    use crate::{BytecodeModule, BytecodeFunction, Instruction, ConstantPool, TypeTable, GlobalVar, Type, Mutability};

    pub struct BytecodeBackend {
        module: BytecodeModule,
    }

    impl BytecodeBackend {
        pub fn new() -> Self {
            Self {
                module: BytecodeModule {
                    version: 1,
                    functions: Vec::new(),
                    constants: ConstantPool {
                        integers: Vec::new(),
                        floats: Vec::new(),
                        strings: Vec::new(),
                        bytes: Vec::new(),
                    },
                    types: TypeTable { types: Vec::new() },
                    globals: Vec::new(),
                    debug_info: None,
                },
            }
        }
    }

    impl CodegenBackend for BytecodeBackend {
        fn codegen_program(&mut self, program: &Program) -> Result<CodegenOutput, CodegenError> {
            for item in &program.items {
                match item {
                    MirItem::Function(func) => {
                        self.codegen_function(func)?;
                    }
                    MirItem::Static(st) => {
                        self.codegen_static(st)?;
                    }
                    MirItem::Const(c) => {
                        self.codegen_const(c)?;
                    }
                    MirItem::TypeDefinition(ty) => {
                        self.codegen_type_def(ty)?;
                    }
                }
            }

            Ok(CodegenOutput::Bytecode(BytecodeArtifact {
                module: std::mem::take(&mut self.module),
            }))
        }

        fn target(&self) -> Target {
            Target::Bytecode
        }
    }

    impl BytecodeBackend {
        fn codegen_function(&mut self, func: &Function) -> Result<(), CodegenError> {
            let mut instructions = Vec::new();
            let mut locals = Vec::new();

            // Collect local types
            for local in &func.body.as_ref().unwrap().local_decls {
                locals.push(local.ty.clone());
            }

            // Codegen basic blocks
            for bb in &func.body.as_ref().unwrap().basic_blocks {
                for stmt in &bb.statements {
                    self.codegen_statement(stmt, &mut instructions)?;
                }
                self.codegen_terminator(&bb.terminator, &mut instructions)?;
            }

            // Add return if missing
            if !matches!(instructions.last(), Some(Instruction::Return)) {
                instructions.push(Instruction::Return);
            }

            let bytecode_func = BytecodeFunction {
                name: func.name.name.clone(),
                params: func.signature.params.iter().map(|p| p.ty.clone()).collect(),
                return_type: func.signature.return_type.clone(),
                locals,
                instructions,
                exception_handlers: Vec::new(),
            };

            self.module.functions.push(bytecode_func);
            Ok(())
        }

        fn codegen_statement(&mut self, stmt: &Statement, instructions: &mut Vec<Instruction>) -> Result<(), CodegenError> {
            match &stmt.kind {
                StatementKind::Assign(place, rvalue) => {
                    // Load rvalue
                    self.codegen_rvalue(rvalue, instructions)?;
                    // Store to place
                    self.codegen_place_store(place, instructions)?;
                }
                _ => {}
            }
            Ok(())
        }

        fn codegen_rvalue(&mut self, rvalue: &Rvalue, instructions: &mut Vec<Instruction>) -> Result<(), CodegenError> {
            match rvalue {
                Rvalue::Use(operand) => self.codegen_operand(operand, instructions)?,
                Rvalue::Copy(place) | Rvalue::Move(place) => self.codegen_place_load(place, instructions)?,
                Rvalue::Ref(place, _) => self.codegen_place_addr(place, instructions)?,
                Rvalue::AddressOf(place) => self.codegen_place_addr(place, instructions)?,
                Rvalue::BinaryOp(op, lhs, rhs) => {
                    self.codegen_operand(lhs, instructions)?;
                    self.codegen_operand(rhs, instructions)?;
                    instructions.push(self.binop_to_instruction(*op));
                }
                Rvalue::UnaryOp(op, operand) => {
                    self.codegen_operand(operand, instructions)?;
                    instructions.push(self.unop_to_instruction(*op));
                }
                Rvalue::Cast(kind, operand, ty) => {
                    self.codegen_operand(operand, instructions)?;
                    instructions.push(Instruction::Intrinsic(0)); // Cast intrinsic
                }
                Rvalue::Aggregate(kind, operands) => {
                    for op in operands {
                        self.codegen_operand(op, instructions)?;
                    }
                    instructions.push(Instruction::MakeStruct(0)); // Type index
                }
                Rvalue::Constant(c) => {
                    self.codegen_constant(c, instructions)?;
                }
                _ => {}
            }
            Ok(())
        }

        fn codegen_operand(&mut self, operand: &Operand, instructions: &mut Vec<Instruction>) -> Result<(), CodegenError> {
            match operand {
                Operand::Copy(place) | Operand::Move(place) => {
                    self.codegen_place_load(place, instructions)?;
                }
                Operand::Constant(c) => {
                    self.codegen_constant(c, instructions)?;
                }
            }
            Ok(())
        }

        fn codegen_place_load(&mut self, place: &Place, instructions: &mut Vec<Instruction>) -> Result<(), CodegenError> {
            match place {
                Place::Local(id) => instructions.push(Instruction::PushLocal(id.0)),
                Place::Static(id) => instructions.push(Instruction::PushGlobal(id.0)),
                Place::Field(base, idx) => {
                    self.codegen_place_load(base, instructions)?;
                    instructions.push(Instruction::GetField(idx.0));
                }
                Place::Index(base, index) => {
                    self.codegen_place_load(base, instructions)?;
                    self.codegen_place_load(index, instructions)?;
                    instructions.push(Instruction::ArrayGet);
                }
                Place::Deref(base) => {
                    self.codegen_place_load(base, instructions)?;
                    instructions.push(Instruction::Deref);
                }
                _ => instructions.push(Instruction::Nop),
            }
            Ok(())
        }

        fn codegen_place_store(&mut self, place: &Place, instructions: &mut Vec<Instruction>) -> Result<(), CodegenError> {
            match place {
                Place::Local(id) => instructions.push(Instruction::Store(id.0)),
                Place::Static(id) => instructions.push(Instruction::StoreGlobal(id.0)),
                Place::Field(base, idx) => {
                    self.codegen_place_load(base, instructions)?;
                    instructions.push(Instruction::SetField(idx.0));
                }
                Place::Index(base, index) => {
                    self.codegen_place_load(base, instructions)?;
                    self.codegen_place_load(index, instructions)?;
                    instructions.push(Instruction::ArraySet);
                }
                _ => {}
            }
            Ok(())
        }

        fn codegen_place_addr(&mut self, place: &Place, instructions: &mut Vec<Instruction>) -> Result<(), CodegenError> {
            match place {
                Place::Local(id) => {
                    // Push address of local
                    instructions.push(Instruction::PushLocal(id.0));
                    instructions.push(Instruction::Ref);
                }
                Place::Static(id) => {
                    instructions.push(Instruction::PushGlobal(id.0));
                    instructions.push(Instruction::Ref);
                }
                _ => instructions.push(Instruction::Nop),
            }
            Ok(())
        }

        fn codegen_constant(&mut self, constant: &Constant, instructions: &mut Vec<Instruction>) -> Result<(), CodegenError> {
            let idx = match &constant.kind {
                ConstantKind::ZeroSized => {
                    // Push unit
                    self.module.constants.integers.push(0);
                    self.module.constants.integers.len() - 1
                }
                ConstantKind::Value(bytes) => {
                    self.module.constants.bytes.push(bytes.clone());
                    self.module.constants.bytes.len() - 1
                }
                ConstantKind::Unevaluated(_, _) => {
                    self.module.constants.integers.push(0);
                    self.module.constants.integers.len() - 1
                }
            };
            instructions.push(Instruction::PushConst(idx as u32));
            Ok(())
        }

        fn codegen_terminator(&mut self, terminator: &Terminator, instructions: &mut Vec<Instruction>) -> Result<(), CodegenError> {
            match terminator {
                Terminator::Goto(target) => {
                    instructions.push(Instruction::Jump(target.0));
                }
                Terminator::SwitchInt(_, targets) => {
                    // Simplified - would need jump table
                    instructions.push(Instruction::Jump(targets.otherwise.0));
                }
                Terminator::Return => {
                    instructions.push(Instruction::Return);
                }
                Terminator::Unreachable => {
                    instructions.push(Instruction::Nop);
                }
                Terminator::Call(call) => {
                    // Load function and args
                    self.codegen_operand(&call.func, instructions)?;
                    for arg in &call.args {
                        self.codegen_operand(arg, instructions)?;
                    }
                    instructions.push(Instruction::Call(0)); // Function index
                    if let Some(dest) = &call.destination {
                        self.codegen_place_store(dest, instructions)?;
                    }
                }
                _ => {}
            }
            Ok(())
        }

        fn binop_to_instruction(&self, op: BinOp) -> Instruction {
            match op {
                BinOp::Add => Instruction::Add,
                BinOp::Sub => Instruction::Sub,
                BinOp::Mul => Instruction::Mul,
                BinOp::Div => Instruction::Div,
                BinOp::Rem => Instruction::Rem,
                BinOp::BitAnd => Instruction::And,
                BinOp::BitOr => Instruction::Or,
                BinOp::BitXor => Instruction::Xor,
                BinOp::Shl => Instruction::Shl,
                BinOp::Shr => Instruction::Shr,
                BinOp::Eq => Instruction::Eq,
                BinOp::Ne => Instruction::Ne,
                BinOp::Lt => Instruction::Lt,
                BinOp::Le => Instruction::Le,
                BinOp::Gt => Instruction::Gt,
                BinOp::Ge => Instruction::Ge,
                BinOp::Offset => Instruction::Add,
            }
        }

        fn unop_to_instruction(&self, op: UnOp) -> Instruction {
            match op {
                UnOp::Not => Instruction::Not,
                UnOp::Neg => Instruction::Neg,
            }
        }

        fn codegen_static(&mut self, st: &crate::mir::Static) -> Result<(), CodegenError> {
            self.module.globals.push(GlobalVar {
                name: st.name.name.clone(),
                ty: st.ty.clone(),
                init: Some(st.value.clone()),
                mutable: st.mutability == Mutability::Mutable,
            });
            Ok(())
        }

        fn codegen_const(&mut self, c: &crate::mir::Const) -> Result<(), CodegenError> {
            self.module.globals.push(GlobalVar {
                name: c.name.name.clone(),
                ty: c.ty.clone(),
                init: Some(c.value.clone()),
                mutable: false,
            });
            Ok(())
        }

        fn codegen_type_def(&mut self, ty: &TypeDefinition) -> Result<(), CodegenError> {
            // Register type in type table
            let type_idx = self.module.types.types.len();
            self.module.types.types.push(Type::Struct(type_idx as u32));
            Ok(())
        }
    }
}

/// WASM backend
#[cfg(feature = "codegen")]
pub mod wasm {
    use super::*;

    pub struct WasmBackend {
        // WASM module builder
    }

    impl WasmBackend {
        pub fn new() -> Result<Self, CodegenError> {
            Ok(Self {})
        }
    }

    impl CodegenBackend for WasmBackend {
        fn codegen_program(&mut self, _program: &Program) -> Result<CodegenOutput, CodegenError> {
            Err(CodegenError::Failed("WASM backend not implemented".to_string()))
        }

        fn target(&self) -> Target {
            Target::Wasm32
        }
    }
}

/// Main codegen function
pub fn codegen(session: &mut Session, mir: crate::mir::Program, target: Target) -> Result<crate::CompilationOutput, Vec<crate::Diagnostic>> {
    let output = match target {
        #[cfg(feature = "codegen")]
        Target::NativeX86_64 | Target::NativeAArch64 | Target::NativeRiscV64 => {
            let mut backend = llvm::LlvmBackend::new(target)?;
            backend.codegen_program(&mir)?
        }
        #[cfg(feature = "codegen")]
        Target::Wasm32 => {
            let mut backend = wasm::WasmBackend::new()?;
            backend.codegen_program(&mir)?
        }
        Target::Bytecode => {
            let mut backend = bytecode::BytecodeBackend::new();
            backend.codegen_program(&mir)?
        }
        _ => {
            return Err(vec![crate::Diagnostic::new(
                crate::Severity::Error,
                format!("Unsupported target: {:?}", target),
                Span::new(crate::FileId(0), 0, 0),
            )]);
        }
    };

    Ok(crate::CompilationOutput {
        artifact: match output {
            CodegenOutput::Native(art) => crate::Artifact::Native(art.bytes),
            CodegenOutput::Wasm(art) => crate::Artifact::Wasm(art.module),
            CodegenOutput::Bytecode(art) => crate::Artifact::Bytecode(art.module),
            CodegenOutput::LlvmIr(ir) => crate::Artifact::LlvmIr(ir),
            CodegenOutput::Assembly(asm) => crate::Artifact::Assembly(asm),
        },
        diagnostics: session.diagnostics.clone(),
    })
}