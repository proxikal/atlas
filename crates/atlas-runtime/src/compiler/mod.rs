//! AST to bytecode compiler
//!
//! Compiles AST directly to stack-based bytecode.
//! - Expressions leave their result on the stack
//! - Statements may or may not leave values on the stack
//! - Locals are tracked by index (stack slots)
//! - Globals are tracked by name (string constants)

mod expr;
mod stmt;

use crate::ast::*;
use crate::bytecode::{Bytecode, Opcode, Optimizer};
use crate::diagnostic::Diagnostic;
use crate::optimizer::{ConstantFoldingPass, DeadCodeEliminationPass, PeepholePass};
use crate::span::Span;

/// Local variable information
#[derive(Debug, Clone)]
pub(super) struct Local {
    pub(super) name: String,
    /// Scope depth of this local (for shadowing resolution)
    #[allow(dead_code)] // TODO: Use in scope resolution (future phase)
    pub(super) depth: usize,
    /// Whether this local is mutable (let vs var)
    #[allow(dead_code)] // TODO: Use for const checking (future phase)
    pub(super) mutable: bool,
    /// Scoped name for nested functions (None for regular variables)
    /// Used to access nested functions globally from siblings
    pub(super) scoped_name: Option<String>,
}

/// Loop context for break/continue
#[derive(Debug, Clone)]
pub(super) struct LoopContext {
    pub(super) start_offset: usize,
    pub(super) break_jumps: Vec<usize>,
}

/// Compiler state
pub struct Compiler {
    /// Output bytecode
    pub(super) bytecode: Bytecode,
    /// Local variables (stack slots)
    pub(super) locals: Vec<Local>,
    /// Current scope depth
    pub(super) scope_depth: usize,
    /// Loop context stack (for break/continue)
    pub(super) loops: Vec<LoopContext>,
    /// Bytecode optimizer (optional)
    optimizer: Option<Optimizer>,
    /// Monomorphizer for generic functions
    #[allow(dead_code)] // Will be used when generic runtime support is fully integrated
    pub(super) monomorphizer: crate::typechecker::generics::Monomorphizer,
    /// Counter for generating unique nested function names
    next_func_id: usize,
    /// Base index for current function's locals (for nested functions)
    /// Used to distinguish parent-scope locals from function-local variables
    pub(super) current_function_base: usize,
}

