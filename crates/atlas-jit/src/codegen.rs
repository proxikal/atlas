//! Bytecode-to-IR translation
//!
//! Translates Atlas bytecode sequences into Cranelift IR for native
//! code generation. Handles arithmetic, comparisons, control flow,
//! and local variable access.

use atlas_runtime::bytecode::{Bytecode, Opcode};
use cranelift_codegen::ir::condcodes::FloatCC;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::{AbiParam, Function, InstBuilder, Signature, UserFuncName};
use cranelift_codegen::isa::CallConv;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};

use crate::{JitError, JitResult};

/// Translates a range of Atlas bytecode to a Cranelift IR function.
///
/// The generated function takes no arguments and returns an f64 (the
/// numeric result). This handles the common case of numeric
/// computations which are the primary JIT compilation targets.
pub struct IrTranslator {
    /// Optimization level string for Cranelift (used by backend)
    _opt_level: &'static str,
}

impl IrTranslator {
    /// Create a translator with the given optimization level
    pub fn new(opt_level: u8) -> Self {
        let opt = match opt_level {
            0 => "none",
            1 => "speed",
            _ => "speed_and_size",
        };
        Self { _opt_level: opt }
    }

    /// Translate a bytecode range into a Cranelift IR function
    ///
    /// `bytecode` - the full bytecode container
    /// `start` - start offset of function body
    /// `end` - end offset (exclusive) of function body
    ///
    /// Returns a Cranelift Function ready for compilation.
    pub fn translate(&self, bytecode: &Bytecode, start: usize, end: usize) -> JitResult<Function> {
        // Function signature: () -> f64
        let mut sig = Signature::new(CallConv::SystemV);
        sig.returns.push(AbiParam::new(types::F64));

        let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);
        let mut func_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

        // Create entry block
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        // Translate bytecode to IR using a simulated value stack
        let result = self.translate_body(&mut builder, bytecode, start, end)?;

        builder.ins().return_(&[result]);
        builder.finalize();

