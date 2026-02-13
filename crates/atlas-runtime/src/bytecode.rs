//! Bytecode instruction set
//!
//! Stack-based bytecode with 30 opcodes organized by category.
//! Operands are encoded separately in the instruction stream.

use crate::span::Span;
use crate::value::Value;

/// Bytecode opcode (30 instructions)
///
/// Stack-based VM with explicit byte values for serialization.
/// Operands are encoded inline after the opcode byte.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // ===== Constants (0x01-0x0F) =====
    /// Push constant from pool [u16 index]
    Constant = 0x01,
    /// Push null
    Null = 0x02,
    /// Push true
    True = 0x03,
    /// Push false
    False = 0x04,

    // ===== Variables (0x10-0x1F) =====
    /// Load local variable [u16 index]
    GetLocal = 0x10,
    /// Store to local variable [u16 index]
    SetLocal = 0x11,
    /// Load global variable [u16 name_index]
    GetGlobal = 0x12,
    /// Store to global variable [u16 name_index]
    SetGlobal = 0x13,

    // ===== Arithmetic (0x20-0x2F) =====
    /// Pop b, pop a, push a + b
    Add = 0x20,
    /// Pop b, pop a, push a - b
    Sub = 0x21,
    /// Pop b, pop a, push a * b
    Mul = 0x22,
    /// Pop b, pop a, push a / b
    Div = 0x23,
    /// Pop b, pop a, push a % b
    Mod = 0x24,
    /// Pop a, push -a
    Negate = 0x25,

    // ===== Comparison (0x30-0x3F) =====
    /// Pop b, pop a, push a == b
    Equal = 0x30,
    /// Pop b, pop a, push a != b
    NotEqual = 0x31,
    /// Pop b, pop a, push a < b
    Less = 0x32,
    /// Pop b, pop a, push a <= b
    LessEqual = 0x33,
    /// Pop b, pop a, push a > b
    Greater = 0x34,
    /// Pop b, pop a, push a >= b
    GreaterEqual = 0x35,

    // ===== Logical (0x40-0x4F) =====
    /// Pop a, push !a
    Not = 0x40,
    /// Short-circuit: if TOS is false, skip next instruction
    And = 0x41,
    /// Short-circuit: if TOS is true, skip next instruction
    Or = 0x42,

    // ===== Control flow (0x50-0x5F) =====
    /// Unconditional jump [i16 offset]
    Jump = 0x50,
    /// Pop condition, jump if false [i16 offset]
    JumpIfFalse = 0x51,
    /// Jump backward [i16 offset]
    Loop = 0x52,

    // ===== Functions (0x60-0x6F) =====
    /// Call function [u8 arg_count]
    Call = 0x60,
    /// Return from function
    Return = 0x61,

    // ===== Arrays (0x70-0x7F) =====
    /// Create array [u16 size] from stack
    Array = 0x70,
    /// Pop index, pop array, push array[index]
    GetIndex = 0x71,
    /// Pop value, pop index, pop array, array[index] = value
    SetIndex = 0x72,

    // ===== Stack manipulation (0x80-0x8F) =====
    /// Pop and discard top of stack
    Pop = 0x80,
    /// Duplicate top of stack
    Dup = 0x81,

    // ===== Special (0xF0-0xFF) =====
    /// End of bytecode
    Halt = 0xFF,
}

impl TryFrom<u8> for Opcode {
    type Error = ();

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0x01 => Ok(Opcode::Constant),
            0x02 => Ok(Opcode::Null),
            0x03 => Ok(Opcode::True),
            0x04 => Ok(Opcode::False),
            0x10 => Ok(Opcode::GetLocal),
            0x11 => Ok(Opcode::SetLocal),
            0x12 => Ok(Opcode::GetGlobal),
            0x13 => Ok(Opcode::SetGlobal),
            0x20 => Ok(Opcode::Add),
            0x21 => Ok(Opcode::Sub),
            0x22 => Ok(Opcode::Mul),
            0x23 => Ok(Opcode::Div),
            0x24 => Ok(Opcode::Mod),
            0x25 => Ok(Opcode::Negate),
            0x30 => Ok(Opcode::Equal),
            0x31 => Ok(Opcode::NotEqual),
            0x32 => Ok(Opcode::Less),
            0x33 => Ok(Opcode::LessEqual),
            0x34 => Ok(Opcode::Greater),
            0x35 => Ok(Opcode::GreaterEqual),
            0x40 => Ok(Opcode::Not),
            0x41 => Ok(Opcode::And),
            0x42 => Ok(Opcode::Or),
            0x50 => Ok(Opcode::Jump),
            0x51 => Ok(Opcode::JumpIfFalse),
            0x52 => Ok(Opcode::Loop),
            0x60 => Ok(Opcode::Call),
            0x61 => Ok(Opcode::Return),
            0x70 => Ok(Opcode::Array),
            0x71 => Ok(Opcode::GetIndex),
            0x72 => Ok(Opcode::SetIndex),
            0x80 => Ok(Opcode::Pop),
            0x81 => Ok(Opcode::Dup),
            0xFF => Ok(Opcode::Halt),
            _ => Err(()),
        }
    }
}

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

