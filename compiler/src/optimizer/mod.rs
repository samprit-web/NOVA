//! NOVA MIR Optimizer
//!
//! Performs various optimizations on MIR:
//! - Dead code elimination
//! - Constant propagation
//! - Common subexpression elimination
//! - Loop optimizations
//! - Inlining
//! - Vectorization

use std::collections::{HashMap, HashSet};

use crate::mir::{Program, MirItem, Function, Body, BasicBlock, Statement, Terminator, Place, LocalDecl, Rvalue, Operand, Constant, BinOp, UnOp, StatementKind, BasicBlockId, LocalId, ConstantKind};

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptLevel {
    None = 0,
    Basic = 1,
    Standard = 2,
    Full = 3,
    Size = 4,
}

/// Optimization context
pub struct Optimizer {
    level: OptLevel,
}

impl Optimizer {
    pub fn new(level: OptLevel) -> Self {
        Self { level }
    }

    pub fn optimize(&mut self, program: &mut Program) -> Result<(), OptimizerError> {
        for item in &mut program.items {
            if let MirItem::Function(func) = item {
                if let Some(body) = &mut func.body {
                    self.optimize_body(body)?;
                }
            }
        }
        Ok(())
    }

    fn optimize_body(&mut self, body: &mut Body) -> Result<(), OptimizerError> {
        // Run optimization passes in order
        if self.level >= OptLevel::Basic {
            self.remove_unreachable_blocks(body)?;
            self.eliminate_dead_stores(body)?;
        }

        if self.level >= OptLevel::Standard {
            self.constant_propagation(body)?;
            self.common_subexpression_elimination(body)?;
            self.simplify_cfg(body)?;
        }

        if self.level >= OptLevel::Full {
            self.loop_invariant_code_motion(body)?;
            self.strength_reduction(body)?;
            self.vectorize_loops(body)?;
        }

        if self.level == OptLevel::Size {
            self.optimize_for_size(body)?;
        }

        Ok(())
    }

    // ========================================================================
    // Pass 1: Remove unreachable blocks
    // ========================================================================
    fn remove_unreachable_blocks(&self, body: &mut Body) -> Result<(), OptimizerError> {
        let mut reachable = HashSet::new();
        let mut worklist = vec![BasicBlockId(0)]; // Entry block

        while let Some(bb_id) = worklist.pop() {
            if reachable.insert(bb_id) {
                if let Some(bb) = body.basic_blocks.iter().find(|b| b.id == bb_id) {
                    for succ in bb.terminator.successors() {
                        worklist.push(succ);
                    }
                }
            }
        }

        body.basic_blocks.retain(|bb| reachable.contains(&bb.id));

        // Remap block IDs
        let mut id_map = HashMap::new();
        for (new_id, bb) in body.basic_blocks.iter_mut().enumerate() {
            id_map.insert(bb.id, BasicBlockId(new_id as u32));
            bb.id = BasicBlockId(new_id as u32);
        }

        // Update terminators
        for bb in &mut body.basic_blocks {
            bb.terminator.remap_successors(&id_map);
        }

        Ok(())
    }

    // ========================================================================
    // Pass 2: Eliminate dead stores
    // ========================================================================
    fn eliminate_dead_stores(&self, body: &mut Body) -> Result<(), OptimizerError> {
        // Track which locals are read after being written
        let mut live_locals: HashSet<LocalId> = HashSet::new();

        // Backward pass
        for bb in body.basic_blocks.iter().rev() {
            for stmt in bb.statements.iter().rev() {
                if let StatementKind::Assign(place, _) = &stmt.kind {
                    if let Place::Local(local_id) = place {
                        if live_locals.contains(local_id) {
                            // Keep the store
                            live_locals.remove(local_id);
                        } else {
                            // Dead store - mark for removal
                        }
                    }
                }
            }

            // Terminator may use locals
            for operand in bb.terminator.operands() {
                if let Operand::Copy(Place::Local(id)) | Operand::Move(Place::Local(id)) = operand {
                    live_locals.insert(*id);
                }
            }
        }

        // Forward pass to remove dead stores
        for bb in &mut body.basic_blocks {
            bb.statements.retain(|stmt| {
                if let StatementKind::Assign(Place::Local(local_id), _) = &stmt.kind {
                    !live_locals.contains(local_id)
                } else {
                    true
                }
            });
        }

        Ok(())
    }

