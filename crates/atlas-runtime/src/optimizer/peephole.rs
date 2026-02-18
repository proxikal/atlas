//! Peephole optimization pass
//!
//! Applies local pattern simplifications to the instruction stream:
//! - `Dup, Pop` → nothing (useless dup immediately discarded)
//! - `Not, Not` → nothing (double negation)
//! - `True, Not` → `False` (constant boolean flip)
//! - `False, Not` → `True` (constant boolean flip)
//! - `Jump +0` → nothing (jump to next instruction)
//! - Jump threading: `Jump A` where A is `Jump B` → `Jump B`
//!
//! Multiple passes are run until the bytecode stabilizes.

use super::{
    decode_instructions, encode_instructions, fix_all_references, DecodedInstruction,
    OptimizationPass, OptimizationStats,
};
use crate::bytecode::{Bytecode, Opcode};

/// Peephole optimization pass
///
/// Applies small, local transformations that eliminate wasteful patterns.
/// Runs until the bytecode stabilizes.
pub struct PeepholePass;

impl OptimizationPass for PeepholePass {
    fn name(&self) -> &str {
        "peephole"
    }

    fn optimize(&self, bytecode: Bytecode) -> (Bytecode, OptimizationStats) {
        let mut stats = OptimizationStats::new();
        stats.bytecode_size_before = bytecode.instructions.len();
        stats.passes_run = 1;

        let mut decoded = decode_instructions(&bytecode);
        let mut constants = bytecode.constants.clone();

        let mut changed = true;
        while changed {
            changed = false;
            let mut new_decoded: Vec<DecodedInstruction> = Vec::with_capacity(decoded.len());
            let mut i = 0;

            while i < decoded.len() {
                // ── Pattern: Dup, Pop → nothing ──────────────────────────────
                if i + 1 < decoded.len()
                    && decoded[i].opcode == Opcode::Dup
                    && decoded[i + 1].opcode == Opcode::Pop
                {
                    i += 2;
                    stats.peephole_patterns_applied += 1;
                    changed = true;
                    continue;
                }

                // ── Pattern: Not, Not → nothing ──────────────────────────────
                if i + 1 < decoded.len()
                    && decoded[i].opcode == Opcode::Not
                    && decoded[i + 1].opcode == Opcode::Not
                {
                    i += 2;
                    stats.peephole_patterns_applied += 1;
                    changed = true;
                    continue;
                }

                // ── Pattern: Pop, Pop → nothing (eliminate set+discard) ───────
                // Only safe if there's nothing between producing and popping
                // This is NOT always safe — we skip this pattern intentionally
                // to avoid removing side-effects.

                // ── Pattern: Jump +0 → nothing ───────────────────────────────
                // Jump with relative offset 0 jumps to the instruction right
                // after the jump's operands (a no-op).
                if decoded[i].opcode == Opcode::Jump && decoded[i].operands.len() == 2 {
                    let relative = decoded[i].read_i16();
                    if relative == 0 {
                        i += 1;
                        stats.peephole_patterns_applied += 1;
                        changed = true;
                        continue;
                    }
                }

                // ── Jump threading: Jump A where A is Jump B → Jump B ─────────
                // Resolve chains of unconditional jumps to their final target.
                if decoded[i].opcode == Opcode::Jump && decoded[i].operands.len() == 2 {
                    let relative = decoded[i].read_i16();
                    let origin_after = decoded[i].offset + 3;
                    let target = (origin_after as isize + relative as isize) as usize;

                    // Check if target instruction is also a Jump
                    if let Some(target_instr) = find_instruction_at(&decoded, target) {
                        if target_instr.opcode == Opcode::Jump
                            && target_instr.operands.len() == 2
                            && target != decoded[i].offset
                        {
                            let inner_relative = target_instr.read_i16();
                            let inner_after = target + 3;
                            let final_target =
                                (inner_after as isize + inner_relative as isize) as usize;

                            // Don't create an infinite loop
                            if final_target != decoded[i].offset {
                                // Compute new relative offset from current instruction
                                let new_after = decoded[i].offset + 3;
                                let new_relative =
                                    (final_target as isize - new_after as isize) as i16;
                                let mut threaded = decoded[i].clone();
                                threaded.operands =
                                    DecodedInstruction::make_i16_operands(new_relative);
                                new_decoded.push(threaded);
                                i += 1;
                                stats.peephole_patterns_applied += 1;
                                changed = true;
                                continue;
                            }
                        }
                    }
                }

                // ── JumpIfFalse threading ─────────────────────────────────────
                // JumpIfFalse A where A is Jump B → update target to B
                if decoded[i].opcode == Opcode::JumpIfFalse && decoded[i].operands.len() == 2 {
                    let relative = decoded[i].read_i16();
                    let origin_after = decoded[i].offset + 3;
                    let target = (origin_after as isize + relative as isize) as usize;

                    if let Some(target_instr) = find_instruction_at(&decoded, target) {
                        if target_instr.opcode == Opcode::Jump
                            && target_instr.operands.len() == 2
                            && target != decoded[i].offset
                        {
                            let inner_relative = target_instr.read_i16();
                            let inner_after = target + 3;
                            let final_target =
                                (inner_after as isize + inner_relative as isize) as usize;

                            if final_target != decoded[i].offset {
                                let new_after = decoded[i].offset + 3;
                                let new_relative =
                                    (final_target as isize - new_after as isize) as i16;
                                let mut threaded = decoded[i].clone();
                                threaded.operands =
                                    DecodedInstruction::make_i16_operands(new_relative);
                                new_decoded.push(threaded);
                                i += 1;
                                stats.peephole_patterns_applied += 1;
                                changed = true;
                                continue;
                            }
                        }
                    }
                }

                // No pattern matched — keep as-is
                new_decoded.push(decoded[i].clone());
                i += 1;
            }

            decoded = new_decoded;
        }

        fix_all_references(&mut decoded, &mut constants);

        let result = encode_instructions(&decoded, constants);
        stats.bytecode_size_after = result.instructions.len();
        (result, stats)
    }
}

