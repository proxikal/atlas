//! Bytecode instruction set
//!
//! Stack-based bytecode with 30 opcodes organized by category.
//! Operands are encoded separately in the instruction stream.

mod disasm;
mod opcode;
mod optimizer;
mod serialize;

pub use disasm::disassemble;
pub use opcode::Opcode;
pub use optimizer::{ConstantFoldingPass, OptimizationPass, Optimizer};
use serialize::{deserialize_span, deserialize_value, serialize_span, serialize_value};

use crate::span::Span;
use crate::value::Value;

/// Current bytecode format version
///
/// This version is incremented when the bytecode format changes in a
/// backward-incompatible way. The VM will reject bytecode files with
/// different version numbers to prevent runtime errors from format mismatches.
///
/// Version history:
/// - Version 1: Initial bytecode format (Phase 10)
pub const BYTECODE_VERSION: u16 = 1;

/// Debug information for bytecode
///
/// Maps instruction offsets to source spans for error reporting
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSpan {
    /// Byte offset of instruction in bytecode
    pub instruction_offset: usize,
    /// Source span for this instruction
    pub span: Span,
}

/// Bytecode container
///
/// Contains raw instruction bytes, constant pool, and debug information.
/// Instructions are encoded as:
/// - Opcode (1 byte)
/// - Operands (variable, depending on opcode)
#[derive(Debug, Clone)]
pub struct Bytecode {
    /// Raw instruction bytes
    pub instructions: Vec<u8>,
    /// Constant pool (referenced by index)
    pub constants: Vec<Value>,
    /// Debug information (instruction offset -> source span)
    pub debug_info: Vec<DebugSpan>,
}

