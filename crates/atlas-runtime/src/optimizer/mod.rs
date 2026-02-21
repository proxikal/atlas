//! Bytecode optimizer
//!
//! Provides three optimization passes:
//! - **Constant folding** — evaluate constant expressions at compile time
//! - **Dead code elimination** — remove unreachable instructions after returns/jumps
//! - **Peephole optimization** — local pattern simplifications (dup-pop, not-not, etc.)
//!
//! # Usage
//!
//! ```
//! use atlas_runtime::optimizer::Optimizer;
//! use atlas_runtime::compiler::Compiler;
//!
//! let mut compiler = Compiler::with_optimization();
//! // optimizer is automatically applied after compilation
//! ```

pub mod constant_folding;
pub mod dead_code;
pub mod peephole;

pub use constant_folding::ConstantFoldingPass;
pub use dead_code::DeadCodeEliminationPass;
pub use peephole::PeepholePass;

use crate::bytecode::{Bytecode, DebugSpan, Opcode};
use crate::span::Span;
use crate::value::Value;

// ============================================================================
// Public API: OptimizationStats
// ============================================================================

/// Statistics collected during an optimization run
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OptimizationStats {
    /// Number of constant expressions folded to a single constant
    pub constants_folded: usize,
    /// Number of unreachable instructions removed
    pub dead_instructions_removed: usize,
    /// Number of peephole pattern matches applied
    pub peephole_patterns_applied: usize,
    /// Bytecode size (instruction bytes) before optimization
    pub bytecode_size_before: usize,
    /// Bytecode size (instruction bytes) after optimization
    pub bytecode_size_after: usize,
    /// Number of optimization passes executed
    pub passes_run: usize,
}

impl OptimizationStats {
    /// Create a new zero-initialized stats object
    pub fn new() -> Self {
        Default::default()
    }

    /// Net bytes saved (positive = smaller, negative = larger after optimization)
    pub fn bytes_saved(&self) -> isize {
        self.bytecode_size_before as isize - self.bytecode_size_after as isize
    }

    /// Percentage size reduction (0–100)
    pub fn size_reduction_percent(&self) -> f64 {
        if self.bytecode_size_before == 0 {
            return 0.0;
        }
        (self.bytes_saved() as f64 / self.bytecode_size_before as f64) * 100.0
    }

    /// Total number of optimizations applied across all passes
    pub fn total_optimizations(&self) -> usize {
        self.constants_folded + self.dead_instructions_removed + self.peephole_patterns_applied
    }

    /// Merge another stats object into this one (sum all counts)
    pub fn merge(&mut self, other: &OptimizationStats) {
        self.constants_folded += other.constants_folded;
        self.dead_instructions_removed += other.dead_instructions_removed;
        self.peephole_patterns_applied += other.peephole_patterns_applied;
        self.passes_run += other.passes_run;
    }
}

// ============================================================================
// Public API: OptimizationPass trait
// ============================================================================

/// A single optimization pass
///
/// Each pass receives bytecode, may transform it, and returns statistics
/// describing what changed.
pub trait OptimizationPass: Send + Sync {
    /// Human-readable name of this pass (for diagnostics)
    fn name(&self) -> &str;

    /// Apply the optimization to bytecode.
    ///
    /// Returns the (possibly transformed) bytecode and statistics.
    /// Implementations MUST preserve program semantics.
    fn optimize(&self, bytecode: Bytecode) -> (Bytecode, OptimizationStats);
}

// ============================================================================
// Public API: Optimizer
// ============================================================================

/// Bytecode optimizer — manages and runs optimization passes
///
/// Runs registered passes repeatedly until the bytecode stabilizes or the
/// iteration limit is reached.
pub struct Optimizer {
    /// Whether optimization is enabled (disabled = pass-through)
    enabled: bool,
    /// Registered optimization passes, run in order
    passes: Vec<Box<dyn OptimizationPass>>,
    /// Maximum number of full-pipeline iterations
    max_iterations: usize,
}

impl Optimizer {
    /// Create a disabled optimizer (no passes, pass-through)
    pub fn new() -> Self {
        Self {
            enabled: false,
            passes: Vec::new(),
            max_iterations: 10,
        }
    }

