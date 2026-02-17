//! Optimized instruction dispatch for the VM
//!
//! Uses a static lookup table for O(1) opcode decoding instead of
//! match-based dispatch, reducing branch mispredictions in the hot loop.

use crate::bytecode::Opcode;

/// Static dispatch table mapping byte values to optional Opcodes.
/// Indexed by the raw u8 opcode byte for O(1) lookup.
static OPCODE_TABLE: [Option<Opcode>; 256] = {
    let mut table: [Option<Opcode>; 256] = [None; 256];

    // Constants (0x01-0x04)
    table[0x01] = Some(Opcode::Constant);
    table[0x02] = Some(Opcode::Null);
    table[0x03] = Some(Opcode::True);
    table[0x04] = Some(Opcode::False);

    // Variables (0x10-0x13)
    table[0x10] = Some(Opcode::GetLocal);
    table[0x11] = Some(Opcode::SetLocal);
    table[0x12] = Some(Opcode::GetGlobal);
    table[0x13] = Some(Opcode::SetGlobal);

    // Arithmetic (0x20-0x25)
    table[0x20] = Some(Opcode::Add);
    table[0x21] = Some(Opcode::Sub);
    table[0x22] = Some(Opcode::Mul);
    table[0x23] = Some(Opcode::Div);
    table[0x24] = Some(Opcode::Mod);
    table[0x25] = Some(Opcode::Negate);

    // Comparison (0x30-0x35)
    table[0x30] = Some(Opcode::Equal);
    table[0x31] = Some(Opcode::NotEqual);
    table[0x32] = Some(Opcode::Less);
    table[0x33] = Some(Opcode::LessEqual);
    table[0x34] = Some(Opcode::Greater);
    table[0x35] = Some(Opcode::GreaterEqual);

    // Logical (0x40-0x42)
    table[0x40] = Some(Opcode::Not);
    table[0x41] = Some(Opcode::And);
    table[0x42] = Some(Opcode::Or);

    // Control flow (0x50-0x52)
    table[0x50] = Some(Opcode::Jump);
    table[0x51] = Some(Opcode::JumpIfFalse);
    table[0x52] = Some(Opcode::Loop);

    // Functions (0x60-0x61)
    table[0x60] = Some(Opcode::Call);
    table[0x61] = Some(Opcode::Return);

    // Arrays (0x70-0x72)
    table[0x70] = Some(Opcode::Array);
    table[0x71] = Some(Opcode::GetIndex);
    table[0x72] = Some(Opcode::SetIndex);

    // Stack manipulation (0x80-0x81)
    table[0x80] = Some(Opcode::Pop);
    table[0x81] = Some(Opcode::Dup);

    // Pattern matching (0x90-0x97)
    table[0x90] = Some(Opcode::IsOptionSome);
    table[0x91] = Some(Opcode::IsOptionNone);
    table[0x92] = Some(Opcode::IsResultOk);
    table[0x93] = Some(Opcode::IsResultErr);
    table[0x94] = Some(Opcode::ExtractOptionValue);
    table[0x95] = Some(Opcode::ExtractResultValue);
    table[0x96] = Some(Opcode::IsArray);
    table[0x97] = Some(Opcode::GetArrayLen);

    // Special
    table[0xFF] = Some(Opcode::Halt);

    table
};

/// Decode an opcode byte using the static lookup table.
/// Returns None for invalid opcode bytes.
#[inline(always)]
pub fn decode_opcode(byte: u8) -> Option<Opcode> {
    unsafe { *OPCODE_TABLE.get_unchecked(byte as usize) }
}

/// Returns the number of operand bytes following an opcode.
/// Used for instruction-length-aware operations (disassembly, skipping).
#[inline(always)]
pub fn operand_size(opcode: Opcode) -> usize {
    match opcode {
        // u16 operand
        Opcode::Constant
        | Opcode::GetLocal
        | Opcode::SetLocal
        | Opcode::GetGlobal
        | Opcode::SetGlobal
        | Opcode::Array => 2,
        // i16 operand
        Opcode::Jump | Opcode::JumpIfFalse | Opcode::Loop => 2,
        // u8 operand
        Opcode::Call => 1,
        // No operand
        _ => 0,
    }
}