        Ok(func)
    }

    /// Translate a bytecode range into a Cranelift IR function that takes
    /// arguments (for parameterized functions).
    ///
    /// `param_count` - number of f64 parameters
    pub fn translate_with_params(
        &self,
        bytecode: &Bytecode,
        start: usize,
        end: usize,
        param_count: usize,
    ) -> JitResult<Function> {
        let mut sig = Signature::new(CallConv::SystemV);
        for _ in 0..param_count {
            sig.params.push(AbiParam::new(types::F64));
        }
        sig.returns.push(AbiParam::new(types::F64));

        let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);
        let mut func_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        // Declare local variables for parameters
        let mut locals = Vec::new();
        for i in 0..param_count {
            let var = builder.declare_var(types::F64);
            let param_val = builder.block_params(entry_block)[i];
            builder.def_var(var, param_val);
            locals.push(var);
        }

        let result =
            self.translate_body_with_locals(&mut builder, bytecode, start, end, &locals)?;

        builder.ins().return_(&[result]);
        builder.finalize();

        Ok(func)
    }

    /// Core translation loop: walks bytecode and emits IR
    fn translate_body(
        &self,
        builder: &mut FunctionBuilder,
        bytecode: &Bytecode,
        start: usize,
        end: usize,
    ) -> JitResult<cranelift_codegen::ir::Value> {
        self.translate_body_with_locals(builder, bytecode, start, end, &[])
    }

    /// Core translation loop with local variable support
    fn translate_body_with_locals(
        &self,
        builder: &mut FunctionBuilder,
        bytecode: &Bytecode,
        start: usize,
        end: usize,
        locals: &[Variable],
    ) -> JitResult<cranelift_codegen::ir::Value> {
        let instructions = &bytecode.instructions;
        let mut ip = start;
        let mut stack: Vec<cranelift_codegen::ir::Value> = Vec::new();

        // Track all declared variables (start with passed-in locals)
        let max_locals = 64; // reasonable upper bound
        let mut declared_vars: Vec<Variable> = locals.to_vec();

        while ip < end && ip < instructions.len() {
            let byte = instructions[ip];
            let opcode = Opcode::try_from(byte).map_err(|_| {
                JitError::InvalidBytecode(format!("invalid opcode byte 0x{:02x} at {}", byte, ip))
            })?;
            ip += 1;

            match opcode {
                Opcode::Constant => {
                    let idx = read_u16(instructions, &mut ip);
                    let val = bytecode.constants.get(idx as usize).ok_or_else(|| {
                        JitError::InvalidBytecode(format!("constant index {} out of bounds", idx))
                    })?;
                    // Only support numeric constants in JIT
                    let f = match val {
                        atlas_runtime::value::Value::Number(n) => *n,
                        _ => {
                            return Err(JitError::InvalidBytecode(
                                "JIT only supports numeric constants".into(),
                            ));
                        }
                    };
                    stack.push(builder.ins().f64const(f));
                }
                Opcode::True => {
                    stack.push(builder.ins().f64const(1.0));
                }
                Opcode::False | Opcode::Null => {
                    stack.push(builder.ins().f64const(0.0));
                }
                Opcode::Add => {
                    let (a, b) = pop2(&mut stack)?;
                    stack.push(builder.ins().fadd(a, b));
                }
                Opcode::Sub => {
                    let (a, b) = pop2(&mut stack)?;
                    stack.push(builder.ins().fsub(a, b));
                }
                Opcode::Mul => {
                    let (a, b) = pop2(&mut stack)?;
                    stack.push(builder.ins().fmul(a, b));
                }
                Opcode::Div => {
                    let (a, b) = pop2(&mut stack)?;
                    stack.push(builder.ins().fdiv(a, b));
                }
                Opcode::Mod => {
                    // f64 modulo: a - floor(a/b) * b
                    let (a, b) = pop2(&mut stack)?;
                    let div = builder.ins().fdiv(a, b);
                    let floored = builder.ins().floor(div);
                    let prod = builder.ins().fmul(floored, b);
                    stack.push(builder.ins().fsub(a, prod));
                }
                Opcode::Negate => {
                    let a = pop1(&mut stack)?;
                    stack.push(builder.ins().fneg(a));
                }
                Opcode::Equal => {
                    let (a, b) = pop2(&mut stack)?;
                    let cmp = builder.ins().fcmp(FloatCC::Equal, a, b);
                    // Convert bool (i8) to f64: 1.0 or 0.0
                    let int_val = builder.ins().uextend(types::I32, cmp);
                    stack.push(builder.ins().fcvt_from_uint(types::F64, int_val));
                }
                Opcode::NotEqual => {
                    let (a, b) = pop2(&mut stack)?;
                    let cmp = builder.ins().fcmp(FloatCC::NotEqual, a, b);
                    let int_val = builder.ins().uextend(types::I32, cmp);
                    stack.push(builder.ins().fcvt_from_uint(types::F64, int_val));
                }
                Opcode::Less => {
                    let (a, b) = pop2(&mut stack)?;
                    let cmp = builder.ins().fcmp(FloatCC::LessThan, a, b);
                    let int_val = builder.ins().uextend(types::I32, cmp);
                    stack.push(builder.ins().fcvt_from_uint(types::F64, int_val));
                }
                Opcode::LessEqual => {
                    let (a, b) = pop2(&mut stack)?;
                    let cmp = builder.ins().fcmp(FloatCC::LessThanOrEqual, a, b);
                    let int_val = builder.ins().uextend(types::I32, cmp);
                    stack.push(builder.ins().fcvt_from_uint(types::F64, int_val));
                }
                Opcode::Greater => {
                    let (a, b) = pop2(&mut stack)?;
                    let cmp = builder.ins().fcmp(FloatCC::GreaterThan, a, b);
                    let int_val = builder.ins().uextend(types::I32, cmp);
                    stack.push(builder.ins().fcvt_from_uint(types::F64, int_val));
                }
                Opcode::GreaterEqual => {
                    let (a, b) = pop2(&mut stack)?;
                    let cmp = builder.ins().fcmp(FloatCC::GreaterThanOrEqual, a, b);
                    let int_val = builder.ins().uextend(types::I32, cmp);
                    stack.push(builder.ins().fcvt_from_uint(types::F64, int_val));
                }
                Opcode::Not => {
                    let a = pop1(&mut stack)?;
                    let zero = builder.ins().f64const(0.0);
                    let cmp = builder.ins().fcmp(FloatCC::Equal, a, zero);
                    let int_val = builder.ins().uextend(types::I32, cmp);
                    stack.push(builder.ins().fcvt_from_uint(types::F64, int_val));
                }
                Opcode::GetLocal => {
                    let idx = read_u16(instructions, &mut ip) as usize;
                    // Declare on-the-fly if needed
                    while declared_vars.len() <= idx && declared_vars.len() < max_locals {
                        let var = builder.declare_var(types::F64);
                        let zero = builder.ins().f64const(0.0);
                        builder.def_var(var, zero);
                        declared_vars.push(var);
                    }
                    if idx < declared_vars.len() {
                        stack.push(builder.use_var(declared_vars[idx]));
                    } else {
                        return Err(JitError::InvalidBytecode(format!(
                            "local index {} exceeds max {}",
                            idx, max_locals
                        )));
                    }
                }
                Opcode::SetLocal => {
                    let idx = read_u16(instructions, &mut ip) as usize;
                    let val = pop1(&mut stack)?;
                    // Ensure variable is declared
                    while declared_vars.len() <= idx && declared_vars.len() < max_locals {
                        let var = builder.declare_var(types::F64);
                        let zero = builder.ins().f64const(0.0);
                        builder.def_var(var, zero);
                        declared_vars.push(var);
                    }
                    if idx < declared_vars.len() {
                        builder.def_var(declared_vars[idx], val);
                    }
                }
                Opcode::Pop => {
                    let _ = pop1(&mut stack)?;
                }
                Opcode::Dup => {
                    let a = pop1(&mut stack)?;
                    stack.push(a);
                    stack.push(a);
                }
                Opcode::Return | Opcode::Halt => {
                    break;
                }
                // Unsupported opcodes — bail out to interpreter
                other => {
                    return Err(JitError::UnsupportedOpcode(other));
                }
            }
        }

        // Return top of stack, or 0.0 if empty
        if let Some(top) = stack.last() {
            Ok(*top)
        } else {
            Ok(builder.ins().f64const(0.0))
        }
    }
}

