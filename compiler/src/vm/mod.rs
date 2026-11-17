//! NOVA Virtual Machine
//!
//! High-performance bytecode interpreter with JIT compilation support.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{BytecodeModule, BytecodeFunction, Instruction, ConstantPool, TypeTable, GlobalVar, Type, Constant, ConstantKind, Span, FileId};

/// VM configuration
#[derive(Debug, Clone)]
pub struct VmConfig {
    pub max_stack_size: usize,
    pub max_call_depth: usize,
    pub enable_jit: bool,
    pub jit_threshold: usize,
    pub gc_threshold: usize,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            max_stack_size: 1024 * 1024, // 1M stack slots
            max_call_depth: 10000,
            enable_jit: true,
            jit_threshold: 1000,
            gc_threshold: 1024 * 1024 * 10, // 10MB
        }
    }
}

/// Virtual Machine
pub struct VirtualMachine {
    config: VmConfig,
    modules: HashMap<String, Arc<BytecodeModule>>,
    globals: Vec<Value>,
    jit_compiler: Option<JitCompiler>,
    gc: GarbageCollector,
    thread_pool: ThreadPool,
}

/// JIT Compiler (simplified)
struct JitCompiler {
    cache: HashMap<usize, CompiledFunction>,
    invocation_counts: HashMap<usize, usize>,
    threshold: usize,
}

struct CompiledFunction {
    code: Box<dyn Fn(&mut VmContext) -> Result<(), VmError>>,
}

/// Garbage Collector
struct GarbageCollector {
    allocated: Vec<GcObject>,
    threshold: usize,
    bytes_allocated: usize,
}

struct GcObject {
    data: Vec<u8>,
    marked: bool,
    finalizer: Option<Box<dyn FnOnce() + Send>>,
}

/// Thread pool for async operations
struct ThreadPool {
    workers: Vec<thread::JoinHandle<()>>,
    job_queue: Arc<Mutex<Vec<Box<dyn FnOnce() + Send>>>>,
}

/// VM execution context
pub struct VmContext<'a> {
    vm: &'a mut VirtualMachine,
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    registers: Vec<Value>,
    instruction_pointer: usize,
    current_function: usize,
    exception_handlers: Vec<ExceptionHandler>,
}

/// Call frame
struct CallFrame {
    function: usize,
    base_pointer: usize,
    return_address: usize,
    locals: Vec<Value>,
}

/// Exception handler
struct ExceptionHandler {
    try_start: usize,
    try_end: usize,
    catch_start: usize,
    catch_type: Option<Type>,
}

/// VM values
#[derive(Debug, Clone)]
pub enum Value {
    Unit,
    Bool(bool),
    Char(char),
    I8(i8), I16(i16), I32(i32), I64(i64), I128(i128),
    U8(u8), U16(u16), U32(u32), U64(u64), U128(u128),
    F16(f32), F32(f32), F64(f64), F128(f64),
    Ref(usize), // Heap reference
    Ptr(*mut u8),
    Function(usize), // Function index
    Closure(usize, Vec<Value>), // Function index, captured values
    Array(Vec<Value>),
    Struct(Vec<Value>),
    Enum(usize, Vec<Value>), // Variant index, fields
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::I8(v) | Value::I16(v) | Value::I32(v) | Value::I64(v) | Value::I128(v) => *v != 0,
            Value::U8(v) | Value::U16(v) | Value::U32(v) | Value::U64(v) | Value::U128(v) => *v != 0,
            Value::F16(v) | Value::F32(v) | Value::F64(v) | Value::F128(v) => *v != 0.0,
            Value::Ref(_) | Value::Ptr(_) | Value::Function(_) | Value::Closure(_, _) => true,
            Value::Array(a) | Value::Struct(a) => !a.is_empty(),
            Value::Enum(_, fields) => !fields.is_empty(),
            Value::Unit => false,
            Value::Char(c) => *c != '\0',
        }
    }
}

