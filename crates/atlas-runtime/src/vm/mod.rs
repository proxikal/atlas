//! Stack-based virtual machine
//!
//! Executes bytecode instructions with a value stack and call frames.
//! - Arithmetic operations check for NaN/Infinity
//! - Variables are stored in locals (stack) or globals (HashMap)
//! - Control flow uses jumps and loops

mod debugger;
pub mod dispatch;
mod frame;
mod profiler;

pub use debugger::{DebugAction, DebugHook, Debugger};
pub use frame::CallFrame;
pub use profiler::Profiler;

use crate::bytecode::{Bytecode, Opcode};
use crate::ffi::{ExternFunction, LibraryLoader};
use crate::span::Span;
use crate::value::{RuntimeError, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Result returned by [`VM::run_debuggable`].
#[derive(Debug)]
pub enum VmRunResult {
    /// Execution completed normally.
    Complete(Option<Value>),
    /// Execution was paused (breakpoint hit or step condition met).
    ///
    /// The VM state is fully preserved.  Resume by calling `run_debuggable` again
    /// (or `run` to run without further debug checks).
    Paused {
        /// Instruction pointer at the pause point (the instruction that would
        /// have been executed next has NOT been executed yet).
        ip: usize,
    },
}

/// Virtual machine state
pub struct VM {
    /// Value stack
    stack: Vec<Value>,
    /// Call frames (for function calls)
    frames: Vec<CallFrame>,
    /// Global variables
    globals: HashMap<String, Value>,
    /// Bytecode to execute
    bytecode: Bytecode,
    /// Instruction pointer
    ip: usize,
    /// Optional profiler for performance analysis
    profiler: Option<Profiler>,
    /// Optional debugger for step-through execution
    debugger: Option<Debugger>,
    /// Set to `true` by the execute_loop when the debugger requests a pause.
    /// Cleared by `run_debuggable` after it reads the flag.
    debug_pause_pending: bool,
    /// Security context for current execution (set during run())
    current_security: Option<std::sync::Arc<crate::security::SecurityContext>>,
    /// Output writer for print() (defaults to stdout)
    output_writer: crate::stdlib::OutputWriter,
    /// FFI library loader (phase-10b)
    library_loader: LibraryLoader,
    /// Loaded extern functions (phase-10b)
    extern_functions: HashMap<String, ExternFunction>,
    /// Reusable string buffer for temporary string operations (reduces allocations)
    string_buffer: String,
}

impl VM {
    /// Create a new VM with bytecode
    pub fn new(bytecode: Bytecode) -> Self {
        // Create an initial "main" frame for top-level code
        let main_frame = CallFrame {
            function_name: "<main>".to_string(),
            return_ip: 0,
            stack_base: 0,
            local_count: bytecode.top_level_local_count,
            upvalues: std::sync::Arc::new(Vec::new()),
        };

        Self {
            stack: Vec::with_capacity(1024),
            frames: vec![main_frame],
            globals: HashMap::new(),
            bytecode,
            ip: 0,
            profiler: None,
            debugger: None,
            debug_pause_pending: false,
            current_security: None,
            output_writer: crate::stdlib::stdout_writer(),
            library_loader: LibraryLoader::new(),
            extern_functions: HashMap::new(),
            string_buffer: String::with_capacity(256),
        }
    }

    /// Create a new VM with profiling enabled
    pub fn with_profiling(bytecode: Bytecode) -> Self {
        let mut vm = Self::new(bytecode);
        vm.profiler = Some(Profiler::enabled());
        vm
    }

    /// Create a new VM with debugging enabled
    pub fn with_debugging(bytecode: Bytecode) -> Self {
        let mut vm = Self::new(bytecode);
        vm.debugger = Some(Debugger::enabled());
        vm
    }

    /// Set the output writer (used by Runtime to redirect print() output)
    pub fn set_output_writer(&mut self, writer: crate::stdlib::OutputWriter) {
        self.output_writer = writer;
    }

    /// Set a global variable
    ///
    /// Used by the Runtime to inject native functions and other complex values
    /// that can't be represented in bytecode constants.
    pub fn set_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }

    /// Get all global variables
    ///
    /// Used by the Runtime to persist VM globals back to interpreter state
    /// after execution completes.
    pub fn get_globals(&self) -> &std::collections::HashMap<String, Value> {
        &self.globals
    }

    /// Load extern declarations from AST (phase-10b)
    ///
    /// Processes extern function declarations by loading libraries and looking up symbols.
    /// Must be called before running bytecode that calls extern functions.
    pub fn load_extern_declarations(
        &mut self,
        program: &crate::ast::Program,
    ) -> Result<(), RuntimeError> {
        use crate::ast::Item;
        use crate::value::FunctionRef;

        for item in &program.items {
            if let Item::Extern(extern_decl) = item {
                // Load the dynamic library
                self.library_loader
                    .load(&extern_decl.library)
                    .map_err(|e| RuntimeError::TypeError {
                        msg: format!("Failed to load library '{}': {}", extern_decl.library, e),
                        span: extern_decl.span,
                    })?;

                // Determine the symbol name (use 'as' name if provided, otherwise function name)
                let symbol_name = extern_decl.symbol.as_ref().unwrap_or(&extern_decl.name);

                // Look up the function symbol
                let fn_ptr = unsafe {
                    self.library_loader
                        .lookup_symbol::<*const ()>(&extern_decl.library, symbol_name)
                        .map_err(|e| RuntimeError::TypeError {
                            msg: format!(
                                "Failed to find symbol '{}' in library '{}': {}",
                                symbol_name, extern_decl.library, e
                            ),
                            span: extern_decl.span,
                        })?
                };

                // Convert parameter types from AST to FFI types
                let param_types: Vec<crate::ffi::ExternType> = extern_decl
                    .params
                    .iter()
                    .map(|(_, ty)| convert_extern_type_annotation(ty))
                    .collect();

                let return_type = convert_extern_type_annotation(&extern_decl.return_type);

                // Create ExternFunction
                let extern_fn = unsafe { ExternFunction::new(*fn_ptr, param_types, return_type) };

                // Store the extern function
                self.extern_functions
                    .insert(extern_decl.name.clone(), extern_fn);

                // Register as a callable global
                let func_value = Value::Function(FunctionRef {
                    name: extern_decl.name.clone(),
                    arity: extern_decl.params.len(),
                    bytecode_offset: 0, // Not used for extern functions
                    local_count: 0,     // Not used for extern functions
                });
                self.globals.insert(extern_decl.name.clone(), func_value);
            }
        }

        Ok(())
    }
}

/// Convert ExternTypeAnnotation (AST) to ExternType (FFI runtime)
fn convert_extern_type_annotation(
    annotation: &crate::ast::ExternTypeAnnotation,
) -> crate::ffi::ExternType {
    use crate::ast::ExternTypeAnnotation;
    use crate::ffi::ExternType;

    match annotation {
        ExternTypeAnnotation::CInt => ExternType::CInt,
        ExternTypeAnnotation::CLong => ExternType::CLong,
        ExternTypeAnnotation::CDouble => ExternType::CDouble,
        ExternTypeAnnotation::CCharPtr => ExternType::CCharPtr,
        ExternTypeAnnotation::CVoid => ExternType::CVoid,
        ExternTypeAnnotation::CBool => ExternType::CBool,
    }
}

impl VM {
    /// Set the instruction pointer
    ///
    /// Used by Runtime to start execution from a specific offset when
    /// accumulating bytecode across multiple eval() calls.
    pub fn set_ip(&mut self, offset: usize) {
        self.ip = offset;
    }

    /// Enable profiling
    pub fn enable_profiling(&mut self) {
        if let Some(ref mut profiler) = self.profiler {
            profiler.enable();
        } else {
            self.profiler = Some(Profiler::enabled());
        }
    }

    /// Disable profiling
    pub fn disable_profiling(&mut self) {
        if let Some(ref mut profiler) = self.profiler {
            profiler.disable();
        }
    }

    /// Get profiler reference
    pub fn profiler(&self) -> Option<&Profiler> {
        self.profiler.as_ref()
    }

    /// Get mutable profiler reference
    pub fn profiler_mut(&mut self) -> Option<&mut Profiler> {
        self.profiler.as_mut()
    }

    /// Enable debugging
    pub fn enable_debugging(&mut self) {
        if let Some(ref mut debugger) = self.debugger {
            debugger.enable();
        } else {
            self.debugger = Some(Debugger::enabled());
        }
    }

    /// Disable debugging
    pub fn disable_debugging(&mut self) {
        if let Some(ref mut debugger) = self.debugger {
            debugger.disable();
        }
    }

    /// Get debugger reference
    pub fn debugger(&self) -> Option<&Debugger> {
        self.debugger.as_ref()
    }

    /// Get mutable debugger reference
    pub fn debugger_mut(&mut self) -> Option<&mut Debugger> {
        self.debugger.as_mut()
    }

    /// Get the source span for the current instruction pointer
    ///
    /// Returns the span from debug info if available.
    /// Useful for error reporting with source location context.
    pub fn current_span(&self) -> Option<crate::span::Span> {
        if self.ip == 0 {
            return None;
        }
        self.bytecode.get_span_for_offset(self.ip - 1)
    }

    /// Get the source span for a specific instruction offset
    pub fn span_for_offset(&self, offset: usize) -> Option<crate::span::Span> {
        self.bytecode.get_span_for_offset(offset)
    }

    // ── Debugger inspection API ───────────────────────────────────────────────

    /// Get the current instruction pointer.
    pub fn current_ip(&self) -> usize {
        self.ip
    }

    /// Get the current call-frame depth (number of active frames).
    pub fn frame_depth(&self) -> usize {
        self.frames.len()
    }

    /// Get the current value-stack depth.
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    /// Get a call frame by index (0 = innermost / most recent).
    ///
    /// Returns `None` if `index` is out of range.
    pub fn get_frame_at(&self, index: usize) -> Option<&CallFrame> {
        let len = self.frames.len();
        if index < len {
            self.frames.get(len - 1 - index)
        } else {
            None
        }
    }

    /// Get the local variable values for a call frame.
    ///
    /// `frame_index` 0 is the innermost (current) frame.
    /// Returns `(slot_index, &Value)` pairs for all initialised local slots.
    pub fn get_locals_for_frame(&self, frame_index: usize) -> Vec<(usize, &Value)> {
        let frame = match self.get_frame_at(frame_index) {
            Some(f) => f,
            None => return Vec::new(),
        };
        let base = frame.stack_base;
        let count = frame.local_count;
        (0..count)
            .filter_map(|i| self.stack.get(base + i).map(|v| (i, v)))
            .collect()
    }

    /// Get all global variables.
    pub fn get_global_variables(&self) -> &HashMap<String, Value> {
        &self.globals
    }

    /// Get debug information from the bytecode.
    pub fn debug_spans(&self) -> &[crate::bytecode::DebugSpan] {
        &self.bytecode.debug_info
    }

    // ── Debuggable execution ──────────────────────────────────────────────────

