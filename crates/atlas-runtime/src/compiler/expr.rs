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
                    // Parent scope variable - use GetGlobal with scoped name
                    // This handles sibling nested functions calling each other
                    let name_to_use = local.scoped_name.as_ref().unwrap_or(&local.name);
                    let name_idx = self
                        .bytecode
                        .add_constant(crate::value::Value::string(name_to_use));
                    self.bytecode.emit(Opcode::GetGlobal, call.span);
                    self.bytecode.emit_u16(name_idx);
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

        Ok(())
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
                // Parent scope variable - use GetGlobal with scoped name (for nested functions)
                let name_to_use = local.scoped_name.as_ref().unwrap_or(&local.name);
                let name_idx = self.bytecode.add_constant(Value::string(name_to_use));
                self.bytecode.emit(Opcode::GetGlobal, ident.span);
                self.bytecode.emit_u16(name_idx);
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
    /// Strategy: Desugar match to if-else chain using existing opcodes.
    /// Each arm tries to match the pattern, jumping to the next arm on failure.
    /// On success, bind variables and evaluate the arm body.
    fn compile_match(&mut self, match_expr: &MatchExpr) -> Result<(), Vec<Diagnostic>> {
        // Compile scrutinee (leaves value on stack)
        self.compile_expr(&match_expr.scrutinee)?;

        let mut arm_end_jumps = Vec::new(); // Jumps to the end after each arm body

        for (arm_idx, arm) in match_expr.arms.iter().enumerate() {
            let is_last_arm = arm_idx == match_expr.arms.len() - 1;

            // Save current scope state for cleanup
            let locals_before = self.locals.len();

            if !is_last_arm {
                // Duplicate scrutinee for this arm (so next arm can use it)
                self.bytecode.emit(Opcode::Dup, arm.span);
            }

            // Try to match pattern
            // Pattern matching leaves: scrutinee consumed, match_success (bool) on stack,
            // and binds variables as locals if successful
            let match_failed_jump =
                self.compile_pattern_check(&arm.pattern, arm.span, locals_before)?;

            // If pattern matched: pop success flag, evaluate body, jump to end
            self.bytecode.emit(Opcode::Pop, arm.span); // Pop the true from pattern check

            // Compile arm body (result stays on stack)
            self.compile_expr(&arm.body)?;

            // Stack cleanup: SetLocal peeks (doesn't pop), leaving orphans for each pattern variable.
            // After body executes, stack = [..., pattern_vars..., result]
            // We need to move the result down and pop the orphans.
            // NOTE: Array patterns handle cleanup internally, so exclude them here.
            let pattern_var_count = self.locals.len() - locals_before;
            if pattern_var_count > 0 && !matches!(arm.pattern, Pattern::Array { .. }) {
                // Move result to the position of the first pattern variable
                let result_dest = locals_before - self.current_function_base;
                self.bytecode.emit(Opcode::SetLocal, arm.span);
                self.bytecode.emit_u16(result_dest as u16);
                // Pop all orphaned pattern variables
                for _ in 0..pattern_var_count {
                    self.bytecode.emit(Opcode::Pop, arm.span);
                }
            }

            // Jump to end (skip other arms)
            if !is_last_arm {
                self.bytecode.emit(Opcode::Jump, arm.span);
                let jump_offset = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF); // Placeholder
                arm_end_jumps.push(jump_offset);
            }

            // Patch the jump for failed pattern match (jump here to try next arm)
            if let Some(failed_jump) = match_failed_jump {
                self.bytecode.patch_jump(failed_jump);
            }

            // Clean up locals from this arm (restore state for next arm)
            self.locals.truncate(locals_before);
        }

        // Patch all end jumps to point here
        for jump_offset in arm_end_jumps {
            self.bytecode.patch_jump(jump_offset);
        }

        Ok(())
    }

    /// Compile pattern matching check
    ///
    /// Expects scrutinee on top of stack.
    /// Leaves: bool (match success) on stack
    /// Side effects: Binds pattern variables as locals if successful
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
                // Pop scrutinee (consumed)
                self.bytecode.emit(Opcode::Pop, span);
                // Push true (match succeeded)
                self.bytecode.emit(Opcode::True, span);
                Ok(None) // No jump needed, always matches
            }

            // Variable: always matches, bind to local
            Pattern::Variable(id) => {
                // Scrutinee is already on stack - that becomes the variable's value
                // Add as local variable
                self.push_local(Local {
                    name: id.name.clone(),
                    depth: self.scope_depth,
                    mutable: false, // Pattern variables are immutable
                    scoped_name: None,
                });
                let local_idx = self.locals.len() - 1;

                // Store scrutinee to local (SetLocal peeks, doesn't pop)
                self.bytecode.emit(Opcode::SetLocal, span);
                self.bytecode.emit_u16(local_idx as u16);

                // Push true (match succeeded)
                self.bytecode.emit(Opcode::True, span);
                Ok(None) // No jump needed, always matches
            }

            // Literal: check equality
            Pattern::Literal(lit, lit_span) => {
                // Scrutinee is on stack
                // Push literal value
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

                // Compare: pops both values, pushes bool
                self.bytecode.emit(Opcode::Equal, span);

                // If false, jump to next arm
                self.bytecode.emit(Opcode::JumpIfFalse, span);
                let jump_offset = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF); // Placeholder

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
        }
    }

    /// Compile constructor pattern (Some, None, Ok, Err)
    ///
    /// Uses dedicated pattern matching opcodes to check variant and extract values.
    fn compile_constructor_pattern(
        &mut self,
        name: &crate::ast::Identifier,
        args: &[Pattern],
        span: Span,
        locals_before: usize,
    ) -> Result<Option<usize>, Vec<Diagnostic>> {
        use crate::bytecode::Opcode;

        // Scrutinee is on stack (Option or Result value)

        match name.name.as_str() {
            "None" => {
                // Check if scrutinee is Option::None
                // Args should be empty for None
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

                // Dup scrutinee (keep original for potential next arm)
                // Actually, we already consumed it - it's on the stack
                // Check if it's None
                self.bytecode.emit(Opcode::IsOptionNone, span);

                // If false (not None), jump to next arm
                self.bytecode.emit(Opcode::JumpIfFalse, span);
                let jump_offset = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF); // Placeholder

                // Push true (pattern matched - None was found)
                self.bytecode.emit(Opcode::True, span);

                Ok(Some(jump_offset))
            }

            "Some" => {
                // Check if scrutinee is Option::Some and extract value
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

                // Dup scrutinee to check and extract
                self.bytecode.emit(Opcode::Dup, span);

                // Check if it's Some
                self.bytecode.emit(Opcode::IsOptionSome, span);

                // If false (not Some), jump to next arm
                self.bytecode.emit(Opcode::JumpIfFalse, span);
                let jump_offset = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF); // Placeholder

                // It is Some! Extract the inner value (pops Option, pushes inner)
                self.bytecode.emit(Opcode::ExtractOptionValue, span);

                // Now match the inner pattern against the extracted value
                let _inner_failed_jump =
                    self.compile_pattern_check(&args[0], span, locals_before)?;

                // Inner pattern succeeded - pop the success flag
                self.bytecode.emit(Opcode::Pop, span);

                // SetLocal peeks (doesn't pop), so if inner pattern is Variable, we have an orphan
                if matches!(&args[0], Pattern::Variable(_)) {
                    self.bytecode.emit(Opcode::Pop, span);
                }

                // Push true (this pattern matched)
                self.bytecode.emit(Opcode::True, span);

                // Note: If the inner pattern can fail, both the outer failure (IsOptionSome = false)
                // and inner failure should jump to the next arm. We can only return one jump offset,
                // so we need to make them converge. For now, we'll just return the outer jump and
                // let the inner pattern's failure be handled separately. This is a known limitation.
                Ok(Some(jump_offset))
            }

            "Ok" => {
                // Similar to Some but for Result
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

                self.bytecode.emit(Opcode::Dup, span);
                self.bytecode.emit(Opcode::IsResultOk, span);

                self.bytecode.emit(Opcode::JumpIfFalse, span);
                let jump_offset = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF);

                self.bytecode.emit(Opcode::ExtractResultValue, span);

                let _inner_failed_jump =
                    self.compile_pattern_check(&args[0], span, locals_before)?;

                self.bytecode.emit(Opcode::Pop, span);

                // SetLocal peeks (doesn't pop), so if inner pattern is Variable, we have an orphan
                if matches!(&args[0], Pattern::Variable(_)) {
                    self.bytecode.emit(Opcode::Pop, span);
                }

                // Push true (this pattern matched)
                self.bytecode.emit(Opcode::True, span);

                Ok(Some(jump_offset))
            }

            "Err" => {
                // Similar to Ok but checks for Err variant
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

                self.bytecode.emit(Opcode::Dup, span);
                self.bytecode.emit(Opcode::IsResultErr, span);

                self.bytecode.emit(Opcode::JumpIfFalse, span);
                let jump_offset = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF);

                self.bytecode.emit(Opcode::ExtractResultValue, span);

                let _inner_failed_jump =
                    self.compile_pattern_check(&args[0], span, locals_before)?;

                self.bytecode.emit(Opcode::Pop, span);

                // SetLocal peeks (doesn't pop), so if inner pattern is Variable, we have an orphan
                if matches!(&args[0], Pattern::Variable(_)) {
                    self.bytecode.emit(Opcode::Pop, span);
                }

                // Push true (this pattern matched)
                self.bytecode.emit(Opcode::True, span);

                Ok(Some(jump_offset))
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

    /// Compile array pattern [x, y, z]
    ///
    /// Checks if scrutinee is an array with correct length, then matches elements.
    fn compile_array_pattern(
        &mut self,
        elements: &[Pattern],
        span: Span,
        locals_before: usize,
    ) -> Result<Option<usize>, Vec<Diagnostic>> {
        use crate::bytecode::Opcode;

        // Scrutinee (array) is on stack
        // Strategy:
        // 1. Dup and check if it's an array
        // 2. Dup and check length
        // 3. For each element: get element, match pattern, bind variables

        // Check if value is an array
        self.bytecode.emit(Opcode::Dup, span);
        self.bytecode.emit(Opcode::IsArray, span);

        self.bytecode.emit(Opcode::JumpIfFalse, span);
        let not_array_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // Check array length
        self.bytecode.emit(Opcode::Dup, span);
        self.bytecode.emit(Opcode::GetArrayLen, span);

        // Push expected length
        let expected_len = elements.len() as f64;
        let const_idx = self.bytecode.add_constant(Value::Number(expected_len));
        self.bytecode.emit(Opcode::Constant, span);
        self.bytecode.emit_u16(const_idx);

        // Compare lengths
        self.bytecode.emit(Opcode::Equal, span);

        self.bytecode.emit(Opcode::JumpIfFalse, span);
        let wrong_length_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // Array is correct type and length!
        // Now match each element
        for (idx, elem_pattern) in elements.iter().enumerate() {
            // Dup array
            self.bytecode.emit(Opcode::Dup, span);

            // Push index
            let idx_const = self.bytecode.add_constant(Value::Number(idx as f64));
            self.bytecode.emit(Opcode::Constant, span);
            self.bytecode.emit_u16(idx_const);

            // Get element at index
            self.bytecode.emit(Opcode::GetIndex, span);

            // Match pattern against element
            let elem_failed_jump = self.compile_pattern_check(elem_pattern, span, locals_before)?;

            // Pattern matched - pop success flag
            self.bytecode.emit(Opcode::Pop, span);

            // SetLocal peeks (doesn't pop), so the element value is still on stack
            // For Variable patterns, we need to pop it here (already stored in local)
            // For other patterns, they consume the value, so no extra pop needed
            if matches!(elem_pattern, Pattern::Variable(_)) {
                self.bytecode.emit(Opcode::Pop, span);
            }

            // If pattern matching failed, jump to next arm
            if let Some(jump) = elem_failed_jump {
                self.bytecode.patch_jump(jump);
            }
        }

        // All elements matched! Pop the array (we're done with it)
        self.bytecode.emit(Opcode::Pop, span);

        // Push true (match succeeded)
        self.bytecode.emit(Opcode::True, span);

        // Jump to after failure handling
        self.bytecode.emit(Opcode::Jump, span);
        let success_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // Patch failure jumps to come here
        self.bytecode.patch_jump(not_array_jump);
        self.bytecode.patch_jump(wrong_length_jump);

        // Failed - push false
        self.bytecode.emit(Opcode::False, span);

        // Patch success jump
        self.bytecode.patch_jump(success_jump);

        // Return jump for failed match
        self.bytecode.emit(Opcode::JumpIfFalse, span);
        let final_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        Ok(Some(final_jump))
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