/// VM errors
#[derive(Debug, thiserror::Error)]
pub enum VmError {
    #[error("Stack overflow")]
    StackOverflow,
    #[error("Call depth exceeded")]
    CallDepthExceeded,
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),
    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },
    #[error("Null pointer dereference")]
    NullPointerDeref,
    #[error("Uninitialized variable")]
    UninitializedVariable,
    #[error("Function not found: {0}")]
    FunctionNotFound(usize),
    #[error("Global not found: {0}")]
    GlobalNotFound(usize),
    #[error("Invalid instruction: {0}")]
    InvalidInstruction(String),
    #[error("Exception: {0}")]
    Exception(String),
    #[error("JIT compilation failed: {0}")]
    JitFailed(String),
    #[error("Out of memory")]
    OutOfMemory,
}

impl VirtualMachine {
    pub fn new(config: VmConfig) -> Self {
        Self {
            config: config.clone(),
            modules: HashMap::new(),
            globals: Vec::new(),
            jit_compiler: if config.enable_jit {
                Some(JitCompiler {
                    cache: HashMap::new(),
                    invocation_counts: HashMap::new(),
                    threshold: config.jit_threshold,
                })
            } else {
                None
            },
            gc: GarbageCollector {
                allocated: Vec::new(),
                threshold: config.gc_threshold,
                bytes_allocated: 0,
            },
            thread_pool: ThreadPool::new(num_cpus::get()),
        }
    }

    pub fn load_module(&mut self, name: String, module: BytecodeModule) {
        self.globals.resize(module.globals.len(), Value::Unit);
        for (i, global) in module.globals.iter().enumerate() {
            if let Some(init) = &global.init {
                self.globals[i] = self.constant_to_value(init);
            }
        }
        self.modules.insert(name, Arc::new(module));
    }

    pub fn execute(&mut self, module_name: &str, function_name: &str, args: Vec<Value>) -> Result<Value, VmError> {
        let module = self.modules.get(module_name)
            .ok_or_else(|| VmError::Exception(format!("Module not found: {}", module_name)))?;

        let func_idx = module.functions.iter()
            .position(|f| f.name == function_name)
            .ok_or_else(|| VmError::FunctionNotFound(0))?;

        let mut context = VmContext::new(self, func_idx, args)?;
        context.run()
    }

    fn constant_to_value(&self, constant: &Constant) -> Value {
        match &constant.kind {
            ConstantKind::ZeroSized => Value::Unit,
            ConstantKind::Value(bytes) => {
                // Simplified - would need proper deserialization
                Value::Unit
            }
            ConstantKind::Unevaluated(_, _) => Value::Unit,
        }
    }
}

