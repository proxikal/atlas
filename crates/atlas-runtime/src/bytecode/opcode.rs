//! Bytecode instruction set
//!
//! Stack-based bytecode with 30 opcodes organized by category.
//! Operands are encoded separately in the instruction stream.

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
