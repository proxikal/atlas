//! Bytecode instruction set
//!
//! Stack-based bytecode with 30 opcodes organized by category.
//! Operands are encoded separately in the instruction stream.

mod opcode;
mod serialize;

pub use opcode::Opcode;
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
}
