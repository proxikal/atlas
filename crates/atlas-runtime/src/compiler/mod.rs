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
use crate::bytecode::{Bytecode, Opcode};
use crate::diagnostic::Diagnostic;
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

    // ===== Constants Pool Compilation Tests =====

    #[test]
    fn test_compile_number_to_constant_pool() {
        // Verify numbers are added to constant pool, not inlined
        let bytecode = compile_source("42; 3.14; -1.5;");

        // Should have 3 number constants (including the one for -1.5)
        assert!(bytecode.constants.len() >= 2); // At least 42 and 3.14
        assert_eq!(bytecode.constants[0], Value::Number(42.0));
        assert_eq!(bytecode.constants[1], Value::Number(3.14));
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
        let has_number = bytecode.constants.iter().any(|c| matches!(c, Value::Number(_)));
        let has_string = bytecode.constants.iter().any(|c| matches!(c, Value::String(_)));
        assert!(has_number, "Should have number constants");
        assert!(has_string, "Should have string constants for variable names");
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

        assert!(bytecode.constants.len() >= 1);
        assert_eq!(bytecode.constants[0], Value::Number(999999999.123456789));
    }

    #[test]
    fn test_compile_empty_string_constant() {
        // Test that empty strings work
        let bytecode = compile_source("\"\";");

        assert!(bytecode.constants.len() >= 1);
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

        assert!(bytecode.constants.len() >= 1);
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
}
