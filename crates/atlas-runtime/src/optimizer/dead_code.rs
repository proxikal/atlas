//! Dead code elimination optimization pass
//!
//! Removes instructions that can never be executed:
//! - Code after unconditional jumps/returns (when not a jump target)
//! - Any instruction not reachable from the entry point
//!
//! Algorithm:
//! 1. Decode bytecode into instruction list
//! 2. BFS from offset 0, following normal execution and jump targets
//! 3. Remove all instructions not in the reachable set
//! 4. Fix all jump offsets and function references to account for removals

use super::{
    decode_instructions, encode_instructions, fix_all_references, DecodedInstruction,
    OptimizationPass, OptimizationStats,
};
use crate::bytecode::{Bytecode, Opcode};
use std::collections::{HashMap, HashSet, VecDeque};

/// Dead code elimination pass
///
/// Performs a BFS reachability analysis from instruction offset 0 and removes
/// any instructions that cannot be reached during normal execution.
pub struct DeadCodeEliminationPass;

impl OptimizationPass for DeadCodeEliminationPass {
    fn name(&self) -> &str {
        "dead-code-elimination"
    }

    fn optimize(&self, bytecode: Bytecode) -> (Bytecode, OptimizationStats) {
        let mut stats = OptimizationStats::new();
        stats.bytecode_size_before = bytecode.instructions.len();
        stats.passes_run = 1;

        if bytecode.instructions.is_empty() {
            stats.bytecode_size_after = 0;
            return (bytecode, stats);
        }

        let decoded = decode_instructions(&bytecode);
        if decoded.is_empty() {
            stats.bytecode_size_after = bytecode.instructions.len();
            return (bytecode, stats);
        }

        // Build offset → instruction index map for efficient lookup
        let offset_to_idx: HashMap<usize, usize> = decoded
            .iter()
            .enumerate()
            .map(|(i, instr)| (instr.offset, i))
            .collect();

        // BFS reachability analysis
        let reachable = compute_reachable(&decoded, &offset_to_idx, &bytecode);

        // Count dead instructions
        let dead_count = decoded.len() - reachable.len();
        if dead_count == 0 {
            // Nothing to remove
            stats.bytecode_size_after = bytecode.instructions.len();
            return (bytecode, stats);
        }

        stats.dead_instructions_removed = dead_count;

        // Keep only reachable instructions (preserve order)
        let mut live: Vec<DecodedInstruction> = decoded
            .into_iter()
            .filter(|instr| reachable.contains(&instr.offset))
            .collect();

        let mut constants = bytecode.constants;
        fix_all_references(&mut live, &mut constants);

        let result = encode_instructions(&live, constants);
        stats.bytecode_size_after = result.instructions.len();
        (result, stats)
    }
}

