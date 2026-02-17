//! Constant folding optimization pass
//!
//! Evaluates constant expressions at compile time:
//! - Binary arithmetic: `Constant(a), Constant(b), Op` → `Constant(a op b)`
//! - Unary negation: `Constant(n), Negate` → `Constant(-n)`
//! - Boolean not: `Constant(bool), Not` → `True`/`False`
//! - Literal not: `True/False, Not` → `False/True`
//!
//! Multiple passes are run until the bytecode stabilizes.

use super::{
    decode_instructions, encode_instructions, fix_all_references, DecodedInstruction,
    OptimizationPass, OptimizationStats,
};
use crate::bytecode::{Bytecode, Opcode};
use crate::value::Value;

/// Constant folding optimization pass
///
/// Repeatedly scans the instruction stream looking for constant expressions
/// and folds them into a single constant.  Runs until stable (fixed point).
pub struct ConstantFoldingPass;

impl OptimizationPass for ConstantFoldingPass {
    fn name(&self) -> &str {
        "constant-folding"
    }

    fn optimize(&self, bytecode: Bytecode) -> (Bytecode, OptimizationStats) {
        let mut stats = OptimizationStats::new();
        stats.bytecode_size_before = bytecode.instructions.len();
        stats.passes_run = 1;

        let mut constants = bytecode.constants.clone();
        let mut decoded = decode_instructions(&bytecode);

        let mut changed = true;
        while changed {
            changed = false;
            let mut new_decoded: Vec<DecodedInstruction> = Vec::with_capacity(decoded.len());
            let mut i = 0;

            while i < decoded.len() {
                // ── Pattern: Constant(a), Constant(b), BinaryOp ───────────────
                if i + 2 < decoded.len()
                    && decoded[i].opcode == Opcode::Constant
                    && decoded[i + 1].opcode == Opcode::Constant
                {
                    let a_idx = decoded[i].read_u16() as usize;
                    let b_idx = decoded[i + 1].read_u16() as usize;
                    let op = decoded[i + 2].opcode;

                    if a_idx < constants.len() && b_idx < constants.len() && is_foldable_binary(op)
                    {
                        if let Some(result) = fold_binary(&constants[a_idx], &constants[b_idx], op)
                        {
                            let new_idx = constants.len() as u16;
                            constants.push(result);
                            let span = decoded[i].span.or(decoded[i + 2].span);
                            new_decoded.push(DecodedInstruction {
                                offset: decoded[i].offset,
                                opcode: Opcode::Constant,
                                operands: DecodedInstruction::make_u16_operands(new_idx),
                                span,
                            });
                            i += 3;
                            stats.constants_folded += 1;
                            changed = true;
                            continue;
                        }
                    }
                }

                // ── Pattern: Constant(n), Negate ──────────────────────────────
                if i + 1 < decoded.len()
                    && decoded[i].opcode == Opcode::Constant
                    && decoded[i + 1].opcode == Opcode::Negate
                {
                    let a_idx = decoded[i].read_u16() as usize;
                    if a_idx < constants.len() {
                        if let Value::Number(n) = constants[a_idx] {
                            let new_idx = constants.len() as u16;
                            constants.push(Value::Number(-n));
                            let span = decoded[i].span;
                            new_decoded.push(DecodedInstruction {
                                offset: decoded[i].offset,
                                opcode: Opcode::Constant,
                                operands: DecodedInstruction::make_u16_operands(new_idx),
                                span,
                            });
                            i += 2;
                            stats.constants_folded += 1;
                            changed = true;
                            continue;
                        }
                    }
                }

                // ── Pattern: Constant(bool), Not ──────────────────────────────
                if i + 1 < decoded.len()
                    && decoded[i].opcode == Opcode::Constant
                    && decoded[i + 1].opcode == Opcode::Not
                {
                    let a_idx = decoded[i].read_u16() as usize;
                    if a_idx < constants.len() {
                        if let Value::Bool(b) = constants[a_idx] {
                            let new_opcode = if b { Opcode::False } else { Opcode::True };
                            let span = decoded[i].span;
                            new_decoded.push(DecodedInstruction {
                                offset: decoded[i].offset,
                                opcode: new_opcode,
                                operands: Vec::new(),
                                span,
                            });
                            i += 2;
                            stats.constants_folded += 1;
                            changed = true;
                            continue;
                        }
                    }
                }

                // ── Pattern: True, Not → False ────────────────────────────────
                if i + 1 < decoded.len()
                    && decoded[i].opcode == Opcode::True
                    && decoded[i + 1].opcode == Opcode::Not
                {
                    let span = decoded[i].span;
                    new_decoded.push(DecodedInstruction {
                        offset: decoded[i].offset,
                        opcode: Opcode::False,
                        operands: Vec::new(),
                        span,
                    });
                    i += 2;
                    stats.constants_folded += 1;
                    changed = true;
                    continue;
                }

                // ── Pattern: False, Not → True ────────────────────────────────
                if i + 1 < decoded.len()
                    && decoded[i].opcode == Opcode::False
                    && decoded[i + 1].opcode == Opcode::Not
                {
                    let span = decoded[i].span;
                    new_decoded.push(DecodedInstruction {
                        offset: decoded[i].offset,
                        opcode: Opcode::True,
                        operands: Vec::new(),
                        span,
                    });
                    i += 2;
                    stats.constants_folded += 1;
                    changed = true;
                    continue;
                }

                // ── Pattern: Null, Not ────────────────────────────────────────
                // null is falsy, so !null = true
                if i + 1 < decoded.len()
                    && decoded[i].opcode == Opcode::Null
                    && decoded[i + 1].opcode == Opcode::Not
                {
                    let span = decoded[i].span;
                    new_decoded.push(DecodedInstruction {
                        offset: decoded[i].offset,
                        opcode: Opcode::True,
                        operands: Vec::new(),
                        span,
                    });
                    i += 2;
                    stats.constants_folded += 1;
                    changed = true;
                    continue;
                }

                // No pattern matched — keep instruction as-is
                new_decoded.push(decoded[i].clone());
                i += 1;
            }

            decoded = new_decoded;
        }

        // Fix jump targets and function offsets after structural changes
        fix_all_references(&mut decoded, &mut constants);

        let result = encode_instructions(&decoded, constants);
        stats.bytecode_size_after = result.instructions.len();
        (result, stats)
    }
}

