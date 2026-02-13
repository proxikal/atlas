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

        // Compile all arguments first (they'll be on the stack)
        for arg in &call.args {
            self.compile_expr(arg)?;
        }

        // Check if it's a builtin function
        if crate::stdlib::is_builtin(func_name) {
            // Create a Function value for the builtin with bytecode_offset = 0
            let func_ref = crate::value::FunctionRef {
                name: func_name.to_string(),
                arity: call.args.len(),
                bytecode_offset: 0, // Builtins have offset 0
            };
            let func_value = crate::value::Value::Function(func_ref);
            let const_idx = self.bytecode.add_constant(func_value);

            // Load the function constant
            self.bytecode.emit(crate::bytecode::Opcode::Constant, call.span);
            self.bytecode.emit_u16(const_idx);

            // Emit call instruction with argument count
            self.bytecode.emit(crate::bytecode::Opcode::Call, call.span);
            self.bytecode.emit_u8(call.args.len() as u8);
        } else {
            // User-defined function (TODO: implement later)
            // For now, just emit a placeholder
            return Ok(());
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
        // Compile left and right operands
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
            BinaryOp::And => Opcode::And,
            BinaryOp::Or => Opcode::Or,
        };
        self.bytecode.emit(opcode, bin.span);
        Ok(())
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
