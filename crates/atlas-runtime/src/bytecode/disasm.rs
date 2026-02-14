//! Bytecode disassembler
//!
//! Converts bytecode back to human-readable assembly-like format.
//! Used for debugging, testing, and `atlas build --disasm` output.

use super::{Bytecode, Opcode};
use std::fmt::Write;

/// Disassemble bytecode to human-readable format
///
/// # Format
/// ```text
/// === Constants ===
/// 0: 42.0
/// 1: "hello"
///
/// === Instructions ===
/// 0000  Constant 0
/// 0003  Add
/// 0004  Halt
/// ```
pub fn disassemble(bytecode: &Bytecode) -> String {
    let mut output = String::new();

    // Constants section
    if !bytecode.constants.is_empty() {
        writeln!(output, "=== Constants ===").unwrap();
        for (idx, constant) in bytecode.constants.iter().enumerate() {
            writeln!(output, "{}: {}", idx, format_value(constant)).unwrap();
        }
        writeln!(output).unwrap();
    }

    // Instructions section
    writeln!(output, "=== Instructions ===").unwrap();
    let mut offset = 0;
    while offset < bytecode.instructions.len() {
        let line = disassemble_instruction(bytecode, &mut offset);
        writeln!(output, "{}", line).unwrap();
    }

    output
}

/// Disassemble a single instruction at the given offset
///
/// Advances offset past the instruction and its operands.
/// Returns formatted instruction string.
fn disassemble_instruction(bytecode: &Bytecode, offset: &mut usize) -> String {
    let start_offset = *offset;

    // Read opcode
    if *offset >= bytecode.instructions.len() {
        return format!("{:04}  <invalid offset>", start_offset);
    }

    let byte = bytecode.instructions[*offset];
    *offset += 1;

    let opcode = match Opcode::try_from(byte) {
        Ok(op) => op,
        Err(_) => return format!("{:04}  <invalid opcode: {:#04x}>", start_offset, byte),
    };

    // Format based on opcode type
    match opcode {
        // Simple opcodes (no operands)
        Opcode::Null
        | Opcode::True
        | Opcode::False
        | Opcode::Add
        | Opcode::Sub
        | Opcode::Mul
        | Opcode::Div
        | Opcode::Mod
        | Opcode::Negate
        | Opcode::Equal
        | Opcode::NotEqual
        | Opcode::Less
        | Opcode::LessEqual
        | Opcode::Greater
        | Opcode::GreaterEqual
        | Opcode::Not
        | Opcode::And
        | Opcode::Or
        | Opcode::Return
        | Opcode::GetIndex
        | Opcode::SetIndex
        | Opcode::Pop
        | Opcode::Dup
        | Opcode::IsOptionSome
        | Opcode::IsOptionNone
        | Opcode::IsResultOk
        | Opcode::IsResultErr
        | Opcode::ExtractOptionValue
        | Opcode::ExtractResultValue
        | Opcode::IsArray
        | Opcode::GetArrayLen
        | Opcode::Halt => {
            format!("{:04}  {:?}", start_offset, opcode)
        }

        // u16 operands (constants, locals, globals)
        Opcode::Constant
        | Opcode::GetLocal
        | Opcode::SetLocal
        | Opcode::GetGlobal
        | Opcode::SetGlobal
        | Opcode::Array => {
            let operand = read_u16(bytecode, offset);
            format!("{:04}  {:?} {}", start_offset, opcode, operand)
        }

        // u8 operand (call arg count)
        Opcode::Call => {
            let operand = read_u8(bytecode, offset);
            format!("{:04}  {:?} {}", start_offset, opcode, operand)
        }

        // i16 operands (jumps)
        Opcode::Jump | Opcode::JumpIfFalse | Opcode::Loop => {
            let jump_offset = read_i16(bytecode, offset);
            let target = (*offset as i32 + jump_offset as i32) as usize;
            format!(
                "{:04}  {:?} {} (-> {:04})",
                start_offset, opcode, jump_offset, target
            )
        }
    }
}

/// Read u8 operand from bytecode
fn read_u8(bytecode: &Bytecode, offset: &mut usize) -> u8 {
    if *offset >= bytecode.instructions.len() {
        return 0;
    }
    let value = bytecode.instructions[*offset];
    *offset += 1;
    value
}

/// Read u16 operand from bytecode (big-endian)
fn read_u16(bytecode: &Bytecode, offset: &mut usize) -> u16 {
    if *offset + 1 >= bytecode.instructions.len() {
        return 0;
    }
    let high = bytecode.instructions[*offset] as u16;
    let low = bytecode.instructions[*offset + 1] as u16;
    *offset += 2;
    (high << 8) | low
}

/// Read i16 operand from bytecode (big-endian, signed)
fn read_i16(bytecode: &Bytecode, offset: &mut usize) -> i16 {
    read_u16(bytecode, offset) as i16
}

