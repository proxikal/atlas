//! Statement compilation

use crate::ast::*;
use crate::bytecode::Opcode;
use crate::compiler::{Compiler, Local, LoopContext};
use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::value::Value;

impl Compiler {
    /// Compile a nested function declaration (Phase 6)
    ///
    /// Nested functions are compiled similarly to top-level functions, but:
    /// - They're stored as local variables instead of globals
    /// - They use scoped names to avoid collisions
    fn compile_nested_function(&mut self, func: &FunctionDecl) -> Result<(), Vec<Diagnostic>> {
        // Create scoped name to avoid collisions between nested functions
        let scoped_name = format!("{}_{}", func.name.name, self.next_func_id);
        self.next_func_id += 1;

        // Create placeholder FunctionRef
        let placeholder_ref = crate::value::FunctionRef {
            name: scoped_name.clone(),
            arity: func.params.len(),
            bytecode_offset: 0, // Will be updated after Jump
            local_count: 0,     // Will be updated after compiling body
        };
        let placeholder_value = crate::value::Value::Function(placeholder_ref);

        // Add placeholder to constant pool
        let const_idx = self.bytecode.add_constant(placeholder_value);
        self.bytecode.emit(Opcode::Constant, func.span);
        self.bytecode.emit_u16(const_idx);

        // Store function both globally (for sibling access) and locally (for outer scope)
        if self.scope_depth == 0 {
            // Top-level: store globally with original name
            let name_idx = self
                .bytecode
                .add_constant(crate::value::Value::string(&func.name.name));
            self.bytecode.emit(Opcode::SetGlobal, func.span);
            self.bytecode.emit_u16(name_idx);
            self.bytecode.emit(Opcode::Pop, func.span);
        } else {
            // Nested function: store globally with scoped name for sibling access
            let scoped_name_idx = self
                .bytecode
                .add_constant(crate::value::Value::string(&scoped_name));
            self.bytecode.emit(Opcode::SetGlobal, func.span);
            self.bytecode.emit_u16(scoped_name_idx);

            // Also add as local variable in outer function's scope
            self.push_local(Local {
                name: func.name.name.clone(),
                depth: self.scope_depth,
                mutable: false,
                scoped_name: Some(scoped_name.clone()), // Store scoped name for sibling access
            });
            // The function value is still on stack - it will be the first local
        }

        // Jump over the function body (so it's not executed during declaration)
        self.bytecode.emit(Opcode::Jump, func.span);
        let skip_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF); // Placeholder

        // Record the function body offset
        let function_offset = self.bytecode.current_offset();

        // STRATEGY FOR NESTED FUNCTIONS:
        // - Keep parent scope locals visible (for sibling function calls)
        // - Add this function's parameters starting from index 0
        // - Use local_base to track where this function's locals start
        // - When emitting GetLocal/SetLocal, adjust index based on whether
        //   it's a parent-scope variable (keep absolute) or function-local (relative)

        let old_scope = self.scope_depth;
        let local_base = self.locals.len(); // Where this function's locals start
        self.scope_depth += 1;

        // Reset watermark for this nested function. Save previous value.
        let prev_watermark = std::mem::replace(&mut self.locals_watermark, local_base);

        // Add parameters as locals
        for param in &func.params {
            self.push_local(Local {
                name: param.name.name.clone(),
                depth: self.scope_depth,
                mutable: true,
                scoped_name: None,
            });
        }

        // Save the local_base before compiling body (for nested function resolution)
        let prev_local_base = std::mem::replace(&mut self.current_function_base, local_base);

        // Compile function body
        self.compile_block(&func.body)?;

        // Restore previous function base
        self.current_function_base = prev_local_base;

        // Calculate total local count using watermark (not self.locals.len() which may
        // have been truncated by match arm cleanup).
        let total_local_count = self.locals_watermark - local_base;

        // Restore watermark for the enclosing function
        self.locals_watermark = prev_watermark;

        // Implicit return null if no explicit return
        self.bytecode.emit(Opcode::Null, func.span);
        self.bytecode.emit(Opcode::Return, func.span);

        // Restore scope and remove this function's locals
        self.scope_depth = old_scope;
        self.locals.truncate(local_base);

