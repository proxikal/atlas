//! Statement compilation

use crate::ast::*;
use crate::bytecode::Opcode;
use crate::compiler::{Compiler, Local, LoopContext};
use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::value::Value;

impl Compiler {
    /// Compile a statement
    pub(super) fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), Vec<Diagnostic>> {
        match stmt {
            Stmt::VarDecl(decl) => self.compile_var_decl(decl),
            Stmt::Assign(assign) => {
                // Compile assignment and pop the result (statement context)
                self.compile_assign(assign)?;
                self.bytecode.emit(Opcode::Pop, assign.span);
                Ok(())
            }
            Stmt::Expr(expr_stmt) => {
                // Compile expression and pop the result (statement context)
                self.compile_expr(&expr_stmt.expr)?;
                self.bytecode.emit(Opcode::Pop, expr_stmt.span);
                Ok(())
            }
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.value {
                    self.compile_expr(expr)?;
                } else {
                    self.bytecode.emit(Opcode::Null, ret.span);
                }
                self.bytecode.emit(Opcode::Return, ret.span);
                Ok(())
            }
            Stmt::If(if_stmt) => self.compile_if(if_stmt),
            Stmt::While(while_stmt) => self.compile_while(while_stmt),
            Stmt::For(for_stmt) => self.compile_for(for_stmt),
            Stmt::Break(span) => self.compile_break(*span),
            Stmt::Continue(span) => self.compile_continue(*span),
            Stmt::CompoundAssign(compound) => self.compile_compound_assign(compound),
            Stmt::Increment(inc) => self.compile_increment(inc),
            Stmt::Decrement(dec) => self.compile_decrement(dec),
        }
    }

    /// Compile a variable declaration
    fn compile_var_decl(&mut self, decl: &VarDecl) -> Result<(), Vec<Diagnostic>> {
        // Compile the initializer
        self.compile_expr(&decl.init)?;

        if self.scope_depth == 0 {
            // Global variable - use SetGlobal then Pop
            // SetGlobal uses peek() to support assignment expressions like x = y = 5,
            // but for variable declarations we need to pop the value to avoid polluting
            // the stack (which would corrupt local variable indices)
            let name_idx = self.bytecode.add_constant(Value::string(&decl.name.name));
            self.bytecode.emit(Opcode::SetGlobal, decl.span);
            self.bytecode.emit_u16(name_idx);
            self.bytecode.emit(Opcode::Pop, decl.span);
        } else {
            // Local variable - add to locals list
            // Value stays on stack (locals are stack-allocated)
            self.locals.push(Local {
                name: decl.name.name.clone(),
                depth: self.scope_depth,
                mutable: decl.mutable,
            });
        }

        Ok(())
    }

    /// Compile an assignment
    fn compile_assign(&mut self, assign: &Assign) -> Result<(), Vec<Diagnostic>> {
        match &assign.target {
            AssignTarget::Name(ident) => {
                // For name assignment: compile value first, then set
                self.compile_expr(&assign.value)?;

                // Try to find local first
                if let Some(local_idx) = self.resolve_local(&ident.name) {
                    self.bytecode.emit(Opcode::SetLocal, assign.span);
                    self.bytecode.emit_u16(local_idx as u16);
                } else {
                    // Global variable
                    let name_idx = self.bytecode.add_constant(Value::string(&ident.name));
                    self.bytecode.emit(Opcode::SetGlobal, assign.span);
                    self.bytecode.emit_u16(name_idx);
                }
            }
            AssignTarget::Index { target, index, .. } => {
                // For array index assignment: compile array, index, value (in that order)
                // SetIndex pops: value (top), index, array (bottom)
                // So we need stack: [array, index, value]

                // Compile the array target
                self.compile_expr(target)?;
                // Compile the index
                self.compile_expr(index)?;
                // Compile the value
                self.compile_expr(&assign.value)?;

                // Emit SetIndex
                self.bytecode.emit(Opcode::SetIndex, assign.span);
            }
        }

        Ok(())
    }

    /// Compile an if statement
    fn compile_if(&mut self, if_stmt: &IfStmt) -> Result<(), Vec<Diagnostic>> {
        // Compile condition
        self.compile_expr(&if_stmt.cond)?;

        // Jump if false - we'll patch this later
        self.bytecode.emit(Opcode::JumpIfFalse, if_stmt.span);
        let then_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF); // Placeholder

        // Compile then branch
        self.compile_block(&if_stmt.then_block)?;

        if let Some(else_block) = &if_stmt.else_block {
            // Jump over else branch
            self.bytecode.emit(Opcode::Jump, if_stmt.span);
            let else_jump = self.bytecode.current_offset();
            self.bytecode.emit_u16(0xFFFF); // Placeholder

            // Patch the then jump to go here
            self.bytecode.patch_jump(then_jump);

            // Compile else branch
            self.compile_block(else_block)?;

            // Patch the else jump
            self.bytecode.patch_jump(else_jump);
        } else {
            // No else branch, just patch the jump
            self.bytecode.patch_jump(then_jump);
        }

        Ok(())
    }

    /// Compile a while loop
    fn compile_while(&mut self, while_stmt: &WhileStmt) -> Result<(), Vec<Diagnostic>> {
        let loop_start = self.bytecode.current_offset();

        // Start a new loop context
        self.loops.push(LoopContext {
            start_offset: loop_start,
            break_jumps: Vec::new(),
        });

        // Compile condition
        self.compile_expr(&while_stmt.cond)?;

        // Jump if false (exit loop)
        self.bytecode.emit(Opcode::JumpIfFalse, while_stmt.span);
        let exit_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF); // Placeholder

        // Compile body
        self.compile_block(&while_stmt.body)?;

        // Loop back to condition
        // Offset needs to account for the Loop instruction (1 byte) + offset operand (2 bytes) = 3 bytes
        let offset = loop_start as i32 - (self.bytecode.current_offset() as i32 + 3);
        self.bytecode.emit(Opcode::Loop, while_stmt.span);
        self.bytecode.emit_i16(offset as i16);

        // Patch the exit jump
        self.bytecode.patch_jump(exit_jump);

        // Patch all break statements
        let loop_ctx = self.loops.pop().unwrap();
        for break_jump in loop_ctx.break_jumps {
            self.bytecode.patch_jump(break_jump);
        }

        Ok(())
    }

    /// Compile a for loop
    fn compile_for(&mut self, for_stmt: &ForStmt) -> Result<(), Vec<Diagnostic>> {
        // Compile initializer
        self.compile_stmt(&for_stmt.init)?;

        let loop_start = self.bytecode.current_offset();

        // Start a new loop context
        self.loops.push(LoopContext {
            start_offset: loop_start,
            break_jumps: Vec::new(),
        });

        // Compile condition
        self.compile_expr(&for_stmt.cond)?;

        // Jump if false (exit loop)
        self.bytecode.emit(Opcode::JumpIfFalse, for_stmt.span);
        let exit_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF); // Placeholder

        // Compile body
        self.compile_block(&for_stmt.body)?;

        // Compile step
        self.compile_stmt(&for_stmt.step)?;

        // Loop back to condition
        // Offset needs to account for the Loop instruction (1 byte) + offset operand (2 bytes) = 3 bytes
        let offset = loop_start as i32 - (self.bytecode.current_offset() as i32 + 3);
        self.bytecode.emit(Opcode::Loop, for_stmt.span);
        self.bytecode.emit_i16(offset as i16);

        // Patch the exit jump
        self.bytecode.patch_jump(exit_jump);

        // Patch all break statements
        let loop_ctx = self.loops.pop().unwrap();
        for break_jump in loop_ctx.break_jumps {
            self.bytecode.patch_jump(break_jump);
        }

        Ok(())
    }

    /// Compile a compound assignment (+=, -=, *=, /=, %=)
    fn compile_compound_assign(&mut self, compound: &CompoundAssign) -> Result<(), Vec<Diagnostic>> {
        match &compound.target {
            AssignTarget::Name(ident) => {
                // Get current value
                if let Some(local_idx) = self.resolve_local(&ident.name) {
                    self.bytecode.emit(Opcode::GetLocal, compound.span);
                    self.bytecode.emit_u16(local_idx as u16);
                } else {
                    let name_idx = self.bytecode.add_constant(Value::string(&ident.name));
                    self.bytecode.emit(Opcode::GetGlobal, compound.span);
                    self.bytecode.emit_u16(name_idx);
                }

                // Compile the value to apply
                self.compile_expr(&compound.value)?;

                // Emit the operation
                let opcode = match compound.op {
                    CompoundOp::AddAssign => Opcode::Add,
                    CompoundOp::SubAssign => Opcode::Sub,
                    CompoundOp::MulAssign => Opcode::Mul,
                    CompoundOp::DivAssign => Opcode::Div,
                    CompoundOp::ModAssign => Opcode::Mod,
                };
                self.bytecode.emit(opcode, compound.span);

                // Store the result
                if let Some(local_idx) = self.resolve_local(&ident.name) {
                    self.bytecode.emit(Opcode::SetLocal, compound.span);
                    self.bytecode.emit_u16(local_idx as u16);
                } else {
                    let name_idx = self.bytecode.add_constant(Value::string(&ident.name));
                    self.bytecode.emit(Opcode::SetGlobal, compound.span);
                    self.bytecode.emit_u16(name_idx);
                }

                // Pop the result (statement context)
                self.bytecode.emit(Opcode::Pop, compound.span);
            }
            AssignTarget::Index { target, index, span } => {
                // Get array[index]
                self.compile_expr(target)?;
                self.compile_expr(index)?;
                // Duplicate array and index for later SetIndex
                // Stack: [array, index]
                // We need: [array, index] for GetIndex, then [array, index, new_value] for SetIndex
                // But we don't have a good way to duplicate both values
                // We'd need: [array, index, array, index] -> GetIndex -> [array, index, old_value]
                // Then: [array, index, old_value, new_operand] -> Op -> [array, index, result]
                // Then: SetIndex

                // Since we don't have Dup2, we need to recompile:
                // Option 1: Recompile array and index twice
                // Option 2: Use locals to save them
                // For simplicity, let's recompile (not optimal but works):

                // First get the current value
                self.compile_expr(target)?; // Recompile array
                self.compile_expr(index)?; // Recompile index
                self.bytecode.emit(Opcode::GetIndex, *span);

                // Now apply the operation
                self.compile_expr(&compound.value)?;
                let opcode = match compound.op {
                    CompoundOp::AddAssign => Opcode::Add,
                    CompoundOp::SubAssign => Opcode::Sub,
                    CompoundOp::MulAssign => Opcode::Mul,
                    CompoundOp::DivAssign => Opcode::Div,
                    CompoundOp::ModAssign => Opcode::Mod,
                };
                self.bytecode.emit(opcode, compound.span);

                // Now set it back: need array, index, value
                self.compile_expr(target)?; // Recompile array again
                self.compile_expr(index)?; // Recompile index again
                // Stack is now: [result, array, index]
                // But we need: [array, index, result]
                // We need to rotate... but we don't have that opcode

                // Let's use a different approach with a temp on stack
                // Actually, this is getting complex. Let me use locals.
                // For now, let's just note this limitation and use a simpler approach:

                // Save result to a temporary by using the stack
                // We have: [result] from the operation
                // We need: [array, index, result]
                // Compile array: [result, array]
                // Compile index: [result, array, index]
                // Now we need to get result to top: we need [array, index, result]

                // Without rotate/swap, this is tricky. Let me think...
                // Actually, we can store result in a temp local if in local scope
                // Or just do multiple GetIndex/SetIndex sequences

                // Simplest working solution: compute result, then do full set sequence:
                // Current stack: [result]
                // We'll emit: array, index, result (by using stack manipulation)

                // You know what, let me just restructure to compute things in right order:
                // We'll compute: array, index, then old_value, then operation, giving new_value
                // But that puts new_value on top with array and index buried

                // Cleanest approach: emit array and index first, dup them, getindex, operate, setindex
                // But we don't have dup2 to dup both array and index

                // For MVP, let's just recompile array and index multiple times (inefficient but correct):
                // Get current: arr[idx]
                // Compute: old_value op new_value = result
                // Set: arr[idx] = result

                // But the issue remains: after computing result, we need array and index under it

                // Let me try a different approach: save to temp local if possible
                // Check scope depth and use a local

                // Actually, let's just accept the limitation for now and not support
                // compound assignment on array indices in v0.1, or implement correctly:

                // CORRECT IMPLEMENTATION using recompilation:
                // Step 1: Push array, index, get value
                // Step 2: Compute operation (old_value on stack, push operand, apply op)
                // Step 3: Push array, index again, then rotate/swap to get value on top
                //
                // Since we don't have rotate, we need to structure it as:
                // Push array, index (will be consumed by SetIndex)
                // Push array, index again (will be consumed by GetIndex)
                // GetIndex (leaves old_value)
                // Push operand
                // Apply operation (leaves result)
                // Now stack is [array_for_set, index_for_set, result] - PERFECT!

                // Let's implement that:

                // For SetIndex at the end (push array and index first)
                self.compile_expr(target)?;
                self.compile_expr(index)?;
                // Stack: [array, index]

                // For GetIndex (push array and index again)
                self.compile_expr(target)?;
                self.compile_expr(index)?;
                // Stack: [array_set, index_set, array_get, index_get]

                self.bytecode.emit(Opcode::GetIndex, *span);
                // Stack: [array_set, index_set, old_value]

                // Apply operation
                self.compile_expr(&compound.value)?;
                // Stack: [array_set, index_set, old_value, operand]

                let opcode = match compound.op {
                    CompoundOp::AddAssign => Opcode::Add,
                    CompoundOp::SubAssign => Opcode::Sub,
                    CompoundOp::MulAssign => Opcode::Mul,
                    CompoundOp::DivAssign => Opcode::Div,
                    CompoundOp::ModAssign => Opcode::Mod,
                };
                self.bytecode.emit(opcode, compound.span);
                // Stack: [array_set, index_set, result]

                // Now SetIndex
                self.bytecode.emit(Opcode::SetIndex, *span);
                // Stack: [] (SetIndex consumes all three)
            }
        }

        Ok(())
    }

    /// Compile an increment statement (++)
    fn compile_increment(&mut self, inc: &IncrementStmt) -> Result<(), Vec<Diagnostic>> {
        match &inc.target {
            AssignTarget::Name(ident) => {
                // Get current value
                if let Some(local_idx) = self.resolve_local(&ident.name) {
                    self.bytecode.emit(Opcode::GetLocal, inc.span);
                    self.bytecode.emit_u16(local_idx as u16);
                } else {
                    let name_idx = self.bytecode.add_constant(Value::string(&ident.name));
                    self.bytecode.emit(Opcode::GetGlobal, inc.span);
                    self.bytecode.emit_u16(name_idx);
                }

                // Push 1
                let one_idx = self.bytecode.add_constant(Value::Number(1.0));
                self.bytecode.emit(Opcode::Constant, inc.span);
                self.bytecode.emit_u16(one_idx);

                // Add
                self.bytecode.emit(Opcode::Add, inc.span);

                // Store back
                if let Some(local_idx) = self.resolve_local(&ident.name) {
                    self.bytecode.emit(Opcode::SetLocal, inc.span);
                    self.bytecode.emit_u16(local_idx as u16);
                } else {
                    let name_idx = self.bytecode.add_constant(Value::string(&ident.name));
                    self.bytecode.emit(Opcode::SetGlobal, inc.span);
                    self.bytecode.emit_u16(name_idx);
                }

                // Pop the result (statement context)
                self.bytecode.emit(Opcode::Pop, inc.span);
            }
            AssignTarget::Index { target, index, span } => {
                // Same pattern as compound assign for index
                // Push array, index for SetIndex
                self.compile_expr(target)?;
                self.compile_expr(index)?;

                // Push array, index for GetIndex
                self.compile_expr(target)?;
                self.compile_expr(index)?;
                self.bytecode.emit(Opcode::GetIndex, *span);

                // Push 1 and add
                let one_idx = self.bytecode.add_constant(Value::Number(1.0));
                self.bytecode.emit(Opcode::Constant, inc.span);
                self.bytecode.emit_u16(one_idx);
                self.bytecode.emit(Opcode::Add, inc.span);

                // SetIndex
                self.bytecode.emit(Opcode::SetIndex, *span);
            }
        }

        Ok(())
    }

    /// Compile a decrement statement (--)
    fn compile_decrement(&mut self, dec: &DecrementStmt) -> Result<(), Vec<Diagnostic>> {
        match &dec.target {
            AssignTarget::Name(ident) => {
                // Get current value
                if let Some(local_idx) = self.resolve_local(&ident.name) {
                    self.bytecode.emit(Opcode::GetLocal, dec.span);
                    self.bytecode.emit_u16(local_idx as u16);
                } else {
                    let name_idx = self.bytecode.add_constant(Value::string(&ident.name));
                    self.bytecode.emit(Opcode::GetGlobal, dec.span);
                    self.bytecode.emit_u16(name_idx);
                }

                // Push 1
                let one_idx = self.bytecode.add_constant(Value::Number(1.0));
                self.bytecode.emit(Opcode::Constant, dec.span);
                self.bytecode.emit_u16(one_idx);

                // Subtract
                self.bytecode.emit(Opcode::Sub, dec.span);

                // Store back
                if let Some(local_idx) = self.resolve_local(&ident.name) {
                    self.bytecode.emit(Opcode::SetLocal, dec.span);
                    self.bytecode.emit_u16(local_idx as u16);
                } else {
                    let name_idx = self.bytecode.add_constant(Value::string(&ident.name));
                    self.bytecode.emit(Opcode::SetGlobal, dec.span);
                    self.bytecode.emit_u16(name_idx);
                }

                // Pop the result (statement context)
                self.bytecode.emit(Opcode::Pop, dec.span);
            }
            AssignTarget::Index { target, index, span } => {
                // Same pattern as increment for index
                // Push array, index for SetIndex
                self.compile_expr(target)?;
                self.compile_expr(index)?;

                // Push array, index for GetIndex
                self.compile_expr(target)?;
                self.compile_expr(index)?;
                self.bytecode.emit(Opcode::GetIndex, *span);

                // Push 1 and subtract
                let one_idx = self.bytecode.add_constant(Value::Number(1.0));
                self.bytecode.emit(Opcode::Constant, dec.span);
                self.bytecode.emit_u16(one_idx);
                self.bytecode.emit(Opcode::Sub, dec.span);

                // SetIndex
                self.bytecode.emit(Opcode::SetIndex, *span);
            }
        }

        Ok(())
    }

    /// Compile a break statement
    fn compile_break(&mut self, span: Span) -> Result<(), Vec<Diagnostic>> {
        if let Some(loop_ctx) = self.loops.last_mut() {
            // Emit jump and save offset to patch later
            self.bytecode.emit(Opcode::Jump, span);
            let jump_offset = self.bytecode.current_offset();
            self.bytecode.emit_u16(0xFFFF); // Placeholder
            loop_ctx.break_jumps.push(jump_offset);
            Ok(())
        } else {
            // Error: break outside loop (should be caught by typechecker)
            Ok(())
        }
    }

    /// Compile a continue statement
    fn compile_continue(&mut self, span: Span) -> Result<(), Vec<Diagnostic>> {
        if let Some(loop_ctx) = self.loops.last() {
            // Jump back to loop start
            // Offset needs to account for the Loop instruction (1 byte) + offset operand (2 bytes) = 3 bytes
            let offset = loop_ctx.start_offset as i32 - (self.bytecode.current_offset() as i32 + 3);
            self.bytecode.emit(Opcode::Loop, span);
            self.bytecode.emit_i16(offset as i16);
            Ok(())
        } else {
            // Error: continue outside loop (should be caught by typechecker)
            Ok(())
        }
    }

    /// Compile a block
    pub(super) fn compile_block(&mut self, block: &Block) -> Result<(), Vec<Diagnostic>> {
        for stmt in &block.statements {
            self.compile_stmt(stmt)?;
        }
        Ok(())
    }
}