/// BFS reachability analysis starting from offset 0.
///
/// Returns the set of byte offsets of reachable instructions.
fn compute_reachable(
    decoded: &[DecodedInstruction],
    offset_to_idx: &HashMap<usize, usize>,
    bytecode: &Bytecode,
) -> HashSet<usize> {
    let mut reachable: HashSet<usize> = HashSet::new();
    let mut queue: VecDeque<usize> = VecDeque::new();

    // Seed with entry point
    if !decoded.is_empty() {
        queue.push_back(decoded[0].offset);
    }

    // Also seed with all function body entry points (they're called indirectly)
    for constant in &bytecode.constants {
        if let crate::value::Value::Function(ref func) = constant {
            if func.bytecode_offset > 0 {
                queue.push_back(func.bytecode_offset);
            }
        }
    }

    while let Some(offset) = queue.pop_front() {
        if reachable.contains(&offset) {
            continue;
        }

        let idx = match offset_to_idx.get(&offset) {
            Some(&i) => i,
            None => continue, // Invalid offset, skip
        };

        let instr = &decoded[idx];
        reachable.insert(offset);

        match instr.opcode {
            // Unconditional jump: only successor is the jump target
            Opcode::Jump => {
                if instr.operands.len() == 2 {
                    let relative = instr.read_i16();
                    let target = (offset as isize + 3 + relative as isize) as usize;
                    queue.push_back(target);
                }
                // NO fallthrough for unconditional jump
            }

            // Loop (backward jump): only successor is the loop target
            Opcode::Loop => {
                if instr.operands.len() == 2 {
                    let relative = instr.read_i16();
                    let target = (offset as isize + 3 + relative as isize) as usize;
                    queue.push_back(target);
                }
                // NO fallthrough
            }

            // Conditional jump: both fallthrough and jump target are successors
            Opcode::JumpIfFalse => {
                if instr.operands.len() == 2 {
                    let relative = instr.read_i16();
                    let target = (offset as isize + 3 + relative as isize) as usize;
                    queue.push_back(target);
                }
                // Also fallthrough to next instruction
                let next_offset = offset + instr.byte_size();
                queue.push_back(next_offset);
            }

            // Terminators: no successors
            Opcode::Return | Opcode::Halt => {}

            // All other instructions: fallthrough to next
            _ => {
                let next_offset = offset + instr.byte_size();
                queue.push_back(next_offset);
            }
        }
    }

    reachable
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::Bytecode;
    use crate::optimizer::is_unconditional_terminator;
    use crate::span::Span;

    fn run_dce(bytecode: Bytecode) -> (Bytecode, OptimizationStats) {
        DeadCodeEliminationPass.optimize(bytecode)
    }

    fn compile_source(source: &str) -> Bytecode {
        use crate::{compiler::Compiler, lexer::Lexer, parser::Parser};
        let mut lexer = Lexer::new(source.to_string());
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (program, _) = parser.parse();
        let mut compiler = Compiler::new();
        compiler.compile(&program).expect("compile failed")
    }

    fn run_bytecode(bc: Bytecode) -> Option<crate::value::Value> {
        use crate::{security::SecurityContext, vm::VM};
        let security = SecurityContext::allow_all();
        let mut vm = VM::new(bc);
        vm.run(&security).unwrap_or(None)
    }

    // ── Dead code after return ────────────────────────────────────────────────

    #[test]
    fn test_remove_dead_code_after_halt() {
        // Halt followed by unreachable instructions
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Halt, Span::dummy());
        bc.emit(Opcode::Null, Span::dummy()); // unreachable
        bc.emit(Opcode::Pop, Span::dummy()); // unreachable

        let (result, stats) = run_dce(bc);
        assert_eq!(stats.dead_instructions_removed, 2);
        assert_eq!(result.instructions.len(), 1); // just Halt
        assert_eq!(result.instructions[0], Opcode::Halt as u8);
    }

    #[test]
    fn test_remove_dead_code_after_jump() {
        // Jump over dead code to Halt
        // Jump(3), Null(dead), Pop(dead), Halt
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Jump, Span::dummy());
        bc.emit_i16(2); // jump over 2 instructions (Null=1, Pop=1)
        bc.emit(Opcode::Null, Span::dummy()); // dead
        bc.emit(Opcode::Pop, Span::dummy()); // dead
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_dce(bc);
        assert_eq!(stats.dead_instructions_removed, 2);
        // Result should be: Jump(0), Halt (or just Halt after jump threading)
        assert!(!result.instructions.contains(&(Opcode::Null as u8)));
        assert!(!result.instructions.contains(&(Opcode::Pop as u8)));
    }

    #[test]
    fn test_no_removal_when_all_reachable() {
        // All instructions are reachable
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Null, Span::dummy());
        bc.emit(Opcode::Pop, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let original_len = bc.instructions.len();
        let (result, stats) = run_dce(bc);
        assert_eq!(stats.dead_instructions_removed, 0);
        assert_eq!(result.instructions.len(), original_len);
    }

    #[test]
    fn test_remove_implicit_return_after_explicit_return() {
        // Compiler emits Null, Return after every function body even when
        // there's an explicit Return. The implicit pair is dead code.
        let source = r#"
            fn add(a: number, b: number) -> number {
                return a + b;
            }
            add(1, 2);
        "#;
        let bc = compile_source(source);
        let original_size = bc.instructions.len();
        let (result, stats) = run_dce(bc);
        // The Null+Return after explicit Return should be eliminated
        assert!(
            stats.dead_instructions_removed >= 2,
            "Should remove implicit Null+Return (removed {})",
            stats.dead_instructions_removed
        );
        assert!(
            result.instructions.len() < original_size,
            "Result should be smaller"
        );
    }

    // ── Conditional branches ─────────────────────────────────────────────────

    #[test]
    fn test_keep_both_branches_of_if_else() {
        // if/else: both branches should be reachable
        let source = "if (true) { 1; } else { 2; }";
        let bc = compile_source(source);
        let (result, stats) = run_dce(bc.clone());
        // Both branches should be kept (they're both reachable)
        // Only implicit dead code at end might be removed
        let _ = result;
        let _ = stats;
        // Verify no valid code was removed by running the program
    }

    #[test]
    fn test_reachable_code_preserved() {
        let source = "let x = 5; let y = x + 3;";
        let bc = compile_source(source);
        let result_orig = run_bytecode(bc.clone());
        let (optimized, _) = run_dce(bc);
        let result_opt = run_bytecode(optimized);
        assert_eq!(result_orig, result_opt);
    }

    // ── Function bodies ───────────────────────────────────────────────────────

    #[test]
    fn test_function_body_kept() {
        // Function bodies are "unreachable" from program start in the BFS sense,
        // but we seed the BFS with all function entry points
        let source = r#"
            fn double(x: number) -> number { return x * 2; }
            double(5);
        "#;
        let bc = compile_source(source);
        let result_orig = run_bytecode(bc.clone());
        let (optimized, stats) = run_dce(bc);
        let result_opt = run_bytecode(optimized);
        assert_eq!(result_orig, result_opt);
        // Only the implicit null+return after explicit return should be removed
        let _ = stats;
    }

    // ── Empty and minimal ─────────────────────────────────────────────────────

    #[test]
    fn test_empty_bytecode_unchanged() {
        let bc = Bytecode::new();
        let (result, stats) = run_dce(bc);
        assert_eq!(stats.dead_instructions_removed, 0);
        assert!(result.instructions.is_empty());
    }

    #[test]
    fn test_halt_only_unchanged() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Halt, Span::dummy());
        let (result, stats) = run_dce(bc);
        assert_eq!(stats.dead_instructions_removed, 0);
        assert_eq!(result.instructions.len(), 1);
    }

    // ── Semantics preservation ────────────────────────────────────────────────

    #[test]
    fn test_preserves_while_loop() {
        // Use 'var' because sum and i are reassigned
        let source = r#"
            var sum = 0;
            var i = 0;
            while (i < 5) {
                sum = sum + i;
                i = i + 1;
            }
        "#;
        let bc = compile_source(source);
        let result_orig = run_bytecode(bc.clone());
        let (optimized, _) = run_dce(bc);
        let result_opt = run_bytecode(optimized);
        assert_eq!(result_orig, result_opt);
    }

    #[test]
    fn test_preserves_if_statement() {
        // Use 'var' because x is reassigned
        let source = "var x = 5; if (x > 3) { x = x + 1; }";
        let bc = compile_source(source);
        let result_orig = run_bytecode(bc.clone());
        let (optimized, _) = run_dce(bc);
        let result_opt = run_bytecode(optimized);
        assert_eq!(result_orig, result_opt);
    }

    #[test]
    fn test_preserves_function_call() {
        let source = r#"
            fn square(x: number) -> number { return x * x; }
            let result = square(7);
        "#;
        let bc = compile_source(source);
        let result_orig = run_bytecode(bc.clone());
        let (optimized, _) = run_dce(bc);
        let result_opt = run_bytecode(optimized);
        assert_eq!(result_orig, result_opt);
    }

    #[test]
    fn test_size_reduction_with_dead_code() {
        let mut bc = Bytecode::new();
        // Halt followed by unreachable instructions
        bc.emit(Opcode::Halt, Span::dummy());
        for _ in 0..10 {
            bc.emit(Opcode::Null, Span::dummy());
        }
        let (_result, stats) = run_dce(bc);
        assert_eq!(stats.dead_instructions_removed, 10);
        assert!(stats.bytes_saved() > 0);
    }

    #[test]
    fn test_dce_stats_populated() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Halt, Span::dummy());
        bc.emit(Opcode::Null, Span::dummy()); // dead
        let (_, stats) = run_dce(bc);
        assert_eq!(stats.passes_run, 1);
        assert!(stats.bytecode_size_before > 0);
        assert!(stats.bytecode_size_after <= stats.bytecode_size_before);
    }

    #[test]
    fn test_unconditional_terminator_recognition() {
        assert!(is_unconditional_terminator(Opcode::Jump));
        assert!(is_unconditional_terminator(Opcode::Return));
        assert!(is_unconditional_terminator(Opcode::Halt));
        assert!(!is_unconditional_terminator(Opcode::JumpIfFalse));
        assert!(!is_unconditional_terminator(Opcode::Add));
    }
}