/// Returns true if `op` is a binary opcode that constant folding can evaluate
fn is_foldable_binary(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::Add
            | Opcode::Sub
            | Opcode::Mul
            | Opcode::Div
            | Opcode::Mod
            | Opcode::Equal
            | Opcode::NotEqual
            | Opcode::Less
            | Opcode::LessEqual
            | Opcode::Greater
            | Opcode::GreaterEqual
    )
}

/// Attempt to fold a binary operation on two constant values.
/// Returns `None` if the operation is not supported, or would produce a
/// runtime error (e.g., division by zero).
fn fold_binary(a: &Value, b: &Value, op: Opcode) -> Option<Value> {
    match (a, b) {
        (Value::Number(an), Value::Number(bn)) => match op {
            Opcode::Add => Some(Value::Number(an + bn)),
            Opcode::Sub => Some(Value::Number(an - bn)),
            Opcode::Mul => Some(Value::Number(an * bn)),
            Opcode::Div => {
                // Preserve runtime semantics: don't fold division by zero
                if *bn == 0.0 {
                    None
                } else {
                    Some(Value::Number(an / bn))
                }
            }
            Opcode::Mod => {
                if *bn == 0.0 {
                    None
                } else {
                    Some(Value::Number(an % bn))
                }
            }
            Opcode::Equal => Some(Value::Bool((an - bn).abs() < f64::EPSILON)),
            Opcode::NotEqual => Some(Value::Bool((an - bn).abs() >= f64::EPSILON)),
            Opcode::Less => Some(Value::Bool(an < bn)),
            Opcode::LessEqual => Some(Value::Bool(an <= bn)),
            Opcode::Greater => Some(Value::Bool(an > bn)),
            Opcode::GreaterEqual => Some(Value::Bool(an >= bn)),
            _ => None,
        },
        (Value::Bool(ab), Value::Bool(bb)) => match op {
            Opcode::Equal => Some(Value::Bool(ab == bb)),
            Opcode::NotEqual => Some(Value::Bool(ab != bb)),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::Bytecode;
    use crate::span::Span;

    fn run_cf(bytecode: Bytecode) -> (Bytecode, OptimizationStats) {
        ConstantFoldingPass.optimize(bytecode)
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

    /// Compile source and apply constant folding
    fn cf_source(source: &str) -> (Bytecode, OptimizationStats) {
        let bc = compile_source(source);
        run_cf(bc)
    }

    // ── Arithmetic folding ────────────────────────────────────────────────────

    #[test]
    fn test_fold_add() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(2.0));
        let b = bc.add_constant(Value::Number(3.0));
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(a);
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(b);
        bc.emit(Opcode::Add, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_cf(bc);
        // 2+3=5 should be folded to Constant(5), Halt
        assert_eq!(stats.constants_folded, 1);
        assert!(result.instructions.len() < 8); // shorter than original
                                                // The last constant in pool should be 5.0
        assert_eq!(*result.constants.last().unwrap(), Value::Number(5.0));
    }

    #[test]
    fn test_fold_sub() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(10.0));
        let b = bc.add_constant(Value::Number(3.0));
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(a);
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(b);
        bc.emit(Opcode::Sub, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 1);
        assert_eq!(*result.constants.last().unwrap(), Value::Number(7.0));
    }

    #[test]
    fn test_fold_mul() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(6.0));
        let b = bc.add_constant(Value::Number(7.0));
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(a);
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(b);
        bc.emit(Opcode::Mul, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 1);
        assert_eq!(*result.constants.last().unwrap(), Value::Number(42.0));
    }

    #[test]
    fn test_fold_div() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(15.0));
        let b = bc.add_constant(Value::Number(3.0));
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(a);
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(b);
        bc.emit(Opcode::Div, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 1);
        assert_eq!(*result.constants.last().unwrap(), Value::Number(5.0));
    }

    #[test]
    fn test_fold_mod() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(10.0));
        let b = bc.add_constant(Value::Number(3.0));
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(a);
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(b);
        bc.emit(Opcode::Mod, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 1);
        assert_eq!(*result.constants.last().unwrap(), Value::Number(1.0));
    }

    #[test]
    fn test_no_fold_div_by_zero() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(10.0));
        let b = bc.add_constant(Value::Number(0.0));
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(a);
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(b);
        bc.emit(Opcode::Div, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let original_len = bc.instructions.len();
        let (result, stats) = run_cf(bc);
        // Division by zero should NOT be folded
        assert_eq!(stats.constants_folded, 0);
        assert_eq!(result.instructions.len(), original_len);
    }

    #[test]
    fn test_no_fold_mod_by_zero() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(10.0));
        let b = bc.add_constant(Value::Number(0.0));
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(a);
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(b);
        bc.emit(Opcode::Mod, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 0);
        assert_eq!(result.instructions.len(), 8);
    }

    // ── Comparison folding ────────────────────────────────────────────────────

    #[test]
    fn test_fold_equal_true() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(5.0));
        let b = bc.add_constant(Value::Number(5.0));
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(a);
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(b);
        bc.emit(Opcode::Equal, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 1);
        assert_eq!(*result.constants.last().unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_fold_equal_false() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(5.0));
        let b = bc.add_constant(Value::Number(6.0));
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(a);
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(b);
        bc.emit(Opcode::Equal, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 1);
        assert_eq!(*result.constants.last().unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_fold_less_than() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(3.0));
        let b = bc.add_constant(Value::Number(5.0));
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(a);
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(b);
        bc.emit(Opcode::Less, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 1);
        assert_eq!(*result.constants.last().unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_fold_greater_equal() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(5.0));
        let b = bc.add_constant(Value::Number(3.0));
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(a);
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(b);
        bc.emit(Opcode::GreaterEqual, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 1);
        assert_eq!(*result.constants.last().unwrap(), Value::Bool(true));
    }

    // ── Unary folding ─────────────────────────────────────────────────────────

    #[test]
    fn test_fold_negate() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(42.0));
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(a);
        bc.emit(Opcode::Negate, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 1);
        assert_eq!(*result.constants.last().unwrap(), Value::Number(-42.0));
    }

    #[test]
    fn test_fold_negate_negative() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(-7.0));
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(a);
        bc.emit(Opcode::Negate, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 1);
        assert_eq!(*result.constants.last().unwrap(), Value::Number(7.0));
    }

    #[test]
    fn test_fold_bool_not_true() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::True, Span::dummy());
        bc.emit(Opcode::Not, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 1);
        assert!(result.instructions.contains(&(Opcode::False as u8)));
        assert!(!result.instructions.contains(&(Opcode::True as u8)));
    }

    #[test]
    fn test_fold_bool_not_false() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::False, Span::dummy());
        bc.emit(Opcode::Not, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 1);
        assert!(result.instructions.contains(&(Opcode::True as u8)));
        assert!(!result.instructions.contains(&(Opcode::False as u8)));
    }

    #[test]
    fn test_fold_null_not() {
        // !null = true
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Null, Span::dummy());
        bc.emit(Opcode::Not, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 1);
        assert!(result.instructions.contains(&(Opcode::True as u8)));
    }

    // ── Nested / chained folding ──────────────────────────────────────────────

    #[test]
    fn test_fold_nested_expression() {
        // (2 + 3) * 4 → 20
        let (result, stats) = cf_source("(2 + 3) * 4;");
        assert!(stats.constants_folded >= 2, "Should fold both operations");
        // The folded result (20.0) should be in the constant pool
        let has_20 = result
            .constants
            .iter()
            .any(|c| matches!(c, Value::Number(n) if (n - 20.0).abs() < f64::EPSILON));
        assert!(has_20, "Should have folded constant 20.0");
    }

    #[test]
    fn test_fold_deeply_nested() {
        // ((1 + 2) + (3 + 4)) + 5 → 15
        let (result, stats) = cf_source("((1 + 2) + (3 + 4)) + 5;");
        assert!(stats.constants_folded >= 3);
        let has_15 = result
            .constants
            .iter()
            .any(|c| matches!(c, Value::Number(n) if (n - 15.0).abs() < f64::EPSILON));
        assert!(has_15, "Should have folded to 15.0");
    }

    #[test]
    fn test_fold_size_reduction() {
        // 2 + 3 generates 7 bytes unoptimized, should become 3 bytes
        let (_result, stats) = cf_source("2 + 3;");
        // Size should be smaller after folding
        assert!(
            stats.bytecode_size_after <= stats.bytecode_size_before,
            "Optimized bytecode should be smaller"
        );
        // Folded result should be present
        assert!(stats.constants_folded > 0);
    }

    // ── Non-constant operands (should NOT fold) ───────────────────────────────

    #[test]
    fn test_no_fold_variable_add() {
        // let x = 5; x + 3 — cannot fold x+3 since x is not constant at compile time
        let (result, _stats) = cf_source("let x = 5; x + 3;");
        // The GetGlobal should still be present
        assert!(result.instructions.contains(&(Opcode::GetGlobal as u8)));
    }

    #[test]
    fn test_no_fold_string_operations() {
        // String concatenation is not foldable by this pass
        let (result, stats) = cf_source("\"hello\" + \" world\";");
        // No folding of string operations
        assert_eq!(stats.constants_folded, 0);
        let _ = result;
    }

    // ── Edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn test_empty_bytecode_unchanged() {
        let bc = Bytecode::new();
        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 0);
        assert!(result.instructions.is_empty());
    }

    #[test]
    fn test_halt_only_unchanged() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Halt, Span::dummy());
        let (result, stats) = run_cf(bc);
        assert_eq!(stats.constants_folded, 0);
        assert_eq!(result.instructions.len(), 1);
    }

    #[test]
    fn test_fold_preserves_semantics() {
        // Compile and run both versions, they should produce the same result
        let source = "2 + 3;";
        let bc = compile_source(source);
        let (optimized, stats) = run_cf(bc.clone());
        assert!(stats.constants_folded > 0);
        let result_orig = run_bytecode(bc);
        let result_opt = run_bytecode(optimized);
        assert_eq!(result_orig, result_opt);
    }

    #[test]
    fn test_fold_arithmetic_chain_preserves_semantics() {
        let source = "10 * 2 + 5 - 3;";
        let bc = compile_source(source);
        let (optimized, stats) = run_cf(bc.clone());
        assert!(stats.constants_folded > 0);
        let result_orig = run_bytecode(bc);
        let result_opt = run_bytecode(optimized);
        assert_eq!(result_orig, result_opt);
    }

    #[test]
    fn test_fold_comparison_preserves_semantics() {
        let source = "let x = 5 > 3;";
        let bc = compile_source(source);
        let (optimized, stats) = run_cf(bc.clone());
        assert!(stats.constants_folded > 0);
        let result_orig = run_bytecode(bc);
        let result_opt = run_bytecode(optimized);
        assert_eq!(result_orig, result_opt);
    }
}
