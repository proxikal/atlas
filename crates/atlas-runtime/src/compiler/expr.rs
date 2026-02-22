//! Expression compilation

use crate::ast::*;
use crate::bytecode::Opcode;
use crate::compiler::{Compiler, Local};
use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::value::Value;

impl Compiler {
    /// Compile an expression
    pub(super) fn compile_expr(&mut self, expr: &Expr) -> Result<(), Vec<Diagnostic>> {
        match expr {
            Expr::Literal(lit, span) => self.compile_literal(lit, *span),
            Expr::Identifier(ident) => self.compile_identifier(ident),
            Expr::Binary(bin) => self.compile_binary(bin),
            Expr::Unary(un) => self.compile_unary(un),
            Expr::Group(group) => self.compile_expr(&group.expr),
            Expr::ArrayLiteral(arr) => self.compile_array_literal(arr),
            Expr::Index(index) => self.compile_index(index),
            Expr::Call(call) => self.compile_call(call),
            Expr::Match(match_expr) => self.compile_match(match_expr),
            Expr::Member(member) => self.compile_member(member),
            Expr::Try(try_expr) => self.compile_try(try_expr),
        }
    }

    /// Compile a function call expression
    fn compile_call(&mut self, call: &CallExpr) -> Result<(), Vec<Diagnostic>> {
        // Extract function name from callee (must be an identifier for now)
        let func_name = match call.callee.as_ref() {
            Expr::Identifier(ident) => &ident.name,
            _ => {
                // Complex callees (like method calls) not supported yet
                return Ok(());
            }
        };

        // Load the function from local or global scope
        // Don't hardcode builtins - let GetGlobal handle them so natives can override
        {
            // Try local first (for nested functions)
            if let Some(local_idx) = self.resolve_local(func_name) {
                let local = &self.locals[local_idx];

                // Check if this local is from current function's scope or parent scope
                if local.depth < self.scope_depth {
                    if let Some(name_to_use) = local.scoped_name.as_ref() {
                        // Nested function in parent scope — accessible via global scoped name
                        let name_idx = self
                            .bytecode
                            .add_constant(crate::value::Value::string(name_to_use));
                        self.bytecode.emit(Opcode::GetGlobal, call.span);
                        self.bytecode.emit_u16(name_idx);
                    } else if !self.upvalue_stack.is_empty() {
                        // Regular closure/variable from outer scope — load via upvalue
                        let upvalue_idx = self.register_upvalue(func_name, local_idx);
                        self.bytecode.emit(Opcode::GetUpvalue, call.span);
                        self.bytecode.emit_u16(upvalue_idx as u16);
                    } else {
                        // Fallback: GetGlobal
                        let name_idx = self
                            .bytecode
                            .add_constant(crate::value::Value::string(func_name));
                        self.bytecode.emit(Opcode::GetGlobal, call.span);
                        self.bytecode.emit_u16(name_idx);
                    }
                } else {
                    // Current function's scope - use GetLocal with function-relative index
                    let function_relative_idx = local_idx - self.current_function_base;
                    self.bytecode.emit(Opcode::GetLocal, call.span);
                    self.bytecode.emit_u16(function_relative_idx as u16);
                }
            } else {
                // Load from global
                let name_idx = self
                    .bytecode
                    .add_constant(crate::value::Value::string(func_name));
                self.bytecode.emit(Opcode::GetGlobal, call.span);
                self.bytecode.emit_u16(name_idx);
            }
        }

        // Compile all arguments (they'll be pushed on top of the function)
        for arg in &call.args {
            self.compile_expr(arg)?;
        }

        // Emit call instruction with argument count
        self.bytecode.emit(Opcode::Call, call.span);
        self.bytecode.emit_u8(call.args.len() as u8);

        // CoW write-back: collection mutation builtins return the new collection.
        // If the first argument is an identifier, write the result back to that variable.
        self.emit_cow_writeback_if_needed(func_name, call);

        Ok(())
    }