    /// Create an optimizer with all three default passes enabled
    ///
    /// Passes run in order: constant folding → dead code elimination → peephole
    pub fn with_default_passes() -> Self {
        let mut opt = Self {
            enabled: true,
            passes: Vec::new(),
            max_iterations: 10,
        };
        opt.add_pass(Box::new(ConstantFoldingPass));
        opt.add_pass(Box::new(DeadCodeEliminationPass));
        opt.add_pass(Box::new(PeepholePass));
        opt
    }

    /// Create an optimizer with a specific optimization level
    ///
    /// - `0` — disabled
    /// - `1` — peephole only
    /// - `2` — constant folding + peephole
    /// - `3+` — all passes (same as `with_default_passes`)
    pub fn with_optimization_level(level: u8) -> Self {
        match level {
            0 => Self::new(),
            1 => {
                let mut opt = Self {
                    enabled: true,
                    passes: Vec::new(),
                    max_iterations: 3,
                };
                opt.add_pass(Box::new(PeepholePass));
                opt
            }
            2 => {
                let mut opt = Self {
                    enabled: true,
                    passes: Vec::new(),
                    max_iterations: 5,
                };
                opt.add_pass(Box::new(ConstantFoldingPass));
                opt.add_pass(Box::new(PeepholePass));
                opt
            }
            _ => Self::with_default_passes(),
        }
    }

    /// Enable or disable the optimizer
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns whether the optimizer is currently enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the number of registered passes
    pub fn passes_count(&self) -> usize {
        self.passes.len()
    }

    /// Add an optimization pass
    pub fn add_pass(&mut self, pass: Box<dyn OptimizationPass>) {
        self.passes.push(pass);
    }

    /// Optimize bytecode, returning the result
    pub fn optimize(&self, bytecode: Bytecode) -> Bytecode {
        if !self.enabled {
            return bytecode;
        }
        let (result, _) = self.optimize_with_stats(bytecode);
        result
    }

