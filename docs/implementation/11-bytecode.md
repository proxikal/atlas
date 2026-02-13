# Bytecode Instruction Set

Complete 30-opcode definition for stack-based VM.

## Opcode Enum

```rust
// bytecode/ module (bytecode/mod.rs + bytecode/opcode.rs + bytecode/serialize.rs)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // Constants (0x01-0x0F)
    Constant = 0x01,     // Push constant from pool [u16 index]
    Null = 0x02,         // Push null
    True = 0x03,         // Push true
    False = 0x04,        // Push false

    // Variables (0x10-0x1F)
    GetLocal = 0x10,     // Load local variable [u16 index]
    SetLocal = 0x11,     // Store to local variable [u16 index]
    GetGlobal = 0x12,    // Load global variable [u16 name_index]
    SetGlobal = 0x13,    // Store to global variable [u16 name_index]

    // Arithmetic (0x20-0x2F)
    Add = 0x20,          // Pop b, pop a, push a + b
    Sub = 0x21,          // Pop b, pop a, push a - b
    Mul = 0x22,          // Pop b, pop a, push a * b
    Div = 0x23,          // Pop b, pop a, push a / b
    Mod = 0x24,          // Pop b, pop a, push a % b
    Negate = 0x25,       // Pop a, push -a

    // Comparison (0x30-0x3F)
    Equal = 0x30,        // Pop b, pop a, push a == b
    NotEqual = 0x31,     // Pop b, pop a, push a != b
    Less = 0x32,         // Pop b, pop a, push a < b
    LessEqual = 0x33,    // Pop b, pop a, push a <= b
    Greater = 0x34,      // Pop b, pop a, push a > b
    GreaterEqual = 0x35, // Pop b, pop a, push a >= b

    // Logical (0x40-0x4F)
    Not = 0x40,          // Pop a, push !a
    And = 0x41,          // Short-circuit: if TOS is false, skip next instruction
    Or = 0x42,           // Short-circuit: if TOS is true, skip next instruction

    // Control flow (0x50-0x5F)
    Jump = 0x50,         // Unconditional jump [i16 offset]
    JumpIfFalse = 0x51,  // Pop condition, jump if false [i16 offset]
    Loop = 0x52,         // Jump backward [i16 offset]

    // Functions (0x60-0x6F)
    Call = 0x60,         // Call function [u8 arg_count]
    Return = 0x61,       // Return from function

    // Arrays (0x70-0x7F)
    Array = 0x70,        // Create array [u16 size] from stack
    GetIndex = 0x71,     // Pop index, pop array, push array[index]
    SetIndex = 0x72,     // Pop value, pop index, pop array, array[index] = value

    // Stack manipulation (0x80-0x8F)
    Pop = 0x80,          // Pop and discard top of stack
    Dup = 0x81,          // Duplicate top of stack

    // Special (0xF0-0xFF)
    Halt = 0xFF,         // End of bytecode
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
```

## Bytecode Container

```rust
pub struct Bytecode {
    pub instructions: Vec<u8>,
    pub constants: Vec<Value>,
    pub debug_info: Vec<DebugSpan>,
}

pub struct DebugSpan {
    pub instruction_offset: usize,
    pub span: Span,
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            debug_info: Vec::new(),
        }
    }

    pub fn emit(&mut self, opcode: Opcode, span: Span) {
        self.debug_info.push(DebugSpan {
            instruction_offset: self.instructions.len(),
            span,
        });
        self.instructions.push(opcode as u8);
    }

    pub fn emit_u8(&mut self, byte: u8) {
        self.instructions.push(byte);
    }

    pub fn emit_u16(&mut self, value: u16) {
        self.instructions.push((value >> 8) as u8);
        self.instructions.push((value & 0xFF) as u8);
    }

    pub fn emit_i16(&mut self, value: i16) {
        self.emit_u16(value as u16);
    }

    pub fn add_constant(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }

    pub fn current_offset(&self) -> usize {
        self.instructions.len()
    }

    pub fn patch_jump(&mut self, offset: usize) {
        let jump = (self.instructions.len() - offset - 2) as i16;
        self.instructions[offset] = ((jump >> 8) & 0xFF) as u8;
        self.instructions[offset + 1] = (jump & 0xFF) as u8;
    }
}
```

## Encoding Format

- **Opcode:** 1 byte
- **u8 operand:** 1 byte (for arg counts)
- **u16 operand:** 2 bytes, big-endian (for indexes)
- **i16 operand:** 2 bytes, big-endian, signed (for jumps)

## Example Bytecode Sequence

```
// Atlas: let x = 2 + 3;
Constant 0x0000    // Push 2.0
Constant 0x0001    // Push 3.0
Add                // Pop both, push 5.0
SetLocal 0x0000    // Store to local[0] (x)
```

## Key Design Decisions

- **Stack-based:** All operations work on implicit stack
- **Big-endian encoding:** For multi-byte operands
- **Constant pool:** Literals stored separately, referenced by index
- **Debug info:** Parallel array mapping instruction offset to source span