impl Compiler {
    /// Create a new compiler
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            locals: Vec::new(),
            scope_depth: 0,
            loops: Vec::new(),
            optimizer: None, // Optimization disabled by default
            monomorphizer: crate::typechecker::generics::Monomorphizer::new(),
            next_func_id: 0,
            current_function_base: 0,
        }
    }

    /// Enable bytecode optimization with all three passes:
    /// constant folding, dead code elimination, and peephole optimizations.
    pub fn with_optimization() -> Self {
        let mut optimizer = Optimizer::new();
        optimizer.set_enabled(true);
        optimizer.add_pass(Box::new(ConstantFoldingPass));
        optimizer.add_pass(Box::new(DeadCodeEliminationPass));
        optimizer.add_pass(Box::new(PeepholePass));

        Self {
            bytecode: Bytecode::new(),
            locals: Vec::new(),
            scope_depth: 0,
            loops: Vec::new(),
            optimizer: Some(optimizer),
            monomorphizer: crate::typechecker::generics::Monomorphizer::new(),
            next_func_id: 0,
            current_function_base: 0,
        }
    }

    /// Set the optimizer to use (or None to disable optimization)
    pub fn set_optimizer(&mut self, optimizer: Option<Optimizer>) {
        self.optimizer = optimizer;
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
        let mut bytecode = std::mem::take(&mut self.bytecode);

        // Apply optimization if enabled
        if let Some(ref optimizer) = self.optimizer {
            bytecode = optimizer.optimize(bytecode);
        }

        Ok(bytecode)
    }

    /// Compile a top-level item
    fn compile_item(&mut self, item: &Item) -> Result<(), Vec<Diagnostic>> {
        match item {
            Item::Function(func) => self.compile_function(func),
            Item::Statement(stmt) => self.compile_stmt(stmt),
            Item::Import(_) => {
                // Imports don't generate code - they're resolved at compile time
                Ok(())
            }
            Item::Export(export_decl) => {
                // Export wraps an item - compile the inner item
                match &export_decl.item {
                    crate::ast::ExportItem::Function(func) => self.compile_function(func),
                    crate::ast::ExportItem::Variable(var) => {
                        self.compile_stmt(&crate::ast::Stmt::VarDecl(var.clone()))
                    }
                    crate::ast::ExportItem::TypeAlias(_) => Ok(()),
                }
            }
            Item::Extern(_) => {
                // Extern declarations don't generate bytecode - they're loaded at runtime
                // Full implementation in phase-10b (FFI infrastructure)
                Ok(())
            }
            Item::TypeAlias(_) => Ok(()),
        }
    }

    /// Compile a function declaration
    fn compile_function(&mut self, func: &FunctionDecl) -> Result<(), Vec<Diagnostic>> {
        // We'll update the function ref after compiling the body to get accurate local_count
        // For now, create a placeholder with bytecode_offset = 0 (will be updated)
        let placeholder_ref = crate::value::FunctionRef {
            name: func.name.name.clone(),
            arity: func.params.len(),
            bytecode_offset: 0, // Placeholder - will be updated after Jump
            local_count: 0,     // Will be updated after compiling body
        };
        let placeholder_value = crate::value::Value::Function(placeholder_ref);

        // Add placeholder to constant pool
        let const_idx = self.bytecode.add_constant(placeholder_value);
        self.bytecode.emit(Opcode::Constant, func.span);
        self.bytecode.emit_u16(const_idx);

        // Store function as a global variable (so it can be called)
        let name_idx = self
            .bytecode
            .add_constant(crate::value::Value::string(&func.name.name));
        self.bytecode.emit(Opcode::SetGlobal, func.span);
        self.bytecode.emit_u16(name_idx);
        self.bytecode.emit(Opcode::Pop, func.span);

        // Jump over the function body (so it's not executed during program init)
        self.bytecode.emit(Opcode::Jump, func.span);
        let skip_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF); // Placeholder

        // NOW record the function body offset (after all setup code)
        let function_offset = self.bytecode.current_offset();

        // Now compile the function body at function_offset
        // The function body is compiled inline in the bytecode
        // When called, the VM will jump here

        // Set up parameters as local variables
        // Parameters are expected to be on the stack when the function is called
        // They're already there from the CALL instruction
        // We just need to track them as locals
        let old_locals_len = self.locals.len();
        let old_scope = self.scope_depth;
        self.scope_depth += 1;

        // Add parameters as locals
        for param in &func.params {
            self.locals.push(Local {
                name: param.name.name.clone(),
                depth: self.scope_depth,
                mutable: true, // Parameters are always mutable
                scoped_name: None,
            });
        }

        // Track function base for nested function support
        let prev_function_base = std::mem::replace(&mut self.current_function_base, old_locals_len);

        // Compile function body
        self.compile_block(&func.body)?;

        // Restore function base
        self.current_function_base = prev_function_base;

        // Calculate total local count (all locals added during function compilation)
        let total_local_count = self.locals.len() - old_locals_len;

        // If function doesn't end with explicit return, add implicit "return null"
        self.bytecode.emit(Opcode::Null, func.span);
        self.bytecode.emit(Opcode::Return, func.span);

        // Restore scope and locals
        self.scope_depth = old_scope;
        self.locals.truncate(old_locals_len);

        // Update the FunctionRef in constants with accurate local_count
        let updated_ref = crate::value::FunctionRef {
            name: func.name.name.clone(),
            arity: func.params.len(),
            bytecode_offset: function_offset,
            local_count: total_local_count,
        };
        self.bytecode.constants[const_idx as usize] = crate::value::Value::Function(updated_ref);

        // Patch the skip jump to go past the function body
        self.bytecode.patch_jump(skip_jump);

        Ok(())
    }

    /// Resolve a local variable by name, returning its index if found
    pub(super) fn resolve_local(&self, name: &str) -> Option<usize> {
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
    use crate::value::Value;

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
        assert!(!bytecode.instructions.is_empty());
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
    fn test_compile_user_function_basic() {
        let bytecode = compile_source("fn add(a: number, b: number) -> number { return a + b; }");
        // Should have instructions for function definition
        assert!(!bytecode.instructions.is_empty());
        // Should have function in constants
        let has_function = bytecode
            .constants
            .iter()
            .any(|c| matches!(c, Value::Function(_)));
        assert!(has_function, "Should have function in constants");
    }

    #[test]
    fn test_compile_function_call_user_defined() {
        let bytecode = compile_source(
            r#"
            fn double(x: number) -> number { return x * 2; }
            double(21);
        "#,
        );
        // Should have Call opcode
        let has_call = bytecode.instructions.contains(&(Opcode::Call as u8));
        assert!(has_call, "Should have Call opcode");
    }

    #[test]
    fn test_compile_array_index_assignment() {
        let bytecode = compile_source("let arr = [1, 2, 3]; arr[0] = 42;");
        // Should have SetIndex opcode
        let has_setindex = bytecode.instructions.contains(&(Opcode::SetIndex as u8));
        assert!(has_setindex, "Should have SetIndex opcode");
    }

    #[test]
    fn test_compile_compound_assignment_add() {
        let bytecode = compile_source("let x = 10; x += 5;");
        // Should have GetGlobal, Constant(5), Add, SetGlobal
        let add_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::Add as u8);
        assert!(add_pos.is_some(), "Should have Add opcode for +=");
    }

    #[test]
    fn test_compile_increment() {
        let bytecode = compile_source("let x = 5; x++;");
        // Should have GetGlobal, Constant(1), Add, SetGlobal
        let add_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::Add as u8);
        assert!(add_pos.is_some(), "Should have Add opcode for ++");
    }

    #[test]
    fn test_compile_decrement() {
        let bytecode = compile_source("let x = 5; x--;");
        // Should have GetGlobal, Constant(1), Sub, SetGlobal
        let sub_pos = bytecode
            .instructions
            .iter()
            .position(|&b| b == Opcode::Sub as u8);
        assert!(sub_pos.is_some(), "Should have Sub opcode for --");
    }

    #[test]
    fn test_compile_short_circuit_and() {
        let bytecode = compile_source("let result = false && true;");
        // Should have Dup and JumpIfFalse for short-circuit
        let has_dup = bytecode.instructions.contains(&(Opcode::Dup as u8));
        let has_jump = bytecode.instructions.contains(&(Opcode::JumpIfFalse as u8));
        assert!(has_dup, "Should have Dup for short-circuit");
        assert!(has_jump, "Should have JumpIfFalse for short-circuit");
    }

    #[test]
    fn test_compile_short_circuit_or() {
        let bytecode = compile_source("let result = true || false;");
        // Should have Dup, Not, and JumpIfFalse for short-circuit
        let has_dup = bytecode.instructions.contains(&(Opcode::Dup as u8));
        let has_not = bytecode.instructions.contains(&(Opcode::Not as u8));
        let has_jump = bytecode.instructions.contains(&(Opcode::JumpIfFalse as u8));
        assert!(has_dup, "Should have Dup for short-circuit");
        assert!(has_not, "Should have Not for || short-circuit");
        assert!(has_jump, "Should have JumpIfFalse for short-circuit");
    }

    #[test]
    fn test_compile_return_statement() {
        let bytecode = compile_source("fn test() -> number { return 42; }");
        // Should have Return opcode
        let has_return = bytecode.instructions.contains(&(Opcode::Return as u8));
        assert!(has_return, "Should have Return opcode");
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

    // ===== Constants Pool Compilation Tests =====

    #[test]
    fn test_compile_number_to_constant_pool() {
        // Verify numbers are added to constant pool, not inlined
        let bytecode = compile_source("42; 2.5; -1.5;");

        // Should have 3 number constants (including the one for -1.5)
        assert!(bytecode.constants.len() >= 2); // At least 42 and 2.5
        assert_eq!(bytecode.constants[0], Value::Number(42.0));
        assert_eq!(bytecode.constants[1], Value::Number(2.5));
    }

    #[test]
    fn test_compile_string_to_constant_pool() {
        // Verify strings are added to constant pool
        let bytecode = compile_source("\"hello\"; \"world\";");

        // Should have at least 2 string constants
        assert!(bytecode.constants.len() >= 2);
        if let Value::String(s) = &bytecode.constants[0] {
            assert_eq!(s.as_ref(), "hello");
        } else {
            panic!("Expected string constant");
        }
        if let Value::String(s) = &bytecode.constants[1] {
            assert_eq!(s.as_ref(), "world");
        } else {
            panic!("Expected string constant");
        }
    }

    #[test]
    fn test_compile_variable_names_to_constant_pool() {
        // Variable names for globals should be in constant pool
        let bytecode = compile_source("let x = 10; let y = 20;");

        // Should have: 10, "x", 20, "y" in constants
        assert!(bytecode.constants.len() >= 4);

        // Verify we have number and string constants
        let has_number = bytecode
            .constants
            .iter()
            .any(|c| matches!(c, Value::Number(_)));
        let has_string = bytecode
            .constants
            .iter()
            .any(|c| matches!(c, Value::String(_)));
        assert!(has_number, "Should have number constants");
        assert!(
            has_string,
            "Should have string constants for variable names"
        );
    }

    #[test]
    fn test_compile_constant_indices_sequential() {
        // Verify that constants are indexed sequentially
        let bytecode = compile_source("1; 2; 3; 4; 5;");

        // Should have at least 5 constants
        assert!(bytecode.constants.len() >= 5);

        // All should be numbers
        for i in 0..5 {
            assert!(matches!(bytecode.constants[i], Value::Number(_)));
        }
    }

    #[test]
    fn test_compile_repeated_constants() {
        // Test that repeated values create separate constant entries
        let bytecode = compile_source("42; 42; 42;");

        // Current implementation doesn't deduplicate, so should have 3 entries
        let count_42 = bytecode
            .constants
            .iter()
            .filter(|&c| *c == Value::Number(42.0))
            .count();
        assert_eq!(count_42, 3, "Should store 42 three times");
    }

    #[test]
    fn test_compile_mixed_constant_types() {
        // Test that different types are properly added to constant pool
        let bytecode = compile_source(r#"42; "hello"; 3.14; "world"; -1;"#);

        // Should have mix of numbers and strings
        let number_count = bytecode
            .constants
            .iter()
            .filter(|c| matches!(c, Value::Number(_)))
            .count();
        let string_count = bytecode
            .constants
            .iter()
            .filter(|c| matches!(c, Value::String(_)))
            .count();

        assert!(number_count >= 3, "Should have at least 3 number constants");
        assert!(string_count >= 2, "Should have at least 2 string constants");
    }

    #[test]
    fn test_compile_constant_in_arithmetic() {
        // Test that constants in arithmetic are properly pooled
        let bytecode = compile_source("1 + 2 + 3;");

        // Should have 3 number constants
        assert!(bytecode.constants.len() >= 3);
        assert_eq!(bytecode.constants[0], Value::Number(1.0));
        assert_eq!(bytecode.constants[1], Value::Number(2.0));
        assert_eq!(bytecode.constants[2], Value::Number(3.0));
    }

    #[test]
    fn test_compile_large_number_constant() {
        // Test that large numbers are handled correctly
        let bytecode = compile_source("999999999.123456789;");

        assert!(!bytecode.constants.is_empty());
        assert_eq!(bytecode.constants[0], Value::Number(999_999_999.123_456_8));
    }

    #[test]
    fn test_compile_empty_string_constant() {
        // Test that empty strings work
        let bytecode = compile_source("\"\";");

        assert!(!bytecode.constants.is_empty());
        if let Value::String(s) = &bytecode.constants[0] {
            assert_eq!(s.as_ref(), "");
        } else {
            panic!("Expected empty string constant");
        }
    }

    #[test]
    fn test_compile_long_string_constant() {
        // Test that long strings work
        let long_str = "a".repeat(1000);
        let source = format!(r#""{}";"#, long_str);
        let bytecode = compile_source(&source);

        assert!(!bytecode.constants.is_empty());
        if let Value::String(s) = &bytecode.constants[0] {
            assert_eq!(s.len(), 1000);
        } else {
            panic!("Expected long string constant");
        }
    }

    #[test]
    fn test_compile_constant_pool_size() {
        // Test that we can handle a reasonable number of constants
        let mut source = String::new();
        for i in 0..100 {
            source.push_str(&format!("{}; ", i));
        }

        let bytecode = compile_source(&source);

        // Should have at least 100 constants
        assert!(bytecode.constants.len() >= 100);
    }

    // ===== Debug Info Default Tests (Phase 15) =====

    #[test]
    fn test_debug_info_present_by_default() {
        // Verify that compiled code contains debug info by default
        let bytecode = compile_source("let x = 42;");

        // Debug info should not be empty
        assert!(
            !bytecode.debug_info.is_empty(),
            "Debug info should be present by default"
        );

        // Should have debug info for each emitted opcode
        // At minimum: Constant, SetGlobal, Pop, Halt
        assert!(
            bytecode.debug_info.len() >= 3,
            "Should have debug spans for multiple instructions"
        );
    }

    #[test]
    fn test_debug_info_for_expressions() {
        // Test that expressions preserve debug info
        let bytecode = compile_source("1 + 2 * 3;");

        // Should have debug info for: Constant(1), Constant(2), Constant(3), Mul, Add, Pop, Halt
        assert!(
            bytecode.debug_info.len() >= 5,
            "Expression should generate debug info"
        );

        // Verify all debug spans have valid instruction offsets
        for debug_span in &bytecode.debug_info {
            assert!(
                debug_span.instruction_offset < bytecode.instructions.len(),
                "Debug span offset should be within instruction bounds"
            );
        }
    }

    #[test]
    fn test_debug_info_for_control_flow() {
        // Test that control flow statements preserve debug info
        let bytecode = compile_source("if (true) { 42; }");

        // Should have debug info for: True, JumpIfFalse, Constant, Pop, Halt
        assert!(
            bytecode.debug_info.len() >= 3,
            "Control flow should generate debug info"
        );

        // Check that we have a JumpIfFalse instruction with debug info
        let has_jump = bytecode.instructions.contains(&(Opcode::JumpIfFalse as u8));
        assert!(has_jump, "Should have JumpIfFalse instruction");
    }

    #[test]
    fn test_debug_info_for_loops() {
        // Test that loops preserve debug info
        let bytecode = compile_source("while (true) { 1; }");

        // Should have debug info for loop instructions (True, JumpIfFalse, Constant, Pop, Loop, ...)
        assert!(
            bytecode.debug_info.len() >= 4,
            "Loops should generate debug info"
        );

        // Verify we have Loop opcode with debug info
        let has_loop = bytecode.instructions.contains(&(Opcode::Loop as u8));
        assert!(has_loop, "Should have Loop instruction");
    }

    #[test]
    fn test_debug_info_for_arrays() {
        // Test that array operations preserve debug info
        let bytecode = compile_source("[1, 2, 3];");

        // Should have debug info for: Constant(1), Constant(2), Constant(3), Array, Pop, Halt
        assert!(
            bytecode.debug_info.len() >= 4,
            "Array operations should generate debug info"
        );

        // Verify we have Array opcode
        let has_array = bytecode.instructions.contains(&(Opcode::Array as u8));
        assert!(has_array, "Should have Array instruction");
    }

    #[test]
    fn test_debug_info_spans_not_dummy() {
        // Verify that most debug spans are not dummy spans (have real source positions)
        let bytecode = compile_source("let x = 1 + 2;");

        // Count non-dummy spans (dummy spans have start == end == 0)
        let non_dummy_count = bytecode
            .debug_info
            .iter()
            .filter(|ds| ds.span.start != 0 || ds.span.end != 0)
            .count();

        // Most spans should be real (only Halt uses Span::dummy())
        assert!(
            non_dummy_count >= bytecode.debug_info.len() - 1,
            "Most debug spans should have real source positions"
        );
    }

    #[test]
    fn test_debug_info_serialization_flag() {
        // Test that serialized bytecode has the debug info flag set
        let bytecode = compile_source("42;");

        let bytes = bytecode.to_bytes();

        // Extract flags from header (bytes 6-7)
        assert!(bytes.len() >= 8, "Bytecode should have valid header");
        let flags = u16::from_be_bytes([bytes[6], bytes[7]]);

        // Bit 0 should be set (debug info present)
        assert_eq!(
            flags & 1,
            1,
            "Debug info flag should be set in serialized bytecode"
        );
    }

    #[test]
    fn test_debug_info_roundtrip_preserves_spans() {
        // Test that debug info survives serialization roundtrip
        let bytecode = compile_source("let x = 10; x + 5;");

        let original_debug_count = bytecode.debug_info.len();
        assert!(
            original_debug_count > 0,
            "Should have debug info before serialization"
        );

        // Serialize and deserialize
        let bytes = bytecode.to_bytes();
        let loaded = Bytecode::from_bytes(&bytes).expect("Deserialization should succeed");

        // Debug info should be preserved
        assert_eq!(
            loaded.debug_info.len(),
            original_debug_count,
            "Debug info count should match after roundtrip"
        );

        // Verify each debug span matches
        for (i, debug_span) in loaded.debug_info.iter().enumerate() {
            assert_eq!(
                debug_span.instruction_offset, bytecode.debug_info[i].instruction_offset,
                "Instruction offset should match for debug span {}",
                i
            );
            assert_eq!(
                debug_span.span, bytecode.debug_info[i].span,
                "Span should match for debug span {}",
                i
            );
        }
    }

    #[test]
    fn test_debug_info_for_complex_program() {
        // Test a more complex program to ensure debug info scales
        let source = r#"
            let a = 1;
            let b = 2;
            let c = a + b;
            if (c > 0) {
                let d = c * 2;
                d;
            }
        "#;

        let bytecode = compile_source(source);

        // Should have substantial debug info for all these operations
        assert!(
            bytecode.debug_info.len() >= 10,
            "Complex program should generate substantial debug info"
        );

        // Verify debug info is monotonically increasing in instruction offsets
        // (each new instruction should have an offset >= the previous)
        for i in 1..bytecode.debug_info.len() {
            assert!(
                bytecode.debug_info[i].instruction_offset
                    >= bytecode.debug_info[i - 1].instruction_offset,
                "Debug spans should be in instruction order"
            );
        }
    }

    #[test]
    fn test_empty_program_has_minimal_debug_info() {
        // Empty program should have debug info only for Halt
        let mut compiler = Compiler::new();
        let program = Program { items: Vec::new() };
        let bytecode = compiler.compile(&program).unwrap();

        // Should have just one debug span for Halt
        assert_eq!(
            bytecode.debug_info.len(),
            1,
            "Empty program should have debug info for Halt only"
        );
        assert_eq!(
            bytecode.debug_info[0].instruction_offset, 0,
            "Halt should be at offset 0"
        );
    }

    #[test]
    fn test_debug_info_with_optimization() {
        // Test that debug info is preserved even with optimization enabled
        let mut lexer = Lexer::new("1 + 2;".to_string());
        let (tokens, lex_diags) = lexer.tokenize();
        assert!(lex_diags.is_empty());

        let mut parser = Parser::new(tokens);
        let (program, parse_diags) = parser.parse();
        assert!(parse_diags.is_empty());

        let mut compiler = Compiler::with_optimization();
        let bytecode = compiler.compile(&program).expect("Compilation failed");

        // Even with optimization, debug info should be present
        assert!(
            !bytecode.debug_info.is_empty(),
            "Debug info should be present even with optimization"
        );

        // Serialization should still include debug info
        let bytes = bytecode.to_bytes();
        let flags = u16::from_be_bytes([bytes[6], bytes[7]]);
        assert_eq!(
            flags & 1,
            1,
            "Debug info flag should be set with optimization"
        );
    }
}