/// Format a Value for constant pool display
fn format_value(value: &crate::value::Value) -> String {
    use crate::value::Value;
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            // Show integers without decimal point
            if n.fract() == 0.0 && n.is_finite() {
                format!("{:.0}", n)
            } else {
                n.to_string()
            }
        }
        Value::String(s) => format!("\"{}\"", s),
        Value::Function(f) => format!("<fn {}({})>", f.name, f.arity),
        Value::Array(_) => "<array>".to_string(),
        Value::JsonValue(_) => "<json>".to_string(),
        Value::Option(_) => "<option>".to_string(),
        Value::Result(_) => "<result>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    #[test]
    fn test_disassemble_empty() {
        let bytecode = Bytecode::new();
        let output = disassemble(&bytecode);
        assert!(output.contains("=== Instructions ==="));
    }

    #[test]
    fn test_disassemble_simple_opcodes() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Null, Span::dummy());
        bytecode.emit(Opcode::True, Span::dummy());
        bytecode.emit(Opcode::Add, Span::dummy());
        bytecode.emit(Opcode::Halt, Span::dummy());

        let output = disassemble(&bytecode);
        assert!(output.contains("0000  Null"));
        assert!(output.contains("0001  True"));
        assert!(output.contains("0002  Add"));
        assert!(output.contains("0003  Halt"));
    }

    #[test]
    fn test_disassemble_with_constants() {
        let mut bytecode = Bytecode::new();
        let idx = bytecode.add_constant(crate::value::Value::Number(42.0));
        bytecode.emit(Opcode::Constant, Span::dummy());
        bytecode.emit_u16(idx);
        bytecode.emit(Opcode::Halt, Span::dummy());

        let output = disassemble(&bytecode);
        assert!(output.contains("=== Constants ==="));
        assert!(output.contains("0: 42"));
        assert!(output.contains("0000  Constant 0"));
        assert!(output.contains("0003  Halt"));
    }

    #[test]
    fn test_disassemble_locals() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::GetLocal, Span::dummy());
        bytecode.emit_u16(5);
        bytecode.emit(Opcode::SetLocal, Span::dummy());
        bytecode.emit_u16(10);

        let output = disassemble(&bytecode);
        assert!(output.contains("0000  GetLocal 5"));
        assert!(output.contains("0003  SetLocal 10"));
    }

    #[test]
    fn test_disassemble_jumps() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Jump, Span::dummy());
        bytecode.emit_i16(10);
        bytecode.emit(Opcode::JumpIfFalse, Span::dummy());
        bytecode.emit_i16(-5);

        let output = disassemble(&bytecode);
        assert!(output.contains("Jump 10"));
        assert!(output.contains("-> 0013")); // offset 3 + 10 = 13
        assert!(output.contains("JumpIfFalse -5"));
        assert!(output.contains("-> 0001")); // offset 6 - 5 = 1
    }

    #[test]
    fn test_disassemble_call() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Call, Span::dummy());
        bytecode.emit_u8(3);

        let output = disassemble(&bytecode);
        assert!(output.contains("0000  Call 3"));
    }

    #[test]
    fn test_disassemble_array() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Array, Span::dummy());
        bytecode.emit_u16(5);

        let output = disassemble(&bytecode);
        assert!(output.contains("0000  Array 5"));
    }

    #[test]
    fn test_format_value_number() {
        use crate::value::Value;
        assert_eq!(format_value(&Value::Number(42.0)), "42");
        assert_eq!(format_value(&Value::Number(3.14)), "3.14");
    }

    #[test]
    fn test_format_value_string() {
        use crate::value::Value;
        assert_eq!(format_value(&Value::string("hello")), "\"hello\"");
    }

    #[test]
    fn test_format_value_bool() {
        use crate::value::Value;
        assert_eq!(format_value(&Value::Bool(true)), "true");
        assert_eq!(format_value(&Value::Bool(false)), "false");
    }

    #[test]
    fn test_disassemble_complete_sequence() {
        // Test: let x = 2 + 3;
        let mut bytecode = Bytecode::new();
        let idx_2 = bytecode.add_constant(crate::value::Value::Number(2.0));
        let idx_3 = bytecode.add_constant(crate::value::Value::Number(3.0));

        bytecode.emit(Opcode::Constant, Span::dummy());
        bytecode.emit_u16(idx_2);
        bytecode.emit(Opcode::Constant, Span::dummy());
        bytecode.emit_u16(idx_3);
        bytecode.emit(Opcode::Add, Span::dummy());
        bytecode.emit(Opcode::SetLocal, Span::dummy());
        bytecode.emit_u16(0);
        bytecode.emit(Opcode::Halt, Span::dummy());

        let output = disassemble(&bytecode);

        // Check constants section
        assert!(output.contains("=== Constants ==="));
        assert!(output.contains("0: 2"));
        assert!(output.contains("1: 3"));

        // Check instructions section
        assert!(output.contains("0000  Constant 0"));
        assert!(output.contains("0003  Constant 1"));
        assert!(output.contains("0006  Add"));
        assert!(output.contains("0007  SetLocal 0"));
        assert!(output.contains("0010  Halt"));
    }
}