    /// Emit CoW write-back bytecode after a collection mutation builtin call.
    ///
    /// - RETURNS_COLLECTION: `SetLocal/SetGlobal(var)` (peek, keeps value on stack)
    /// - RETURNS_PAIR `[extracted, new_col]`: dup → index(1) → set_var → pop → index(0)
    ///   Result on stack becomes just `extracted` (item), new_col is written to var.
    fn emit_cow_writeback_if_needed(&mut self, func_name: &str, call: &CallExpr) {
        const RETURNS_COLLECTION: &[&str] = &[
            // HashMap
            "hashMapPut",
            "hashMapClear",
            // HashSet
            "hashSetAdd",
            "hashSetClear",
            // Queue
            "queueEnqueue",
            "queueClear",
            // Stack
            "stackPush",
            "stackClear",
            // Array (free-function variants)
            "unshift",
            "reverse",
            "flatten",
        ];
        const RETURNS_PAIR: &[&str] = &[
            // HashMap / HashSet / Queue / Stack
            "hashMapRemove",
            "hashSetRemove",
            "queueDequeue",
            "stackPop",
            // Array (free-function variants)
            "pop",
            "shift",
        ];

        let first_ident = call.args.first().and_then(|e| {
            if let Expr::Identifier(id) = e {
                Some(id.name.as_str())
            } else {
                None
            }
        });
        let var_name = match first_ident {
            Some(n) => n,
            None => return,
        };

        if RETURNS_COLLECTION.contains(&func_name) {
            // Stack: new_collection
            // Emit SetLocal/SetGlobal (peek — keeps value on stack for caller)
            self.emit_force_writeback(var_name, call.span);
        } else if RETURNS_PAIR.contains(&func_name) {
            // Stack: [extracted, new_collection]
            // Dup → [..., pair, pair]
            // Constant(1), GetIndex → [..., pair, new_collection]
            // SetLocal/SetGlobal(var) — peek, keeps new_collection on stack
            // Pop → [..., pair]
            // Constant(0), GetIndex → [..., extracted]
            self.bytecode.emit(Opcode::Dup, call.span);
            let idx1 = self.bytecode.add_constant(crate::value::Value::Number(1.0));
            self.bytecode.emit(Opcode::Constant, call.span);
            self.bytecode.emit_u16(idx1);
            self.bytecode.emit(Opcode::GetIndex, call.span);
            self.emit_force_writeback(var_name, call.span);
            self.bytecode.emit(Opcode::Pop, call.span);
            let idx0 = self.bytecode.add_constant(crate::value::Value::Number(0.0));
            self.bytecode.emit(Opcode::Constant, call.span);
            self.bytecode.emit_u16(idx0);
            self.bytecode.emit(Opcode::GetIndex, call.span);
        }
    }

    /// Emit SetLocal or SetGlobal for `var_name`, bypassing mutability checks.
    ///
    /// This mirrors `force_set_collection` in the interpreter: container content
    /// mutation is not a variable rebinding, so mutability doesn't apply.
    fn emit_force_writeback(&mut self, var_name: &str, span: Span) {
        if let Some(local_idx) = self.resolve_local(var_name) {
            let local = &self.locals[local_idx];
            if local.depth < self.scope_depth && !self.upvalue_stack.is_empty() {
                let upvalue_idx = self.register_upvalue(var_name, local_idx);
                self.bytecode.emit(Opcode::SetUpvalue, span);
                self.bytecode.emit_u16(upvalue_idx as u16);
            } else {
                let function_relative_idx = if local.depth < self.scope_depth {
                    local_idx
                } else {
                    local_idx - self.current_function_base
                };
                self.bytecode.emit(Opcode::SetLocal, span);
                self.bytecode.emit_u16(function_relative_idx as u16);
            }
        } else {
            // Global variable (or doesn't exist — silently skip)
            let name_idx = self
                .bytecode
                .add_constant(crate::value::Value::string(var_name));
            self.bytecode.emit(Opcode::SetGlobal, span);
            self.bytecode.emit_u16(name_idx);
        }
    }

    /// Compile a member expression (method call)
    ///
    /// Desugars method calls to stdlib function calls at compile time.
    /// The function name is determined from the method name using a standard mapping:
    ///   value.as_string() → jsonAsString(value)
    fn compile_member(&mut self, member: &MemberExpr) -> Result<(), Vec<Diagnostic>> {
        // Resolve method via shared dispatch table (type tag set by typechecker)
        let type_tag = member
            .type_tag
            .get()
            .expect("TypeTag not set — typechecker must run before compile");
        let func_name = crate::method_dispatch::resolve_method(type_tag, &member.member.name)
            .ok_or_else(|| {
                vec![crate::diagnostic::Diagnostic::error(
                    format!("No method '{}' on type {:?}", member.member.name, type_tag),
                    member.span,
                )]
            })?;

        // Load the stdlib function as a Builtin constant
        let func_value = crate::value::Value::Builtin(std::sync::Arc::from(func_name.as_str()));
        let const_idx = self.bytecode.add_constant(func_value);

        // Load the function constant
        self.bytecode.emit(Opcode::Constant, member.span);
        self.bytecode.emit_u16(const_idx);

        // Compile target (becomes first argument)
        self.compile_expr(&member.target)?;

        // Compile method arguments
        if let Some(args) = &member.args {
            for arg in args {
                self.compile_expr(arg)?;
            }
        }

        // Emit call instruction with total argument count (target + args)
        let arg_count = 1 + member.args.as_ref().map(|a| a.len()).unwrap_or(0);
        self.bytecode.emit(Opcode::Call, member.span);
        self.bytecode.emit_u8(arg_count as u8);

        // CoW write-back: for mutating array methods, update the receiver variable.
        // Only possible when the target is a simple identifier.
        if let crate::ast::Expr::Identifier(id) = member.target.as_ref() {
            let var_name = id.name.as_str();
            if crate::method_dispatch::is_array_mutating_collection(&func_name) {
                // Stack: new_array — peek-set to receiver, value stays on stack
                self.emit_force_writeback(var_name, member.span);
            } else if crate::method_dispatch::is_array_mutating_pair(&func_name) {
                // Stack: [extracted, new_array]
                // Dup → get index 1 (new_array) → set receiver → pop → get index 0 (extracted)
                self.bytecode.emit(Opcode::Dup, member.span);
                let idx1 = self.bytecode.add_constant(crate::value::Value::Number(1.0));
                self.bytecode.emit(Opcode::Constant, member.span);
                self.bytecode.emit_u16(idx1);
                self.bytecode.emit(Opcode::GetIndex, member.span);
                self.emit_force_writeback(var_name, member.span);
                self.bytecode.emit(Opcode::Pop, member.span);
                let idx0 = self.bytecode.add_constant(crate::value::Value::Number(0.0));
                self.bytecode.emit(Opcode::Constant, member.span);
                self.bytecode.emit_u16(idx0);
                self.bytecode.emit(Opcode::GetIndex, member.span);
            }
        }

        Ok(())
    }