impl VmContext<'_> {
    fn new(vm: &mut VirtualMachine, function: usize, args: Vec<Value>) -> Result<Self, VmError> {
        let module = vm.modules.values().next().unwrap(); // Simplified

        if args.len() != module.functions[function].params.len() {
            return Err(VmError::TypeMismatch {
                expected: module.functions[function].params.len().to_string(),
                got: args.len().to_string(),
            });
        }

        let mut locals = vec![Value::Unit; module.functions[function].locals.len()];
        locals[..args.len()].clone_from_slice(&args);

        Ok(Self {
            vm,
            stack: Vec::with_capacity(1024),
            frames: vec![CallFrame {
                function,
                base_pointer: 0,
                return_address: 0,
                locals,
            }],
            registers: Vec::new(),
            instruction_pointer: 0,
            current_function: function,
            exception_handlers: Vec::new(),
        })
    }

    fn run(&mut self) -> Result<Value, VmError> {
        let module = self.vm.modules.values().next().unwrap();

        loop {
            // Check JIT threshold
            if let Some(jit) = &mut self.vm.jit_compiler {
                let count = jit.invocation_counts.entry(self.current_function).or_insert(0);
                *count += 1;
                if *count >= jit.threshold {
                    // Trigger JIT compilation
                    self.compile_jit()?;
                }
            }

            let func = &module.functions[self.current_function];
            if self.instruction_pointer >= func.instructions.len() {
                // Function ended without return
                return Ok(Value::Unit);
            }

            let instruction = &func.instructions[self.instruction_pointer];
            self.instruction_pointer += 1;

            self.execute_instruction(instruction)?;

            // Check for stack overflow
            if self.stack.len() > self.vm.config.max_stack_size {
                return Err(VmError::StackOverflow);
            }
        }
    }

    fn execute_instruction(&mut self, instruction: &Instruction) -> Result<(), VmError> {
        use Instruction::*;

        match instruction {
            PushConst(idx) => {
                let module = self.vm.modules.values().next().unwrap();
                let constant = self.get_constant(module, *idx)?;
                self.stack.push(self.constant_to_value(constant));
            }
            PushLocal(idx) => {
                let frame = self.current_frame()?;
                self.stack.push(frame.locals[*idx as usize].clone());
            }
            PushGlobal(idx) => {
                self.stack.push(self.vm.globals[*idx as usize].clone());
            }
            Pop => { self.stack.pop(); }
            Dup => {
                let val = self.stack.last().cloned().ok_or(VmError::UninitializedVariable)?;
                self.stack.push(val);
            }
            Swap => {
                let len = self.stack.len();
                if len < 2 { return Err(VmError::UninitializedVariable); }
                self.stack.swap(len - 1, len - 2);
            }
            Add | Sub | Mul | Div | Rem => self.binary_op(instruction)?,
            Neg | Not => self.unary_op(instruction)?,
            And | Or | Xor => self.binary_op(instruction)?,
            Shl | Shr | Rotl | Rotr => self.binary_op(instruction)?,
            Eq | Ne | Lt | Le | Gt | Ge => self.compare_op(instruction)?,
            Jump(target) => self.instruction_pointer = *target as usize,
            JumpIf(target) => {
                let cond = self.pop_bool()?;
                if cond { self.instruction_pointer = *target as usize; }
            }
            JumpIfNot(target) => {
                let cond = self.pop_bool()?;
                if !cond { self.instruction_pointer = *target as usize; }
            }
            Call(idx) => self.call_function(*idx)?,
            CallIndirect(idx) => self.call_indirect(*idx)?,
            Return => return self.return_from_function(),
            TailCall(idx) => return self.tail_call(*idx),
            Load(idx) => {
                let frame = self.current_frame()?;
                self.stack.push(frame.locals[*idx as usize].clone());
            }
            Store(idx) => {
                let val = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
                let frame = self.current_frame_mut()?;
                frame.locals[*idx as usize] = val;
            }
            LoadGlobal(idx) => {
                self.stack.push(self.vm.globals[*idx as usize].clone());
            }
            StoreGlobal(idx) => {
                let val = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
                self.vm.globals[*idx as usize] = val;
            }
            Alloc(idx) => {
                let module = self.vm.modules.values().next().unwrap();
                let ty = &module.types.types[*idx as usize];
                let obj = self.allocate_object(ty)?;
                self.stack.push(Value::Ref(obj));
            }
            Free => {
                // Handled by GC
            }
            Ref => {
                let val = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
                let obj = self.allocate_value(val)?;
                self.stack.push(Value::Ref(obj));
            }
            Deref => {
                let val = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
                if let Value::Ref(idx) = val {
                    let obj = &self.vm.gc.allocated[idx];
                    // Simplified - would need proper deserialization
                    self.stack.push(Value::Unit);
                } else {
                    return Err(VmError::TypeMismatch { expected: "Ref".to_string(), got: "Other".to_string() });
                }
            }
            Borrow | BorrowMut => {
                // Reference creation
                let val = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
                let obj = self.allocate_value(val)?;
                self.stack.push(Value::Ref(obj));
            }
            MakeStruct(idx) => {
                let module = self.vm.modules.values().next().unwrap();
                let ty = &module.types.types[*idx as usize];
                // Pop fields from stack
                let field_count = self.get_struct_field_count(ty);
                let mut fields = Vec::new();
                for _ in 0..field_count {
                    fields.push(self.stack.pop().ok_or(VmError::UninitializedVariable)?);
                }
                fields.reverse();
                self.stack.push(Value::Struct(fields));
            }
            MakeEnum(idx, variant) => {
                // Similar to struct but with variant tag
                let module = self.vm.modules.values().next().unwrap();
                let ty = &module.types.types[*idx as usize];
                let field_count = self.get_enum_variant_field_count(ty, *variant);
                let mut fields = Vec::new();
                for _ in 0..field_count {
                    fields.push(self.stack.pop().ok_or(VmError::UninitializedVariable)?);
                }
                fields.reverse();
                self.stack.push(Value::Enum(*variant as usize, fields));
            }
            GetField(idx) => {
                let val = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
                if let Value::Struct(fields) = val {
                    self.stack.push(fields[*idx as usize].clone());
                } else if let Value::Enum(_, fields) = val {
                    self.stack.push(fields[*idx as usize].clone());
                } else {
                    return Err(VmError::TypeMismatch { expected: "Struct/Enum".to_string(), got: "Other".to_string() });
                }
            }
            SetField(idx) => {
                let val = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
                let obj = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
                if let Value::Struct(mut fields) = obj {
                    fields[*idx as usize] = val;
                    self.stack.push(Value::Struct(fields));
                } else {
                    return Err(VmError::TypeMismatch { expected: "Struct".to_string(), got: "Other".to_string() });
                }
            }
            GetVariant => {
                let val = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
                if let Value::Enum(variant, _) = val {
                    self.stack.push(Value::I64(variant as i64));
                } else {
                    return Err(VmError::TypeMismatch { expected: "Enum".to_string(), got: "Other".to_string() });
                }
            }
            MatchEnum(_) => {
                // Pattern matching - simplified
            }
            NewArray(idx) => {
                let module = self.vm.modules.values().next().unwrap();
                let ty = &module.types.types[*idx as usize];
                let len = self.pop_usize()?;
                let mut arr = Vec::with_capacity(len);
                for _ in 0..len {
                    arr.push(self.stack.pop().ok_or(VmError::UninitializedVariable)?);
                }
                arr.reverse();
                self.stack.push(Value::Array(arr));
            }
            NewSlice(idx) => {
                // Similar to array
            }
            ArrayLen => {
                let val = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
                let len = match val {
                    Value::Array(arr) => arr.len(),
                    Value::Struct(fields) => fields.len(),
                    _ => return Err(VmError::TypeMismatch { expected: "Array".to_string(), got: "Other".to_string() }),
                };
                self.stack.push(Value::USize(len));
            }
            ArrayGet => {
                let index = self.pop_usize()?;
                let val = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
                if let Value::Array(arr) = val {
                    let elem = arr.get(index).cloned().ok_or(VmError::IndexOutOfBounds(index))?;
                    self.stack.push(elem);
                } else {
                    return Err(VmError::TypeMismatch { expected: "Array".to_string(), got: "Other".to_string() });
                }
            }
            ArraySet => {
                let val = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
                let index = self.pop_usize()?;
                let arr_val = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
                if let Value::Array(mut arr) = arr_val {
                    if index < arr.len() {
                        arr[index] = val;
                        self.stack.push(Value::Array(arr));
                    } else {
                        return Err(VmError::IndexOutOfBounds(index));
                    }
                } else {
                    return Err(VmError::TypeMismatch { expected: "Array".to_string(), got: "Other".to_string() });
                }
            }
            Await => {
                // Async await - would need async runtime
            }
            Yield => {
                // Generator yield
            }
            Spawn => {
                // Task spawn
            }
            Try(handler) => {
                self.exception_handlers.push(ExceptionHandler {
                    try_start: self.instruction_pointer,
                    try_end: 0, // Would be set by EndTry
                    catch_start: *handler as usize,
                    catch_type: None,
                });
            }
            EndTry => {
                self.exception_handlers.pop();
            }
            Throw => {
                // Exception throwing
            }
            Rethrow => {
                // Rethrow current exception
            }
            Intrinsic(idx) => {
                self.call_intrinsic(*idx)?;
            }
            Nop => {}
            Line(_) => {}
        }
        Ok(())
    }

    // Helper methods
    fn current_frame(&self) -> Result<&CallFrame, VmError> {
        self.frames.last().ok_or(VmError::Exception("No call frame".to_string()))
    }

    fn current_frame_mut(&mut self) -> Result<&mut CallFrame, VmError> {
        self.frames.last_mut().ok_or(VmError::Exception("No call frame".to_string()))
    }

    fn pop_bool(&mut self) -> Result<bool, VmError> {
        Ok(self.stack.pop().ok_or(VmError::UninitializedVariable)?.is_truthy())
    }

    fn pop_usize(&mut self) -> Result<usize, VmError> {
        match self.stack.pop().ok_or(VmError::UninitializedVariable)? {
            Value::USize(v) | Value::U64(v) | Value::U32(v) | Value::U16(v) | Value::U8(v) => Ok(v as usize),
            Value::ISize(v) | Value::I64(v) | Value::I32(v) | Value::I16(v) | Value::I8(v) => Ok(v as usize),
            _ => Err(VmError::TypeMismatch { expected: "Integer".to_string(), got: "Other".to_string() }),
        }
    }

    fn binary_op(&mut self, op: &Instruction) -> Result<(), VmError> {
        let rhs = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
        let lhs = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
        let result = self.eval_binary(op, lhs, rhs)?;
        self.stack.push(result);
        Ok(())
    }

    fn unary_op(&mut self, op: &Instruction) -> Result<(), VmError> {
        let val = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
        let result = self.eval_unary(op, val)?;
        self.stack.push(result);
        Ok(())
    }

    fn compare_op(&mut self, op: &Instruction) -> Result<(), VmError> {
        let rhs = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
        let lhs = self.stack.pop().ok_or(VmError::UninitializedVariable)?;
        let result = self.eval_compare(op, lhs, rhs)?;
        self.stack.push(Value::Bool(result));
        Ok(())
    }

    fn eval_binary(&self, op: &Instruction, lhs: Value, rhs: Value) -> Result<Value, VmError> {
        // Simplified - would need proper type handling
        Ok(Value::Unit)
    }

    fn eval_unary(&self, op: &Instruction, val: Value) -> Result<Value, VmError> {
        Ok(val)
    }

    fn eval_compare(&self, op: &Instruction, lhs: Value, rhs: Value) -> Result<bool, VmError> {
        Ok(false)
    }

    fn call_function(&mut self, idx: usize) -> Result<(), VmError> {
        let module = self.vm.modules.values().next().unwrap();
        if idx >= module.functions.len() {
            return Err(VmError::FunctionNotFound(idx));
        }

        let func = &module.functions[idx];
        let arg_count = func.params.len();

        // Pop arguments
        let mut args = Vec::with_capacity(arg_count);
        for _ in 0..arg_count {
            args.push(self.stack.pop().ok_or(VmError::UninitializedVariable)?);
        }
        args.reverse();

        // Create new frame
        if self.frames.len() >= self.vm.config.max_call_depth {
            return Err(VmError::CallDepthExceeded);
        }

        let mut locals = vec![Value::Unit; func.locals.len()];
        locals[..arg_count].clone_from_slice(&args);

        self.frames.push(CallFrame {
            function: idx,
            base_pointer: self.stack.len(),
            return_address: self.instruction_pointer,
            locals,
        });

        self.current_function = idx;
        self.instruction_pointer = 0;
        Ok(())
    }

    fn call_indirect(&mut self, _idx: usize) -> Result<(), VmError> {
        // Function pointer call
        Ok(())
    }

    fn return_from_function(&mut self) -> Result<Value, VmError> {
        let return_val = self.stack.pop().unwrap_or(Value::Unit);

        if self.frames.len() == 1 {
            // Returning from top-level
            return Ok(return_val);
        }

        // Pop frame
        let frame = self.frames.pop().unwrap();
        self.instruction_pointer = frame.return_address;
        self.current_function = self.frames.last().unwrap().function;

        // Push return value
        self.stack.push(return_val);
        Ok(Value::Unit) // Will continue loop
    }

    fn tail_call(&mut self, idx: usize) -> Result<Value, VmError> {
        // Reuse current frame
        let module = self.vm.modules.values().next().unwrap();
        let func = &module.functions[idx];
        let arg_count = func.params.len();

        let mut args = Vec::with_capacity(arg_count);
        for _ in 0..arg_count {
            args.push(self.stack.pop().ok_or(VmError::UninitializedVariable)?);
        }
        args.reverse();

        let frame = self.current_frame_mut()?;
        frame.function = idx;
        frame.locals = vec![Value::Unit; func.locals.len()];
        frame.locals[..arg_count].clone_from_slice(&args);
        frame.return_address = 0; // No return
        self.instruction_pointer = 0;
        self.current_function = idx;
        Ok(Value::Unit)
    }

    fn get_constant(&self, module: &BytecodeModule, idx: u32) -> Result<&Constant, VmError> {
        // Simplified constant lookup
        Err(VmError::InvalidInstruction("Constant lookup not implemented".to_string()))
    }

    fn constant_to_value(&self, constant: &Constant) -> Value {
        Value::Unit
    }

    fn allocate_object(&mut self, _ty: &Type) -> Result<usize, VmError> {
        let idx = self.vm.gc.allocated.len();
        self.vm.gc.allocated.push(GcObject {
            data: Vec::new(),
            marked: false,
            finalizer: None,
        });
        Ok(idx)
    }

    fn allocate_value(&mut self, val: Value) -> Result<usize, VmError> {
        let idx = self.vm.gc.allocated.len();
        self.vm.gc.allocated.push(GcObject {
            data: Vec::new(), // Would serialize value
            marked: false,
            finalizer: None,
        });
        Ok(idx)
    }

    fn get_struct_field_count(&self, _ty: &Type) -> usize { 0 }
    fn get_enum_variant_field_count(&self, _ty: &Type, _variant: u32) -> usize { 0 }

    fn call_intrinsic(&mut self, _idx: u32) -> Result<(), VmError> { Ok(()) }

    fn compile_jit(&mut self) -> Result<(), VmError> {
        // JIT compilation would go here
        Ok(())
    }
}

