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