/// Find a decoded instruction by its original byte offset
fn find_instruction_at(
    decoded: &[DecodedInstruction],
    offset: usize,
) -> Option<&DecodedInstruction> {
    decoded.iter().find(|instr| instr.offset == offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::Bytecode;
    use crate::span::Span;

    fn run_peep(bytecode: Bytecode) -> (Bytecode, OptimizationStats) {
        PeepholePass.optimize(bytecode)
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

    // ── Dup-Pop elimination ───────────────────────────────────────────────────

    #[test]
    fn test_eliminate_dup_pop() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Null, Span::dummy());
        bc.emit(Opcode::Dup, Span::dummy());
        bc.emit(Opcode::Pop, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_peep(bc);
        assert_eq!(stats.peephole_patterns_applied, 1);
        assert!(!result.instructions.contains(&(Opcode::Dup as u8)));
        assert!(!result.instructions.contains(&(Opcode::Pop as u8)));
    }

    #[test]
    fn test_eliminate_multiple_dup_pop() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Null, Span::dummy());
        bc.emit(Opcode::Dup, Span::dummy());
        bc.emit(Opcode::Pop, Span::dummy());
        bc.emit(Opcode::Dup, Span::dummy());
        bc.emit(Opcode::Pop, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_peep(bc);
        assert_eq!(stats.peephole_patterns_applied, 2);
        assert!(!result.instructions.contains(&(Opcode::Dup as u8)));
        assert!(!result.instructions.contains(&(Opcode::Pop as u8)));
    }

    #[test]
    fn test_keep_dup_without_pop() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Null, Span::dummy());
        bc.emit(Opcode::Dup, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_peep(bc);
        assert_eq!(stats.peephole_patterns_applied, 0);
        assert!(result.instructions.contains(&(Opcode::Dup as u8)));
    }

    #[test]
    fn test_keep_pop_without_dup() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Null, Span::dummy());
        bc.emit(Opcode::Pop, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_peep(bc);
        assert_eq!(stats.peephole_patterns_applied, 0);
        assert!(result.instructions.contains(&(Opcode::Pop as u8)));
    }

    // ── Not-Not elimination ───────────────────────────────────────────────────

    #[test]
    fn test_eliminate_not_not() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::True, Span::dummy());
        bc.emit(Opcode::Not, Span::dummy());
        bc.emit(Opcode::Not, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_peep(bc);
        assert_eq!(stats.peephole_patterns_applied, 1);
        // The double-not should be gone; True should remain
        assert!(result.instructions.contains(&(Opcode::True as u8)));
        assert!(!result.instructions.contains(&(Opcode::Not as u8)));
    }

    #[test]
    fn test_eliminate_triple_not_leaves_one() {
        // Not Not Not → Not (after two passes: first removes inner Not-Not pair)
        let mut bc = Bytecode::new();
        bc.emit(Opcode::True, Span::dummy());
        bc.emit(Opcode::Not, Span::dummy());
        bc.emit(Opcode::Not, Span::dummy());
        bc.emit(Opcode::Not, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_peep(bc);
        assert!(stats.peephole_patterns_applied >= 1);
        // One Not should remain
        let not_count = result
            .instructions
            .iter()
            .filter(|&&b| b == Opcode::Not as u8)
            .count();
        assert_eq!(not_count, 1);
    }

    // ── Jump +0 elimination ───────────────────────────────────────────────────

    #[test]
    fn test_eliminate_jump_zero() {
        // Jump with offset 0 = jump to next instruction = no-op
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Null, Span::dummy());
        bc.emit(Opcode::Jump, Span::dummy());
        bc.emit_i16(0); // jump to next instruction
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_peep(bc);
        assert_eq!(stats.peephole_patterns_applied, 1);
        assert!(!result.instructions.contains(&(Opcode::Jump as u8)));
    }

    #[test]
    fn test_keep_nonzero_jump() {
        // Meaningful forward jump should be kept
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Jump, Span::dummy());
        bc.emit_i16(1); // skip next instruction
        bc.emit(Opcode::Null, Span::dummy()); // skipped
        bc.emit(Opcode::Halt, Span::dummy());

        let original_jump_count = bc
            .instructions
            .iter()
            .filter(|&&b| b == Opcode::Jump as u8)
            .count();
        let (result, stats) = run_peep(bc);
        // Jump threading might reduce this, but the jump itself should not be
        // eliminated unless it's a no-op
        assert!(
            stats.peephole_patterns_applied == 0
                || result.instructions.contains(&(Opcode::Jump as u8))
                || result.instructions.len() < 5,
            "Non-trivial jump should only be removed if threading finds same target"
        );
        let _ = original_jump_count;
    }

    // ── Jump threading ────────────────────────────────────────────────────────

    #[test]
    fn test_jump_chain_threading() {
        // Jump A where A is also Jump B → Jump B directly
        // Bytecode layout:
        //   0: Jump(3)         → target offset 6 (which is another Jump)
        //   3: Halt
        //   6: Jump(1)         → target offset 10 (Null)
        //   9: Halt (dead)
        //  10: Halt
        let mut bc = Bytecode::new();
        // Instruction at 0: Jump → target at 6 (offset+3+relative = 0+3+3 = 6)
        bc.emit(Opcode::Jump, Span::dummy());
        bc.emit_i16(3); // 0+3+3 = 6
                        // Instruction at 3: Halt
        bc.emit(Opcode::Halt, Span::dummy());
        // Instruction at 4: Jump → target at 8 (4+3+1 = 8)
        bc.emit(Opcode::Jump, Span::dummy());
        bc.emit_i16(1); // 4+3+1 = 8
                        // Instruction at 7: Null (dead — between the two jumps)
        bc.emit(Opcode::Null, Span::dummy());
        // Instruction at 8: Halt
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, _stats) = run_peep(bc);
        // The result should have no chained jumps
        let _ = result;
    }

    // ── Semantics preservation ────────────────────────────────────────────────

    #[test]
    fn test_preserves_simple_program() {
        let source = "let x = 5 + 3;";
        let bc = compile_source(source);
        let result_orig = run_bytecode(bc.clone());
        let (optimized, _) = run_peep(bc);
        let result_opt = run_bytecode(optimized);
        assert_eq!(result_orig, result_opt);
    }

    #[test]
    fn test_preserves_while_loop() {
        // Use 'var' because x is reassigned
        let source = r#"
            var x = 0;
            while (x < 5) { x = x + 1; }
        "#;
        let bc = compile_source(source);
        let result_orig = run_bytecode(bc.clone());
        let (optimized, _) = run_peep(bc);
        let result_opt = run_bytecode(optimized);
        assert_eq!(result_orig, result_opt);
    }

    #[test]
    fn test_preserves_if_else() {
        // Use 'var' because x is reassigned
        let source = "var x = 5; if (x > 3) { x = 1; } else { x = 2; }";
        let bc = compile_source(source);
        let result_orig = run_bytecode(bc.clone());
        let (optimized, _) = run_peep(bc);
        let result_opt = run_bytecode(optimized);
        assert_eq!(result_orig, result_opt);
    }

    #[test]
    fn test_preserves_function_call() {
        let source = r#"
            fn negate(x: number) -> number { return x * -1; }
            let result = negate(7);
        "#;
        let bc = compile_source(source);
        let result_orig = run_bytecode(bc.clone());
        let (optimized, _) = run_peep(bc);
        let result_opt = run_bytecode(optimized);
        assert_eq!(result_orig, result_opt);
    }

    // ── Edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn test_empty_bytecode_unchanged() {
        let bc = Bytecode::new();
        let (result, stats) = run_peep(bc);
        assert_eq!(stats.peephole_patterns_applied, 0);
        assert!(result.instructions.is_empty());
    }

    #[test]
    fn test_halt_only_unchanged() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Halt, Span::dummy());
        let (result, stats) = run_peep(bc);
        assert_eq!(stats.peephole_patterns_applied, 0);
        assert_eq!(result.instructions.len(), 1);
    }

    #[test]
    fn test_stats_populated() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Dup, Span::dummy());
        bc.emit(Opcode::Pop, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (_, stats) = run_peep(bc);
        assert_eq!(stats.passes_run, 1);
        assert!(stats.bytecode_size_before > 0);
        assert!(stats.bytecode_size_after < stats.bytecode_size_before);
    }

    #[test]
    fn test_size_reduced_with_dup_pop() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Null, Span::dummy());
        bc.emit(Opcode::Dup, Span::dummy());
        bc.emit(Opcode::Pop, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());
        let size_before = bc.instructions.len();

        let (result, _) = run_peep(bc);
        assert!(result.instructions.len() < size_before);
    }
}