/// Serialize a Value to bytes
fn serialize_value(value: &Value, bytes: &mut Vec<u8>) {
    match value {
        Value::Null => {
            bytes.push(0x00); // Type tag
        }
        Value::Bool(b) => {
            bytes.push(0x01); // Type tag
            bytes.push(if *b { 1 } else { 0 });
        }
        Value::Number(n) => {
            bytes.push(0x02); // Type tag
            bytes.extend_from_slice(&n.to_be_bytes());
        }
        Value::String(s) => {
            bytes.push(0x03); // Type tag
            let s_bytes = s.as_bytes();
            bytes.extend_from_slice(&(s_bytes.len() as u32).to_be_bytes());
            bytes.extend_from_slice(s_bytes);
        }
        Value::Function(func) => {
            bytes.push(0x04); // Type tag
            // Serialize function name
            let name_bytes = func.name.as_bytes();
            bytes.extend_from_slice(&(name_bytes.len() as u32).to_be_bytes());
            bytes.extend_from_slice(name_bytes);
            // Serialize arity
            bytes.push(func.arity as u8);
            // Serialize bytecode offset
            bytes.extend_from_slice(&(func.bytecode_offset as u32).to_be_bytes());
        }
        Value::Array(_) => {
            // Arrays cannot be serialized in constant pool
            // They are runtime-only values
            panic!("Cannot serialize array values in bytecode constants");
        }
    }
}

/// Deserialize a Value from bytes, returns (Value, bytes_consumed)
fn deserialize_value(bytes: &[u8]) -> Result<(Value, usize), String> {
    if bytes.is_empty() {
        return Err("Unexpected end of data while reading value".to_string());
    }

    let tag = bytes[0];
    match tag {
        0x00 => Ok((Value::Null, 1)),
        0x01 => {
            if bytes.len() < 2 {
                return Err("Truncated bool value".to_string());
            }
            Ok((Value::Bool(bytes[1] != 0), 2))
        }
        0x02 => {
            if bytes.len() < 9 {
                return Err("Truncated number value".to_string());
            }
            let num_bytes: [u8; 8] = bytes[1..9].try_into().unwrap();
            Ok((Value::Number(f64::from_be_bytes(num_bytes)), 9))
        }
        0x03 => {
            if bytes.len() < 5 {
                return Err("Truncated string value".to_string());
            }
            let len = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
            if bytes.len() < 5 + len {
                return Err("Truncated string data".to_string());
            }
            let s = String::from_utf8(bytes[5..5 + len].to_vec())
                .map_err(|e| format!("Invalid UTF-8 in string: {}", e))?;
            Ok((Value::string(&s), 5 + len))
        }
        0x04 => {
            if bytes.len() < 5 {
                return Err("Truncated function value".to_string());
            }
            let name_len = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
            if bytes.len() < 5 + name_len + 1 + 4 {
                return Err("Truncated function data".to_string());
            }
            let name = String::from_utf8(bytes[5..5 + name_len].to_vec())
                .map_err(|e| format!("Invalid UTF-8 in function name: {}", e))?;
            let arity = bytes[5 + name_len] as usize;
            let offset = u32::from_be_bytes([
                bytes[6 + name_len],
                bytes[7 + name_len],
                bytes[8 + name_len],
                bytes[9 + name_len],
            ]) as usize;
            Ok((
                Value::Function(crate::value::FunctionRef {
                    name,
                    arity,
                    bytecode_offset: offset,
                }),
                10 + name_len,
            ))
        }
        _ => Err(format!("Unknown value type tag: {:#x}", tag)),
    }
}

/// Serialize a Span to bytes
fn serialize_span(span: &Span, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&(span.start as u32).to_be_bytes());
    bytes.extend_from_slice(&(span.end as u32).to_be_bytes());
}

/// Deserialize a Span from bytes, returns (Span, bytes_consumed)
fn deserialize_span(bytes: &[u8]) -> Result<(Span, usize), String> {
    if bytes.len() < 8 {
        return Err("Truncated span data".to_string());
    }
    let start = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    let end = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    Ok((Span { start, end }, 8))
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
        bytes.extend_from_slice(&1u16.to_be_bytes()); // Version
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
        let mut offset = 0;

        // Read header
        if bytes.len() < 8 {
            return Err("Invalid bytecode file: too short".to_string());
        }
        if &bytes[0..4] != b"ATB\0" {
            return Err("Invalid bytecode file: bad magic number".to_string());
        }
        let version = u16::from_be_bytes([bytes[4], bytes[5]]);
        if version != 1 {
            return Err(format!("Unsupported bytecode version: {}", version));
        }
        let flags = u16::from_be_bytes([bytes[6], bytes[7]]);
        let has_debug_info = (flags & 1) != 0;
        offset = 8;

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
        assert!(result.unwrap_err().contains("Unsupported bytecode version"));
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
