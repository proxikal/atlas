//! AST to bytecode compiler
//!
//! Compiles AST directly to stack-based bytecode.
//! - Expressions leave their result on the stack
//! - Statements may or may not leave values on the stack
//! - Locals are tracked by index (stack slots)
//! - Globals are tracked by name (string constants)

use crate::ast::*;
use crate::bytecode::{Bytecode, Opcode};
use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::value::Value;

/// Local variable information
#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: usize,
    mutable: bool,
}

/// Loop context for break/continue
#[derive(Debug, Clone)]
struct LoopContext {
    start_offset: usize,
    break_jumps: Vec<usize>,
}

/// Compiler state
pub struct Compiler {
    /// Output bytecode
    bytecode: Bytecode,
    /// Local variables (stack slots)
    locals: Vec<Local>,
    /// Current scope depth
    scope_depth: usize,
    /// Loop context stack (for break/continue)
    loops: Vec<LoopContext>,
}

impl Compiler {
    /// Create a new compiler
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            locals: Vec::new(),
            scope_depth: 0,
            loops: Vec::new(),
        }
    }

    /// Compile an AST to bytecode
    pub fn compile(&mut self, program: &Program) -> Result<Bytecode, Vec<Diagnostic>> {
        // Compile all top-level items
        for item in &program.items {
            self.compile_item(item)?;
        }

        // Emit halt at the end
        self.bytecode.emit(Opcode::Halt, Span::dummy());

        // Take ownership of the bytecode
        Ok(std::mem::replace(&mut self.bytecode, Bytecode::new()))
    }

    /// Compile a top-level item
    fn compile_item(&mut self, item: &Item) -> Result<(), Vec<Diagnostic>> {
        match item {
            Item::Function(_func) => {
                // TODO: Function compilation in later iteration
                Ok(())
            }
            Item::Statement(stmt) => self.compile_stmt(stmt),
        }
    }

    /// Compile a statement
    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), Vec<Diagnostic>> {
        match stmt {
            Stmt::VarDecl(decl) => self.compile_var_decl(decl),
            Stmt::Assign(assign) => self.compile_assign(assign),
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
            Stmt::CompoundAssign(_) | Stmt::Increment(_) | Stmt::Decrement(_) => {
                // TODO: Compound assignment and increment/decrement
                Ok(())
            }
        }
    }

    /// Compile a variable declaration
    fn compile_var_decl(&mut self, decl: &VarDecl) -> Result<(), Vec<Diagnostic>> {
        // Compile the initializer
        self.compile_expr(&decl.init)?;

        if self.scope_depth == 0 {
            // Global variable - use SetGlobal
            let name_idx = self.bytecode.add_constant(Value::string(&decl.name.name));
            self.bytecode.emit(Opcode::SetGlobal, decl.span);
            self.bytecode.emit_u16(name_idx);
        } else {
            // Local variable - add to locals list
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
        // Compile the value
        self.compile_expr(&assign.value)?;

        match &assign.target {
            AssignTarget::Name(ident) => {
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
            AssignTarget::Index { .. } => {
                // TODO: Array index assignment
            }
        }

        Ok(())
    }

    /// Compile an expression
    fn compile_expr(&mut self, expr: &Expr) -> Result<(), Vec<Diagnostic>> {
        match expr {
            Expr::Literal(lit, span) => self.compile_literal(lit, *span),
            Expr::Identifier(ident) => self.compile_identifier(ident),
            Expr::Binary(bin) => self.compile_binary(bin),
            Expr::Unary(un) => self.compile_unary(un),
            Expr::Group(group) => self.compile_expr(&group.expr),
            Expr::ArrayLiteral(arr) => self.compile_array_literal(arr),
            Expr::Index(index) => self.compile_index(index),
            Expr::Call(_) => {
                // TODO: Function calls in next phase
                Ok(())
            }
        }
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
            BinaryOp::And | BinaryOp::Or => {
                // TODO: Short-circuit evaluation
                return Ok(());
            }
        };

        self.bytecode.emit(opcode, bin.span);
        Ok(())
    }

    /// Compile a unary expression
    fn compile_unary(&mut self, un: &UnaryExpr) -> Result<(), Vec<Diagnostic>> {
        // Compile the operand
        self.compile_expr(&un.expr)?;

        // Emit the opcode
        let opcode = match un.op {
            UnaryOp::Negate => Opcode::Negate,
            UnaryOp::Not => Opcode::Not,
        };

        self.bytecode.emit(opcode, un.span);
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

        // Loop back to start
        let offset = self.bytecode.current_offset() - loop_start + 2;
        self.bytecode.emit(Opcode::Loop, while_stmt.span);
        self.bytecode.emit_i16(-(offset as i16));

        // Patch exit jump
        self.bytecode.patch_jump(exit_jump);

        // Patch all break jumps
        let loop_ctx = self.loops.pop().unwrap();
        for break_jump in loop_ctx.break_jumps {
            self.bytecode.patch_jump(break_jump);
        }

        Ok(())
    }

    /// Compile a for loop (desugared to while)
    fn compile_for(&mut self, for_stmt: &ForStmt) -> Result<(), Vec<Diagnostic>> {
        // Enter a new scope for the loop variable
        self.scope_depth += 1;

        // Compile initializer
        self.compile_stmt(&for_stmt.init)?;

        let loop_start = self.bytecode.current_offset();

        // Start loop context
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

        // Compile step expression
        self.compile_stmt(&for_stmt.step)?;

        // Loop back to condition
        let offset = self.bytecode.current_offset() - loop_start + 2;
        self.bytecode.emit(Opcode::Loop, for_stmt.span);
        self.bytecode.emit_i16(-(offset as i16));

        // Patch exit jump
        self.bytecode.patch_jump(exit_jump);

        // Patch all break jumps
        let loop_ctx = self.loops.pop().unwrap();
        for break_jump in loop_ctx.break_jumps {
            self.bytecode.patch_jump(break_jump);
        }

        // Exit scope and pop loop variable
        self.scope_depth -= 1;
        while !self.locals.is_empty() && self.locals.last().unwrap().depth > self.scope_depth {
            self.locals.pop();
            self.bytecode.emit(Opcode::Pop, for_stmt.span);
        }

        Ok(())
    }

    /// Compile a break statement
    fn compile_break(&mut self, span: Span) -> Result<(), Vec<Diagnostic>> {
        if let Some(loop_ctx) = self.loops.last_mut() {
            // Emit jump and record it for patching later
            self.bytecode.emit(Opcode::Jump, span);
            let jump_offset = self.bytecode.current_offset();
            self.bytecode.emit_u16(0xFFFF); // Placeholder
            loop_ctx.break_jumps.push(jump_offset);
            Ok(())
        } else {
            // Error: break outside loop (would be caught by typechecker)
            Ok(())
        }
    }

    /// Compile a continue statement
    fn compile_continue(&mut self, span: Span) -> Result<(), Vec<Diagnostic>> {
        if let Some(loop_ctx) = self.loops.last() {
            // Jump back to loop start
            let offset = self.bytecode.current_offset() - loop_ctx.start_offset + 2;
            self.bytecode.emit(Opcode::Loop, span);
            self.bytecode.emit_i16(-(offset as i16));
            Ok(())
        } else {
            // Error: continue outside loop (would be caught by typechecker)
            Ok(())
        }
    }

    /// Compile a block of statements
    fn compile_block(&mut self, block: &Block) -> Result<(), Vec<Diagnostic>> {
        self.scope_depth += 1;

        for stmt in &block.statements {
            self.compile_stmt(stmt)?;
        }

        // Pop all locals from this scope
        self.scope_depth -= 1;
        while !self.locals.is_empty() && self.locals.last().unwrap().depth > self.scope_depth {
            self.locals.pop();
            self.bytecode.emit(Opcode::Pop, block.span);
        }

        Ok(())
    }

    /// Compile an array literal
    fn compile_array_literal(&mut self, arr: &ArrayLiteral) -> Result<(), Vec<Diagnostic>> {
        // Compile all elements (they'll be on the stack)
        for elem in &arr.elements {
            self.compile_expr(elem)?;
        }

        // Emit Array opcode with element count
        self.bytecode.emit(Opcode::Array, arr.span);
        self.bytecode.emit_u16(arr.elements.len() as u16);

        Ok(())
    }

    /// Compile an index expression
    fn compile_index(&mut self, index: &IndexExpr) -> Result<(), Vec<Diagnostic>> {
        // Compile target and index
        self.compile_expr(&index.target)?;
        self.compile_expr(&index.index)?;

        // Emit GetIndex opcode
        self.bytecode.emit(Opcode::GetIndex, index.span);

        Ok(())
    }

    /// Resolve a local variable by name, returning its index if found
    fn resolve_local(&self, name: &str) -> Option<usize> {
        // Search from most recent to oldest (for shadowing)
        for (idx, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(idx);
            }
        }
        None
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn compile_source(source: &str) -> Bytecode {
        let mut lexer = Lexer::new(source.to_string());
        let (tokens, lex_diags) = lexer.tokenize();
        assert!(lex_diags.is_empty(), "Lexer errors: {:?}", lex_diags);

        let mut parser = Parser::new(tokens);
        let (program, parse_diags) = parser.parse();
        assert!(parse_diags.is_empty(), "Parser errors: {:?}", parse_diags);

        let mut compiler = Compiler::new();
        compiler.compile(&program).expect("Compilation failed")
    }

    #[test]
    fn test_compiler_creation() {
        let mut compiler = Compiler::new();
        let program = Program { items: Vec::new() };
        let bytecode = compiler.compile(&program).unwrap();
        // Empty program should just have Halt
        assert_eq!(bytecode.instructions.len(), 1);
        assert_eq!(bytecode.instructions[0], Opcode::Halt as u8);
    }

    #[test]
    fn test_compile_number_literal() {
        let bytecode = compile_source("42;");
        // Should have: Constant, Pop, Halt
        assert!(bytecode.instructions.len() > 0);
        assert_eq!(bytecode.instructions[0], Opcode::Constant as u8);
        assert_eq!(bytecode.constants.len(), 1);
        assert_eq!(bytecode.constants[0], Value::Number(42.0));
    }

    #[test]
    fn test_compile_arithmetic() {
        let bytecode = compile_source("2 + 3;");
        // Should have: Constant(2), Constant(3), Add, Pop, Halt
        assert_eq!(bytecode.constants.len(), 2);
        assert_eq!(bytecode.constants[0], Value::Number(2.0));
        assert_eq!(bytecode.constants[1], Value::Number(3.0));

        // Find the Add opcode
        let add_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::Add as u8);
        assert!(add_pos.is_some(), "Add opcode not found");
    }

    #[test]
    fn test_compile_comparison() {
        let bytecode = compile_source("1 < 2;");
        // Should have: Constant(1), Constant(2), Less, Pop, Halt
        let less_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::Less as u8);
        assert!(less_pos.is_some(), "Less opcode not found");
    }

    #[test]
    fn test_compile_unary() {
        let bytecode = compile_source("-42;");
        // Should have: Constant(42), Negate, Pop, Halt
        let negate_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::Negate as u8);
        assert!(negate_pos.is_some(), "Negate opcode not found");
    }

    #[test]
    fn test_compile_var_decl_global() {
        let bytecode = compile_source("let x = 42;");
        // Should have: Constant(42), SetGlobal(name_idx), Halt
        let setglobal_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::SetGlobal as u8);
        assert!(setglobal_pos.is_some(), "SetGlobal opcode not found");

        // Check that variable name was added to constants
        assert!(bytecode.constants.len() >= 2); // number + name
    }

    #[test]
    fn test_compile_var_access_global() {
        let bytecode = compile_source("let x = 10; x;");
        // Should have GetGlobal opcode for variable access
        let getglobal_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::GetGlobal as u8);
        assert!(getglobal_pos.is_some(), "GetGlobal opcode not found");
    }

    #[test]
    fn test_compile_bool_literals() {
        let bytecode = compile_source("true; false;");
        let true_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::True as u8);
        let false_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::False as u8);
        assert!(true_pos.is_some(), "True opcode not found");
        assert!(false_pos.is_some(), "False opcode not found");
    }

    #[test]
    fn test_compile_null_literal() {
        let bytecode = compile_source("null;");
        let null_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::Null as u8);
        assert!(null_pos.is_some(), "Null opcode not found");
    }

    #[test]
    fn test_compile_string_literal() {
        let bytecode = compile_source("\"hello\";");
        assert_eq!(bytecode.constants.len(), 1);
        if let Value::String(s) = &bytecode.constants[0] {
            assert_eq!(s.as_ref(), "hello");
        } else {
            panic!("Expected string constant");
        }
    }

    #[test]
    fn test_compile_array_literal() {
        let bytecode = compile_source("[1, 2, 3];");
        // Should have constants for 1, 2, 3 and Array opcode
        let array_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::Array as u8);
        assert!(array_pos.is_some(), "Array opcode not found");
    }

    #[test]
    fn test_compile_array_index() {
        let bytecode = compile_source("let arr = [1, 2, 3]; arr[0];");
        // Should have GetIndex opcode
        let getindex_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::GetIndex as u8);
        assert!(getindex_pos.is_some(), "GetIndex opcode not found");
    }

    #[test]
    fn test_compile_if_statement() {
        let bytecode = compile_source("if (true) { 42; }");
        // Should have JumpIfFalse opcode
        let jump_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::JumpIfFalse as u8);
        assert!(jump_pos.is_some(), "JumpIfFalse opcode not found");
    }

    #[test]
    fn test_compile_if_else_statement() {
        let bytecode = compile_source("if (true) { 1; } else { 2; }");
        // Should have both JumpIfFalse and Jump opcodes
        let jumpif_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::JumpIfFalse as u8);
        let jump_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::Jump as u8);
        assert!(jumpif_pos.is_some(), "JumpIfFalse opcode not found");
        assert!(jump_pos.is_some(), "Jump opcode not found");
    }

    #[test]
    fn test_compile_while_loop() {
        let bytecode = compile_source("while (true) { 42; }");
        // Should have JumpIfFalse and Loop opcodes
        let jumpif_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::JumpIfFalse as u8);
        let loop_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::Loop as u8);
        assert!(jumpif_pos.is_some(), "JumpIfFalse opcode not found");
        assert!(loop_pos.is_some(), "Loop opcode not found");
    }

    #[test]
    fn test_compile_for_loop() {
        let bytecode = compile_source("for (let i = 0; i < 10; i = i + 1) { 42; }");
        // Should have Loop and JumpIfFalse opcodes
        let loop_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::Loop as u8);
        assert!(loop_pos.is_some(), "Loop opcode not found");
    }

    #[test]
    fn test_compile_return_statement() {
        let bytecode = compile_source("fn test() -> number { return 42; }");
        // Function compilation is TODO, but we should handle the syntax
        // For now, just make sure it doesn't crash
        assert!(bytecode.instructions.len() > 0);
    }

    #[test]
    fn test_compile_complex_expression() {
        let bytecode = compile_source("(1 + 2) * (3 - 4);");
        // Should have multiple arithmetic opcodes
        assert!(bytecode.constants.len() >= 4);
    }

    #[test]
    fn test_compile_logical_not() {
        let bytecode = compile_source("!true;");
        let not_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::Not as u8);
        assert!(not_pos.is_some(), "Not opcode not found");
    }
}