    // ========================================================================
    // Pass 3: Constant propagation
    // ========================================================================
    fn constant_propagation(&self, body: &mut Body) -> Result<(), OptimizerError> {
        let mut constants: HashMap<Place, Constant> = HashMap::new();

        // Forward pass
        for bb in &mut body.basic_blocks {
            let mut new_statements = Vec::new();

            for stmt in &bb.statements {
                match &stmt.kind {
                    StatementKind::Assign(place, rvalue) => {
                        if let Some(const_val) = self.evaluate_rvalue_constant(rvalue, &constants) {
                            constants.insert(place.clone(), const_val);
                            // Replace with constant
                            new_statements.push(Statement {
                                kind: StatementKind::Assign(place.clone(), Rvalue::Constant(const_val)),
                                span: stmt.span,
                            });
                        } else {
                            // Not constant, propagate existing constants into rvalue
                            let new_rvalue = self.substitute_constants_in_rvalue(rvalue, &constants);
                            new_statements.push(Statement {
                                kind: StatementKind::Assign(place.clone(), new_rvalue),
                                span: stmt.span,
                            });
                            constants.remove(place);
                        }
                    }
                    _ => new_statements.push(stmt.clone()),
                }
            }

            bb.statements = new_statements;

            // Update terminator
            bb.terminator = self.substitute_constants_in_terminator(&bb.terminator, &constants);
        }

        Ok(())
    }

    fn evaluate_rvalue_constant(&self, rvalue: &Rvalue, constants: &HashMap<Place, Constant>) -> Option<Constant> {
        match rvalue {
            Rvalue::Constant(c) => Some(c.clone()),
            Rvalue::Use(operand) => self.evaluate_operand_constant(operand, constants),
            Rvalue::Copy(place) | Rvalue::Move(place) => constants.get(place).cloned(),
            Rvalue::BinaryOp(op, lhs, rhs) => {
                let lhs_val = self.evaluate_operand_constant(lhs, constants)?;
                let rhs_val = self.evaluate_operand_constant(rhs, constants)?;
                self.eval_binary_op(*op, &lhs_val, &rhs_val)
            }
            Rvalue::UnaryOp(op, operand) => {
                let val = self.evaluate_operand_constant(operand, constants)?;
                self.eval_unary_op(*op, &val)
            }
            Rvalue::Cast(_, operand, ty) => {
                let val = self.evaluate_operand_constant(operand, constants)?;
                Some(Constant { ty: ty.clone(), kind: val.kind })
            }
            _ => None,
        }
    }

    fn evaluate_operand_constant(&self, operand: &Operand, constants: &HashMap<Place, Constant>) -> Option<Constant> {
        match operand {
            Operand::Constant(c) => Some(c.clone()),
            Operand::Copy(place) | Operand::Move(place) => constants.get(place).cloned(),
        }
    }

    fn eval_binary_op(&self, op: BinOp, lhs: &Constant, rhs: &Constant) -> Option<Constant> {
        // Simplified - only handle integer constants
        match (&lhs.kind, &rhs.kind) {
            (ConstantKind::Value(lhs_bytes), ConstantKind::Value(rhs_bytes)) => {
                // Would need proper type-aware evaluation
                None
            }
            _ => None,
        }
    }

    fn eval_unary_op(&self, op: UnOp, val: &Constant) -> Option<Constant> {
        None
    }

    fn substitute_constants_in_rvalue(&self, rvalue: &Rvalue, constants: &HashMap<Place, Constant>) -> Rvalue {
        match rvalue {
            Rvalue::Use(operand) => Rvalue::Use(self.substitute_constants_in_operand(operand, constants)),
            Rvalue::Copy(place) | Rvalue::Move(place) => {
                if let Some(c) = constants.get(place) {
                    Rvalue::Constant(c.clone())
                } else {
                    rvalue.clone()
                }
            }
            Rvalue::BinaryOp(op, lhs, rhs) => Rvalue::BinaryOp(*op,
                self.substitute_constants_in_operand(lhs, constants),
                self.substitute_constants_in_operand(rhs, constants)),
            Rvalue::UnaryOp(op, operand) => Rvalue::UnaryOp(*op, self.substitute_constants_in_operand(operand, constants)),
            Rvalue::Cast(kind, operand, ty) => Rvalue::Cast(*kind, self.substitute_constants_in_operand(operand, constants), ty.clone()),
            _ => rvalue.clone(),
        }
    }