impl ThreadPool {
    fn new(num_threads: usize) -> Self {
        let job_queue = Arc::new(Mutex::new(Vec::new()));
        let mut workers = Vec::new();

        for _ in 0..num_threads {
            let queue = job_queue.clone();
            workers.push(thread::spawn(move || {
                loop {
                    let job = {
                        let mut q = queue.lock().unwrap();
                        q.pop()
                    };
                    if let Some(job) = job {
                        job();
                    } else {
                        thread::park();
                    }
                }
            }));
        }

        Self { workers, job_queue }
    }

    fn spawn<F: FnOnce() + Send + 'static>(&self, f: F) {
        self.job_queue.lock().unwrap().push(Box::new(f));
        // Wake up a worker
        if let Some(worker) = self.workers.first() {
            worker.thread().unpark();
        }
    }
}

/// JIT Compiler (stub for future implementation)
mod jit {
    use super::*;

    pub struct JitCompiler {
        // JIT state
    }

    impl JitCompiler {
        pub fn new() -> Self {
            Self {}
        }

        pub fn compile(&mut self, _func: &BytecodeFunction) -> Result<CompiledFunction, VmError> {
            Ok(CompiledFunction {
                code: Box::new(|_ctx| Ok(())),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_creation() {
        let vm = VirtualMachine::new(VmConfig::default());
        assert!(vm.modules.is_empty());
    }

    #[test]
    fn test_value_truthy() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(Value::I32(42).is_truthy());
        assert!(!Value::I32(0).is_truthy());
        assert!(Value::Ref(0).is_truthy());
        assert!(!Value::Unit.is_truthy());
    }
}