    /// Compile a literal
    fn compile_literal(&mut self, lit: &Literal, span: Span) -> Result<(), Vec<Diagnostic>> {
        match lit {
            Literal::Number(n) => {
                let idx = self.bytecode.add_constant(Value::Number(*n));
                self.bytecode.emit(Opcode::Constant, span);
                self.bytecode.emit_u16(idx);
            }
            Literal::String(s) => {
                let idx = self.bytecode.add_constant(Value::string(s));
                self.bytecode.emit(Opcode::Constant, span);
                self.bytecode.emit_u16(idx);
            }
            Literal::Bool(b) => {
                let opcode = if *b { Opcode::True } else { Opcode::False };
                self.bytecode.emit(opcode, span);
            }
            Literal::Null => {
                self.bytecode.emit(Opcode::Null, span);
            }
        }
        Ok(())
    }

    /// Compile an identifier (variable access)
    fn compile_identifier(&mut self, ident: &Identifier) -> Result<(), Vec<Diagnostic>> {
        // Try to resolve as local first
        if let Some(local_idx) = self.resolve_local(&ident.name) {
            let local = &self.locals[local_idx];

            // Check if this local is from current function's scope or parent scope
            if local.depth < self.scope_depth {
                if let Some(name_to_use) = local.scoped_name.as_ref() {
                    // Nested function in parent scope — accessible via its global scoped name
                    let name_idx = self.bytecode.add_constant(Value::string(name_to_use));
                    self.bytecode.emit(Opcode::GetGlobal, ident.span);
                    self.bytecode.emit_u16(name_idx);
                } else if !self.upvalue_stack.is_empty() {
                    // Regular variable from outer function scope — capture as upvalue
                    let upvalue_idx = self.register_upvalue(&ident.name, local_idx);
                    self.bytecode.emit(Opcode::GetUpvalue, ident.span);
                    self.bytecode.emit_u16(upvalue_idx as u16);
                } else {
                    // Outer scope but not in a nested function — use GetGlobal fallback
                    let name_idx = self.bytecode.add_constant(Value::string(&ident.name));
                    self.bytecode.emit(Opcode::GetGlobal, ident.span);
                    self.bytecode.emit_u16(name_idx);
                }
            } else {
                // Current function's scope - use GetLocal with function-relative index
                let function_relative_idx = local_idx - self.current_function_base;
                self.bytecode.emit(Opcode::GetLocal, ident.span);
                self.bytecode.emit_u16(function_relative_idx as u16);
            }
        } else {
            // Global variable
            let name_idx = self.bytecode.add_constant(Value::string(&ident.name));
            self.bytecode.emit(Opcode::GetGlobal, ident.span);
            self.bytecode.emit_u16(name_idx);
        }
        Ok(())
    }