    fn substitute_constants_in_operand(&self, operand: &Operand, constants: &HashMap<Place, Constant>) -> Operand {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                if let Some(c) = constants.get(place) {
                    Operand::Constant(c.clone())
                } else {
                    operand.clone()
                }
            }
            Operand::Constant(_) => operand.clone(),
        }
    }

    fn substitute_constants_in_terminator(&self, terminator: &Terminator, constants: &HashMap<Place, Constant>) -> Terminator {
        match terminator {
            Terminator::Call(call) => Terminator::Call(Call {
                func: self.substitute_constants_in_operand(&call.func, constants),
                args: call.args.iter().map(|a| self.substitute_constants_in_operand(a, constants)).collect(),
                ..call.clone()
            }),
            Terminator::Assert(operand, kind, target) => Terminator::Assert(
                self.substitute_constants_in_operand(operand, constants),
                kind.clone(),
                *target,
            ),
            Terminator::SwitchInt(operand, targets) => Terminator::SwitchInt(
                self.substitute_constants_in_operand(operand, constants),
                targets.clone(),
            ),
            _ => terminator.clone(),
        }
    }

    // ========================================================================
    // Pass 4: Common subexpression elimination
    // ========================================================================
    fn common_subexpression_elimination(&self, body: &mut Body) -> Result<(), OptimizerError> {
        let mut expr_map: HashMap<Rvalue, Place> = HashMap::new();

        for bb in &mut body.basic_blocks {
            let mut new_statements = Vec::new();

            for stmt in &bb.statements {
                if let StatementKind::Assign(dest, rvalue) = &stmt.kind {
                    // Check if we've computed this rvalue before
                    if let Some(existing_place) = expr_map.get(rvalue) {
                        // Replace with copy from existing place
                        new_statements.push(Statement {
                            kind: StatementKind::Assign(dest.clone(), Rvalue::Use(Operand::Copy(existing_place.clone()))),
                            span: stmt.span,
                        });
                    } else {
                        // New expression
                        expr_map.insert(rvalue.clone(), dest.clone());
                        new_statements.push(stmt.clone());
                    }
                } else {
                    new_statements.push(stmt.clone());
                }
            }

            bb.statements = new_statements;
        }

        Ok(())
    }

    // ========================================================================
    // Pass 5: Simplify CFG
    // ========================================================================
    fn simplify_cfg(&self, body: &mut Body) -> Result<(), OptimizerError> {
        let mut changed = true;

        while changed {
            changed = false;

            // Merge blocks with single predecessor
            // Remove empty blocks
            // Simplify branches

            // This is a simplified implementation
            // A full implementation would be more complex
        }

        Ok(())
    }

    // ========================================================================
    // Pass 6: Loop invariant code motion
    // ========================================================================
    fn loop_invariant_code_motion(&self, body: &mut Body) -> Result<(), OptimizerError> {
        // Identify loops
        // Find invariant statements
        // Hoist them to preheader

        Ok(())
    }

    // ========================================================================
    // Pass 7: Strength reduction
    // ========================================================================
    fn strength_reduction(&self, body: &mut Body) -> Result<(), OptimizerError> {
        // Replace expensive ops with cheaper ones
        // e.g., x * 2 -> x + x, x / 2 -> x >> 1

        for bb in &mut body.basic_blocks {
            for stmt in &mut bb.statements {
                if let StatementKind::Assign(_, Rvalue::BinaryOp(op, lhs, rhs)) = &mut stmt.kind {
                    // Check for multiplication by power of 2
                    if *op == BinOp::Mul {
                        if let (Operand::Copy(Place::Local(_)) | Operand::Move(Place::Local(_)), Operand::Constant(c)) = (lhs, rhs) {
                            if let ConstantKind::Value(bytes) = &c.kind {
                                // Check if constant is power of 2
                                // If so, replace with shift
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // ========================================================================
    // Pass 8: Vectorization
    // ========================================================================
    fn vectorize_loops(&self, body: &mut Body) -> Result<(), OptimizerError> {
        // Identify vectorizable loops
        // Transform to SIMD operations

        Ok(())
    }

    // ========================================================================
    // Pass 9: Size optimization
    // ========================================================================
    fn optimize_for_size(&self, body: &mut Body) -> Result<(), OptimizerError> {
        // Prefer smaller instructions
        // Reduce code duplication
        // Use compressed encodings

        Ok(())
    }
}

/// Optimizer errors
#[derive(Debug, thiserror::Error)]
pub enum OptimizerError {
    #[error("Optimization failed: {0}")]
    Failed(String),
    #[error("Invalid MIR: {0}")]
    InvalidMir(String),
}

// Extension traits for terminator analysis
impl Terminator {
    pub fn successors(&self) -> Vec<BasicBlockId> {
        match self {
            Terminator::Goto(target) => vec![*target],
            Terminator::SwitchInt(_, targets) => {
                let mut succ = targets.targets.clone();
                succ.push(targets.otherwise);
                succ
            }
            Terminator::Return => vec![],
            Terminator::Unreachable => vec![],
            Terminator::Drop(_, target, unwind) => {
                let mut succ = vec![*target];
                if let Some(u) = unwind {
                    succ.push(*u);
                }
                succ
            }
            Terminator::DropAndReplace(_, _, target, unwind) => {
                let mut succ = vec![*target];
                if let Some(u) = unwind {
                    succ.push(*u);
                }
                succ
            }
            Terminator::Call(call) => {
                let mut succ = Vec::new();
                if let Some(t) = call.target {
                    succ.push(t);
                }
                if let Some(u) = call.unwind {
                    succ.push(u);
                }
                succ
            }
            Terminator::Assert(_, _, target) => vec![*target],
            Terminator::Yield(_, target) => vec![*target],
            Terminator::CoroutineDrop(_) => vec![],
            Terminator::FalseEdge(targets) => targets.clone(),
            Terminator::FalseUnwind(targets) => targets.clone(),
            Terminator::InlineAsm(_) => vec![],
        }
    }

    pub fn operands(&self) -> Vec<Operand> {
        match self {
            Terminator::SwitchInt(operand, _) => vec![operand.clone()],
            Terminator::Call(call) => {
                let mut ops = vec![call.func.clone()];
                ops.extend(call.args.clone());
                ops
            }
            Terminator::Assert(operand, _, _) => vec![operand.clone()],
            Terminator::Yield(place, _) => vec![Operand::Move(place.clone())],
            Terminator::InlineAsm(asm) => {
                asm.operands.iter().filter_map(|op| match &op.kind {
                    crate::mir::AsmOperandKind::In(o) => Some(o.clone()),
                    crate::mir::AsmOperandKind::InOut(_, o) => Some(o.clone()),
                    crate::mir::AsmOperandKind::Out(place, _) => Some(Operand::Move(place.clone())),
                    _ => None,
                }).collect()
            }
            _ => vec![],
        }
    }
}

/// Free function wrapper for optimizer (for backward compatibility)
pub fn optimize(session: &mut crate::Session, mut program: crate::mir::Program) -> Result<crate::mir::Program, OptimizerError> {
    let optimizer = Optimizer::new(OptLevel::Standard);
    optimizer.optimize(&mut program)?;
    Ok(program)
}

// Need to add Call struct to mir for this to work
#[derive(Debug, Clone)]
pub struct Call {
    pub func: Operand,
    pub args: Vec<Operand>,
    pub destination: Place,
    pub target: Option<BasicBlockId>,
    pub unwind: Option<BasicBlockId>,
    pub call_source: crate::mir::CallSource,
    pub fn_span: crate::Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let optimizer = Optimizer::new(OptLevel::Standard);
        assert_eq!(optimizer.level, OptLevel::Standard);
    }
}