impl Bytecode {
    /// Create a new empty bytecode container
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            debug_info: Vec::new(),
        }
    }

    /// Emit an opcode and track debug information
    pub fn emit(&mut self, opcode: Opcode, span: Span) {
        self.debug_info.push(DebugSpan {
            instruction_offset: self.instructions.len(),
            span,
        });
        self.instructions.push(opcode as u8);
    }

    /// Emit a single byte operand
    pub fn emit_u8(&mut self, byte: u8) {
        self.instructions.push(byte);
    }

    /// Emit a u16 operand (big-endian)
    pub fn emit_u16(&mut self, value: u16) {
        self.instructions.push((value >> 8) as u8);
        self.instructions.push((value & 0xFF) as u8);
    }

    /// Emit an i16 operand (big-endian, signed)
    pub fn emit_i16(&mut self, value: i16) {
        self.emit_u16(value as u16);
    }

    /// Add a constant to the pool and return its index
    pub fn add_constant(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }

    /// Get current instruction offset (for jump targets)
    pub fn current_offset(&self) -> usize {
        self.instructions.len()
    }

    /// Patch a jump instruction with the correct offset
    ///
    /// Used for forward jumps where the target isn't known yet
    pub fn patch_jump(&mut self, offset: usize) {
        let jump = (self.instructions.len() - offset - 2) as i16;
        self.instructions[offset] = ((jump >> 8) & 0xFF) as u8;
        self.instructions[offset + 1] = (jump & 0xFF) as u8;
    }

    /// Look up the source span for a given instruction offset
    ///
    /// Returns the span of the instruction at or before the given offset.
    /// This is useful for error reporting in the VM.
    pub fn get_span_for_offset(&self, offset: usize) -> Option<Span> {
        // Find the most recent debug info entry at or before the offset
        self.debug_info
            .iter()
            .rev()
            .find(|debug_span| debug_span.instruction_offset <= offset)
            .map(|debug_span| debug_span.span)
    }

    /// Serialize bytecode to binary format (.atb file)
    ///
    /// Format:
    /// - Header: Magic "ATB\0" + version u16 + flags u16
    /// - Constants: count u32 + serialized values
    /// - Instructions: length u32 + bytecode bytes
    /// - Debug info (optional): count u32 + debug spans
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Header
        bytes.extend_from_slice(b"ATB\0"); // Magic number
        bytes.extend_from_slice(&BYTECODE_VERSION.to_be_bytes()); // Version
        let flags = if self.debug_info.is_empty() { 0u16 } else { 1u16 };
        bytes.extend_from_slice(&flags.to_be_bytes()); // Flags

        // Constants section
        bytes.extend_from_slice(&(self.constants.len() as u32).to_be_bytes());
        for value in &self.constants {
            serialize_value(value, &mut bytes);
        }

        // Instructions section
        bytes.extend_from_slice(&(self.instructions.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&self.instructions);

        // Debug info section (optional)
        if !self.debug_info.is_empty() {
            bytes.extend_from_slice(&(self.debug_info.len() as u32).to_be_bytes());
            for debug_span in &self.debug_info {
                bytes.extend_from_slice(&(debug_span.instruction_offset as u32).to_be_bytes());
                serialize_span(&debug_span.span, &mut bytes);
            }
        }

        bytes
    }

    /// Deserialize bytecode from binary format (.atb file)
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        // Read and validate header
        if bytes.len() < 8 {
            return Err("Invalid bytecode file: too short".to_string());
        }
        if &bytes[0..4] != b"ATB\0" {
            return Err("Invalid bytecode file: bad magic number. Expected 'ATB\\0', this may not be an Atlas bytecode file.".to_string());
        }
        let version = u16::from_be_bytes([bytes[4], bytes[5]]);
        if version != BYTECODE_VERSION {
            return Err(format!(
                "Bytecode version mismatch: file has version {}, but this VM supports version {}. \
                 Recompile the source file with the current Atlas compiler.",
                version, BYTECODE_VERSION
            ));
        }
        let flags = u16::from_be_bytes([bytes[6], bytes[7]]);
        let has_debug_info = (flags & 1) != 0;

        // Start reading sections after header (8 bytes)
        let mut offset = 8;

        // Read constants
        if offset + 4 > bytes.len() {
            return Err("Invalid bytecode: constants section truncated".to_string());
        }
        let const_count = u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;

        let mut constants = Vec::with_capacity(const_count);
        for _ in 0..const_count {
            let (value, consumed) = deserialize_value(&bytes[offset..])?;
            constants.push(value);
            offset += consumed;
        }

        // Read instructions
        if offset + 4 > bytes.len() {
            return Err("Invalid bytecode: instructions section truncated".to_string());
        }
        let instr_len = u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;

        if offset + instr_len > bytes.len() {
            return Err("Invalid bytecode: instructions data truncated".to_string());
        }
        let instructions = bytes[offset..offset + instr_len].to_vec();
        offset += instr_len;

        // Read debug info (optional)
        let mut debug_info = Vec::new();
        if has_debug_info {
            if offset + 4 > bytes.len() {
                return Err("Invalid bytecode: debug info section truncated".to_string());
            }
            let debug_count = u32::from_be_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            offset += 4;

            for _ in 0..debug_count {
                if offset + 4 > bytes.len() {
                    return Err("Invalid bytecode: debug span truncated".to_string());
                }
                let instruction_offset = u32::from_be_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]) as usize;
                offset += 4;

                let (span, consumed) = deserialize_span(&bytes[offset..])?;
                debug_info.push(DebugSpan {
                    instruction_offset,
                    span,
                });
                offset += consumed;
            }
        }

        // Verify we consumed exactly the expected amount of data
        if offset != bytes.len() {
            return Err(format!(
                "Invalid bytecode: expected {} bytes, but only consumed {}",
                bytes.len(),
                offset
            ));
        }

        Ok(Bytecode {
            instructions,
            constants,
            debug_info,
        })
    }
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_to_u8() {
        assert_eq!(Opcode::Constant as u8, 0x01);
        assert_eq!(Opcode::Null as u8, 0x02);
        assert_eq!(Opcode::Add as u8, 0x20);
        assert_eq!(Opcode::Jump as u8, 0x50);
        assert_eq!(Opcode::Halt as u8, 0xFF);
    }

    #[test]
    fn test_opcode_from_u8() {
        assert_eq!(Opcode::try_from(0x01), Ok(Opcode::Constant));
        assert_eq!(Opcode::try_from(0x02), Ok(Opcode::Null));
        assert_eq!(Opcode::try_from(0x20), Ok(Opcode::Add));
        assert_eq!(Opcode::try_from(0x50), Ok(Opcode::Jump));
        assert_eq!(Opcode::try_from(0xFF), Ok(Opcode::Halt));
        assert_eq!(Opcode::try_from(0x99), Err(())); // Invalid opcode
    }

    #[test]
    fn test_all_opcodes_roundtrip() {
        let opcodes = vec![
            Opcode::Constant,
            Opcode::Null,
            Opcode::True,
            Opcode::False,
            Opcode::GetLocal,
            Opcode::SetLocal,
            Opcode::GetGlobal,
            Opcode::SetGlobal,
            Opcode::Add,
            Opcode::Sub,
            Opcode::Mul,
            Opcode::Div,
            Opcode::Mod,
            Opcode::Negate,
            Opcode::Equal,
            Opcode::NotEqual,
            Opcode::Less,
            Opcode::LessEqual,
            Opcode::Greater,
            Opcode::GreaterEqual,
            Opcode::Not,
            Opcode::And,
            Opcode::Or,
            Opcode::Jump,
            Opcode::JumpIfFalse,
            Opcode::Loop,
            Opcode::Call,
            Opcode::Return,
            Opcode::Array,
            Opcode::GetIndex,
            Opcode::SetIndex,
            Opcode::Pop,
            Opcode::Dup,
            Opcode::Halt,
        ];

        for opcode in opcodes {
            let byte = opcode as u8;
            let decoded = Opcode::try_from(byte).unwrap();
            assert_eq!(opcode, decoded);
        }
    }

    #[test]
    fn test_bytecode_creation() {
        let bytecode = Bytecode::new();
        assert_eq!(bytecode.instructions.len(), 0);
        assert_eq!(bytecode.constants.len(), 0);
        assert_eq!(bytecode.debug_info.len(), 0);
    }

    #[test]
    fn test_emit_opcode() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Null, Span::dummy());
        assert_eq!(bytecode.instructions.len(), 1);
        assert_eq!(bytecode.instructions[0], 0x02); // Null opcode
        assert_eq!(bytecode.debug_info.len(), 1);
        assert_eq!(bytecode.debug_info[0].instruction_offset, 0);
    }

    #[test]
    fn test_emit_u8() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Call, Span::dummy());
        bytecode.emit_u8(3); // 3 arguments
        assert_eq!(bytecode.instructions.len(), 2);
        assert_eq!(bytecode.instructions[0], 0x60); // Call opcode
        assert_eq!(bytecode.instructions[1], 3);
    }

    #[test]
    fn test_emit_u16_big_endian() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Constant, Span::dummy());
        bytecode.emit_u16(0x1234);
        assert_eq!(bytecode.instructions.len(), 3);
        assert_eq!(bytecode.instructions[0], 0x01); // Constant opcode
        assert_eq!(bytecode.instructions[1], 0x12); // High byte
        assert_eq!(bytecode.instructions[2], 0x34); // Low byte
    }

    #[test]
    fn test_emit_i16() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Jump, Span::dummy());
        bytecode.emit_i16(100);
        assert_eq!(bytecode.instructions.len(), 3);
        assert_eq!(bytecode.instructions[0], 0x50); // Jump opcode
        // 100 as i16 -> u16 -> bytes
        assert_eq!(bytecode.instructions[1], 0x00);
        assert_eq!(bytecode.instructions[2], 0x64);
    }

    #[test]
    fn test_constant_pool() {
        use std::rc::Rc;
        let mut bytecode = Bytecode::new();
        let idx1 = bytecode.add_constant(Value::Number(42.0));
        let idx2 = bytecode.add_constant(Value::String(Rc::new("hello".to_string())));
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(bytecode.constants.len(), 2);
        assert_eq!(bytecode.constants[0], Value::Number(42.0));
    }

    #[test]
    fn test_current_offset() {
        let mut bytecode = Bytecode::new();
        assert_eq!(bytecode.current_offset(), 0);
        bytecode.emit(Opcode::Null, Span::dummy());
        assert_eq!(bytecode.current_offset(), 1);
        bytecode.emit_u16(0x1234);
        assert_eq!(bytecode.current_offset(), 3);
    }

    #[test]
    fn test_patch_jump() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::JumpIfFalse, Span::dummy());
        let jump_offset = bytecode.current_offset();
        bytecode.emit_u16(0xFFFF); // Placeholder
        bytecode.emit(Opcode::Null, Span::dummy());
        bytecode.emit(Opcode::Null, Span::dummy());

        // Patch the jump to point to current position
        bytecode.patch_jump(jump_offset);

        // Jump should now be 2 (from offset+2 to current position)
        let jump = ((bytecode.instructions[jump_offset] as i16) << 8)
            | (bytecode.instructions[jump_offset + 1] as i16);
        assert_eq!(jump, 2);
    }

    #[test]
    fn test_debug_info_tracking() {
        let mut bytecode = Bytecode::new();
        let span1 = Span::new(0, 10);
        let span2 = Span::new(10, 20);

        bytecode.emit(Opcode::Constant, span1);
        bytecode.emit_u16(0);
        bytecode.emit(Opcode::Constant, span2);
        bytecode.emit_u16(1);

        assert_eq!(bytecode.debug_info.len(), 2);
        assert_eq!(bytecode.debug_info[0].instruction_offset, 0);
        assert_eq!(bytecode.debug_info[0].span, span1);
        assert_eq!(bytecode.debug_info[1].instruction_offset, 3);
        assert_eq!(bytecode.debug_info[1].span, span2);
    }

    #[test]
    fn test_bytecode_sequence() {
        // Test: let x = 2 + 3;
        // Bytecode:
        //   Constant 0  (push 2.0)
        //   Constant 1  (push 3.0)
        //   Add         (pop both, push 5.0)
        //   SetLocal 0  (store to x)

        let mut bytecode = Bytecode::new();
        let idx_2 = bytecode.add_constant(Value::Number(2.0));
        let idx_3 = bytecode.add_constant(Value::Number(3.0));

        bytecode.emit(Opcode::Constant, Span::dummy());
        bytecode.emit_u16(idx_2);
        bytecode.emit(Opcode::Constant, Span::dummy());
        bytecode.emit_u16(idx_3);
        bytecode.emit(Opcode::Add, Span::dummy());
        bytecode.emit(Opcode::SetLocal, Span::dummy());
        bytecode.emit_u16(0); // Local index 0

        assert_eq!(bytecode.instructions.len(), 10);
        assert_eq!(bytecode.constants.len(), 2);
        assert_eq!(bytecode.debug_info.len(), 4);
    }

    // ===== Constants Pool Specific Tests =====

    #[test]
    fn test_constant_pool_indexing() {
        use std::rc::Rc;
        let mut bytecode = Bytecode::new();

        // Add multiple constants of different types
        let idx0 = bytecode.add_constant(Value::Number(42.0));
        let idx1 = bytecode.add_constant(Value::String(Rc::new("hello".to_string())));
        let idx2 = bytecode.add_constant(Value::Bool(true));
        let idx3 = bytecode.add_constant(Value::Null);

        // Verify sequential indexing
        assert_eq!(idx0, 0);
        assert_eq!(idx1, 1);
        assert_eq!(idx2, 2);
        assert_eq!(idx3, 3);

        // Verify values are stored correctly
        assert_eq!(bytecode.constants[0], Value::Number(42.0));
        assert_eq!(
            bytecode.constants[1],
            Value::String(Rc::new("hello".to_string()))
        );
        assert_eq!(bytecode.constants[2], Value::Bool(true));
        assert_eq!(bytecode.constants[3], Value::Null);
    }

    #[test]
    fn test_constant_pool_multiple_same_values() {
        // Test that adding the same value multiple times creates separate entries
        // (no deduplication in current implementation)
        let mut bytecode = Bytecode::new();

        let idx1 = bytecode.add_constant(Value::Number(42.0));
        let idx2 = bytecode.add_constant(Value::Number(42.0));
        let idx3 = bytecode.add_constant(Value::Number(42.0));

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 2);
        assert_eq!(bytecode.constants.len(), 3);
    }

    #[test]
    fn test_constant_pool_large() {
        // Test that we can handle a large constant pool
        let mut bytecode = Bytecode::new();

        for i in 0..1000 {
            let idx = bytecode.add_constant(Value::Number(i as f64));
            assert_eq!(idx, i as u16);
        }

        assert_eq!(bytecode.constants.len(), 1000);
        assert_eq!(bytecode.constants[500], Value::Number(500.0));
    }

    #[test]
    fn test_constant_pool_mixed_types() {
        use std::rc::Rc;
        let mut bytecode = Bytecode::new();

        // Add a variety of constant types
        bytecode.add_constant(Value::Number(1.0));
        bytecode.add_constant(Value::String(Rc::new("a".to_string())));
        bytecode.add_constant(Value::Number(2.0));
        bytecode.add_constant(Value::Bool(false));
        bytecode.add_constant(Value::String(Rc::new("b".to_string())));
        bytecode.add_constant(Value::Null);
        bytecode.add_constant(Value::Number(3.0));
        bytecode.add_constant(Value::Bool(true));

        assert_eq!(bytecode.constants.len(), 8);

        // Verify types are preserved
        assert!(matches!(bytecode.constants[0], Value::Number(_)));
        assert!(matches!(bytecode.constants[1], Value::String(_)));
        assert!(matches!(bytecode.constants[2], Value::Number(_)));
        assert!(matches!(bytecode.constants[3], Value::Bool(false)));
        assert!(matches!(bytecode.constants[4], Value::String(_)));
        assert!(matches!(bytecode.constants[5], Value::Null));
        assert!(matches!(bytecode.constants[6], Value::Number(_)));
        assert!(matches!(bytecode.constants[7], Value::Bool(true)));
    }

    #[test]
    fn test_constant_loading_sequence() {
        // Test complete sequence of adding and using constants
        let mut bytecode = Bytecode::new();

        let num_idx = bytecode.add_constant(Value::Number(100.0));
        let str_idx = bytecode.add_constant(Value::string("test"));

        // Emit instructions to load these constants
        bytecode.emit(Opcode::Constant, Span::dummy());
        bytecode.emit_u16(num_idx);
        bytecode.emit(Opcode::Constant, Span::dummy());
        bytecode.emit_u16(str_idx);
        bytecode.emit(Opcode::Halt, Span::dummy());

        // Verify bytecode structure
        assert_eq!(bytecode.constants.len(), 2);
        assert_eq!(bytecode.instructions.len(), 7); // Constant + u16 + Constant + u16 + Halt

        // Verify the constant references in bytecode
        let idx1_bytes = ((bytecode.instructions[1] as u16) << 8) | (bytecode.instructions[2] as u16);
        let idx2_bytes = ((bytecode.instructions[4] as u16) << 8) | (bytecode.instructions[5] as u16);

        assert_eq!(idx1_bytes, num_idx);
        assert_eq!(idx2_bytes, str_idx);
    }

    #[test]
    fn test_constant_pool_edge_values() {
        use std::rc::Rc;
        let mut bytecode = Bytecode::new();

        // Test edge case values
        bytecode.add_constant(Value::Number(0.0));
        bytecode.add_constant(Value::Number(-0.0));
        bytecode.add_constant(Value::Number(f64::MIN));
        bytecode.add_constant(Value::Number(f64::MAX));
        bytecode.add_constant(Value::String(Rc::new("".to_string()))); // Empty string
        bytecode.add_constant(Value::String(Rc::new("a".repeat(1000)))); // Long string

        assert_eq!(bytecode.constants.len(), 6);
        assert_eq!(bytecode.constants[0], Value::Number(0.0));
        assert_eq!(bytecode.constants[2], Value::Number(f64::MIN));
        assert_eq!(bytecode.constants[3], Value::Number(f64::MAX));
    }

    // ===== Debug Info Lookup Tests (Phase 14) =====

    #[test]
    fn test_get_span_for_offset_empty() {
        let bytecode = Bytecode::new();
        assert_eq!(bytecode.get_span_for_offset(0), None);
        assert_eq!(bytecode.get_span_for_offset(10), None);
    }

    #[test]
    fn test_get_span_for_offset_exact_match() {
        let mut bytecode = Bytecode::new();
        let span1 = Span::new(0, 10);
        let span2 = Span::new(10, 20);

        bytecode.emit(Opcode::Constant, span1);
        bytecode.emit_u16(0);
        bytecode.emit(Opcode::Add, span2);

        // Exact match for first instruction (offset 0)
        assert_eq!(bytecode.get_span_for_offset(0), Some(span1));

        // Exact match for second instruction (offset 3)
        assert_eq!(bytecode.get_span_for_offset(3), Some(span2));
    }

    #[test]
    fn test_get_span_for_offset_between_instructions() {
        let mut bytecode = Bytecode::new();
        let span1 = Span::new(0, 10);
        let span2 = Span::new(20, 30);

        bytecode.emit(Opcode::Constant, span1);
        bytecode.emit_u16(0); // Takes 2 more bytes (offset 1, 2)
        bytecode.emit(Opcode::Add, span2); // At offset 3

        // Offset 1 and 2 are operand bytes, should return span1
        assert_eq!(bytecode.get_span_for_offset(1), Some(span1));
        assert_eq!(bytecode.get_span_for_offset(2), Some(span1));

        // Offset 3 is next instruction, should return span2
        assert_eq!(bytecode.get_span_for_offset(3), Some(span2));

        // Offset beyond last instruction, should return last span
        assert_eq!(bytecode.get_span_for_offset(10), Some(span2));
    }

    #[test]
    fn test_get_span_for_offset_complex_sequence() {
        let mut bytecode = Bytecode::new();
        let span1 = Span::new(0, 5);
        let span2 = Span::new(6, 11);
        let span3 = Span::new(12, 17);

        // Instruction at offset 0
        bytecode.emit(Opcode::Constant, span1);
        bytecode.emit_u16(0);

        // Instruction at offset 3
        bytecode.emit(Opcode::Constant, span2);
        bytecode.emit_u16(1);

        // Instruction at offset 6
        bytecode.emit(Opcode::Add, span3);

        // Test various offsets
        assert_eq!(bytecode.get_span_for_offset(0), Some(span1));
        assert_eq!(bytecode.get_span_for_offset(1), Some(span1));
        assert_eq!(bytecode.get_span_for_offset(2), Some(span1));
        assert_eq!(bytecode.get_span_for_offset(3), Some(span2));
        assert_eq!(bytecode.get_span_for_offset(4), Some(span2));
        assert_eq!(bytecode.get_span_for_offset(5), Some(span2));
        assert_eq!(bytecode.get_span_for_offset(6), Some(span3));
        assert_eq!(bytecode.get_span_for_offset(100), Some(span3));
    }

    // ===== Serialization Tests (Phase 10) =====

    #[test]
    fn test_bytecode_serialize_empty() {
        // Test serializing empty bytecode
        let bytecode = Bytecode::new();
        let bytes = bytecode.to_bytes();

        // Check header
        assert_eq!(&bytes[0..4], b"ATB\0");
        // Version should be 1
        assert_eq!(u16::from_be_bytes([bytes[4], bytes[5]]), 1);
        // Flags should be 0 (no debug info)
        assert_eq!(u16::from_be_bytes([bytes[6], bytes[7]]), 0);
        // Constants count should be 0
        assert_eq!(
            u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            0
        );
        // Instructions length should be 0
        assert_eq!(
            u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
            0
        );
    }

    #[test]
    fn test_bytecode_serialize_with_constants() {
        use std::rc::Rc;
        let mut bytecode = Bytecode::new();
        bytecode.add_constant(Value::Number(42.0));
        bytecode.add_constant(Value::String(Rc::new("hello".to_string())));
        bytecode.add_constant(Value::Bool(true));
        bytecode.add_constant(Value::Null);

        let bytes = bytecode.to_bytes();
        let loaded = Bytecode::from_bytes(&bytes).unwrap();

        assert_eq!(loaded.constants.len(), 4);
        assert_eq!(loaded.constants[0], Value::Number(42.0));
        assert_eq!(loaded.constants[1], Value::string("hello"));
        assert_eq!(loaded.constants[2], Value::Bool(true));
        assert_eq!(loaded.constants[3], Value::Null);
    }

    #[test]
    fn test_bytecode_serialize_with_instructions() {
        let mut bytecode = Bytecode::new();
        bytecode.add_constant(Value::Number(10.0));
        bytecode.emit(Opcode::Constant, Span::dummy());
        bytecode.emit_u16(0);
        bytecode.emit(Opcode::Pop, Span::dummy());
        bytecode.emit(Opcode::Halt, Span::dummy());

        let bytes = bytecode.to_bytes();
        let loaded = Bytecode::from_bytes(&bytes).unwrap();

        assert_eq!(loaded.instructions.len(), bytecode.instructions.len());
        assert_eq!(loaded.instructions, bytecode.instructions);
    }

    #[test]
    fn test_bytecode_roundtrip() {
        // Test full roundtrip: bytecode -> bytes -> bytecode
        use std::rc::Rc;
        let mut bytecode = Bytecode::new();

        // Add constants
        bytecode.add_constant(Value::Number(3.14));
        bytecode.add_constant(Value::String(Rc::new("test".to_string())));

        // Add instructions
        bytecode.emit(Opcode::Constant, Span { start: 0, end: 5 });
        bytecode.emit_u16(0);
        bytecode.emit(Opcode::Constant, Span { start: 6, end: 12 });
        bytecode.emit_u16(1);
        bytecode.emit(Opcode::Add, Span { start: 13, end: 14 });
        bytecode.emit(Opcode::Halt, Span { start: 15, end: 16 });

        // Serialize
        let bytes = bytecode.to_bytes();

        // Deserialize
        let loaded = Bytecode::from_bytes(&bytes).unwrap();

        // Verify constants
        assert_eq!(loaded.constants.len(), 2);
        assert_eq!(loaded.constants[0], Value::Number(3.14));
        assert_eq!(loaded.constants[1], Value::string("test"));

        // Verify instructions
        assert_eq!(loaded.instructions, bytecode.instructions);

        // Verify debug info
        assert_eq!(loaded.debug_info.len(), bytecode.debug_info.len());
        for (i, debug_span) in loaded.debug_info.iter().enumerate() {
            assert_eq!(
                debug_span.instruction_offset,
                bytecode.debug_info[i].instruction_offset
            );
            assert_eq!(debug_span.span, bytecode.debug_info[i].span);
        }
    }

    #[test]
    fn test_bytecode_deserialize_invalid_magic() {
        let bytes = b"XXX\0\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let result = Bytecode::from_bytes(bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("bad magic number"));
    }

    #[test]
    fn test_bytecode_deserialize_invalid_version() {
        let bytes = b"ATB\0\x00\x99\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let result = Bytecode::from_bytes(bytes);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("version mismatch"));
        assert!(err.contains("153")); // 0x0099 in decimal
    }

    #[test]
    fn test_bytecode_version_constant() {
        // Verify that BYTECODE_VERSION constant is used in serialization
        let bytecode = Bytecode::new();
        let bytes = bytecode.to_bytes();

        // Extract version from header (bytes 4-5)
        let version = u16::from_be_bytes([bytes[4], bytes[5]]);
        assert_eq!(version, BYTECODE_VERSION);
    }

    #[test]
    fn test_bytecode_version_too_old() {
        // Simulate loading bytecode from an older version (version 0)
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"ATB\0"); // Magic
        bytes.extend_from_slice(&0u16.to_be_bytes()); // Version 0 (too old)
        bytes.extend_from_slice(&0u16.to_be_bytes()); // Flags
        bytes.extend_from_slice(&0u32.to_be_bytes()); // Constants count
        bytes.extend_from_slice(&0u32.to_be_bytes()); // Instructions length

        let result = Bytecode::from_bytes(&bytes);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("version mismatch"));
        assert!(err.contains("0")); // Old version
        assert!(err.contains(&BYTECODE_VERSION.to_string())); // Expected version
    }

    #[test]
    fn test_bytecode_version_too_new() {
        // Simulate loading bytecode from a newer version (version 99)
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"ATB\0"); // Magic
        bytes.extend_from_slice(&99u16.to_be_bytes()); // Version 99 (too new)
        bytes.extend_from_slice(&0u16.to_be_bytes()); // Flags
        bytes.extend_from_slice(&0u32.to_be_bytes()); // Constants count
        bytes.extend_from_slice(&0u32.to_be_bytes()); // Instructions length

        let result = Bytecode::from_bytes(&bytes);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("version mismatch"));
        assert!(err.contains("99")); // New version
        assert!(err.contains(&BYTECODE_VERSION.to_string())); // Expected version
        assert!(err.contains("Recompile")); // Helpful suggestion
    }

    #[test]
    fn test_bytecode_current_version_accepted() {
        // Verify that bytecode with current version is accepted
        let bytecode = Bytecode::new();
        let bytes = bytecode.to_bytes();

        let result = Bytecode::from_bytes(&bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bytecode_magic_number_error_message() {
        // Test improved magic number error message
        let bytes = b"XXX\0\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let result = Bytecode::from_bytes(bytes);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("bad magic number"));
        assert!(err.contains("ATB")); // Mentions expected magic
    }

    #[test]
    fn test_bytecode_deserialize_truncated() {
        let bytes = b"ATB\0";
        let result = Bytecode::from_bytes(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_bytecode_serialize_function_constant() {
        use crate::value::FunctionRef;

        let mut bytecode = Bytecode::new();
        bytecode.add_constant(Value::Function(FunctionRef {
            name: "test_func".to_string(),
            arity: 2,
            bytecode_offset: 100,
            local_count: 0,
        }));

        let bytes = bytecode.to_bytes();
        let loaded = Bytecode::from_bytes(&bytes).unwrap();

        assert_eq!(loaded.constants.len(), 1);
        match &loaded.constants[0] {
            Value::Function(func) => {
                assert_eq!(func.name, "test_func");
                assert_eq!(func.arity, 2);
                assert_eq!(func.bytecode_offset, 100);
            }
            _ => panic!("Expected Function value"),
        }
    }

    // ===== Debug Info Default Tests (Phase 15) =====

    #[test]
    fn test_bytecode_with_debug_info_sets_flag() {
        // Verify that bytecode with debug info sets the flag correctly
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Constant, Span::new(0, 5));
        bytecode.emit_u16(0);
        bytecode.emit(Opcode::Halt, Span::new(6, 7));

        assert!(!bytecode.debug_info.is_empty(), "Should have debug info");

        let bytes = bytecode.to_bytes();

        // Check header flags (bytes 6-7)
        let flags = u16::from_be_bytes([bytes[6], bytes[7]]);
        assert_eq!(flags & 1, 1, "Bit 0 should be set when debug info is present");
    }

    #[test]
    fn test_bytecode_without_debug_info_clears_flag() {
        // Verify that bytecode without debug info has flag cleared
        let mut bytecode = Bytecode::new();
        // Use lower-level operations that don't track debug info
        bytecode.instructions.push(Opcode::Halt as u8);

        assert!(bytecode.debug_info.is_empty(), "Should have no debug info");

        let bytes = bytecode.to_bytes();

        // Check header flags (bytes 6-7)
        let flags = u16::from_be_bytes([bytes[6], bytes[7]]);
        assert_eq!(flags & 1, 0, "Bit 0 should be clear when debug info is absent");
    }

    #[test]
    fn test_serialized_debug_info_section_present() {
        // Test that the debug info section is present in serialized bytecode
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Constant, Span::new(10, 20));
        bytecode.emit_u16(0);
        bytecode.emit(Opcode::Pop, Span::new(21, 22));

        let bytes = bytecode.to_bytes();

        // Deserialize and verify debug info is present
        let loaded = Bytecode::from_bytes(&bytes).unwrap();
        assert_eq!(loaded.debug_info.len(), 2, "Should have 2 debug spans");
        assert_eq!(loaded.debug_info[0].span, Span::new(10, 20));
        assert_eq!(loaded.debug_info[1].span, Span::new(21, 22));
    }

    #[test]
    fn test_debug_info_instruction_offsets_accurate() {
        // Verify that debug info instruction offsets are accurate
        let mut bytecode = Bytecode::new();

        // Emit several instructions and track expected offsets
        bytecode.emit(Opcode::Constant, Span::new(0, 1)); // Offset 0
        bytecode.emit_u16(0); // Operands at 1-2
        bytecode.emit(Opcode::Constant, Span::new(2, 3)); // Offset 3
        bytecode.emit_u16(1); // Operands at 4-5
        bytecode.emit(Opcode::Add, Span::new(4, 5)); // Offset 6
        bytecode.emit(Opcode::Halt, Span::new(6, 7)); // Offset 7

        // Verify debug info offsets
        assert_eq!(bytecode.debug_info[0].instruction_offset, 0);
        assert_eq!(bytecode.debug_info[1].instruction_offset, 3);
        assert_eq!(bytecode.debug_info[2].instruction_offset, 6);
        assert_eq!(bytecode.debug_info[3].instruction_offset, 7);

        // Verify these offsets survive serialization
        let bytes = bytecode.to_bytes();
        let loaded = Bytecode::from_bytes(&bytes).unwrap();

        assert_eq!(loaded.debug_info[0].instruction_offset, 0);
        assert_eq!(loaded.debug_info[1].instruction_offset, 3);
        assert_eq!(loaded.debug_info[2].instruction_offset, 6);
        assert_eq!(loaded.debug_info[3].instruction_offset, 7);
    }

    #[test]
    fn test_debug_info_with_many_instructions() {
        // Test that debug info works with many instructions
        let mut bytecode = Bytecode::new();

        for i in 0..100 {
            bytecode.emit(Opcode::Constant, Span::new(i * 10, i * 10 + 5));
            bytecode.emit_u16(i as u16);
            bytecode.emit(Opcode::Pop, Span::new(i * 10 + 6, i * 10 + 7));
        }
        bytecode.emit(Opcode::Halt, Span::dummy());

        // Should have 201 debug spans (100 Constants + 100 Pops + 1 Halt)
        assert_eq!(bytecode.debug_info.len(), 201);

        // Serialize and verify
        let bytes = bytecode.to_bytes();
        let loaded = Bytecode::from_bytes(&bytes).unwrap();

        assert_eq!(loaded.debug_info.len(), 201);
        // Spot check a few spans
        assert_eq!(loaded.debug_info[0].span, Span::new(0, 5));
        assert_eq!(loaded.debug_info[1].span, Span::new(6, 7));
        assert_eq!(loaded.debug_info[100].span, Span::new(500, 505));
    }

    #[test]
    fn test_debug_info_span_lookup_after_deserialization() {
        // Test that span lookup works correctly after deserialization
        let mut bytecode = Bytecode::new();
        let span1 = Span::new(0, 10);
        let span2 = Span::new(20, 30);

        bytecode.emit(Opcode::Constant, span1);
        bytecode.emit_u16(0); // Takes bytes 1-2
        bytecode.emit(Opcode::Add, span2); // At offset 3

        // Serialize and deserialize
        let bytes = bytecode.to_bytes();
        let loaded = Bytecode::from_bytes(&bytes).unwrap();

        // Test span lookup on deserialized bytecode
        assert_eq!(loaded.get_span_for_offset(0), Some(span1));
        assert_eq!(loaded.get_span_for_offset(1), Some(span1)); // Operand byte
        assert_eq!(loaded.get_span_for_offset(2), Some(span1)); // Operand byte
        assert_eq!(loaded.get_span_for_offset(3), Some(span2));
    }

    #[test]
    fn test_debug_info_default_emit_behavior() {
        // Test that using emit() automatically tracks debug info
        let mut bytecode = Bytecode::new();

        // emit() should automatically add debug info
        bytecode.emit(Opcode::True, Span::new(5, 9));

        assert_eq!(bytecode.debug_info.len(), 1);
        assert_eq!(bytecode.debug_info[0].instruction_offset, 0);
        assert_eq!(bytecode.debug_info[0].span, Span::new(5, 9));

        // Verify it's included in serialization by default
        let bytes = bytecode.to_bytes();
        let flags = u16::from_be_bytes([bytes[6], bytes[7]]);
        assert_eq!(flags & 1, 1, "Debug flag should be set when using emit()");
    }

    // ===== Bytecode Format Tests (Phase 16) =====

    #[test]
    fn test_bytecode_format_header() {
        // Test that header format matches specification
        let bytecode = Bytecode::new();
        let bytes = bytecode.to_bytes();

        // Magic number: "ATB\0" (4 bytes)
        assert_eq!(&bytes[0..4], b"ATB\0", "Magic number should be ATB\\0");

        // Version: u16 (bytes 4-5)
        let version = u16::from_be_bytes([bytes[4], bytes[5]]);
        assert_eq!(version, BYTECODE_VERSION, "Version should match BYTECODE_VERSION");

        // Flags: u16 (bytes 6-7)
        let flags = u16::from_be_bytes([bytes[6], bytes[7]]);
        assert_eq!(flags, 0, "Empty bytecode should have flags = 0");
    }

    #[test]
    fn test_bytecode_format_minimal() {
        // Test minimal valid bytecode (just Halt instruction)
        let mut bytecode = Bytecode::new();
        bytecode.instructions.push(Opcode::Halt as u8);

        let bytes = bytecode.to_bytes();

        // Header (8 bytes)
        assert_eq!(&bytes[0..4], b"ATB\0");
        assert_eq!(u16::from_be_bytes([bytes[4], bytes[5]]), BYTECODE_VERSION);
        assert_eq!(u16::from_be_bytes([bytes[6], bytes[7]]), 0); // No debug info

        // Constants count (4 bytes) = 0
        assert_eq!(u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]), 0);

        // Instructions length (4 bytes) = 1
        assert_eq!(u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]), 1);

        // Halt opcode
        assert_eq!(bytes[16], Opcode::Halt as u8);

        // Total size should be 8 (header) + 4 (const count) + 4 (instr len) + 1 (halt) = 17
        assert_eq!(bytes.len(), 17);
    }

    #[test]
    fn test_bytecode_format_with_constants() {
        // Test bytecode with constant pool
        let mut bytecode = Bytecode::new();
        bytecode.add_constant(Value::Number(42.0));
        bytecode.add_constant(Value::string("hello"));
        bytecode.add_constant(Value::Bool(true));
        bytecode.add_constant(Value::Null);

        let bytes = bytecode.to_bytes();

        // Skip header (8 bytes)
        let mut offset = 8;

        // Constants count should be 4
        let const_count = u32::from_be_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]);
        assert_eq!(const_count, 4);
        offset += 4;

        // Verify each constant format
        // Constant 0: Number (tag 0x02 + f64)
        assert_eq!(bytes[offset], 0x02); // Number tag
        offset += 1;
        let num = f64::from_be_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        assert_eq!(num, 42.0);
        offset += 8;

        // Constant 1: String (tag 0x03 + u32 len + bytes)
        assert_eq!(bytes[offset], 0x03); // String tag
        offset += 1;
        let str_len = u32::from_be_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]) as usize;
        assert_eq!(str_len, 5);
        offset += 4;
        assert_eq!(&bytes[offset..offset + 5], b"hello");
        offset += 5;

        // Constant 2: Bool (tag 0x01 + u8)
        assert_eq!(bytes[offset], 0x01); // Bool tag
        offset += 1;
        assert_eq!(bytes[offset], 1); // true
        offset += 1;

        // Constant 3: Null (tag 0x00)
        assert_eq!(bytes[offset], 0x00); // Null tag
    }

    #[test]
    fn test_bytecode_format_roundtrip_instructions() {
        // Test that various instruction patterns survive roundtrip
        let mut bytecode = Bytecode::new();

        // Add some constants
        let idx0 = bytecode.add_constant(Value::Number(10.0));
        let idx1 = bytecode.add_constant(Value::Number(20.0));

        // Emit various instructions
        bytecode.emit(Opcode::Constant, Span::new(0, 2));
        bytecode.emit_u16(idx0);
        bytecode.emit(Opcode::Constant, Span::new(3, 5));
        bytecode.emit_u16(idx1);
        bytecode.emit(Opcode::Add, Span::new(6, 7));
        bytecode.emit(Opcode::Dup, Span::new(8, 9));
        bytecode.emit(Opcode::Pop, Span::new(10, 11));
        bytecode.emit(Opcode::Halt, Span::new(12, 13));

        // Serialize and deserialize
        let bytes = bytecode.to_bytes();
        let loaded = Bytecode::from_bytes(&bytes).unwrap();

        // Verify instructions match exactly
        assert_eq!(loaded.instructions, bytecode.instructions);
        assert_eq!(loaded.constants.len(), bytecode.constants.len());
        assert_eq!(loaded.debug_info.len(), bytecode.debug_info.len());
    }

    #[test]
    fn test_bytecode_format_large_constant_pool() {
        // Test bytecode with many constants
        let mut bytecode = Bytecode::new();

        for i in 0..1000 {
            bytecode.add_constant(Value::Number(i as f64));
        }

        let bytes = bytecode.to_bytes();
        let loaded = Bytecode::from_bytes(&bytes).unwrap();

        assert_eq!(loaded.constants.len(), 1000);
        assert_eq!(loaded.constants[500], Value::Number(500.0));
        assert_eq!(loaded.constants[999], Value::Number(999.0));
    }

    #[test]
    fn test_bytecode_format_all_value_types() {
        use std::rc::Rc;
        // Test that all serializable value types work
        let mut bytecode = Bytecode::new();

        bytecode.add_constant(Value::Null);
        bytecode.add_constant(Value::Bool(true));
        bytecode.add_constant(Value::Bool(false));
        bytecode.add_constant(Value::Number(3.14159));
        bytecode.add_constant(Value::Number(-273.15));
        bytecode.add_constant(Value::Number(0.0));
        bytecode.add_constant(Value::String(Rc::new("".to_string())));
        bytecode.add_constant(Value::String(Rc::new("test".to_string())));
        bytecode.add_constant(Value::Function(crate::value::FunctionRef {
            name: "myFunc".to_string(),
            arity: 3,
            bytecode_offset: 42,
            local_count: 0,
        }));

        let bytes = bytecode.to_bytes();
        let loaded = Bytecode::from_bytes(&bytes).unwrap();

        assert_eq!(loaded.constants.len(), 9);
        assert_eq!(loaded.constants[0], Value::Null);
        assert_eq!(loaded.constants[1], Value::Bool(true));
        assert_eq!(loaded.constants[2], Value::Bool(false));
        assert_eq!(loaded.constants[3], Value::Number(3.14159));
        assert_eq!(loaded.constants[4], Value::Number(-273.15));
        assert_eq!(loaded.constants[5], Value::Number(0.0));
        assert_eq!(loaded.constants[6], Value::string(""));
        assert_eq!(loaded.constants[7], Value::string("test"));

        if let Value::Function(f) = &loaded.constants[8] {
            assert_eq!(f.name, "myFunc");
            assert_eq!(f.arity, 3);
            assert_eq!(f.bytecode_offset, 42);
        } else {
            panic!("Expected function constant");
        }
    }

    #[test]
    fn test_bytecode_format_version_check() {
        // Test that version checking works correctly
        let bytecode = Bytecode::new();
        let mut bytes = bytecode.to_bytes();

        // Modify version to something else
        bytes[4] = 0xFF;
        bytes[5] = 0xFF;

        let result = Bytecode::from_bytes(&bytes);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("version mismatch"));
        assert!(err.contains("65535")); // 0xFFFF in decimal
    }

    #[test]
    fn test_bytecode_format_version_too_old() {
        // Simulate bytecode from version 0 (too old)
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"ATB\0");
        bytes.extend_from_slice(&0u16.to_be_bytes()); // Version 0
        bytes.extend_from_slice(&0u16.to_be_bytes()); // Flags
        bytes.extend_from_slice(&0u32.to_be_bytes()); // Constants count
        bytes.extend_from_slice(&0u32.to_be_bytes()); // Instructions length

        let result = Bytecode::from_bytes(&bytes);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("version mismatch"));
        assert!(err.contains("file has version 0"));
    }

    #[test]
    fn test_bytecode_format_corrupted_magic() {
        // Test various corrupted magic numbers
        let test_cases = vec![
            b"XTB\0", // Wrong first byte
            b"AXB\0", // Wrong second byte
            b"ATX\0", // Wrong third byte
            b"ATB1", // Wrong fourth byte (should be null)
            b"atb\0", // Wrong case
        ];

        for &magic in &test_cases {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(magic);
            bytes.extend_from_slice(&1u16.to_be_bytes());
            bytes.extend_from_slice(&0u16.to_be_bytes());
            bytes.extend_from_slice(&0u32.to_be_bytes());
            bytes.extend_from_slice(&0u32.to_be_bytes());

            let result = Bytecode::from_bytes(&bytes);
            assert!(result.is_err(), "Should reject magic {:?}", magic);
            assert!(result.unwrap_err().contains("bad magic number"));
        }
    }

    #[test]
    fn test_bytecode_format_truncated_header() {
        // Test various truncated headers
        for len in 0..8 {
            let bytes = vec![0u8; len];
            let result = Bytecode::from_bytes(&bytes);
            assert!(result.is_err(), "Should reject {} byte header", len);
            assert!(result.unwrap_err().contains("too short"));
        }
    }

    #[test]
    fn test_bytecode_format_truncated_constants() {
        // Header is valid, but constants section is truncated
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"ATB\0");
        bytes.extend_from_slice(&BYTECODE_VERSION.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes());
        bytes.extend_from_slice(&2u32.to_be_bytes()); // Claims 2 constants
        // But no constant data follows

        let result = Bytecode::from_bytes(&bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unexpected end of data"));
    }

    #[test]
    fn test_bytecode_format_truncated_instructions() {
        // Constants section is valid, but instructions section is truncated
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"ATB\0");
        bytes.extend_from_slice(&BYTECODE_VERSION.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes());
        bytes.extend_from_slice(&0u32.to_be_bytes()); // 0 constants
        bytes.extend_from_slice(&10u32.to_be_bytes()); // Claims 10 instruction bytes
        bytes.extend_from_slice(&[0x01, 0x02]); // But only 2 bytes follow

        let result = Bytecode::from_bytes(&bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("truncated"));
    }

    #[test]
    fn test_bytecode_format_exact_size_validation() {
        // Test that deserializer requires exact byte count
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Halt, Span::new(0, 1));

        let mut bytes = bytecode.to_bytes();

        // Add extra garbage bytes
        bytes.push(0xFF);
        bytes.push(0xEE);

        let result = Bytecode::from_bytes(&bytes);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("expected") && err.contains("consumed"));
    }

    #[test]
    fn test_bytecode_format_complex_program() {
        // Test a realistic complex program
        let mut bytecode = Bytecode::new();

        // Add constants for: let x = 10; let y = 20; let z = x + y;
        let num10 = bytecode.add_constant(Value::Number(10.0));
        let var_x = bytecode.add_constant(Value::string("x"));
        let num20 = bytecode.add_constant(Value::Number(20.0));
        let var_y = bytecode.add_constant(Value::string("y"));
        let var_z = bytecode.add_constant(Value::string("z"));

        // let x = 10
        bytecode.emit(Opcode::Constant, Span::new(0, 10));
        bytecode.emit_u16(num10);
        bytecode.emit(Opcode::SetGlobal, Span::new(0, 10));
        bytecode.emit_u16(var_x);
        bytecode.emit(Opcode::Pop, Span::new(0, 10));

        // let y = 20
        bytecode.emit(Opcode::Constant, Span::new(11, 21));
        bytecode.emit_u16(num20);
        bytecode.emit(Opcode::SetGlobal, Span::new(11, 21));
        bytecode.emit_u16(var_y);
        bytecode.emit(Opcode::Pop, Span::new(11, 21));

        // let z = x + y
        bytecode.emit(Opcode::GetGlobal, Span::new(22, 32));
        bytecode.emit_u16(var_x);
        bytecode.emit(Opcode::GetGlobal, Span::new(22, 32));
        bytecode.emit_u16(var_y);
        bytecode.emit(Opcode::Add, Span::new(22, 32));
        bytecode.emit(Opcode::SetGlobal, Span::new(22, 32));
        bytecode.emit_u16(var_z);
        bytecode.emit(Opcode::Pop, Span::new(22, 32));

        bytecode.emit(Opcode::Halt, Span::new(33, 34));

        // Round-trip test
        let bytes = bytecode.to_bytes();
        let loaded = Bytecode::from_bytes(&bytes).unwrap();

        // Verify all components match
        assert_eq!(loaded.instructions, bytecode.instructions);
        assert_eq!(loaded.constants.len(), bytecode.constants.len());
        assert_eq!(loaded.debug_info.len(), bytecode.debug_info.len());

        // Verify specific constants
        assert_eq!(loaded.constants[0], Value::Number(10.0));
        assert_eq!(loaded.constants[1], Value::string("x"));
        assert_eq!(loaded.constants[2], Value::Number(20.0));
        assert_eq!(loaded.constants[3], Value::string("y"));
        assert_eq!(loaded.constants[4], Value::string("z"));
    }
}