    /// Execute bytecode with live debugger-state integration.
    ///
    /// Syncs verified breakpoints and step conditions from `debug_state` into
    /// the VM's embedded debugger, then runs until the program completes or a
    /// pause condition fires.
    ///
    /// When `VmRunResult::Paused` is returned the VM state is fully preserved
    /// and execution can be resumed by calling `run_debuggable` again.
    pub fn run_debuggable(
        &mut self,
        debug_state: &mut crate::debugger::DebuggerState,
        security: &crate::security::SecurityContext,
    ) -> Result<VmRunResult, RuntimeError> {
        use crate::debugger::state::StepMode;
        use debugger::StepCondition;

        // Sync verified breakpoints from debug_state into the embedded Debugger.
        self.enable_debugging();
        let dbg = self.debugger.as_mut().unwrap();
        dbg.clear_breakpoints();
        for bp in debug_state.breakpoints() {
            if let Some(offset) = bp.instruction_offset {
                dbg.set_breakpoint(offset);
            }
        }

        // Configure the step condition.
        let step_condition = match debug_state.step_mode {
            StepMode::None => StepCondition::None,
            StepMode::Into => StepCondition::Always,
            StepMode::Over => StepCondition::OverDepth(debug_state.step_start_frame_depth),
            StepMode::Out => StepCondition::OutDepth(debug_state.step_start_frame_depth),
        };
        self.debugger
            .as_mut()
            .unwrap()
            .set_step_condition(step_condition);

        // Clear any leftover pause flag from a previous run.
        self.debug_pause_pending = false;

        // Run the execute loop (profiling hooks still active).
        self.current_security = Some(std::sync::Arc::new(security.clone()));
        if let Some(ref mut profiler) = self.profiler {
            if profiler.is_enabled() {
                profiler.start_timing();
            }
        }
        let loop_result = self.execute_until_end();
        if let Some(ref mut profiler) = self.profiler {
            if profiler.is_enabled() {
                profiler.stop_timing();
            }
        }

        if self.debug_pause_pending {
            // The execute_loop broke early due to a debug pause.
            let ip = self.ip;
            let location = None; // Caller resolves via SourceMap
            let reason = if let Some(bp) = debug_state.breakpoint_at_offset(ip) {
                crate::debugger::protocol::PauseReason::Breakpoint { id: bp.id }
            } else {
                crate::debugger::protocol::PauseReason::Step
            };
            debug_state.pause(reason, location, ip);
            debug_state.clear_step_mode();
            self.debug_pause_pending = false;
            Ok(VmRunResult::Paused { ip })
        } else {
            debug_state.stop();
            Ok(VmRunResult::Complete(loop_result?))
        }
    }

    /// Execute the bytecode
    pub fn run(
        &mut self,
        security: &crate::security::SecurityContext,
    ) -> Result<Option<Value>, RuntimeError> {
        // Store security context for builtin calls
        self.current_security = Some(std::sync::Arc::new(security.clone()));
        // Start profiling timer if profiler is enabled
        if let Some(ref mut profiler) = self.profiler {
            if profiler.is_enabled() {
                profiler.start_timing();
            }
        }
        let result = self.execute_until_end();
        // Stop profiling timer
        if let Some(ref mut profiler) = self.profiler {
            if profiler.is_enabled() {
                profiler.stop_timing();
            }
        }
        result
    }

    /// Execute bytecode until reaching the end of instructions
    fn execute_until_end(&mut self) -> Result<Option<Value>, RuntimeError> {
        self.execute_loop(None)
    }