    /// Optimize bytecode and return statistics
    ///
    /// Runs all registered passes in a loop until no further changes occur or
    /// `max_iterations` is reached.
    pub fn optimize_with_stats(&self, bytecode: Bytecode) -> (Bytecode, OptimizationStats) {
        if !self.enabled {
            let size = bytecode.instructions.len();
            let mut stats = OptimizationStats::new();
            stats.bytecode_size_before = size;
            stats.bytecode_size_after = size;
            return (bytecode, stats);
        }

        let size_before = bytecode.instructions.len();
        let mut result = bytecode;
        let mut total_stats = OptimizationStats::new();
        total_stats.bytecode_size_before = size_before;

        for _ in 0..self.max_iterations {
            let mut changed = false;
            for pass in &self.passes {
                let (new_bytecode, pass_stats) = pass.optimize(result);
                if pass_stats.total_optimizations() > 0 {
                    changed = true;
                }
                total_stats.merge(&pass_stats);
                result = new_bytecode;
            }
            if !changed {
                break;
            }
        }

        total_stats.bytecode_size_after = result.instructions.len();
        (result, total_stats)
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Internal helpers shared by passes
// ============================================================================

/// Decoded representation of a single bytecode instruction
#[derive(Debug, Clone)]
pub(crate) struct DecodedInstruction {
    /// Original byte offset in the source bytecode
    pub offset: usize,
    /// The opcode
    pub opcode: Opcode,
    /// Raw operand bytes (0, 1, or 2 bytes depending on opcode)
    pub operands: Vec<u8>,
    /// Source span, if available from debug info
    pub span: Option<Span>,
}

impl DecodedInstruction {
    /// Total byte size of this instruction (opcode + operands)
    pub fn byte_size(&self) -> usize {
        1 + self.operands.len()
    }

    /// Read operands as a u16 (big-endian)
    pub fn read_u16(&self) -> u16 {
        debug_assert_eq!(self.operands.len(), 2, "Expected 2-byte operand");
        ((self.operands[0] as u16) << 8) | (self.operands[1] as u16)
    }

    /// Read operands as an i16 (big-endian)
    pub fn read_i16(&self) -> i16 {
        self.read_u16() as i16
    }

    /// Read operand as u8
    #[allow(dead_code)]
    pub fn read_u8(&self) -> u8 {
        debug_assert_eq!(self.operands.len(), 1, "Expected 1-byte operand");
        self.operands[0]
    }

    /// Encode a u16 as 2-byte big-endian operands
    pub fn make_u16_operands(value: u16) -> Vec<u8> {
        vec![(value >> 8) as u8, (value & 0xFF) as u8]
    }

    /// Encode an i16 as 2-byte big-endian operands
    pub fn make_i16_operands(value: i16) -> Vec<u8> {
        Self::make_u16_operands(value as u16)
    }
}

/// Returns the number of operand bytes for a given opcode
pub(crate) fn operand_size(opcode: Opcode) -> usize {
    match opcode {
        Opcode::Constant
        | Opcode::GetLocal
        | Opcode::SetLocal
        | Opcode::GetGlobal
        | Opcode::SetGlobal
        | Opcode::Array
        | Opcode::Jump
        | Opcode::JumpIfFalse
        | Opcode::Loop => 2,
        Opcode::Call => 1,
        _ => 0,
    }
}

/// Returns true if this opcode is a jump instruction (has an i16 relative offset)
pub(crate) fn is_jump_opcode(opcode: Opcode) -> bool {
    matches!(opcode, Opcode::Jump | Opcode::JumpIfFalse | Opcode::Loop)
}

/// Returns true if this opcode terminates a basic block unconditionally.
///
/// Used for dead code analysis: code after these instructions is unreachable
/// (unless targeted by a jump).
#[allow(dead_code)] // used in tests and future passes
pub(crate) fn is_unconditional_terminator(opcode: Opcode) -> bool {
    matches!(
        opcode,
        Opcode::Jump | Opcode::Return | Opcode::Halt | Opcode::Loop
    )
}

/// Decode a bytecode instruction stream into a list of [`DecodedInstruction`]s
///
/// Malformed bytes (unknown opcodes, truncated instructions) are skipped
/// gracefully to avoid panics in the optimizer.
pub(crate) fn decode_instructions(bytecode: &Bytecode) -> Vec<DecodedInstruction> {
    let instructions = &bytecode.instructions;
    let mut result = Vec::new();
    let mut i = 0;

    while i < instructions.len() {
        let offset = i;
        let opcode_byte = instructions[i];

        let opcode = match Opcode::try_from(opcode_byte) {
            Ok(op) => op,
            Err(_) => {
                // Unknown opcode — skip 1 byte, treat as Halt for safety
                result.push(DecodedInstruction {
                    offset,
                    opcode: Opcode::Halt,
                    operands: Vec::new(),
                    span: bytecode.get_span_for_offset(offset),
                });
                i += 1;
                continue;
            }
        };

        let op_size = operand_size(opcode);

        // Guard against truncated instruction stream
        if i + 1 + op_size > instructions.len() {
            // Truncated — push what we can
            let remaining = &instructions[i + 1..];
            result.push(DecodedInstruction {
                offset,
                opcode,
                operands: remaining.to_vec(),
                span: bytecode.get_span_for_offset(offset),
            });
            break;
        }

        let operands = instructions[i + 1..i + 1 + op_size].to_vec();
        let span = bytecode.get_span_for_offset(offset);

        result.push(DecodedInstruction {
            offset,
            opcode,
            operands,
            span,
        });

        i += 1 + op_size;
    }

    result
}

/// Re-encode a list of decoded instructions back into a [`Bytecode`]
///
/// Reconstructs debug information based on the spans stored in each
/// [`DecodedInstruction`].
pub(crate) fn encode_instructions(
    decoded: &[DecodedInstruction],
    constants: Vec<Value>,
    top_level_local_count: usize,
) -> Bytecode {
    let mut instructions = Vec::new();
    let mut debug_info = Vec::new();

    for instr in decoded {
        if let Some(span) = instr.span {
            debug_info.push(DebugSpan {
                instruction_offset: instructions.len(),
                span,
            });
        }
        instructions.push(instr.opcode as u8);
        instructions.extend_from_slice(&instr.operands);
    }

    Bytecode {
        instructions,
        constants,
        debug_info,
        top_level_local_count,
    }
}

/// Fix jump targets and function bytecode offsets after instructions have
/// been inserted or removed.
///
/// This function:
/// 1. Assigns new byte offsets to each instruction in `decoded`
/// 2. For each jump instruction, recalculates the relative i16 offset
/// 3. Updates `Function` values in `constants` if their bytecode_offset changed
pub(crate) fn fix_all_references(decoded: &mut [DecodedInstruction], constants: &mut [Value]) {
    // Build old_offset → new_offset mapping
    let mut new_offsets = Vec::with_capacity(decoded.len());
    let mut current = 0usize;
    for instr in decoded.iter() {
        new_offsets.push(current);
        current += instr.byte_size();
    }

    let old_to_new: std::collections::HashMap<usize, usize> = decoded
        .iter()
        .zip(new_offsets.iter())
        .map(|(instr, &new_off)| (instr.offset, new_off))
        .collect();

    // Fix jump operands
    for (idx, instr) in decoded.iter_mut().enumerate() {
        if is_jump_opcode(instr.opcode) && instr.operands.len() == 2 {
            let old_relative = instr.read_i16();
            let old_ip_after = instr.offset + 3; // opcode(1) + operand(2)
            let old_target = (old_ip_after as isize + old_relative as isize) as usize;

            if let Some(&new_target) = old_to_new.get(&old_target) {
                let new_ip_after = new_offsets[idx] + 3;
                let new_relative = (new_target as isize - new_ip_after as isize) as i16;
                instr.operands = DecodedInstruction::make_i16_operands(new_relative);
            }
            // If target not in map (dead target), leave offset as-is
        }
    }

    // Fix function bytecode offsets in constants
    for constant in constants.iter_mut() {
        if let Value::Function(ref mut func_ref) = constant {
            if let Some(&new_offset) = old_to_new.get(&func_ref.bytecode_offset) {
                func_ref.bytecode_offset = new_offset;
            }
        }
    }

    // Update offsets in decoded list
    for (instr, &new_off) in decoded.iter_mut().zip(new_offsets.iter()) {
        instr.offset = new_off;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::Opcode;
    use crate::span::Span;

    fn _make_bytecode(instrs: &[(Opcode, Vec<u8>)]) -> Bytecode {
        let mut bc = Bytecode::new();
        for (op, operands) in instrs {
            bc.emit(*op, Span::dummy());
            for &b in operands {
                bc.instructions.push(b);
            }
        }
        bc
    }

    #[test]
    fn test_optimizer_new_disabled() {
        let opt = Optimizer::new();
        assert!(!opt.is_enabled());
        assert_eq!(opt.passes_count(), 0);
    }

    #[test]
    fn test_optimizer_with_default_passes() {
        let opt = Optimizer::with_default_passes();
        assert!(opt.is_enabled());
        assert_eq!(opt.passes_count(), 3);
    }

    #[test]
    fn test_optimizer_disabled_is_passthrough() {
        let opt = Optimizer::new();
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Null, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());
        let original_len = bc.instructions.len();
        let result = opt.optimize(bc);
        assert_eq!(result.instructions.len(), original_len);
    }

    #[test]
    fn test_optimizer_enable_disable() {
        let mut opt = Optimizer::new();
        assert!(!opt.is_enabled());
        opt.set_enabled(true);
        assert!(opt.is_enabled());
        opt.set_enabled(false);
        assert!(!opt.is_enabled());
    }

    #[test]
    fn test_optimizer_level_0_disabled() {
        let opt = Optimizer::with_optimization_level(0);
        assert!(!opt.is_enabled());
    }

    #[test]
    fn test_optimizer_level_1_peephole_only() {
        let opt = Optimizer::with_optimization_level(1);
        assert!(opt.is_enabled());
        assert_eq!(opt.passes_count(), 1);
    }

    #[test]
    fn test_optimizer_level_2_cf_and_peephole() {
        let opt = Optimizer::with_optimization_level(2);
        assert!(opt.is_enabled());
        assert_eq!(opt.passes_count(), 2);
    }

    #[test]
    fn test_optimizer_level_3_all_passes() {
        let opt = Optimizer::with_optimization_level(3);
        assert!(opt.is_enabled());
        assert_eq!(opt.passes_count(), 3);
    }

    #[test]
    fn test_optimization_stats_default() {
        let stats = OptimizationStats::default();
        assert_eq!(stats.constants_folded, 0);
        assert_eq!(stats.dead_instructions_removed, 0);
        assert_eq!(stats.peephole_patterns_applied, 0);
        assert_eq!(stats.bytes_saved(), 0);
    }

    #[test]
    fn test_optimization_stats_bytes_saved() {
        let mut stats = OptimizationStats::new();
        stats.bytecode_size_before = 100;
        stats.bytecode_size_after = 75;
        assert_eq!(stats.bytes_saved(), 25);
        assert!((stats.size_reduction_percent() - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_optimization_stats_merge() {
        let mut a = OptimizationStats::new();
        a.constants_folded = 3;
        a.dead_instructions_removed = 2;

        let mut b = OptimizationStats::new();
        b.constants_folded = 1;
        b.peephole_patterns_applied = 5;

        a.merge(&b);
        assert_eq!(a.constants_folded, 4);
        assert_eq!(a.dead_instructions_removed, 2);
        assert_eq!(a.peephole_patterns_applied, 5);
    }

    #[test]
    fn test_optimization_stats_total() {
        let mut stats = OptimizationStats::new();
        stats.constants_folded = 3;
        stats.dead_instructions_removed = 2;
        stats.peephole_patterns_applied = 5;
        assert_eq!(stats.total_optimizations(), 10);
    }

    #[test]
    fn test_decode_simple_instructions() {
        let mut bc = Bytecode::new();
        let idx = bc.add_constant(Value::Number(42.0));
        bc.emit(Opcode::Constant, Span::dummy());
        bc.emit_u16(idx);
        bc.emit(Opcode::Halt, Span::dummy());

        let decoded = decode_instructions(&bc);
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].opcode, Opcode::Constant);
        assert_eq!(decoded[0].read_u16(), 0);
        assert_eq!(decoded[1].opcode, Opcode::Halt);
    }

    #[test]
    fn test_decode_no_operand_instructions() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Add, Span::dummy());
        bc.emit(Opcode::Sub, Span::dummy());
        bc.emit(Opcode::Halt, Span::dummy());

        let decoded = decode_instructions(&bc);
        assert_eq!(decoded.len(), 3);
        assert!(decoded[0].operands.is_empty());
        assert!(decoded[1].operands.is_empty());
    }

    #[test]
    fn test_decode_roundtrip() {
        let mut bc = Bytecode::new();
        let idx = bc.add_constant(Value::Number(10.0));
        bc.emit(Opcode::Constant, Span::new(0, 5));
        bc.emit_u16(idx);
        bc.emit(Opcode::Null, Span::new(5, 6));
        bc.emit(Opcode::Halt, Span::new(6, 7));

        let constants = bc.constants.clone();
        let decoded = decode_instructions(&bc);
        let rebuilt = encode_instructions(&decoded, constants, 0);

        assert_eq!(rebuilt.instructions, bc.instructions);
        assert_eq!(rebuilt.debug_info.len(), bc.debug_info.len());
    }

    #[test]
    fn test_operand_size() {
        assert_eq!(operand_size(Opcode::Constant), 2);
        assert_eq!(operand_size(Opcode::GetLocal), 2);
        assert_eq!(operand_size(Opcode::SetLocal), 2);
        assert_eq!(operand_size(Opcode::GetGlobal), 2);
        assert_eq!(operand_size(Opcode::SetGlobal), 2);
        assert_eq!(operand_size(Opcode::Array), 2);
        assert_eq!(operand_size(Opcode::Jump), 2);
        assert_eq!(operand_size(Opcode::JumpIfFalse), 2);
        assert_eq!(operand_size(Opcode::Loop), 2);
        assert_eq!(operand_size(Opcode::Call), 1);
        assert_eq!(operand_size(Opcode::Add), 0);
        assert_eq!(operand_size(Opcode::Halt), 0);
        assert_eq!(operand_size(Opcode::Pop), 0);
    }

    #[test]
    fn test_fix_jump_targets_after_removal() {
        // Layout: Jump@0 (target=Halt@4), Null@3, Halt@4
        // Jump offset: target=4, ip_after=3, relative = 4-3 = 1
        // After removing Null@3:
        //   New layout: Jump@0, Halt@3
        //   new_ip_after = 3, new_target = 3, new_relative = 3-3 = 0
        let mut decoded = vec![
            DecodedInstruction {
                offset: 0,
                opcode: Opcode::Jump,
                operands: DecodedInstruction::make_i16_operands(1), // target = 0+3+1 = 4 (Halt)
                span: Some(Span::dummy()),
            },
            DecodedInstruction {
                offset: 3,
                opcode: Opcode::Null,
                operands: Vec::new(),
                span: Some(Span::dummy()),
            },
            DecodedInstruction {
                offset: 4,
                opcode: Opcode::Halt,
                operands: Vec::new(),
                span: Some(Span::dummy()),
            },
        ];

        // Remove the Null instruction
        decoded.remove(1);

        let mut constants = Vec::new();
        fix_all_references(&mut decoded, &mut constants);

        // After removal:
        // Jump is at new offset 0, ip_after = 3
        // Halt is at new offset 3
        // new_relative = 3 - 3 = 0
        assert_eq!(decoded[0].read_i16(), 0);
    }

    #[test]
    fn test_decoded_instruction_byte_size() {
        let no_operand = DecodedInstruction {
            offset: 0,
            opcode: Opcode::Add,
            operands: Vec::new(),
            span: None,
        };
        assert_eq!(no_operand.byte_size(), 1);

        let two_operand = DecodedInstruction {
            offset: 0,
            opcode: Opcode::Constant,
            operands: vec![0, 1],
            span: None,
        };
        assert_eq!(two_operand.byte_size(), 3);
    }

    #[test]
    fn test_is_jump_opcode() {
        assert!(is_jump_opcode(Opcode::Jump));
        assert!(is_jump_opcode(Opcode::JumpIfFalse));
        assert!(is_jump_opcode(Opcode::Loop));
        assert!(!is_jump_opcode(Opcode::Add));
        assert!(!is_jump_opcode(Opcode::Halt));
    }

    #[test]
    fn test_is_unconditional_terminator() {
        assert!(is_unconditional_terminator(Opcode::Jump));
        assert!(is_unconditional_terminator(Opcode::Return));
        assert!(is_unconditional_terminator(Opcode::Halt));
        assert!(is_unconditional_terminator(Opcode::Loop));
        assert!(!is_unconditional_terminator(Opcode::JumpIfFalse));
        assert!(!is_unconditional_terminator(Opcode::Add));
    }

    #[test]
    fn test_make_u16_operands() {
        let operands = DecodedInstruction::make_u16_operands(0x1234);
        assert_eq!(operands, vec![0x12, 0x34]);
    }

    #[test]
    fn test_make_i16_operands_negative() {
        let operands = DecodedInstruction::make_i16_operands(-5);
        let val = ((operands[0] as u16) << 8 | operands[1] as u16) as i16;
        assert_eq!(val, -5);
    }

    #[test]
    fn test_optimizer_with_stats_no_passes() {
        let mut opt = Optimizer::new();
        opt.set_enabled(true);
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Halt, Span::dummy());

        let (result, stats) = opt.optimize_with_stats(bc);
        assert_eq!(result.instructions.len(), 1);
        assert_eq!(stats.total_optimizations(), 0);
    }