        // Update the FunctionRef in constants with accurate offsets
        let updated_ref = crate::value::FunctionRef {
            name: scoped_name,
            arity: func.params.len(),
            bytecode_offset: function_offset,
            local_count: total_local_count,
        };
        self.bytecode.constants[const_idx as usize] = crate::value::Value::Function(updated_ref);

        // Patch the skip jump to go past the function body
        self.bytecode.patch_jump(skip_jump);

        Ok(())
    }

    /// Compile a statement
    pub(super) fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), Vec<Diagnostic>> {
        match stmt {
            Stmt::VarDecl(decl) => self.compile_var_decl(decl),
            Stmt::FunctionDecl(func) => {
                // Nested function declaration - compile it
                self.compile_nested_function(func)
            }
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
            Stmt::ForIn(for_in_stmt) => self.compile_for_in(for_in_stmt),
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

            // Track global mutability
            self.global_mutability
                .insert(decl.name.name.clone(), decl.mutable);
        } else {
            // Local variable - add to locals list
            // Value stays on stack (locals are stack-allocated)
            self.push_local(Local {
                name: decl.name.name.clone(),
                depth: self.scope_depth,
                mutable: decl.mutable,
                scoped_name: None,
            });
        }

        Ok(())
    }

    /// Compile an assignment
    fn compile_assign(&mut self, assign: &Assign) -> Result<(), Vec<Diagnostic>> {
        match &assign.target {
            AssignTarget::Name(ident) => {
                // Check mutability before compiling
                if let Some((local_idx, mutable)) = self.resolve_local_with_mutability(&ident.name)
                {
                    // Local variable - check mutability
                    if !mutable {
                        return Err(vec![Diagnostic::error(
                            format!(
                                "Cannot assign to immutable variable '{}' — declared with 'let'",
                                ident.name
                            ),
                            assign.span,
                        )
                        .with_label("assignment to immutable variable")
                        .with_note(
                            "Use 'var' instead of 'let' to declare a mutable variable".to_string(),
                        )]);
                    }

                    // Compile value and emit SetLocal
                    self.compile_expr(&assign.value)?;
                    self.bytecode.emit(Opcode::SetLocal, assign.span);
                    self.bytecode.emit_u16(local_idx as u16);
                } else {
                    // Global variable - check mutability
                    if let Some(mutable) = self.is_global_mutable(&ident.name) {
                        if !mutable {
                            return Err(vec![Diagnostic::error(
                                format!(
                                    "Cannot assign to immutable variable '{}' — declared with 'let'",
                                    ident.name
                                ),
                                assign.span,
                            )
                            .with_label("assignment to immutable variable")
                            .with_note(
                                "Use 'var' instead of 'let' to declare a mutable variable"
                                    .to_string(),
                            )]);
                        }
                    }
                    // If global not found in mutability map, it's either:
                    // - An undeclared variable (runtime error)
                    // - A builtin function (shouldn't be assigned to, but not our concern here)

                    // Compile value and emit SetGlobal
                    self.compile_expr(&assign.value)?;
                    let name_idx = self.bytecode.add_constant(Value::string(&ident.name));
                    self.bytecode.emit(Opcode::SetGlobal, assign.span);
                    self.bytecode.emit_u16(name_idx);
                }
            }
            AssignTarget::Index { target, index, .. } => {
                // For array index assignment: compile array, index, value (in that order)
                // SetIndex pops: value (top), index, array (bottom)
                // So we need stack: [array, index, value]
                // NOTE: Index assignment does NOT mutate the binding itself,
                // it mutates the array contents. This is allowed even for `let` bindings.

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

    /// Compile a for-in loop
    fn compile_for_in(&mut self, for_in_stmt: &ForInStmt) -> Result<(), Vec<Diagnostic>> {
        // TODO(Phase-20c): Full VM support requires array length opcode
        // For now, for-in loops work in interpreter mode only
        // VM support will be added in a future phase when ArrayLength opcode is available
        Err(vec![crate::diagnostic::Diagnostic::error_with_code(
            "AT9998",
            "For-in loops are not yet supported in VM mode (use interpreter mode)",
            for_in_stmt.span,
        )
        .with_help(
            "For-in loops work in interpreter mode. VM support coming soon.".to_string(),
        )])
    }

    /// Compile a compound assignment (+=, -=, *=, /=, %=)
    fn compile_compound_assign(
        &mut self,
        compound: &CompoundAssign,
    ) -> Result<(), Vec<Diagnostic>> {
        match &compound.target {
            AssignTarget::Name(ident) => {
                // Check mutability before compiling
                if let Some((local_idx, mutable)) = self.resolve_local_with_mutability(&ident.name)
                {
                    // Local variable - check mutability
                    if !mutable {
                        return Err(vec![Diagnostic::error(
                            format!(
                                "Cannot assign to immutable variable '{}' — declared with 'let'",
                                ident.name
                            ),
                            compound.span,
                        )
                        .with_label("assignment to immutable variable")
                        .with_note(
                            "Use 'var' instead of 'let' to declare a mutable variable".to_string(),
                        )]);
                    }

                    // Get current value
                    self.bytecode.emit(Opcode::GetLocal, compound.span);
                    self.bytecode.emit_u16(local_idx as u16);
                } else {
                    // Global variable - check mutability
                    if let Some(mutable) = self.is_global_mutable(&ident.name) {
                        if !mutable {
                            return Err(vec![Diagnostic::error(
                                format!(
                                    "Cannot assign to immutable variable '{}' — declared with 'let'",
                                    ident.name
                                ),
                                compound.span,
                            )
                            .with_label("assignment to immutable variable")
                            .with_note(
                                "Use 'var' instead of 'let' to declare a mutable variable"
                                    .to_string(),
                            )]);
                        }
                    }

                    // Get current value
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
            AssignTarget::Index {
                target,
                index,
                span,
            } => {
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
                // Check mutability before compiling
                if let Some((local_idx, mutable)) = self.resolve_local_with_mutability(&ident.name)
                {
                    // Local variable - check mutability
                    if !mutable {
                        return Err(vec![Diagnostic::error(
                            format!(
                                "Cannot assign to immutable variable '{}' — declared with 'let'",
                                ident.name
                            ),
                            inc.span,
                        )
                        .with_label("assignment to immutable variable")
                        .with_note(
                            "Use 'var' instead of 'let' to declare a mutable variable".to_string(),
                        )]);
                    }

                    // Get current value
                    self.bytecode.emit(Opcode::GetLocal, inc.span);
                    self.bytecode.emit_u16(local_idx as u16);
                } else {
                    // Global variable - check mutability
                    if let Some(mutable) = self.is_global_mutable(&ident.name) {
                        if !mutable {
                            return Err(vec![Diagnostic::error(
                                format!(
                                    "Cannot assign to immutable variable '{}' — declared with 'let'",
                                    ident.name
                                ),
                                inc.span,
                            )
                            .with_label("assignment to immutable variable")
                            .with_note(
                                "Use 'var' instead of 'let' to declare a mutable variable"
                                    .to_string(),
                            )]);
                        }
                    }

                    // Get current value
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
            AssignTarget::Index {
                target,
                index,
                span,
            } => {
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
                // Check mutability before compiling
                if let Some((local_idx, mutable)) = self.resolve_local_with_mutability(&ident.name)
                {
                    // Local variable - check mutability
                    if !mutable {
                        return Err(vec![Diagnostic::error(
                            format!(
                                "Cannot assign to immutable variable '{}' — declared with 'let'",
                                ident.name
                            ),
                            dec.span,
                        )
                        .with_label("assignment to immutable variable")
                        .with_note(
                            "Use 'var' instead of 'let' to declare a mutable variable".to_string(),
                        )]);
                    }

                    // Get current value
                    self.bytecode.emit(Opcode::GetLocal, dec.span);
                    self.bytecode.emit_u16(local_idx as u16);
                } else {
                    // Global variable - check mutability
                    if let Some(mutable) = self.is_global_mutable(&ident.name) {
                        if !mutable {
                            return Err(vec![Diagnostic::error(
                                format!(
                                    "Cannot assign to immutable variable '{}' — declared with 'let'",
                                    ident.name
                                ),
                                dec.span,
                            )
                            .with_label("assignment to immutable variable")
                            .with_note(
                                "Use 'var' instead of 'let' to declare a mutable variable"
                                    .to_string(),
                            )]);
                        }
                    }

                    // Get current value
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
            AssignTarget::Index {
                target,
                index,
                span,
            } => {
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