/// Read a big-endian u16 from the instruction stream and advance ip
fn read_u16(instructions: &[u8], ip: &mut usize) -> u16 {
    let hi = instructions.get(*ip).copied().unwrap_or(0) as u16;
    let lo = instructions.get(*ip + 1).copied().unwrap_or(0) as u16;
    *ip += 2;
    (hi << 8) | lo
}

/// Pop one value from the IR value stack
fn pop1(stack: &mut Vec<cranelift_codegen::ir::Value>) -> JitResult<cranelift_codegen::ir::Value> {
    stack
        .pop()
        .ok_or_else(|| JitError::InvalidBytecode("stack underflow".into()))
}

/// Pop two values: first popped is `b`, second is `a` (for a op b)
fn pop2(
    stack: &mut Vec<cranelift_codegen::ir::Value>,
) -> JitResult<(cranelift_codegen::ir::Value, cranelift_codegen::ir::Value)> {
    let b = pop1(stack)?;
    let a = pop1(stack)?;
    Ok((a, b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_runtime::bytecode::Bytecode;
    use atlas_runtime::span::Span;
    use atlas_runtime::value::Value;

    fn dummy_span() -> Span {
        Span::dummy()
    }

    #[test]
    fn test_translate_constant() {
        let mut bc = Bytecode::new();
        let idx = bc.add_constant(Value::Number(42.0));
        bc.emit(Opcode::Constant, dummy_span());
        bc.emit_u16(idx);
        bc.emit(Opcode::Return, dummy_span());

        let translator = IrTranslator::new(0);
        let func = translator.translate(&bc, 0, bc.instructions.len());
        assert!(func.is_ok());
    }

    #[test]
    fn test_translate_add() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(10.0));
        let b = bc.add_constant(Value::Number(20.0));
        bc.emit(Opcode::Constant, dummy_span());
        bc.emit_u16(a);
        bc.emit(Opcode::Constant, dummy_span());
        bc.emit_u16(b);
        bc.emit(Opcode::Add, dummy_span());
        bc.emit(Opcode::Return, dummy_span());

        let translator = IrTranslator::new(0);
        let func = translator.translate(&bc, 0, bc.instructions.len());
        assert!(func.is_ok());
    }

    #[test]
    fn test_translate_unsupported() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::GetGlobal, dummy_span());
        bc.emit_u16(0);

        let translator = IrTranslator::new(0);
        let result = translator.translate(&bc, 0, bc.instructions.len());
        assert!(result.is_err());
        match result.unwrap_err() {
            JitError::UnsupportedOpcode(Opcode::GetGlobal) => {}
            other => panic!("expected UnsupportedOpcode, got {:?}", other),
        }
    }

    #[test]
    fn test_translate_negate() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(5.0));
        bc.emit(Opcode::Constant, dummy_span());
        bc.emit_u16(a);
        bc.emit(Opcode::Negate, dummy_span());
        bc.emit(Opcode::Return, dummy_span());

        let translator = IrTranslator::new(0);
        assert!(translator.translate(&bc, 0, bc.instructions.len()).is_ok());
    }

    #[test]
    fn test_translate_comparison() {
        let mut bc = Bytecode::new();
        let a = bc.add_constant(Value::Number(1.0));
        let b = bc.add_constant(Value::Number(2.0));
        bc.emit(Opcode::Constant, dummy_span());
        bc.emit_u16(a);
        bc.emit(Opcode::Constant, dummy_span());
        bc.emit_u16(b);
        bc.emit(Opcode::Less, dummy_span());
        bc.emit(Opcode::Return, dummy_span());

        let translator = IrTranslator::new(0);
        assert!(translator.translate(&bc, 0, bc.instructions.len()).is_ok());
    }

    #[test]
    fn test_translate_stack_underflow() {
        let mut bc = Bytecode::new();
        bc.emit(Opcode::Add, dummy_span());

        let translator = IrTranslator::new(0);
        assert!(translator.translate(&bc, 0, bc.instructions.len()).is_err());
    }
}
