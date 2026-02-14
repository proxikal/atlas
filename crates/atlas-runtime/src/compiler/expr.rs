//! Expression compilation

use crate::ast::*;
use crate::bytecode::Opcode;
use crate::compiler::Compiler;
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
            Expr::Match(_match_expr) => {
                // Pattern matching runtime execution is BLOCKER 03-B
                // This is BLOCKER 03-A (syntax & type checking only)
                Err(vec![Diagnostic::error_with_code(
                    "AT9999",
                    "Pattern matching runtime execution not yet implemented (BLOCKER 03-B)",
                    expr.span(),
                )
                .with_label("not implemented")])
            }
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

        // Load the function (either builtin or user-defined)
        // For builtins, we create a FunctionRef; for user-defined, we load from globals
        if crate::stdlib::is_builtin(func_name) {
            // Create a Function value for the builtin with bytecode_offset = 0
            let func_ref = crate::value::FunctionRef {
                name: func_name.to_string(),
                arity: call.args.len(),
                bytecode_offset: 0, // Builtins have offset 0
                local_count: 0,     // Builtins don't use local variables
            };
            let func_value = crate::value::Value::Function(func_ref);
            let const_idx = self.bytecode.add_constant(func_value);

            // Load the function constant
            self.bytecode.emit(Opcode::Constant, call.span);
            self.bytecode.emit_u16(const_idx);
        } else {
            // User-defined function - load from global
            // Try local first (for nested functions in the future)
            if let Some(local_idx) = self.resolve_local(func_name) {
                self.bytecode.emit(Opcode::GetLocal, call.span);
                self.bytecode.emit_u16(local_idx as u16);
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
            self.bytecode.emit(Opcode::GetLocal, ident.span);
            self.bytecode.emit_u16(local_idx as u16);
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
}