    /// Compile a binary expression
    fn compile_binary(&mut self, bin: &BinaryExpr) -> Result<(), Vec<Diagnostic>> {
        // Handle short-circuit evaluation for && and ||
        match bin.op {
            BinaryOp::And => {
                // For &&: if left is false, result is false (don't eval right)
                // Compile left
                self.compile_expr(&bin.left)?;
                // Duplicate for the check
                self.bytecode.emit(Opcode::Dup, bin.span);
                // Jump to end if false (keeping false on stack)
                self.bytecode.emit(Opcode::JumpIfFalse, bin.span);
                let end_jump = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF); // Placeholder

                // Left was true, pop it and eval right
                self.bytecode.emit(Opcode::Pop, bin.span);
                self.compile_expr(&bin.right)?;

                // Patch jump
                self.bytecode.patch_jump(end_jump);
                Ok(())
            }
            BinaryOp::Or => {
                // For ||: if left is true, result is true (don't eval right)
                // Compile left
                self.compile_expr(&bin.left)?;
                // Duplicate for the check
                self.bytecode.emit(Opcode::Dup, bin.span);
                // If true, jump to end (keeping true on stack)
                // We need "jump if true" but we only have "jump if false"
                // So: if NOT false, jump to end
                // Actually, we need to negate the logic:
                // Dup, Not, JumpIfFalse (jumps if original was true)
                self.bytecode.emit(Opcode::Not, bin.span);
                self.bytecode.emit(Opcode::JumpIfFalse, bin.span);
                let end_jump = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF); // Placeholder

                // Left was false, pop it and eval right
                self.bytecode.emit(Opcode::Pop, bin.span);
                self.compile_expr(&bin.right)?;

                // Patch jump
                self.bytecode.patch_jump(end_jump);
                Ok(())
            }
            _ => {
                // For all other operators, evaluate both sides
                self.compile_expr(&bin.left)?;
                self.compile_expr(&bin.right)?;

                // Emit the appropriate opcode
                let opcode = match bin.op {
                    BinaryOp::Add => Opcode::Add,
                    BinaryOp::Sub => Opcode::Sub,
                    BinaryOp::Mul => Opcode::Mul,
                    BinaryOp::Div => Opcode::Div,
                    BinaryOp::Mod => Opcode::Mod,
                    BinaryOp::Eq => Opcode::Equal,
                    BinaryOp::Ne => Opcode::NotEqual,
                    BinaryOp::Lt => Opcode::Less,
                    BinaryOp::Le => Opcode::LessEqual,
                    BinaryOp::Gt => Opcode::Greater,
                    BinaryOp::Ge => Opcode::GreaterEqual,
                    BinaryOp::And | BinaryOp::Or => unreachable!(), // Handled above
                };
                self.bytecode.emit(opcode, bin.span);
                Ok(())
            }
        }
    }

    /// Compile a unary expression
    fn compile_unary(&mut self, un: &UnaryExpr) -> Result<(), Vec<Diagnostic>> {
        // Compile the operand
        self.compile_expr(&un.expr)?;

        // Emit the appropriate opcode
        let opcode = match un.op {
            UnaryOp::Negate => Opcode::Negate,
            UnaryOp::Not => Opcode::Not,
        };
        self.bytecode.emit(opcode, un.span);
        Ok(())
    }

    /// Compile an array literal
    fn compile_array_literal(&mut self, arr: &ArrayLiteral) -> Result<(), Vec<Diagnostic>> {
        // Compile all elements (leaves them on stack)
        for elem in &arr.elements {
            self.compile_expr(elem)?;
        }

        // Emit Array instruction with element count
        self.bytecode.emit(Opcode::Array, arr.span);
        self.bytecode.emit_u16(arr.elements.len() as u16);

        Ok(())
    }

    /// Compile an index expression
    fn compile_index(&mut self, index: &IndexExpr) -> Result<(), Vec<Diagnostic>> {
        // Compile the target (array)
        self.compile_expr(&index.target)?;

        // Compile the index
        self.compile_expr(&index.index)?;

        // Emit GetIndex instruction
        self.bytecode.emit(Opcode::GetIndex, index.span);

        Ok(())
    }

    /// Compile a match expression
    ///
    /// Strategy: Use Dup for non-last arms so the scrutinee is available for
    /// subsequent arms. Pattern check consumes the copy (dup or original for last
    /// arm). After body compilation, a temp global is used to save/restore the
    /// result while popping extra stack values (scrutinee, pattern variables).
    ///
    /// This avoids using hidden locals (which break when match is used inside
    /// other expressions due to temporaries corrupting local indices).
    fn compile_match(&mut self, match_expr: &MatchExpr) -> Result<(), Vec<Diagnostic>> {
        // Compile scrutinee (leaves value on stack as a temporary)
        self.compile_expr(&match_expr.scrutinee)?;

        // Temp global name for saving the match result during cleanup
        let temp_name = "$match_result";
        let temp_name_idx = self.bytecode.add_constant(Value::string(temp_name));

        let mut arm_end_jumps = Vec::new();

        for (arm_idx, arm) in match_expr.arms.iter().enumerate() {
            let is_last_arm = arm_idx == match_expr.arms.len() - 1;
            let locals_before = self.locals.len();

            if !is_last_arm {
                // Dup scrutinee so next arm can use the original
                self.bytecode.emit(Opcode::Dup, arm.span);
            }

            // Pattern check consumes the top value (dup or scrutinee for last arm).
            // On success: pushes True, may add pattern variable locals.
            // On failure: stack clean (copy consumed), jumps to fail target.
            let match_failed_jump =
                self.compile_pattern_check(&arm.pattern, arm.span, locals_before)?;

            // Pop True (pattern success flag)
            self.bytecode.emit(Opcode::Pop, arm.span);

            // Compile guard if present — guard failure jumps to next arm
            let guard_failed_jump = if let Some(guard_expr) = &arm.guard {
                self.compile_expr(guard_expr)?;
                // JumpIfFalse: if guard is false, skip this arm
                self.bytecode.emit(Opcode::JumpIfFalse, arm.span);
                let guard_jump = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF);
                Some(guard_jump)
            } else {
                None
            };

            // Compile arm body (result on top of stack)
            self.compile_expr(&arm.body)?;

            // Cleanup: remove extras (pattern vars + scrutinee for non-last) from
            // below the body result. Save result to temp global, pop extras, restore.
            let pattern_var_count = self.locals.len() - locals_before;
            let extras = pattern_var_count + if !is_last_arm { 1 } else { 0 };

            if extras > 0 {
                // Save result to temp global (SetGlobal peeks, value stays)
                self.bytecode.emit(Opcode::SetGlobal, arm.span);
                self.bytecode.emit_u16(temp_name_idx);
                // Pop result copy + all extras
                for _ in 0..=extras {
                    self.bytecode.emit(Opcode::Pop, arm.span);
                }
                // Restore result from temp global
                self.bytecode.emit(Opcode::GetGlobal, arm.span);
                self.bytecode.emit_u16(temp_name_idx);
            }

            // Jump to end (skip other arms)
            if !is_last_arm {
                self.bytecode.emit(Opcode::Jump, arm.span);
                let jump_offset = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF);
                arm_end_jumps.push(jump_offset);
            }

            // Guard cleanup (if guard was present): guard failure jumps here, pops stack items,
            // then falls through to next arm. Pattern failure jumps AFTER cleanup (no pops needed).
            if let Some(guard_jump) = guard_failed_jump {
                // Patch guard jump to point here (start of cleanup)
                self.bytecode.patch_jump(guard_jump);
                // Pop all extras: pattern vars + dup copy (for non-last arms).
                // This mirrors the `extras` formula used in the success path, ensuring the stack
                // is restored to the base scrutinee level for the next arm.
                let pattern_var_count_guard = self.locals.len() - locals_before;
                let guard_cleanup_count =
                    pattern_var_count_guard + if !is_last_arm { 1 } else { 0 };
                for _ in 0..guard_cleanup_count {
                    self.bytecode.emit(Opcode::Pop, arm.span);
                }
                // Fall through to next arm (match_failed_jump patches below)
            }
            // Patch the failed pattern jump (next arm starts here, after any guard cleanup)
            if let Some(failed_jump) = match_failed_jump {
                self.bytecode.patch_jump(failed_jump);
            }

            // Clean up locals tracking
            self.locals.truncate(locals_before);
        }

        // Patch all end jumps to point here
        for jump_offset in arm_end_jumps {
            self.bytecode.patch_jump(jump_offset);
        }

        // Stack: [body_result] — match expression produces exactly one value
        Ok(())
    }

    /// Compile pattern matching check
    ///
    /// Contract:
    /// - INPUT: scrutinee copy on top of stack (from GetLocal in compile_match)
    /// - SUCCESS: scrutinee copy consumed, True pushed on stack, pattern vars added as locals
    /// - FAILURE: scrutinee copy consumed, stack clean, jumps to returned offset
    ///
    /// Returns: Optional jump offset to patch if match fails (None for wildcard/variable)
    fn compile_pattern_check(
        &mut self,
        pattern: &Pattern,
        span: Span,
        locals_before: usize,
    ) -> Result<Option<usize>, Vec<Diagnostic>> {
        use crate::ast::{Literal, Pattern};

        match pattern {
            // Wildcard: always matches, no bindings
            Pattern::Wildcard(_) => {
                // Pop scrutinee copy (consumed)
                self.bytecode.emit(Opcode::Pop, span);
                // Push true (match succeeded)
                self.bytecode.emit(Opcode::True, span);
                Ok(None) // No jump needed, always matches
            }

            // Variable: always matches, bind to local
            Pattern::Variable(id) => {
                // Register local for the pattern variable
                self.push_local(Local {
                    name: id.name.clone(),
                    depth: self.scope_depth,
                    mutable: false,
                    scoped_name: None,
                });
                let local_idx = (self.locals.len() - 1 - self.current_function_base) as u16;

                // Copy value from stack top to the local's slot position.
                // This is necessary because temporaries (from enclosing
                // expressions, scrutinee copies, etc.) may sit between the
                // previous locals and the stack top, so the value isn't
                // naturally at the right position.
                self.bytecode.emit(Opcode::SetLocal, span);
                self.bytecode.emit_u16(local_idx);

                // Push true (match succeeded)
                self.bytecode.emit(Opcode::True, span);
                Ok(None) // No jump needed, always matches
            }

            // Literal: check equality
            Pattern::Literal(lit, lit_span) => {
                // Scrutinee copy is on stack. Push literal value for comparison.
                match lit {
                    Literal::Number(n) => {
                        let const_idx = self.bytecode.add_constant(Value::Number(*n));
                        self.bytecode.emit(Opcode::Constant, *lit_span);
                        self.bytecode.emit_u16(const_idx);
                    }
                    Literal::String(s) => {
                        let const_idx = self.bytecode.add_constant(Value::string(s.clone()));
                        self.bytecode.emit(Opcode::Constant, *lit_span);
                        self.bytecode.emit_u16(const_idx);
                    }
                    Literal::Bool(true) => {
                        self.bytecode.emit(Opcode::True, *lit_span);
                    }
                    Literal::Bool(false) => {
                        self.bytecode.emit(Opcode::False, *lit_span);
                    }
                    Literal::Null => {
                        self.bytecode.emit(Opcode::Null, *lit_span);
                    }
                }

                // Compare: pops both (scrutinee copy + literal), pushes bool
                self.bytecode.emit(Opcode::Equal, span);

                // If false, jump to fail target
                self.bytecode.emit(Opcode::JumpIfFalse, span);
                let jump_offset = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF);

                // On success path: Equal consumed the copy, JumpIfFalse consumed the bool.
                // Push True to satisfy the contract.
                self.bytecode.emit(Opcode::True, span);

                Ok(Some(jump_offset))
            }

            // Constructor: Some(x), None, Ok(x), Err(e)
            Pattern::Constructor { name, args, span } => {
                self.compile_constructor_pattern(name, args, *span, locals_before)
            }

            // Array: [x, y, z]
            Pattern::Array { elements, span } => {
                self.compile_array_pattern(elements, *span, locals_before)
            }

            // OR pattern: sub1 | sub2 | sub3
            Pattern::Or(alternatives, or_span) => {
                self.compile_or_pattern(alternatives, *or_span, locals_before)
            }
        }
    }

    /// Compile constructor pattern (Some, None, Ok, Err)
    ///
    /// Contract (same as compile_pattern_check):
    /// - INPUT: scrutinee copy on top of stack
    /// - SUCCESS: copy consumed, True on stack, pattern vars as locals
    /// - FAILURE: copy consumed, stack clean, jumps to returned offset
    fn compile_constructor_pattern(
        &mut self,
        name: &crate::ast::Identifier,
        args: &[Pattern],
        span: Span,
        locals_before: usize,
    ) -> Result<Option<usize>, Vec<Diagnostic>> {
        use crate::bytecode::Opcode;

        match name.name.as_str() {
            "None" => {
                if !args.is_empty() {
                    return Err(vec![Diagnostic::error_with_code(
                        "AT9995",
                        "None pattern should not have arguments",
                        span,
                    )
                    .with_help(
                        "use 'None' without arguments to match empty Option values",
                    )]);
                }

                // IsOptionNone: pops copy, pushes bool
                self.bytecode.emit(Opcode::IsOptionNone, span);
                // JumpIfFalse: pops bool. On failure: stack clean, jumps.
                self.bytecode.emit(Opcode::JumpIfFalse, span);
                let jump_offset = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF);
                // On success: copy consumed, bool consumed. Push True.
                self.bytecode.emit(Opcode::True, span);

                Ok(Some(jump_offset))
            }

            "Some" => {
                if args.len() != 1 {
                    return Err(vec![Diagnostic::error_with_code(
                        "AT9995",
                        "Some pattern requires exactly one argument",
                        span,
                    )
                    .with_help(
                        "use 'Some(value)' to match and extract the inner value from Option",
                    )]);
                }

                self.compile_wrapping_constructor_pattern(
                    Opcode::IsOptionSome,
                    Opcode::ExtractOptionValue,
                    &args[0],
                    span,
                    locals_before,
                )
            }

            "Ok" => {
                if args.len() != 1 {
                    return Err(vec![Diagnostic::error_with_code(
                        "AT9995",
                        "Ok pattern requires exactly one argument",
                        span,
                    )
                    .with_help(
                        "use 'Ok(value)' to match and extract the success value from Result",
                    )]);
                }

                self.compile_wrapping_constructor_pattern(
                    Opcode::IsResultOk,
                    Opcode::ExtractResultValue,
                    &args[0],
                    span,
                    locals_before,
                )
            }

            "Err" => {
                if args.len() != 1 {
                    return Err(vec![Diagnostic::error_with_code(
                        "AT9995",
                        "Err pattern requires exactly one argument",
                        span,
                    )
                    .with_help(
                        "use 'Err(error)' to match and extract the error value from Result",
                    )]);
                }

                self.compile_wrapping_constructor_pattern(
                    Opcode::IsResultErr,
                    Opcode::ExtractResultValue,
                    &args[0],
                    span,
                    locals_before,
                )
            }

            _ => Err(vec![Diagnostic::error_with_code(
                "AT9995",
                format!("Unknown constructor pattern: {}", name.name),
                span,
            )
            .with_help(
                "valid constructor patterns are: Some, None (for Option) and Ok, Err (for Result)",
            )]),
        }
    }

    /// Compile a wrapping constructor pattern (Some(x), Ok(x), Err(x))
    ///
    /// These patterns check a variant, extract the inner value, then recursively
    /// match the inner pattern.
    ///
    /// Stack protocol (same as compile_pattern_check):
    /// - INPUT: [copy] on stack
    /// - SUCCESS: copy consumed, [inner_locals...] [True] on stack
    /// - FAILURE: copy consumed, stack clean, jumps to returned offset
    ///
    /// Emitted code structure:
    /// ```text
    ///   Dup                              // [copy, dup]
    ///   check_opcode                     // [copy, bool]
    ///   JumpIfFalse → outer_fail         // [copy]
    ///   extract_opcode                   // [inner]
    ///   <inner pattern check>            // [inner_locals..., True] or jump
    ///   Jump → success_exit              // skip failure code
    ///   [inner_fail: Jump → fail_exit]   // (only if inner can fail)
    ///   outer_fail: Pop                  // [] clean
    ///   fail_exit: Jump → ???            // compile_match patches this
    ///   success_exit:                    // [inner_locals..., True]
    /// ```
    fn compile_wrapping_constructor_pattern(
        &mut self,
        check_opcode: Opcode,
        extract_opcode: Opcode,
        inner_pattern: &Pattern,
        span: Span,
        locals_before: usize,
    ) -> Result<Option<usize>, Vec<Diagnostic>> {
        // Stack: [copy]
        self.bytecode.emit(Opcode::Dup, span);
        // Stack: [copy, dup]

        self.bytecode.emit(check_opcode, span);
        // Stack: [copy, bool]

        self.bytecode.emit(Opcode::JumpIfFalse, span);
        let outer_fail = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);
        // Stack (success path): [copy]

        self.bytecode.emit(extract_opcode, span);
        // Stack: [inner]

        // Recursively match inner pattern
        let inner_failed = self.compile_pattern_check(inner_pattern, span, locals_before)?;
        // Success: [inner_locals..., True]

        // Jump over failure code
        self.bytecode.emit(Opcode::Jump, span);
        let success_exit = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // --- Failure paths ---

        // Inner pattern failure (if inner can fail)
        if let Some(inner_jump) = inner_failed {
            self.bytecode.patch_jump(inner_jump);
            // Inner pattern guarantees stack is clean (inner value consumed).
            // Jump over the outer_fail Pop to the shared fail exit.
            self.bytecode.emit(Opcode::Jump, span);
            let inner_to_fail = self.bytecode.current_offset();
            self.bytecode.emit_u16(0xFFFF);

            // Outer variant check failure: copy still on stack
            self.bytecode.patch_jump(outer_fail);
            self.bytecode.emit(Opcode::Pop, span); // Pop copy → clean

            // Shared fail exit (inner and outer paths converge)
            self.bytecode.patch_jump(inner_to_fail);
        } else {
            // No inner failure possible. Only outer fail path.
            self.bytecode.patch_jump(outer_fail);
            self.bytecode.emit(Opcode::Pop, span); // Pop copy → clean
        }

        // Emit fail jump for compile_match to patch
        self.bytecode.emit(Opcode::Jump, span);
        let fail_exit = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // Success exit: inner pattern's True is already on stack
        self.bytecode.patch_jump(success_exit);

        Ok(Some(fail_exit))
    }

    /// Compile array pattern [x, y, z]
    ///
    /// Stack protocol (same as compile_pattern_check):
    /// - INPUT: [copy] (array value) on stack
    /// - SUCCESS: copy consumed, [element_locals...] [True] on stack
    /// - FAILURE: copy consumed, stack clean, jumps to returned offset
    ///
    /// The array is stored in a temp global ("$match_array") so it can be
    /// accessed for each element without interfering with stack/local positions.
    fn compile_array_pattern(
        &mut self,
        elements: &[Pattern],
        span: Span,
        locals_before: usize,
    ) -> Result<Option<usize>, Vec<Diagnostic>> {
        use crate::bytecode::Opcode;

        let array_global_name = "$match_array";
        let array_name_idx = self.bytecode.add_constant(Value::string(array_global_name));

        // Stack: [copy] (the array)
        // Store array to temp global for repeated access
        self.bytecode.emit(Opcode::SetGlobal, span);
        self.bytecode.emit_u16(array_name_idx);
        // Pop the array from stack (SetGlobal peeks)
        self.bytecode.emit(Opcode::Pop, span);
        // Stack: [] (array is in temp global)

        // Check if value is an array
        self.bytecode.emit(Opcode::GetGlobal, span);
        self.bytecode.emit_u16(array_name_idx);
        self.bytecode.emit(Opcode::IsArray, span);

        self.bytecode.emit(Opcode::JumpIfFalse, span);
        let not_array_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // Check array length
        self.bytecode.emit(Opcode::GetGlobal, span);
        self.bytecode.emit_u16(array_name_idx);
        self.bytecode.emit(Opcode::GetArrayLen, span);

        let expected_len = elements.len() as f64;
        let const_idx = self.bytecode.add_constant(Value::Number(expected_len));
        self.bytecode.emit(Opcode::Constant, span);
        self.bytecode.emit_u16(const_idx);

        self.bytecode.emit(Opcode::Equal, span);

        self.bytecode.emit(Opcode::JumpIfFalse, span);
        let wrong_length_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // Array type and length match. Match each element.
        // Track element failure info: (jump_offset, element_locals_above_baseline)
        let mut elem_fail_info: Vec<(usize, usize)> = Vec::new();

        for (idx, elem_pattern) in elements.iter().enumerate() {
            // Get array from temp global
            self.bytecode.emit(Opcode::GetGlobal, span);
            self.bytecode.emit_u16(array_name_idx);

            // Push index
            let idx_const = self.bytecode.add_constant(Value::Number(idx as f64));
            self.bytecode.emit(Opcode::Constant, span);
            self.bytecode.emit_u16(idx_const);

            // Get element (pops array_copy and index, pushes element)
            self.bytecode.emit(Opcode::GetIndex, span);

            // Match element pattern
            let elem_failed = self.compile_pattern_check(elem_pattern, span, locals_before)?;

            // Pop True from successful element match
            self.bytecode.emit(Opcode::Pop, span);

            if let Some(jump) = elem_failed {
                // Record how many element locals exist at this failure point
                let elem_locals = self.locals.len() - locals_before;
                elem_fail_info.push((jump, elem_locals));
            }
        }

        // All elements matched! Push True (contract).
        self.bytecode.emit(Opcode::True, span);

        // Jump over failure code
        self.bytecode.emit(Opcode::Jump, span);
        let success_exit = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // --- Failure paths ---

        // Element failure handlers: pop element locals, jump to fail_exit
        let mut handler_jumps = Vec::new();
        for (jump, elem_locals) in &elem_fail_info {
            self.bytecode.patch_jump(*jump);
            // Pop element locals from earlier successful elements
            for _ in 0..*elem_locals {
                self.bytecode.emit(Opcode::Pop, span);
            }
            self.bytecode.emit(Opcode::Jump, span);
            handler_jumps.push(self.bytecode.current_offset());
            self.bytecode.emit_u16(0xFFFF);
        }

        // Type/length check failures: stack is already clean
        self.bytecode.patch_jump(not_array_jump);
        self.bytecode.patch_jump(wrong_length_jump);

        // All failure paths converge here
        for j in &handler_jumps {
            self.bytecode.patch_jump(*j);
        }

        // Fail exit: compile_match patches this
        self.bytecode.emit(Opcode::Jump, span);
        let fail_exit = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // Success exit
        self.bytecode.patch_jump(success_exit);
        // Stack: [element_locals..., True]

        Ok(Some(fail_exit))
    }

    /// Compile OR pattern: pat1 | pat2 | pat3
    ///
    /// Contract (same as compile_pattern_check):
    /// - INPUT: scrutinee on stack top
    /// - SUCCESS: scrutinee consumed, True on stack, pattern vars added as locals
    /// - FAILURE: scrutinee consumed, stack clean, jumps to returned offset
    fn compile_or_pattern(
        &mut self,
        alternatives: &[crate::ast::Pattern],
        span: Span,
        locals_before: usize,
    ) -> Result<Option<usize>, Vec<Diagnostic>> {
        // For each alternative except last: Dup scrutinee, try sub-pattern
        // On success: jump to success_exit (pop extra dup)
        // On failure: try next alternative
        // Last alternative: no Dup, result determines overall success/failure

        let mut success_jumps: Vec<usize> = Vec::new();

        for (i, alt) in alternatives.iter().enumerate() {
            let is_last = i == alternatives.len() - 1;

            if !is_last {
                // Dup scrutinee for this alternative (pattern check consumes it)
                self.bytecode.emit(Opcode::Dup, span);
            }

            // Try this sub-pattern
            let sub_failed_jump = self.compile_pattern_check(alt, span, locals_before)?;

            if !is_last {
                // Sub-pattern succeeded: pop True, emit jump to success block
                self.bytecode.emit(Opcode::Pop, span);
                // Push True for overall match success
                self.bytecode.emit(Opcode::True, span);
                self.bytecode.emit(Opcode::Jump, span);
                success_jumps.push(self.bytecode.current_offset());
                self.bytecode.emit_u16(0xFFFF);

                // Sub-pattern failed: patch its fail jump here to try next alt
                if let Some(failed_jump) = sub_failed_jump {
                    self.bytecode.patch_jump(failed_jump);
                }
            } else {
                // Last alternative: its result IS the OR result
                // If failed, its fail jump is our overall fail jump
                // Success exit patches below
                self.bytecode.emit(Opcode::Jump, span);
                let last_success_exit = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF);

                // Fail exit for last alt (and overall fail)
                let overall_fail_jump = if let Some(failed_jump) = sub_failed_jump {
                    // Patch last alt failure to emit overall fail jump
                    self.bytecode.patch_jump(failed_jump);
                    self.bytecode.emit(Opcode::Jump, span);
                    let fail_exit = self.bytecode.current_offset();
                    self.bytecode.emit_u16(0xFFFF);
                    // Patch success exits from earlier alternatives here
                    for sj in &success_jumps {
                        self.bytecode.patch_jump(*sj);
                    }
                    self.bytecode.patch_jump(last_success_exit);
                    Some(fail_exit)
                } else {
                    // Last alt always matches (wildcard/variable) — overall always succeeds
                    for sj in &success_jumps {
                        self.bytecode.patch_jump(*sj);
                    }
                    self.bytecode.patch_jump(last_success_exit);
                    None
                };

                return Ok(overall_fail_jump);
            }
        }

        // Should never reach here (alternatives is non-empty, last handled above)
        Ok(None)
    }

    /// Compile try expression (error propagation operator ?)
    ///
    /// Desugars to match-based early return:
    /// ```atlas
    /// value?
    /// // becomes:
    /// match value {
    ///     Ok(v) => v,
    ///     Err(e) => return Err(e)
    /// }
    /// ```
    fn compile_try(&mut self, try_expr: &TryExpr) -> Result<(), Vec<Diagnostic>> {
        // 1. Compile the expression being tried
        self.compile_expr(&try_expr.expr)?;

        // 2. Duplicate the result value for pattern matching
        self.bytecode.emit(Opcode::Dup, try_expr.span);

        // 3. Check if it's an Ok variant
        self.bytecode.emit(Opcode::IsResultOk, try_expr.span);

        // 4. Jump to error handling if false (it's an Err)
        self.bytecode.emit(Opcode::JumpIfFalse, try_expr.span);
        let err_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF); // Placeholder

        // 5. Ok path: extract the Ok value
        self.bytecode
            .emit(Opcode::ExtractResultValue, try_expr.span);

        // Skip error handling
        self.bytecode.emit(Opcode::Jump, try_expr.span);
        let ok_skip = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF); // Placeholder

        // 6. Err path: Result value is still on stack from Dup, return it
        self.bytecode.patch_jump(err_jump);
        self.bytecode.emit(Opcode::Return, try_expr.span);

        // 7. Patch ok skip jump
        self.bytecode.patch_jump(ok_skip);

        Ok(())
    }
}