    #[test]
    fn test_optimizer_add_custom_pass() {
        let mut opt = Optimizer::new();
        opt.add_pass(Box::new(ConstantFoldingPass));
        assert_eq!(opt.passes_count(), 1);
        opt.add_pass(Box::new(PeepholePass));
        assert_eq!(opt.passes_count(), 2);
    }

    #[test]
    fn test_size_reduction_percent_zero_before() {
        let stats = OptimizationStats::new();
        // Should not panic
        assert_eq!(stats.size_reduction_percent(), 0.0);
    }

    #[test]
    fn test_fix_function_bytecode_offsets() {
        use crate::value::FunctionRef;
        // Function body originally at offset 10
        // After removing 4 bytes before it, should be at offset 6
        let mut decoded = vec![
            DecodedInstruction {
                offset: 0,
                opcode: Opcode::Null,
                operands: Vec::new(),
                span: None,
            },
            DecodedInstruction {
                offset: 1,
                opcode: Opcode::Pop,
                operands: Vec::new(),
                span: None,
            },
            // "Function body" starts here at offset 2 (was 10)
            DecodedInstruction {
                offset: 10,
                opcode: Opcode::Return,
                operands: Vec::new(),
                span: None,
            },
        ];
        let mut constants = vec![Value::Function(FunctionRef {
            name: "test".to_string(),
            arity: 0,
            bytecode_offset: 10, // old offset
            local_count: 0,
        })];

        fix_all_references(&mut decoded, &mut constants);

        if let Value::Function(ref func) = constants[0] {
            assert_eq!(func.bytecode_offset, 2); // updated to new offset
        } else {
            panic!("Expected function constant");
        }
    }
}