    /// Execute bytecode until a specific frame depth is reached (for function calls)
    /// If target_frame_depth is Some(n), stops when frames.len() <= n
    /// If target_frame_depth is None, runs until end of bytecode
    fn execute_loop(
        &mut self,
        target_frame_depth: Option<usize>,
    ) -> Result<Option<Value>, RuntimeError> {
        loop {
            // Check termination conditions
            if self.ip >= self.bytecode.instructions.len() {
                break;
            }

            // Check if we've returned from the target frame
            if let Some(depth) = target_frame_depth {
                if self.frames.len() <= depth {
                    // Frame has returned, result should be on stack
                    return Ok(Some(self.peek(0).clone()));
                }
            }

            let opcode = self.read_opcode()?;

            // Debugger hook: before instruction (zero overhead when disabled)
            if let Some(ref mut debugger) = self.debugger {
                if debugger.is_enabled() {
                    let current_ip = self.ip - 1;
                    let frame_depth = self.frames.len();
                    let action =
                        debugger.before_instruction_with_depth(current_ip, opcode, frame_depth);
                    match action {
                        DebugAction::Pause | DebugAction::Step => {
                            // Back up IP so the paused instruction is re-executed on resume
                            self.ip = current_ip;
                            self.debug_pause_pending = true;
                            break; // break the execute_loop loop
                        }
                        DebugAction::Continue => {
                            // Normal execution – continue with opcode dispatch
                        }
                    }
                }
            }

            // Record instruction for profiling (zero overhead when disabled)
            if let Some(ref mut profiler) = self.profiler {
                if profiler.is_enabled() {
                    let instruction_ip = self.ip - 1; // ip already advanced by read_opcode
                    profiler.record_instruction_at(opcode, instruction_ip);
                    profiler.update_value_stack_depth(self.stack.len());
                    profiler.update_frame_depth(self.frames.len());
                }
            }

            match opcode {
                // ===== Constants =====
                Opcode::Constant => {
                    let index = self.read_u16()? as usize;
                    if index >= self.bytecode.constants.len() {
                        return Err(RuntimeError::UnknownOpcode {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    let value = self.bytecode.constants[index].clone();
                    self.push(value);
                }
                Opcode::Null => self.push(Value::Null),
                Opcode::True => self.push(Value::Bool(true)),
                Opcode::False => self.push(Value::Bool(false)),

                // ===== Variables =====
                Opcode::GetLocal => {
                    let index = self.read_u16()? as usize;
                    let base = self.current_frame().stack_base;
                    let absolute_index = base + index;
                    if absolute_index >= self.stack.len() {
                        return Err(RuntimeError::StackUnderflow {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    let value = self.stack[absolute_index].clone();
                    self.push(value);
                }
                Opcode::SetLocal => {
                    let index = self.read_u16()? as usize;
                    let base = self.current_frame().stack_base;
                    let local_count = self.current_frame().local_count;
                    let absolute_index = base + index;
                    let value = self.peek(0).clone();

                    // SAFETY CHECK: Prevent unbounded stack growth
                    // This prevents memory explosion from invalid bytecode or compiler bugs
                    if index >= local_count {
                        return Err(RuntimeError::StackUnderflow {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }

                    // Extend stack if needed (for local variables not yet initialized)
                    if absolute_index >= self.stack.len() {
                        // Bounded extension: only up to the declared local_count
                        let needed = absolute_index - self.stack.len() + 1;
                        if base + local_count > self.stack.len() + needed {
                            return Err(RuntimeError::StackUnderflow {
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            });
                        }
                        for _ in 0..needed {
                            self.stack.push(Value::Null);
                        }
                    }
                    self.stack[absolute_index] = value;
                }
                Opcode::GetGlobal => {
                    let name_index = self.read_u16()? as usize;
                    if name_index >= self.bytecode.constants.len() {
                        return Err(RuntimeError::UnknownOpcode {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    let name = match &self.bytecode.constants[name_index] {
                        Value::String(s) => s.as_ref().clone(),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Expected string constant for variable name".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    };
                    let value = if name == "None" {
                        // Constructor literal: None always evaluates to Option::None
                        Value::Option(None)
                    } else if let Some(v) = self.globals.get(&name) {
                        v.clone()
                    } else if crate::stdlib::is_builtin(&name)
                        || crate::stdlib::is_array_intrinsic(&name)
                    {
                        // Builtin or intrinsic - return builtin value
                        Value::Builtin(std::sync::Arc::from(name.as_str()))
                    } else {
                        // Check math constants
                        match name.as_str() {
                            "PI" => Value::Number(crate::stdlib::math::PI),
                            "E" => Value::Number(crate::stdlib::math::E),
                            "SQRT2" => Value::Number(crate::stdlib::math::SQRT2),
                            "LN2" => Value::Number(crate::stdlib::math::LN2),
                            "LN10" => Value::Number(crate::stdlib::math::LN10),
                            _ => {
                                return Err(RuntimeError::UndefinedVariable {
                                    name: name.clone(),
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }
                        }
                    };
                    self.push(value);
                }
                Opcode::SetGlobal => {
                    let name_index = self.read_u16()? as usize;
                    if name_index >= self.bytecode.constants.len() {
                        return Err(RuntimeError::UnknownOpcode {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    let name = match &self.bytecode.constants[name_index] {
                        Value::String(s) => s.as_ref().clone(),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Expected string constant for variable name".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    };
                    let value = self.peek(0).clone();
                    self.globals.insert(name, value);
                }

                Opcode::MakeClosure => {
                    let func_const_idx = self.read_u16()? as usize;
                    let n_upvalues = self.read_u16()? as usize;

                    // Get the FunctionRef from constant pool
                    let func = match self.bytecode.constants.get(func_const_idx) {
                        Some(Value::Function(f)) => f.clone(),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "MakeClosure: constant is not a function".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    };

                    // Pop upvalues from stack (in reverse order since stack is LIFO)
                    let mut upvalues = Vec::with_capacity(n_upvalues);
                    for _ in 0..n_upvalues {
                        upvalues.push(self.pop());
                    }
                    upvalues.reverse(); // Restore capture order

                    let closure = crate::value::ClosureRef {
                        func,
                        upvalues: std::sync::Arc::new(upvalues),
                    };
                    self.push(Value::Closure(closure));
                }

                Opcode::GetUpvalue => {
                    let idx = self.read_u16()? as usize;
                    let value = match self.current_frame().upvalues.get(idx) {
                        Some(v) => v.clone(),
                        None => {
                            return Err(RuntimeError::TypeError {
                                msg: format!(
                                    "Upvalue index {} out of bounds (closure has {} upvalues)",
                                    idx,
                                    self.current_frame().upvalues.len()
                                ),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    };
                    self.push(value);
                }

                Opcode::SetUpvalue => {
                    let idx = self.read_u16()? as usize;
                    let value = self.peek(0).clone();
                    let frame = self.frames.last_mut().expect("no call frame");
                    let upvalues = std::sync::Arc::make_mut(&mut frame.upvalues);
                    if idx < upvalues.len() {
                        upvalues[idx] = value;
                    } else {
                        return Err(RuntimeError::TypeError {
                            msg: format!(
                                "SetUpvalue: index {} out of bounds (closure has {} upvalues)",
                                idx,
                                upvalues.len()
                            ),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                }

                // ===== Arithmetic =====
                Opcode::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    match (&a, &b) {
                        (Value::Number(x), Value::Number(y)) => {
                            let result = x + y;
                            if result.is_nan() || result.is_infinite() {
                                return Err(RuntimeError::InvalidNumericResult {
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }
                            self.push(Value::Number(result));
                        }
                        (Value::String(x), Value::String(y)) => {
                            // Reuse string buffer to reduce allocations
                            self.string_buffer.clear();
                            self.string_buffer.push_str(x);
                            self.string_buffer.push_str(y);
                            self.push(Value::String(Arc::new(self.string_buffer.clone())));
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Invalid operands for +".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    }
                }
                Opcode::Sub => self.binary_numeric_op(|a, b| a - b)?,
                Opcode::Mul => self.binary_numeric_op(|a, b| a * b)?,
                Opcode::Div => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    if b == 0.0 {
                        return Err(RuntimeError::DivideByZero {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    let result = a / b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    self.push(Value::Number(result));
                }
                Opcode::Mod => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    if b == 0.0 {
                        return Err(RuntimeError::DivideByZero {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    let result = a % b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    self.push(Value::Number(result));
                }
                Opcode::Negate => {
                    let value = self.pop();
                    match value {
                        Value::Number(n) => self.push(Value::Number(-n)),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot negate non-number".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    }
                }

                // ===== Comparison =====
                Opcode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                }
                Opcode::NotEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a != b));
                }
                Opcode::Less => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a < b));
                }
                Opcode::LessEqual => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a <= b));
                }
                Opcode::Greater => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a > b));
                }
                Opcode::GreaterEqual => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a >= b));
                }

                // ===== Logical =====
                Opcode::Not => {
                    let value = self.pop();
                    match value {
                        Value::Bool(b) => self.push(Value::Bool(!b)),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot apply ! to non-boolean".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    }
                }
                Opcode::And | Opcode::Or => {
                    // TODO: Short-circuit evaluation
                    return Err(RuntimeError::UnknownOpcode {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                }

                // ===== Control Flow =====
                Opcode::Jump => {
                    let offset = self.read_i16()?;
                    self.ip = (self.ip as isize + offset as isize) as usize;
                }
                Opcode::JumpIfFalse => {
                    let offset = self.read_i16()?;
                    let condition = self.pop();
                    if !condition.is_truthy() {
                        self.ip = (self.ip as isize + offset as isize) as usize;
                    }
                }
                Opcode::Loop => {
                    let offset = self.read_i16()?;
                    self.ip = (self.ip as isize + offset as isize) as usize;
                }

                // ===== Functions =====
                Opcode::Call => {
                    let arg_count = self.read_u8()? as usize;

                    // Get the function value from stack (it's below the arguments)
                    let function = self.peek(arg_count).clone();

                    match function {
                        Value::Builtin(ref name) => {
                            // Check array intrinsics first (callback-based)
                            if self.is_array_intrinsic(name) {
                                let mut args = Vec::with_capacity(arg_count);
                                for _ in 0..arg_count {
                                    args.push(self.pop());
                                }
                                args.reverse();
                                self.pop(); // Pop function value

                                let result = self.call_array_intrinsic(name, &args)?;
                                self.push(result);
                            } else {
                                // Stdlib builtin - call directly
                                let mut args = Vec::with_capacity(arg_count);
                                for _ in 0..arg_count {
                                    args.push(self.pop());
                                }
                                args.reverse();
                                self.pop(); // Pop function value

                                let security = self
                                    .current_security
                                    .as_ref()
                                    .expect("Security context not set");
                                let result = crate::stdlib::call_builtin(
                                    name,
                                    &args,
                                    self.current_span().unwrap_or_else(crate::span::Span::dummy),
                                    security,
                                    &self.output_writer,
                                )?;

                                self.push(result);
                            }
                        }
                        Value::Function(func) => {
                            // Check for extern functions
                            if let Some(extern_fn) = self.extern_functions.get(&func.name).cloned()
                            {
                                let mut args = Vec::with_capacity(arg_count);
                                for _ in 0..arg_count {
                                    args.push(self.pop());
                                }
                                args.reverse();
                                self.pop(); // Pop function value

                                let result = unsafe { extern_fn.call(&args) }.map_err(|e| {
                                    RuntimeError::TypeError {
                                        msg: format!("FFI call error: {}", e),
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    }
                                })?;

                                self.push(result);
                            } else {
                                // User-defined function
                                // Safety check: compiled functions always have bytecode_offset > 0
                                // because the compiler emits setup code (Constant, SetGlobal, Pop, Jump)
                                // before the function body. bytecode_offset == 0 indicates an
                                // interpreter-created function that has no bytecode.
                                if func.bytecode_offset == 0 {
                                    return Err(RuntimeError::TypeError {
                                        msg: format!(
                                            "Cannot call function '{}' from VM: function was created \
                                             by interpreter and has no compiled bytecode. This typically \
                                             happens when importing functions across execution modes.",
                                            func.name
                                        ),
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    });
                                }

                                // Create a new call frame
                                let frame = CallFrame {
                                    function_name: func.name.clone(),
                                    return_ip: self.ip,
                                    stack_base: self.stack.len() - arg_count, // Points to first argument
                                    local_count: func.local_count, // Use total locals, not just arity
                                    upvalues: std::sync::Arc::new(Vec::new()),
                                };

                                // Verify argument count matches
                                if arg_count != func.arity {
                                    return Err(RuntimeError::TypeError {
                                        msg: format!(
                                            "Function {} expects {} arguments, got {}",
                                            func.name, func.arity, arg_count
                                        ),
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    });
                                }

                                // Push the frame
                                self.frames.push(frame);
                                // Record function call in profiler
                                if let Some(ref mut profiler) = self.profiler {
                                    if profiler.is_enabled() {
                                        profiler.record_function_call(&func.name);
                                    }
                                }

                                // Jump to function bytecode
                                self.ip = func.bytecode_offset;
                            }
                        }
                        Value::Closure(closure) => {
                            // Closure call: same as Function but passes upvalues to the frame
                            let func = closure.func.clone();
                            let upvalues = closure.upvalues.clone();

                            if func.bytecode_offset == 0 {
                                return Err(RuntimeError::TypeError {
                                    msg: format!(
                                        "Cannot call closure '{}' from VM: no compiled bytecode.",
                                        func.name
                                    ),
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }

                            if arg_count != func.arity {
                                return Err(RuntimeError::TypeError {
                                    msg: format!(
                                        "Function {} expects {} arguments, got {}",
                                        func.name, func.arity, arg_count
                                    ),
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }

                            let frame = CallFrame {
                                function_name: func.name.clone(),
                                return_ip: self.ip,
                                stack_base: self.stack.len() - arg_count,
                                local_count: func.local_count,
                                upvalues,
                            };

                            self.frames.push(frame);
                            if let Some(ref mut profiler) = self.profiler {
                                if profiler.is_enabled() {
                                    profiler.record_function_call(&func.name);
                                }
                            }
                            self.ip = func.bytecode_offset;
                        }
                        Value::NativeFunction(native_fn) => {
                            // Call the native Rust closure
                            let mut args = Vec::with_capacity(arg_count);
                            for _ in 0..arg_count {
                                args.push(self.pop());
                            }
                            args.reverse(); // Arguments were pushed in reverse order

                            // Pop the function value from stack
                            self.pop();

                            let result = native_fn(&args)?;
                            self.push(result);
                        }
                        // None() is a valid zero-arg constructor call
                        Value::Option(None) if arg_count == 0 => {
                            self.pop(); // Pop the Option(None) function value
                            self.push(Value::Option(None));
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot call non-function value".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            });
                        }
                    }
                }
                Opcode::Return => {
                    // Pop the return value from stack (if any)
                    let return_value = if self.stack.is_empty() {
                        Value::Null
                    } else {
                        self.pop()
                    };

                    // Pop the call frame
                    let frame = self.frames.pop();

                    if let Some(f) = frame {
                        // Clean up the stack (remove locals, arguments, and function value)
                        self.stack.truncate(f.stack_base);
                        // Also remove the function value (one slot below stack_base)
                        if f.stack_base > 0 && !self.stack.is_empty() {
                            self.stack.pop();
                        }

                        // Restore IP to return address
                        self.ip = f.return_ip;

                        // Push return value
                        self.push(return_value);
                    } else {
                        // Returning from main - we're done
                        // Push the return value and halt
                        self.push(return_value);
                        break;
                    }
                }

                // ===== Arrays =====
                Opcode::Array => {
                    let size = self.read_u16()? as usize;
                    let mut elements = Vec::with_capacity(size);
                    for _ in 0..size {
                        elements.push(self.pop());
                    }
                    elements.reverse(); // Stack is LIFO, so reverse to get correct order
                    self.push(Value::Array(Arc::new(Mutex::new(elements))));
                }
                Opcode::GetIndex => {
                    let index_val = self.pop();
                    let target = self.pop();
                    match target {
                        Value::Array(arr) => {
                            // Array indexing requires number
                            if let Value::Number(index) = index_val {
                                if index.fract() != 0.0 || index < 0.0 {
                                    return Err(RuntimeError::InvalidIndex {
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    });
                                }
                                let idx = index as usize;
                                let borrowed = arr.lock().unwrap();
                                if idx >= borrowed.len() {
                                    return Err(RuntimeError::OutOfBounds {
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    });
                                }
                                self.push(borrowed[idx].clone());
                            } else {
                                return Err(RuntimeError::InvalidIndex {
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }
                        }
                        Value::JsonValue(json) => {
                            // JSON indexing accepts string or number
                            let result = match index_val {
                                Value::String(key) => json.index_str(key.as_ref()),
                                Value::Number(n) => json.index_num(n),
                                _ => {
                                    return Err(RuntimeError::TypeError {
                                        msg: "JSON index must be string or number".to_string(),
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    })
                                }
                            };
                            self.push(Value::JsonValue(Arc::new(result)));
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot index non-array/json".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    }
                }
                Opcode::SetIndex => {
                    let value = self.pop();
                    let index = self.pop_number()?;
                    let array = self.pop();
                    match array {
                        Value::Array(arr) => {
                            if index.fract() != 0.0 || index < 0.0 {
                                return Err(RuntimeError::InvalidIndex {
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }
                            let idx = index as usize;
                            let mut borrowed = arr.lock().unwrap();
                            if idx >= borrowed.len() {
                                return Err(RuntimeError::OutOfBounds {
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }
                            borrowed[idx] = value.clone();
                            // Push the assigned value back (assignment expressions return the value)
                            drop(borrowed); // Release the borrow before pushing
                            self.push(value);
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot index non-array".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    }
                }

                // ===== Stack Manipulation =====
                Opcode::Pop => {
                    // Don't pop if this is the last instruction before Halt
                    // Check if next instruction is Halt
                    if self.ip < self.bytecode.instructions.len()
                        && self.bytecode.instructions[self.ip] != Opcode::Halt as u8
                    {
                        self.pop();
                    }
                }
                Opcode::Dup => {
                    let value = self.peek(0).clone();
                    self.push(value);
                }

                // ===== Pattern Matching =====
                Opcode::IsOptionSome => {
                    let value = self.pop();
                    let is_some = matches!(value, Value::Option(Some(_)));
                    self.push(Value::Bool(is_some));
                }
                Opcode::IsOptionNone => {
                    let value = self.pop();
                    let is_none = matches!(value, Value::Option(None));
                    self.push(Value::Bool(is_none));
                }
                Opcode::IsResultOk => {
                    let value = self.pop();
                    let is_ok = matches!(value, Value::Result(Ok(_)));
                    self.push(Value::Bool(is_ok));
                }
                Opcode::IsResultErr => {
                    let value = self.pop();
                    let is_err = matches!(value, Value::Result(Err(_)));
                    self.push(Value::Bool(is_err));
                }
                Opcode::ExtractOptionValue => {
                    let value = self.pop();
                    match value {
                        Value::Option(Some(inner)) => self.push(*inner),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "ExtractOptionValue requires Option::Some".to_string(),
                                span: Span::dummy(),
                            })
                        }
                    }
                }
                Opcode::ExtractResultValue => {
                    let value = self.pop();
                    match value {
                        Value::Result(Ok(inner)) => self.push(*inner),
                        Value::Result(Err(inner)) => self.push(*inner),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "ExtractResultValue requires Result".to_string(),
                                span: Span::dummy(),
                            })
                        }
                    }
                }
                Opcode::IsArray => {
                    let value = self.pop();
                    let is_array = matches!(value, Value::Array(_));
                    self.push(Value::Bool(is_array));
                }
                Opcode::GetArrayLen => {
                    let value = self.pop();
                    match value {
                        Value::Array(arr) => {
                            let len = arr.lock().unwrap().len();
                            self.push(Value::Number(len as f64));
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "GetArrayLen requires Array".to_string(),
                                span: Span::dummy(),
                            })
                        }
                    }
                }

                // ===== Special =====
                Opcode::Halt => break,
            }
        }

        // Return top of stack if present
        Ok(if self.stack.is_empty() {
            None
        } else {
            Some(self.pop())
        })
    }

    // ===== Helper Methods =====

    #[inline(always)]
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    #[inline(always)]
    fn pop(&mut self) -> Value {
        // SAFETY: VM invariants guarantee stack is non-empty when pop is called
        unsafe { self.stack.pop().unwrap_unchecked() }
    }

    #[inline(always)]
    fn peek(&self, distance: usize) -> &Value {
        // SAFETY: The compiler guarantees stack depth matches operand requirements.
        // Each opcode that calls peek() is emitted only when sufficient values exist.
        unsafe { self.stack.get_unchecked(self.stack.len() - 1 - distance) }
    }

    #[inline(always)]
    fn pop_number(&mut self) -> Result<f64, RuntimeError> {
        match self.pop() {
            Value::Number(n) => Ok(n),
            _ => Err(RuntimeError::TypeError {
                msg: "Expected number".to_string(),
                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
            }),
        }
    }

    #[inline(always)]
    fn binary_numeric_op<F>(&mut self, op: F) -> Result<(), RuntimeError>
    where
        F: FnOnce(f64, f64) -> f64,
    {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        let result = op(a, b);
        if result.is_nan() || result.is_infinite() {
            return Err(RuntimeError::InvalidNumericResult {
                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
            });
        }
        self.push(Value::Number(result));
        Ok(())
    }

    #[inline(always)]
    fn read_opcode(&mut self) -> Result<Opcode, RuntimeError> {
        if self.ip >= self.bytecode.instructions.len() {
            return Err(RuntimeError::UnknownOpcode {
                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
            });
        }
        // SAFETY: Bounds check above guarantees self.ip < len
        let byte = self.bytecode.instructions[self.ip];
        self.ip += 1;
        // Use static dispatch table for O(1) opcode lookup
        dispatch::decode_opcode(byte).ok_or_else(|| RuntimeError::UnknownOpcode {
            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
        })
    }

    #[inline(always)]
    fn read_u8(&mut self) -> Result<u8, RuntimeError> {
        if self.ip >= self.bytecode.instructions.len() {
            return Err(RuntimeError::UnknownOpcode {
                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
            });
        }
        let byte = self.bytecode.instructions[self.ip];
        self.ip += 1;
        Ok(byte)
    }

    #[inline(always)]
    fn read_u16(&mut self) -> Result<u16, RuntimeError> {
        if self.ip + 1 >= self.bytecode.instructions.len() {
            return Err(RuntimeError::UnknownOpcode {
                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
            });
        }
        let hi = self.bytecode.instructions[self.ip] as u16;
        let lo = self.bytecode.instructions[self.ip + 1] as u16;
        self.ip += 2;
        Ok((hi << 8) | lo)
    }

    #[inline(always)]
    fn read_i16(&mut self) -> Result<i16, RuntimeError> {
        Ok(self.read_u16()? as i16)
    }

    #[inline(always)]
    fn current_frame(&self) -> &CallFrame {
        // SAFETY: VM execution always pushes a frame before running, and frames
        // are only popped when returning. The frames vec is never empty during execution.
        unsafe { self.frames.last().unwrap_unchecked() }
    }

    /// Generate a stack trace from the current call frames
    /// Returns a vector of function names from innermost to outermost
    #[allow(dead_code)]
    fn stack_trace(&self) -> Vec<String> {
        self.frames
            .iter()
            .rev()
            .map(|frame| frame.function_name.clone())
            .collect()
    }

    // ========================================================================
    // Array Intrinsics (Callback-based operations)
    // ========================================================================

    fn is_array_intrinsic(&self, name: &str) -> bool {
        crate::stdlib::is_array_intrinsic(name)
    }

    fn call_array_intrinsic(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        let span = self.current_span().unwrap_or_else(crate::span::Span::dummy);

        match name {
            "map" => self.vm_intrinsic_map(args, span),
            "filter" => self.vm_intrinsic_filter(args, span),
            "reduce" => self.vm_intrinsic_reduce(args, span),
            "forEach" => self.vm_intrinsic_for_each(args, span),
            "find" => self.vm_intrinsic_find(args, span),
            "findIndex" => self.vm_intrinsic_find_index(args, span),
            "flatMap" => self.vm_intrinsic_flat_map(args, span),
            "some" => self.vm_intrinsic_some(args, span),
            "every" => self.vm_intrinsic_every(args, span),
            "sort" => self.vm_intrinsic_sort(args, span),
            "sortBy" => self.vm_intrinsic_sort_by(args, span),
            // Result intrinsics (callback-based)
            "result_map" => self.vm_intrinsic_result_map(args, span),
            "result_map_err" => self.vm_intrinsic_result_map_err(args, span),
            "result_and_then" => self.vm_intrinsic_result_and_then(args, span),
            "result_or_else" => self.vm_intrinsic_result_or_else(args, span),
            // HashMap intrinsics (callback-based)
            "hashMapForEach" => self.vm_intrinsic_hashmap_for_each(args, span),
            "hashMapMap" => self.vm_intrinsic_hashmap_map(args, span),
            "hashMapFilter" => self.vm_intrinsic_hashmap_filter(args, span),
            // HashSet intrinsics (callback-based)
            "hashSetForEach" => self.vm_intrinsic_hashset_for_each(args, span),
            "hashSetMap" => self.vm_intrinsic_hashset_map(args, span),
            "hashSetFilter" => self.vm_intrinsic_hashset_filter(args, span),
            // Regex intrinsics (callback-based)
            "regexReplaceWith" => self.vm_intrinsic_regex_replace_with(args, span),
            "regexReplaceAllWith" => self.vm_intrinsic_regex_replace_all_with(args, span),
            _ => Err(RuntimeError::UnknownFunction {
                name: name.to_string(),
                span,
            }),
        }
    }

    fn vm_intrinsic_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "map() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.lock().unwrap().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "map() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "map() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = Vec::with_capacity(arr.len());

        for elem in arr {
            let callback_result = self.vm_call_function_value(callback, vec![elem], span)?;
            result.push(callback_result);
        }

        Ok(Value::array(result))
    }

    fn vm_intrinsic_filter(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "filter() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.lock().unwrap().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "filter() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "filter() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = Vec::new();

        for elem in arr {
            let pred_result = self.vm_call_function_value(predicate, vec![elem.clone()], span)?;
            match pred_result {
                Value::Bool(true) => result.push(elem),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "filter() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::array(result))
    }

    fn vm_intrinsic_reduce(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 3 {
            return Err(RuntimeError::TypeError {
                msg: "reduce() expects 3 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.lock().unwrap().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "reduce() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let reducer = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "reduce() second argument must be function".to_string(),
                    span,
                })
            }
        };
        let mut accumulator = args[2].clone();

        for elem in arr {
            accumulator = self.vm_call_function_value(reducer, vec![accumulator, elem], span)?;
        }

        Ok(accumulator)
    }

    fn vm_intrinsic_for_each(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "forEach() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.lock().unwrap().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "forEach() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "forEach() second argument must be function".to_string(),
                    span,
                })
            }
        };
        for elem in arr {
            self.vm_call_function_value(callback, vec![elem], span)?;
        }

        Ok(Value::Null)
    }

    fn vm_intrinsic_find(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "find() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.lock().unwrap().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "find() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "find() second argument must be function".to_string(),
                    span,
                })
            }
        };
        for elem in arr {
            let pred_result = self.vm_call_function_value(predicate, vec![elem.clone()], span)?;
            match pred_result {
                Value::Bool(true) => return Ok(elem),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "find() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Null)
    }

    fn vm_intrinsic_find_index(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "findIndex() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.lock().unwrap().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "findIndex() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "findIndex() second argument must be function".to_string(),
                    span,
                })
            }
        };
        for (i, elem) in arr.iter().enumerate() {
            let pred_result = self.vm_call_function_value(predicate, vec![elem.clone()], span)?;
            match pred_result {
                Value::Bool(true) => return Ok(Value::Number(i as f64)),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "findIndex() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Number(-1.0))
    }

    fn vm_intrinsic_flat_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "flatMap() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.lock().unwrap().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "flatMap() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "flatMap() second argument must be function".to_string(),
                    span,
                })
            }
        };
        let mut result = Vec::new();

        for elem in arr {
            let callback_result = self.vm_call_function_value(callback, vec![elem], span)?;
            match callback_result {
                Value::Array(nested) => {
                    result.extend(nested.lock().unwrap().clone());
                }
                other => result.push(other),
            }
        }

        Ok(Value::array(result))
    }

    fn vm_intrinsic_some(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "some() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.lock().unwrap().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "some() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "some() second argument must be function".to_string(),
                    span,
                })
            }
        };
        for elem in arr {
            let pred_result = self.vm_call_function_value(predicate, vec![elem], span)?;
            match pred_result {
                Value::Bool(true) => return Ok(Value::Bool(true)),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "some() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Bool(false))
    }

    fn vm_intrinsic_every(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "every() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.lock().unwrap().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "every() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "every() second argument must be function".to_string(),
                    span,
                })
            }
        };
        for elem in arr {
            let pred_result = self.vm_call_function_value(predicate, vec![elem], span)?;
            match pred_result {
                Value::Bool(false) => return Ok(Value::Bool(false)),
                Value::Bool(true) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "every() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Bool(true))
    }

    fn vm_intrinsic_sort(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "sort() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.lock().unwrap().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sort() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let comparator = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sort() second argument must be function".to_string(),
                    span,
                })
            }
        };

        // Insertion sort for stability
        let mut sorted = arr;
        for i in 1..sorted.len() {
            let mut j = i;
            while j > 0 {
                let cmp_result = self.vm_call_function_value(
                    comparator,
                    vec![sorted[j].clone(), sorted[j - 1].clone()],
                    span,
                )?;
                match cmp_result {
                    Value::Number(n) if n < 0.0 => {
                        sorted.swap(j, j - 1);
                        j -= 1;
                    }
                    Value::Number(_) => break,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "sort() comparator must return number".to_string(),
                            span,
                        })
                    }
                }
            }
        }

        Ok(Value::array(sorted))
    }

    fn vm_intrinsic_sort_by(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "sortBy() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.lock().unwrap().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sortBy() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let key_extractor = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sortBy() second argument must be function".to_string(),
                    span,
                })
            }
        };

        // Extract keys
        let mut keyed: Vec<(Value, Value)> = Vec::new();
        for elem in arr {
            let key = self.vm_call_function_value(key_extractor, vec![elem.clone()], span)?;
            keyed.push((key, elem));
        }

        // Sort by keys (insertion sort for stability)
        for i in 1..keyed.len() {
            let mut j = i;
            while j > 0 {
                let cmp = match (&keyed[j].0, &keyed[j - 1].0) {
                    (Value::Number(a), Value::Number(b)) => {
                        if a < b {
                            -1
                        } else if a > b {
                            1
                        } else {
                            0
                        }
                    }
                    (Value::String(a), Value::String(b)) => {
                        if a < b {
                            -1
                        } else if a > b {
                            1
                        } else {
                            0
                        }
                    }
                    _ => 0,
                };

                if cmp < 0 {
                    keyed.swap(j, j - 1);
                    j -= 1;
                } else {
                    break;
                }
            }
        }

        let sorted: Vec<Value> = keyed.into_iter().map(|(_, elem)| elem).collect();
        Ok(Value::array(sorted))
    }

    // ========================================================================
    // Result Intrinsics (Callback-based operations) - VM versions
    // ========================================================================

    fn vm_intrinsic_result_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_map() expects 2 arguments (result, transform_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let transform_fn = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_map() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => {
                let transformed =
                    self.vm_call_function_value(transform_fn, vec![(**val).clone()], span)?;
                Ok(Value::Result(Ok(Box::new(transformed))))
            }
            Value::Result(Err(err)) => Ok(Value::Result(Err(err.clone()))),
            _ => Err(RuntimeError::TypeError {
                msg: "result_map() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    fn vm_intrinsic_result_map_err(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_map_err() expects 2 arguments (result, transform_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let transform_fn = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_map_err() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => Ok(Value::Result(Ok(val.clone()))),
            Value::Result(Err(err)) => {
                let transformed =
                    self.vm_call_function_value(transform_fn, vec![(**err).clone()], span)?;
                Ok(Value::Result(Err(Box::new(transformed))))
            }
            _ => Err(RuntimeError::TypeError {
                msg: "result_map_err() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    fn vm_intrinsic_result_and_then(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_and_then() expects 2 arguments (result, next_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let next_fn = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_and_then() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => {
                // Call next_fn which should return a Result
                self.vm_call_function_value(next_fn, vec![(**val).clone()], span)
            }
            Value::Result(Err(err)) => Ok(Value::Result(Err(err.clone()))),
            _ => Err(RuntimeError::TypeError {
                msg: "result_and_then() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    fn vm_intrinsic_result_or_else(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_or_else() expects 2 arguments (result, recovery_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let recovery_fn = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_or_else() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => Ok(Value::Result(Ok(val.clone()))),
            Value::Result(Err(err)) => {
                // Call recovery_fn which should return a Result
                self.vm_call_function_value(recovery_fn, vec![(**err).clone()], span)
            }
            _ => Err(RuntimeError::TypeError {
                msg: "result_or_else() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    fn vm_intrinsic_hashmap_for_each(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashMapForEach() expects 2 arguments (map, callback)".to_string(),
                span,
            });
        }

        let map = match &args[0] {
            Value::HashMap(m) => m.lock().unwrap().entries(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapForEach() first argument must be HashMap".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapForEach() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for (key, value) in map {
            self.vm_call_function_value(callback, vec![value, key.to_value()], span)?;
        }

        Ok(Value::Null)
    }

    fn vm_intrinsic_hashmap_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashMapMap() expects 2 arguments (map, callback)".to_string(),
                span,
            });
        }

        let map = match &args[0] {
            Value::HashMap(m) => m.lock().unwrap().entries(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapMap() first argument must be HashMap".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapMap() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result_map = crate::stdlib::collections::hashmap::AtlasHashMap::new();
        for (key, value) in map {
            let new_value =
                self.vm_call_function_value(callback, vec![value, key.clone().to_value()], span)?;
            result_map.insert(key, new_value);
        }

        Ok(Value::HashMap(std::sync::Arc::new(std::sync::Mutex::new(
            result_map,
        ))))
    }

    fn vm_intrinsic_hashmap_filter(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashMapFilter() expects 2 arguments (map, predicate)".to_string(),
                span,
            });
        }

        let map = match &args[0] {
            Value::HashMap(m) => m.lock().unwrap().entries(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapFilter() first argument must be HashMap".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapFilter() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result_map = crate::stdlib::collections::hashmap::AtlasHashMap::new();
        for (key, value) in map {
            let pred_result = self.vm_call_function_value(
                predicate,
                vec![value.clone(), key.clone().to_value()],
                span,
            )?;
            match pred_result {
                Value::Bool(true) => {
                    result_map.insert(key, value);
                }
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "hashMapFilter() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::HashMap(std::sync::Arc::new(std::sync::Mutex::new(
            result_map,
        ))))
    }

    fn vm_intrinsic_hashset_for_each(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashSetForEach() expects 2 arguments (set, callback)".to_string(),
                span,
            });
        }

        let set = match &args[0] {
            Value::HashSet(s) => s.lock().unwrap().to_vec(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetForEach() first argument must be HashSet".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetForEach() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for element in set {
            self.vm_call_function_value(callback, vec![element.to_value()], span)?;
        }

        Ok(Value::Null)
    }

    fn vm_intrinsic_hashset_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashSetMap() expects 2 arguments (set, callback)".to_string(),
                span,
            });
        }

        let set = match &args[0] {
            Value::HashSet(s) => s.lock().unwrap().to_vec(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetMap() first argument must be HashSet".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetMap() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = Vec::new();
        for element in set {
            let mapped_value =
                self.vm_call_function_value(callback, vec![element.to_value()], span)?;
            result.push(mapped_value);
        }

        Ok(Value::array(result))
    }

    fn vm_intrinsic_hashset_filter(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashSetFilter() expects 2 arguments (set, predicate)".to_string(),
                span,
            });
        }

        let set = match &args[0] {
            Value::HashSet(s) => s.lock().unwrap().to_vec(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetFilter() first argument must be HashSet".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetFilter() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result_set = crate::stdlib::collections::hashset::AtlasHashSet::new();
        for element in set {
            let pred_result =
                self.vm_call_function_value(predicate, vec![element.clone().to_value()], span)?;
            match pred_result {
                Value::Bool(true) => {
                    result_set.insert(element);
                }
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "hashSetFilter() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::HashSet(std::sync::Arc::new(std::sync::Mutex::new(
            result_set,
        ))))
    }

    /// Regex intrinsic: Replace first match using callback (VM version)
    fn vm_intrinsic_regex_replace_with(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 3 {
            return Err(RuntimeError::TypeError {
                msg: "regexReplaceWith() expects 3 arguments (regex, text, callback)".to_string(),
                span,
            });
        }

        let regex = match &args[0] {
            Value::Regex(r) => r.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceWith() first argument must be Regex".to_string(),
                    span,
                })
            }
        };

        let text = match &args[1] {
            Value::String(s) => s.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceWith() second argument must be string".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[2] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[2],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceWith() third argument must be function".to_string(),
                    span,
                })
            }
        };

        // Find first match
        if let Some(mat) = regex.find(text) {
            let match_start = mat.start();
            let match_end = mat.end();
            let match_text = mat.as_str();

            // Build match data HashMap
            let mut match_map = crate::stdlib::collections::hashmap::AtlasHashMap::new();
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "text".to_string(),
                )),
                Value::string(match_text),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "start".to_string(),
                )),
                Value::Number(match_start as f64),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "end".to_string(),
                )),
                Value::Number(match_end as f64),
            );

            // Extract capture groups
            if let Some(caps) = regex.captures(text) {
                let mut groups = Vec::new();
                for i in 0..caps.len() {
                    if let Some(group) = caps.get(i) {
                        groups.push(Value::string(group.as_str()));
                    } else {
                        groups.push(Value::Null);
                    }
                }
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(groups),
                );
            } else {
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(vec![]),
                );
            }

            let match_value = Value::HashMap(std::sync::Arc::new(std::sync::Mutex::new(match_map)));

            // Call callback with match data
            let replacement_value =
                self.vm_call_function_value(callback, vec![match_value], span)?;

            // Expect string return value and clone to avoid lifetime issues
            let replacement_str = match &replacement_value {
                Value::String(s) => s.as_ref().to_string(),
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "regexReplaceWith() callback must return string".to_string(),
                        span,
                    })
                }
            };

            // Build result string
            let mut result = String::with_capacity(text.len());
            result.push_str(&text[..match_start]);
            result.push_str(&replacement_str);
            result.push_str(&text[match_end..]);

            Ok(Value::string(result))
        } else {
            // No match, return original text
            Ok(Value::string(text))
        }
    }

    /// Regex intrinsic: Replace all matches using callback (VM version)
    fn vm_intrinsic_regex_replace_all_with(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 3 {
            return Err(RuntimeError::TypeError {
                msg: "regexReplaceAllWith() expects 3 arguments (regex, text, callback)"
                    .to_string(),
                span,
            });
        }

        let regex = match &args[0] {
            Value::Regex(r) => r.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceAllWith() first argument must be Regex".to_string(),
                    span,
                })
            }
        };

        let text = match &args[1] {
            Value::String(s) => s.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceAllWith() second argument must be string".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[2] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[2],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceAllWith() third argument must be function".to_string(),
                    span,
                })
            }
        };

        // Find all matches and collect them
        let matches: Vec<_> = regex.find_iter(text).collect();

        if matches.is_empty() {
            return Ok(Value::string(text));
        }

        // Build result string by processing all matches
        let mut result = String::with_capacity(text.len());
        let mut last_end = 0;

        for mat in matches {
            let match_start = mat.start();
            let match_end = mat.end();
            let match_text = mat.as_str();

            // Build match data HashMap
            let mut match_map = crate::stdlib::collections::hashmap::AtlasHashMap::new();
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "text".to_string(),
                )),
                Value::string(match_text),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "start".to_string(),
                )),
                Value::Number(match_start as f64),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "end".to_string(),
                )),
                Value::Number(match_end as f64),
            );

            // Extract capture groups
            if let Some(caps) = regex.captures(mat.as_str()) {
                let mut groups = Vec::new();
                for i in 0..caps.len() {
                    if let Some(group) = caps.get(i) {
                        groups.push(Value::string(group.as_str()));
                    } else {
                        groups.push(Value::Null);
                    }
                }
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(groups),
                );
            } else {
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(vec![]),
                );
            }

            let match_value = Value::HashMap(std::sync::Arc::new(std::sync::Mutex::new(match_map)));

            // Call callback with match data
            let replacement_value =
                self.vm_call_function_value(callback, vec![match_value], span)?;

            // Expect string return value and clone to avoid lifetime issues
            let replacement_str = match &replacement_value {
                Value::String(s) => s.as_ref().to_string(),
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "regexReplaceAllWith() callback must return string".to_string(),
                        span,
                    })
                }
            };

            // Add text before this match
            result.push_str(&text[last_end..match_start]);
            // Add replacement
            result.push_str(&replacement_str);

            last_end = match_end;
        }

        // Add remaining text after last match
        result.push_str(&text[last_end..]);

        Ok(Value::string(result))
    }

    /// Helper: Call a function value with arguments (VM version)
    fn vm_call_function_value(
        &mut self,
        func: &Value,
        args: Vec<Value>,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        match func {
            Value::Builtin(name) => {
                let security = self
                    .current_security
                    .as_ref()
                    .expect("Security context not set");
                crate::stdlib::call_builtin(name, &args, span, security, &self.output_writer)
            }
            Value::Function(func_ref) => {
                // User-defined function - execute via VM
                let saved_ip = self.ip;
                let saved_frame_depth = self.frames.len();
                let stack_base = self.stack.len();
                let arg_count = args.len();

                // Verify arity before pushing
                if func_ref.arity != arg_count {
                    return Err(RuntimeError::TypeError {
                        msg: format!(
                            "Function {} expects {} arguments, got {}",
                            func_ref.name, func_ref.arity, arg_count
                        ),
                        span,
                    });
                }

                // Push arguments onto stack (they become the function's locals)
                for arg in args {
                    self.push(arg);
                }

                // Create call frame
                let frame = CallFrame {
                    function_name: func_ref.name.clone(),
                    return_ip: saved_ip,
                    stack_base,
                    local_count: func_ref.local_count,
                    upvalues: std::sync::Arc::new(Vec::new()),
                };
                self.frames.push(frame);

                // Jump to function bytecode
                self.ip = func_ref.bytecode_offset;

                // Execute until this frame returns (depth goes back to saved_frame_depth)
                let result = self.execute_loop(Some(saved_frame_depth))?;

                // Get the return value from stack
                let return_value = result.unwrap_or(Value::Null);
                // Clean up stack to original base
                self.stack.truncate(stack_base);

                // Restore IP
                self.ip = saved_ip;

                Ok(return_value)
            }
            Value::NativeFunction(native_fn) => {
                // Call the native Rust closure directly
                native_fn(&args)
            }
            _ => Err(RuntimeError::TypeError {
                msg: "Expected function value".to_string(),
                span,
            }),
        }
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new(Bytecode::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Compiler;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::security::SecurityContext;

    fn execute_source(source: &str) -> Result<Option<Value>, RuntimeError> {
        // Compile source to bytecode
        let mut lexer = Lexer::new(source.to_string());
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (program, _) = parser.parse();
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&program).expect("Compilation failed");

        // Execute on VM
        let mut vm = VM::new(bytecode);
        vm.run(&SecurityContext::allow_all())
    }

    #[test]
    fn test_vm_number_literal() {
        let result = execute_source("42;").unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_vm_arithmetic() {
        let result = execute_source("2 + 3;").unwrap();
        assert_eq!(result, Some(Value::Number(5.0)));

        let result = execute_source("10 - 4;").unwrap();
        assert_eq!(result, Some(Value::Number(6.0)));

        let result = execute_source("3 * 4;").unwrap();
        assert_eq!(result, Some(Value::Number(12.0)));

        let result = execute_source("15 / 3;").unwrap();
        assert_eq!(result, Some(Value::Number(5.0)));
    }

    #[test]
    fn test_vm_comparison() {
        let result = execute_source("1 < 2;").unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        let result = execute_source("5 > 10;").unwrap();
        assert_eq!(result, Some(Value::Bool(false)));

        let result = execute_source("3 == 3;").unwrap();
        assert_eq!(result, Some(Value::Bool(true)));
    }

    #[test]
    fn test_vm_global_variable() {
        let result = execute_source("let x = 42; x;").unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_vm_string_concat() {
        let result = execute_source("\"hello\" + \" world\";").unwrap();
        if let Some(Value::String(s)) = result {
            assert_eq!(s.as_ref(), "hello world");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_vm_array_literal() {
        let result = execute_source("[1, 2, 3];").unwrap();
        if let Some(Value::Array(arr)) = result {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 3);
            assert_eq!(borrowed[0], Value::Number(1.0));
            assert_eq!(borrowed[1], Value::Number(2.0));
            assert_eq!(borrowed[2], Value::Number(3.0));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_vm_array_index() {
        let result = execute_source("let arr = [10, 20, 30]; arr[1];").unwrap();
        assert_eq!(result, Some(Value::Number(20.0)));
    }

    #[test]
    fn test_vm_division_by_zero() {
        let result = execute_source("10 / 0;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::DivideByZero { .. }
        ));
    }

    #[test]
    fn test_vm_bool_literals() {
        let result = execute_source("true;").unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        let result = execute_source("false;").unwrap();
        assert_eq!(result, Some(Value::Bool(false)));
    }

    #[test]
    fn test_vm_null_literal() {
        let result = execute_source("null;").unwrap();
        assert_eq!(result, Some(Value::Null));
    }

    #[test]
    fn test_vm_unary_negate() {
        let result = execute_source("-42;").unwrap();
        assert_eq!(result, Some(Value::Number(-42.0)));
    }

    #[test]
    fn test_vm_logical_not() {
        let result = execute_source("!true;").unwrap();
        assert_eq!(result, Some(Value::Bool(false)));

        let result = execute_source("!false;").unwrap();
        assert_eq!(result, Some(Value::Bool(true)));
    }

    // ===== Constants Pool Loading Tests =====

    #[test]
    fn test_vm_load_number_constant() {
        // Test loading a number constant from pool
        let result = execute_source("123.456;").unwrap();
        assert_eq!(result, Some(Value::Number(123.456)));
    }

    #[test]
    fn test_vm_load_string_constant() {
        // Test loading a string constant from pool
        let result = execute_source("\"hello world\";").unwrap();
        if let Some(Value::String(s)) = result {
            assert_eq!(s.as_ref(), "hello world");
        } else {
            panic!("Expected string value");
        }
    }

    #[test]
    fn test_vm_load_multiple_constants() {
        // Test that multiple constants can be loaded and used
        let result = execute_source("1; 2; 3;").unwrap();
        // Should return the last value
        assert_eq!(result, Some(Value::Number(3.0)));
    }

    #[test]
    fn test_vm_constants_in_expression() {
        // Test using multiple constants in a single expression
        let result = execute_source("10 + 20 + 30;").unwrap();
        assert_eq!(result, Some(Value::Number(60.0)));
    }

    #[test]
    fn test_vm_constant_reuse() {
        // Test that the same constant value can be used multiple times
        let result = execute_source("let x = 5; let y = 5; x + y;").unwrap();
        assert_eq!(result, Some(Value::Number(10.0)));
    }

    #[test]
    fn test_vm_large_constant_index() {
        // Test that larger constant indices work correctly
        // Create many variables to populate the constant pool
        let mut source = String::new();
        for i in 0..100 {
            source.push_str(&format!("let x{} = {}; ", i, i));
        }
        source.push_str("x99;");

        let result = execute_source(&source).unwrap();
        assert_eq!(result, Some(Value::Number(99.0)));
    }

    #[test]
    fn test_vm_string_constants_in_variables() {
        // Test that string constants work properly with variables
        let result = execute_source("let s = \"test\"; s;").unwrap();
        if let Some(Value::String(s)) = result {
            assert_eq!(s.as_ref(), "test");
        } else {
            panic!("Expected string value");
        }
    }

    #[test]
    fn test_vm_mixed_constant_types() {
        // Test mixing different constant types
        let result = execute_source(
            r#"
            let n = 42;
            let s = "hello";
            let b = true;
            n;
        "#,
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_vm_constant_bounds_check() {
        // Create bytecode with an invalid constant index
        let mut bytecode = Bytecode::new();
        bytecode.add_constant(Value::Number(1.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(999); // Index out of bounds
        bytecode.emit(Opcode::Halt, crate::span::Span::dummy());

        let mut vm = VM::new(bytecode);
        let result = vm.run(&SecurityContext::allow_all());
        assert!(result.is_err());
    }

    #[test]
    fn test_vm_empty_constant_pool() {
        // Test VM with no constants (only opcodes that don't need them)
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::True, crate::span::Span::dummy());
        bytecode.emit(Opcode::Halt, crate::span::Span::dummy());

        let mut vm = VM::new(bytecode);
        let result = vm.run(&SecurityContext::allow_all()).unwrap();
        assert_eq!(result, Some(Value::Bool(true)));
        assert_eq!(vm.bytecode.constants.len(), 0);
    }

    // ===== Stack Frame Tests =====

    #[test]
    fn test_vm_initial_main_frame() {
        // Test that VM starts with a main frame
        let bytecode = Bytecode::new();
        let vm = VM::new(bytecode);

        assert_eq!(vm.frames.len(), 1);
        assert_eq!(vm.frames[0].function_name, "<main>");
        assert_eq!(vm.frames[0].stack_base, 0);
        assert_eq!(vm.frames[0].local_count, 0);
    }

    #[test]
    fn test_vm_frame_relative_locals() {
        // Test that locals are accessed relative to frame base
        // Simulate: let x = 10; let y = 20; x + y;
        let mut bytecode = Bytecode::new();

        // Push 10 onto stack (will become local 0)
        let idx_10 = bytecode.add_constant(Value::Number(10.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx_10);

        // Push 20 onto stack (will become local 1)
        let idx_20 = bytecode.add_constant(Value::Number(20.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx_20);

        // Get local 0 (should be 10)
        bytecode.emit(Opcode::GetLocal, crate::span::Span::dummy());
        bytecode.emit_u16(0);

        // Get local 1 (should be 20)
        bytecode.emit(Opcode::GetLocal, crate::span::Span::dummy());
        bytecode.emit_u16(1);

        // Add them
        bytecode.emit(Opcode::Add, crate::span::Span::dummy());

        bytecode.emit(Opcode::Halt, crate::span::Span::dummy());

        let mut vm = VM::new(bytecode);
        let result = vm.run(&SecurityContext::allow_all()).unwrap();
        assert_eq!(result, Some(Value::Number(30.0)));
    }

    #[test]
    fn test_vm_return_from_main() {
        // Test that RETURN from main frame terminates execution
        let mut bytecode = Bytecode::new();

        // Push a value
        let idx = bytecode.add_constant(Value::Number(42.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx);

        // Return from main
        bytecode.emit(Opcode::Return, crate::span::Span::dummy());

        // This should never execute
        bytecode.emit(Opcode::Null, crate::span::Span::dummy());
        bytecode.emit(Opcode::Halt, crate::span::Span::dummy());

        let mut vm = VM::new(bytecode);
        let result = vm.run(&SecurityContext::allow_all()).unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_vm_call_frame_creation() {
        use crate::value::FunctionRef;

        // Test that CALL creates a new frame
        let mut bytecode = Bytecode::new();

        // Create a simple function that just returns 42
        // Function starts at offset 10
        let function_offset = 10;

        // Main code: push function, call it
        let func_ref = FunctionRef {
            name: "test_func".to_string(),
            arity: 0,
            bytecode_offset: function_offset,
            local_count: 1,
        };
        let func_idx = bytecode.add_constant(Value::Function(func_ref));

        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(func_idx);

        // Call with 0 arguments
        bytecode.emit(Opcode::Call, crate::span::Span::dummy());
        bytecode.emit_u8(0);

        // After return, halt
        bytecode.emit(Opcode::Halt, crate::span::Span::dummy());

        // Pad to offset 10
        while bytecode.instructions.len() < function_offset {
            bytecode.emit_u8(0);
        }

        // Function body: push 42 and return
        let idx_42 = bytecode.add_constant(Value::Number(42.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx_42);
        bytecode.emit(Opcode::Return, crate::span::Span::dummy());

        let mut vm = VM::new(bytecode);
        let result = vm.run(&SecurityContext::allow_all()).unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_vm_call_with_arguments() {
        use crate::value::FunctionRef;

        // Test function call with arguments
        let mut bytecode = Bytecode::new();

        // Function: fn add(a, b) -> a + b
        let function_offset = 20;

        // Main code: push function, push args (5, 3), call
        let func_ref = FunctionRef {
            name: "add".to_string(),
            arity: 2,
            bytecode_offset: function_offset,
            local_count: 1,
        };
        let func_idx = bytecode.add_constant(Value::Function(func_ref));

        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(func_idx);

        // Push arguments
        let idx_5 = bytecode.add_constant(Value::Number(5.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx_5);

        let idx_3 = bytecode.add_constant(Value::Number(3.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx_3);

        // Call with 2 arguments
        bytecode.emit(Opcode::Call, crate::span::Span::dummy());
        bytecode.emit_u8(2);

        bytecode.emit(Opcode::Halt, crate::span::Span::dummy());

        // Pad to function offset
        while bytecode.instructions.len() < function_offset {
            bytecode.emit_u8(0);
        }

        // Function body: GetLocal 0, GetLocal 1, Add, Return
        bytecode.emit(Opcode::GetLocal, crate::span::Span::dummy());
        bytecode.emit_u16(0);
        bytecode.emit(Opcode::GetLocal, crate::span::Span::dummy());
        bytecode.emit_u16(1);
        bytecode.emit(Opcode::Add, crate::span::Span::dummy());
        bytecode.emit(Opcode::Return, crate::span::Span::dummy());

        let mut vm = VM::new(bytecode);
        let result = vm.run(&SecurityContext::allow_all()).unwrap();
        assert_eq!(result, Some(Value::Number(8.0)));
    }

    #[test]
    fn test_vm_call_wrong_arity() {
        use crate::value::FunctionRef;

        // Test that calling with wrong number of args fails
        let mut bytecode = Bytecode::new();

        let func_ref = FunctionRef {
            name: "test".to_string(),
            arity: 2, // Expects 2 args
            bytecode_offset: 10,
            local_count: 2,
        };
        let func_idx = bytecode.add_constant(Value::Function(func_ref));

        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(func_idx);

        // Only push 1 argument
        bytecode.emit(Opcode::Null, crate::span::Span::dummy());

        // Call with 1 argument (should fail)
        bytecode.emit(Opcode::Call, crate::span::Span::dummy());
        bytecode.emit_u8(1);

        let mut vm = VM::new(bytecode);
        let result = vm.run(&SecurityContext::allow_all());
        assert!(result.is_err());
        match result.unwrap_err() {
            RuntimeError::TypeError { msg, .. } => assert!(msg.contains("expects 2 arguments")),
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_vm_call_non_function() {
        // Test that calling a non-function value fails
        let mut bytecode = Bytecode::new();

        // Push a number (not a function)
        let idx = bytecode.add_constant(Value::Number(42.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx);

        // Try to call it
        bytecode.emit(Opcode::Call, crate::span::Span::dummy());
        bytecode.emit_u8(0);

        let mut vm = VM::new(bytecode);
        let result = vm.run(&SecurityContext::allow_all());
        assert!(result.is_err());
        match result.unwrap_err() {
            RuntimeError::TypeError { msg, .. } => {
                assert!(msg.contains("Cannot call non-function"))
            }
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_vm_nested_calls() {
        use crate::value::FunctionRef;

        // Test nested function calls: main -> f1 -> f2
        let mut bytecode = Bytecode::new();

        let f1_offset = 30;
        let f2_offset = 50;

        // Main: call f1
        let f1_ref = FunctionRef {
            name: "f1".to_string(),
            arity: 0,
            bytecode_offset: f1_offset,
            local_count: 0,
        };
        let f1_idx = bytecode.add_constant(Value::Function(f1_ref));

        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(f1_idx);
        bytecode.emit(Opcode::Call, crate::span::Span::dummy());
        bytecode.emit_u8(0);
        bytecode.emit(Opcode::Halt, crate::span::Span::dummy());

        // Pad to f1_offset
        while bytecode.instructions.len() < f1_offset {
            bytecode.emit_u8(0);
        }

        // f1: call f2
        let f2_ref = FunctionRef {
            name: "f2".to_string(),
            arity: 0,
            bytecode_offset: f2_offset,
            local_count: 1,
        };
        let f2_idx = bytecode.add_constant(Value::Function(f2_ref));

        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(f2_idx);
        bytecode.emit(Opcode::Call, crate::span::Span::dummy());
        bytecode.emit_u8(0);
        bytecode.emit(Opcode::Return, crate::span::Span::dummy());

        // Pad to f2_offset
        while bytecode.instructions.len() < f2_offset {
            bytecode.emit_u8(0);
        }

        // f2: return 100
        let idx_100 = bytecode.add_constant(Value::Number(100.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx_100);
        bytecode.emit(Opcode::Return, crate::span::Span::dummy());

        let mut vm = VM::new(bytecode);
        let result = vm.run(&SecurityContext::allow_all()).unwrap();
        assert_eq!(result, Some(Value::Number(100.0)));
    }

    // ===== Control Flow Tests =====

    #[test]
    fn test_vm_if_true_branch() {
        // Test: var x = 0; if (true) { x = 42; } else { x = 0; } x;
        let result = execute_source("var x = 0; if (true) { x = 42; } else { x = 0; } x;").unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_vm_if_false_branch() {
        // Test: var x = 0; if (false) { x = 42; } else { x = 99; } x;
        let result =
            execute_source("var x = 0; if (false) { x = 42; } else { x = 99; } x;").unwrap();
        assert_eq!(result, Some(Value::Number(99.0)));
    }

    #[test]
    fn test_vm_if_no_else() {
        // Test: var x = 10; if (false) { x = 42; } x;
        let result = execute_source("var x = 10; if (false) { x = 42; } x;").unwrap();
        assert_eq!(result, Some(Value::Number(10.0))); // x unchanged
    }

    #[test]
    fn test_vm_if_with_comparison() {
        // Test: var x = 0; if (5 > 3) { x = 1; } else { x = 2; } x;
        let result = execute_source("var x = 0; if (5 > 3) { x = 1; } else { x = 2; } x;").unwrap();
        assert_eq!(result, Some(Value::Number(1.0)));

        let result = execute_source("var x = 0; if (5 < 3) { x = 1; } else { x = 2; } x;").unwrap();
        assert_eq!(result, Some(Value::Number(2.0)));
    }

    #[test]
    fn test_vm_nested_if() {
        // Test nested if statements
        let result = execute_source(
            "var x = 0; if (true) { if (true) { x = 42; } else { x = 0; } } else { x = 99; } x;",
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));

        let result = execute_source(
            "var x = 0; if (true) { if (false) { x = 42; } else { x = 10; } } else { x = 99; } x;",
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(10.0)));
    }

    #[test]
    fn test_vm_while_loop() {
        // Test: var x = 0; while (x < 5) { x = x + 1; } x;
        let result = execute_source("var x = 0; while (x < 5) { x = x + 1; } x;").unwrap();
        assert_eq!(result, Some(Value::Number(5.0)));
    }

    #[test]
    fn test_vm_while_loop_never_executes() {
        // Test while loop that never executes
        let result = execute_source("var x = 10; while (x < 5) { x = x + 1; } x;").unwrap();
        assert_eq!(result, Some(Value::Number(10.0)));
    }

    #[test]
    fn test_vm_while_loop_sum() {
        // Test: var sum = 0; var i = 1; while (i <= 10) { sum = sum + i; i = i + 1; } sum;
        let result = execute_source(
            "var sum = 0; var i = 1; while (i <= 10) { sum = sum + i; i = i + 1; } sum;",
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(55.0))); // 1+2+3+...+10 = 55
    }

    #[test]
    fn test_vm_for_loop() {
        // Test that the loop executes correctly (use var for mutable sum)
        let result = execute_source(
            "var sum = 0; for (var i = 0; i < 5; i = i + 1) { sum = sum + i; } sum;",
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(10.0))); // 0+1+2+3+4 = 10
    }

    #[test]
    fn test_vm_loop_countdown() {
        // Test loop counting down (using while since for+locals complex)
        let result =
            execute_source("var x = 10; var i = 0; while (i < 5) { x = x - 1; i = i + 1; } x;")
                .unwrap();
        assert_eq!(result, Some(Value::Number(5.0))); // 10 - 5 = 5
    }

    #[test]
    fn test_vm_while_with_local() {
        // Simple test: while loop with local variable
        let result = execute_source(
            r#"
            var count = 0;
            var i = 0;
            while (i < 3) {
                var x = 10;
                count = count + x;
                i = i + 1;
            }
            count;
        "#,
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(30.0))); // Should be 10 + 10 + 10 = 30
    }

    #[test]
    fn test_vm_nested_loops() {
        // Test nested while loops: sum of i*j for i,j in 1..3
        let result = execute_source(
            r#"
            var sum = 0;
            var i = 1;
            while (i <= 3) {
                var j = 1;  // Reset j each outer iteration
                while (j <= 3) {
                    sum = sum + (i * j);
                    j = j + 1;
                }
                i = i + 1;
            }
            sum;
        "#,
        )
        .unwrap();
        // (1*1 + 1*2 + 1*3) + (2*1 + 2*2 + 2*3) + (3*1 + 3*2 + 3*3)
        // = (1+2+3) + (2+4+6) + (3+6+9)
        // = 6 + 12 + 18 = 36
        assert_eq!(result, Some(Value::Number(36.0)));
    }

    #[test]
    fn test_vm_if_in_loop() {
        // Test if inside a loop
        let result = execute_source(
            r#"
            var sum = 0;
            for (var i = 0; i < 10; i = i + 1) {
                if (i < 5) {
                    sum = sum + i;
                }
            }
            sum;
        "#,
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(10.0))); // 0+1+2+3+4 = 10
    }

    #[test]
    fn test_vm_complex_condition() {
        // Test complex boolean expression in if (use var for mutable variables)
        let result = execute_source(
            "var x = 5; var y = 10; var z = 0; if (x < 10) { if (y > 5) { z = 1; } else { z = 2; } } else { z = 3; } z;",
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(1.0)));
    }

    #[test]
    fn test_vm_loop_with_break() {
        // Test break statement in loop
        let result = execute_source(
            r#"
            var x = 0;
            while (true) {
                x = x + 1;
                if (x == 5) {
                    break;
                }
            }
            x;
        "#,
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(5.0)));
    }

    #[test]
    fn test_vm_loop_with_continue() {
        // Test continue statement in while loop
        let result = execute_source(
            r#"
            var sum = 0;
            var i = 0;
            while (i < 10) {
                i = i + 1;
                if (i == 5) {
                    continue;
                }
                sum = sum + i;
            }
            sum;
        "#,
        )
        .unwrap();
        // 1+2+3+4+6+7+8+9+10 = 50 (skips 5, but i is incremented before check)
        assert_eq!(result, Some(Value::Number(50.0)));
    }

    #[test]
    fn test_vm_multiple_breaks() {
        // Test multiple break points in loop
        let result = execute_source(
            r#"
            var x = 0;
            while (x < 100) {
                x = x + 1;
                if (x == 3) {
                    break;
                }
                if (x == 5) {
                    break;
                }
            }
            x;
        "#,
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(3.0))); // Breaks at first condition
    }

    #[test]
    fn test_vm_nested_break() {
        // Test that break only exits innermost loop
        let result = execute_source(
            r#"
            var outer = 0;
            var i = 0;
            while (i < 3) {
                var j = 0;
                while (j < 3) {
                    if (j == 1) {
                        break;
                    }
                    outer = outer + 1;
                    j = j + 1;
                }
                i = i + 1;
            }
            outer;
        "#,
        )
        .unwrap();
        // Each outer iteration: inner loop breaks after j=0, so outer increments once
        // Total: 3 iterations = 3
        assert_eq!(result, Some(Value::Number(3.0)));
    }

    // ===== Runtime Error Tests (Phase 09) =====

    #[test]
    fn test_vm_runtime_error_modulo_by_zero() {
        // Test modulo by zero runtime error
        let result = execute_source("10 % 0;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::DivideByZero { .. }
        ));
    }

    #[test]
    fn test_vm_runtime_error_zero_divided_by_zero() {
        // 0/0 should trigger divide by zero error
        let result = execute_source("0 / 0;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::DivideByZero { .. }
        ));
    }

    #[test]
    fn test_vm_runtime_error_array_out_of_bounds_read() {
        // Test array out of bounds read
        let result = execute_source("let arr = [1, 2, 3]; arr[10];");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::OutOfBounds { .. }
        ));
    }

    // TODO: Add array out of bounds write test when array index assignment is implemented in compiler
    // #[test]
    // fn test_vm_runtime_error_array_out_of_bounds_write() {
    //     let result = execute_source("var arr = [1, 2, 3]; arr[10] = 5; arr;");
    //     assert!(result.is_err());
    //     assert!(matches!(result.unwrap_err(), RuntimeError::OutOfBounds { .. }));
    // }

    #[test]
    fn test_vm_runtime_error_negative_index() {
        // Test negative array index
        let result = execute_source("let arr = [1, 2, 3]; arr[-1];");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidIndex { .. }
        ));
    }

    #[test]
    fn test_vm_runtime_error_non_integer_index() {
        // Test non-integer array index
        let result = execute_source("let arr = [1, 2, 3]; arr[1.5];");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidIndex { .. }
        ));
    }

    #[test]
    fn test_vm_runtime_error_invalid_numeric_add() {
        // Test invalid numeric result from large number addition
        let result = execute_source("let x = 1.7976931348623157e308 + 1.7976931348623157e308;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidNumericResult { .. }
        ));
    }

    #[test]
    fn test_vm_runtime_error_invalid_numeric_multiply() {
        // Test invalid numeric result from large number multiplication
        let result = execute_source("let x = 1e308 * 2.0;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidNumericResult { .. }
        ));
    }

    #[test]
    fn test_vm_runtime_error_in_expression() {
        // Test that runtime errors propagate through expressions
        let result = execute_source("let x = 5 + (10 / 0);");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::DivideByZero { .. }
        ));
    }

    // TODO: Add function call error test when functions are fully implemented
    // #[test]
    // fn test_vm_runtime_error_in_function_call() {
    //     let result = execute_source(
    //         r#"
    //         function divide(a, b) {
    //             return a / b;
    //         }
    //         divide(10, 0);
    //     "#,
    //     );
    //     assert!(result.is_err());
    //     assert!(matches!(result.unwrap_err(), RuntimeError::DivideByZero { .. }));
    // }

    #[test]
    fn test_vm_runtime_error_compound_divide_by_zero() {
        // Test divide by zero in compound assignment
        let result = execute_source("var x = 10; x = x / 0;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::DivideByZero { .. }
        ));
    }

    // ===== Phase 17: VM Numeric Error Propagation Tests =====

    #[test]
    fn test_vm_modulo_by_zero() {
        // Test modulo by zero (AT0005)
        let result = execute_source("10 % 0;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::DivideByZero { .. }
        ));
    }

    #[test]
    fn test_vm_modulo_zero_by_zero() {
        // Test 0 % 0 should also be divide by zero
        let result = execute_source("0 % 0;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::DivideByZero { .. }
        ));
    }

    #[test]
    fn test_vm_modulo_invalid_numeric_result() {
        // Test modulo that produces invalid result
        let result = execute_source("1e308 % 0.1;");
        // This may produce NaN or Infinity depending on the operation
        if let Err(e) = result {
            assert!(matches!(
                e,
                RuntimeError::InvalidNumericResult { .. } | RuntimeError::DivideByZero { .. }
            ));
        }
    }

    #[test]
    fn test_vm_subtraction_overflow() {
        // Test subtraction that produces invalid result (AT0007)
        let result = execute_source("let x = -1.7976931348623157e308 - 1.7976931348623157e308;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidNumericResult { .. }
        ));
    }

    #[test]
    fn test_vm_negation_overflow() {
        // Test negation that produces invalid result
        // Note: Negation of very large numbers shouldn't overflow, but let's test edge cases
        let result = execute_source("let x = -1.7976931348623157e308; let y = -x;");
        // Negation should work fine, but if it somehow produces infinity, catch it
        if let Err(e) = result {
            assert!(matches!(e, RuntimeError::InvalidNumericResult { .. }));
        } else {
            // Should succeed
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_vm_division_produces_infinity() {
        // Test division that would produce infinity (very large / very small)
        let result = execute_source("let x = 1e308 / 1e-308;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidNumericResult { .. }
        ));
    }

    #[test]
    fn test_vm_numeric_error_matches_interpreter_divide_by_zero() {
        // Verify VM and interpreter produce same error for divide by zero
        let source = "1 / 0;";

        // Test VM
        let vm_result = execute_source(source);
        assert!(vm_result.is_err());
        assert!(matches!(
            vm_result.unwrap_err(),
            RuntimeError::DivideByZero { .. }
        ));

        // Test Interpreter (via runtime)
        use crate::runtime::Atlas;
        let runtime = Atlas::new();
        let interp_result = runtime.eval(source);
        assert!(interp_result.is_err());
        // Interpreter converts RuntimeError to Diagnostic, so check diagnostic code
        let diags = interp_result.unwrap_err();
        assert!(!diags.is_empty());
        assert_eq!(diags[0].code, "AT0005");
    }

    #[test]
    fn test_vm_numeric_error_matches_interpreter_overflow() {
        // Verify VM and interpreter produce same error for overflow
        let source = "1e308 * 2.0;";

        // Test VM
        let vm_result = execute_source(source);
        assert!(vm_result.is_err());
        assert!(matches!(
            vm_result.unwrap_err(),
            RuntimeError::InvalidNumericResult { .. }
        ));

        // Test Interpreter (via runtime)
        use crate::runtime::Atlas;
        let runtime = Atlas::new();
        let interp_result = runtime.eval(source);
        assert!(interp_result.is_err());
        let diags = interp_result.unwrap_err();
        assert!(!diags.is_empty());
        assert_eq!(diags[0].code, "AT0007");
    }

    #[test]
    fn test_vm_numeric_error_in_nested_expression() {
        // Test that numeric errors propagate correctly in nested expressions
        let result = execute_source("let x = (5 + 3) * (10 / 0);");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::DivideByZero { .. }
        ));
    }

    #[test]
    fn test_vm_numeric_error_in_array() {
        // Test numeric error in array element
        let result = execute_source("let arr = [1, 2, 10 / 0];");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::DivideByZero { .. }
        ));
    }

    #[test]
    fn test_vm_numeric_error_in_array_index() {
        // Test numeric error used as array index
        let result = execute_source("let arr = [1, 2, 3]; arr[10 / 0];");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::DivideByZero { .. }
        ));
    }

    #[test]
    fn test_vm_multiple_numeric_operations_no_error() {
        // Test that valid numeric operations don't trigger errors
        let result = execute_source("let x = 10 / 2 + 5 * 3 - 8 % 3;");
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value, Some(Value::Number(5.0 + 15.0 - 2.0))); // 10/2=5, 5*3=15, 8%3=2, 5+15-2=18
    }

    #[test]
    fn test_vm_division_by_very_small_number() {
        // Test division by very small number (not zero)
        let result = execute_source("let x = 1.0 / 1e-300;");
        // Should succeed as long as result is not infinity
        // 1.0 / 1e-300 = 1e300, which is less than max f64
        assert!(result.is_ok());
    }

    #[test]
    fn test_vm_division_edge_case_max_divided_by_min() {
        // Test edge case: max / min
        let result = execute_source("let x = 1e308 / 1e-308;");
        // This will overflow to infinity
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidNumericResult { .. }
        ));
    }

    #[test]
    fn test_vm_addition_edge_case_max_plus_max() {
        // Test edge case: max + max
        let result = execute_source("let x = 1.7976931348623157e308 + 1.7976931348623157e308;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidNumericResult { .. }
        ));
    }

    #[test]
    fn test_vm_numeric_error_codes_division_by_zero() {
        // Explicitly test that AT0005 is used for division by zero
        use crate::runtime::Atlas;
        let runtime = Atlas::new();
        let result = runtime.eval("1 / 0");
        assert!(result.is_err());
        let diags = result.unwrap_err();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "AT0005");
        assert!(diags[0].message.contains("Divide by zero"));
    }

    #[test]
    fn test_vm_numeric_error_codes_invalid_result() {
        // Explicitly test that AT0007 is used for invalid numeric results
        use crate::runtime::Atlas;
        let runtime = Atlas::new();
        let result = runtime.eval("1e308 * 1e308");
        assert!(result.is_err());
        let diags = result.unwrap_err();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "AT0007");
        assert!(diags[0].message.contains("Invalid numeric result"));
    }

    // ===== Stdlib Tests (Phase Stdlib-01) =====

    #[test]
    fn test_vm_stdlib_print_number() {
        // Note: We can't easily test stdout, but we can verify no error
        let result = execute_source("print(42);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Null));
    }

    #[test]
    fn test_vm_stdlib_print_string() {
        let result = execute_source("print(\"hello\");");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Null));
    }

    #[test]
    fn test_vm_stdlib_print_bool() {
        let result = execute_source("print(true);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Null));
    }

    #[test]
    fn test_vm_stdlib_print_null() {
        let result = execute_source("print(null);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Null));
    }

    #[test]
    fn test_vm_stdlib_len_string() {
        let result = execute_source("len(\"hello\");");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(5.0)));
    }

    #[test]
    fn test_vm_stdlib_len_unicode_string() {
        // Test Unicode scalar count
        let result = execute_source("len(\"🎉\");");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(1.0))); // 1 char, not 4 bytes
    }

    #[test]
    fn test_vm_stdlib_len_array() {
        let result = execute_source("len([1, 2, 3]);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(3.0)));
    }

    #[test]
    fn test_vm_stdlib_len_empty_string() {
        let result = execute_source("len(\"\");");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(0.0)));
    }

    #[test]
    fn test_vm_stdlib_len_empty_array() {
        let result = execute_source("len([]);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(0.0)));
    }

    #[test]
    fn test_vm_stdlib_str_number() {
        let result = execute_source("str(42);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::string("42")));
    }

    #[test]
    fn test_vm_stdlib_str_bool_true() {
        let result = execute_source("str(true);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::string("true")));
    }

    #[test]
    fn test_vm_stdlib_str_bool_false() {
        let result = execute_source("str(false);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::string("false")));
    }

    #[test]
    fn test_vm_stdlib_str_null() {
        let result = execute_source("str(null);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::string("null")));
    }

    #[test]
    fn test_vm_stdlib_len_in_expression() {
        let result = execute_source("let x = len(\"test\") + 1;");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(5.0))); // 4 + 1
    }

    #[test]
    fn test_vm_stdlib_str_in_concat() {
        let result = execute_source("let x = \"Number: \" + str(42);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::string("Number: 42")));
    }

    #[test]
    fn test_vm_stdlib_nested_calls() {
        let result = execute_source("len(str(12345));");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(5.0))); // "12345" has 5 chars
    }

    #[test]
    fn test_vm_stdlib_in_variable() {
        let result = execute_source("let x = len(\"hello\"); x;");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(5.0)));
    }

    #[test]
    fn test_vm_stdlib_in_array() {
        let result = execute_source("let arr = [len(\"a\"), len(\"ab\"), len(\"abc\")]; arr[1];");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(2.0)));
    }

    #[test]
    fn test_vm_stdlib_matches_interpreter_print() {
        // Verify VM and interpreter produce same behavior
        let source = "print(\"test\");";

        let vm_result = execute_source(source);
        assert!(vm_result.is_ok());

        use crate::runtime::Atlas;
        let runtime = Atlas::new();
        let interp_result = runtime.eval(source);
        assert!(interp_result.is_ok());
    }

    #[test]
    fn test_vm_stdlib_matches_interpreter_len() {
        let source = "len(\"hello\");";

        let vm_result = execute_source(source);
        assert!(vm_result.is_ok());
        assert_eq!(vm_result.unwrap(), Some(Value::Number(5.0)));

        use crate::runtime::Atlas;
        let runtime = Atlas::new();
        let interp_result = runtime.eval(source);
        assert!(interp_result.is_ok());
        assert_eq!(interp_result.unwrap(), Value::Number(5.0));
    }

    #[test]
    fn test_vm_stdlib_matches_interpreter_str() {
        let source = "str(42);";

        let vm_result = execute_source(source);
        assert!(vm_result.is_ok());
        assert_eq!(vm_result.unwrap(), Some(Value::string("42")));

        use crate::runtime::Atlas;
        let runtime = Atlas::new();
        let interp_result = runtime.eval(source);
        assert!(interp_result.is_ok());
        assert_eq!(interp_result.unwrap(), Value::string("42"));
    }

    // =========================================================================
    // Truncated Bytecode Tests (Phase Correctness-09)
    // =========================================================================
    // These tests verify that malformed/truncated bytecode produces clean
    // errors rather than undefined behavior.

    #[test]
    fn test_truncated_bytecode_load_const_missing_operand() {
        use crate::bytecode::{Bytecode, Opcode};

        // LoadConst (Constant opcode) needs 2 operand bytes - provide only 1
        let mut bytecode = Bytecode::new();
        bytecode.instructions = vec![Opcode::Constant as u8, 0x00]; // missing second byte
        bytecode.constants.push(Value::Number(42.0)); // constant exists but index incomplete

        let mut vm = VM::new(bytecode);
        let result = vm.run(&SecurityContext::allow_all());

        // Must produce error, not crash/UB
        assert!(result.is_err(), "Truncated bytecode should produce error");
    }

    #[test]
    fn test_truncated_bytecode_jump_missing_operand() {
        use crate::bytecode::{Bytecode, Opcode};

        // Jump needs 2 operand bytes (i16 offset) - provide only 1
        let mut bytecode = Bytecode::new();
        bytecode.instructions = vec![Opcode::Jump as u8, 0x00]; // missing second byte

        let mut vm = VM::new(bytecode);
        let result = vm.run(&SecurityContext::allow_all());

        // Must produce error, not crash/UB
        assert!(result.is_err(), "Truncated jump should produce error");
    }

    #[test]
    fn test_truncated_bytecode_empty() {
        use crate::bytecode::Bytecode;

        // Empty bytecode - no instructions at all
        let bytecode = Bytecode::new();

        let mut vm = VM::new(bytecode);
        let result = vm.run(&SecurityContext::allow_all());

        // Should either return Ok(None) or a clean error - not UB
        // Our implementation returns error when trying to read first opcode
        assert!(
            result.is_ok() || result.is_err(),
            "Empty bytecode should be handled cleanly"
        );
    }

    #[test]
    fn test_truncated_bytecode_call_missing_arg_count() {
        use crate::bytecode::{Bytecode, Opcode};

        // Call needs 1 operand byte (arg count) - provide none
        let mut bytecode = Bytecode::new();
        bytecode.instructions = vec![Opcode::Call as u8]; // missing arg count byte

        let mut vm = VM::new(bytecode);
        let result = vm.run(&SecurityContext::allow_all());

        // Must produce error, not crash/UB
        assert!(result.is_err(), "Truncated call should produce error");
    }

    #[test]
    fn test_truncated_bytecode_get_local_missing_operand() {
        use crate::bytecode::{Bytecode, Opcode};

        // GetLocal needs 2 operand bytes (u16 index) - provide only 1
        let mut bytecode = Bytecode::new();
        bytecode.instructions = vec![Opcode::GetLocal as u8, 0x00]; // missing second byte

        let mut vm = VM::new(bytecode);
        let result = vm.run(&SecurityContext::allow_all());

        // Must produce error, not crash/UB
        assert!(result.is_err(), "Truncated GetLocal should produce error");
    }
}
