//! AST interpreter (tree-walking)
//!
//! Direct AST evaluation with environment-based variable storage.
//! Supports:
//! - Expression evaluation (literals, binary/unary ops, calls, indexing)
//! - Statement execution (declarations, assignments, control flow)
//! - Function calls and stack frames
//! - Block scoping with shadowing

mod expr;
mod stmt;

use crate::ast::{Block, Item, Param, Program};
use crate::ffi::{ExternFunction, LibraryLoader};
use crate::value::{FunctionRef, RuntimeError, Value};
use std::collections::HashMap;

/// Control flow signal for handling break, continue, and return
#[derive(Debug, Clone, PartialEq)]
pub(super) enum ControlFlow {
    None,
    Break,
    Continue,
    Return(Value),
}

/// User-defined function
#[derive(Debug, Clone)]
pub(super) struct UserFunction {
    pub(super) name: String,
    pub(super) params: Vec<Param>,
    pub(super) body: Block,
}

/// Interpreter state
pub struct Interpreter {
    /// Global variables
    pub(super) globals: HashMap<String, Value>,
    /// Local scopes (stack of environments)
    pub(super) locals: Vec<HashMap<String, Value>>,
    /// User-defined function bodies (accessed via Value::Function references)
    pub(super) function_bodies: HashMap<String, UserFunction>,
    /// Current control flow state
    pub(super) control_flow: ControlFlow,
    /// Monomorphizer for generic functions (tracks type substitutions)
    #[allow(dead_code)] // Will be used when generic runtime support is fully integrated
    pub(super) monomorphizer: crate::typechecker::generics::Monomorphizer,
    /// Security context for current evaluation (set during eval())
    pub(super) current_security: Option<*const crate::security::SecurityContext>,
    /// Counter for generating unique nested function names
    next_func_id: usize,
    /// FFI library loader (phase-10b)
    library_loader: LibraryLoader,
    /// Loaded extern functions (phase-10b)
    extern_functions: HashMap<String, ExternFunction>,
}

impl Interpreter {
    /// Create a new interpreter
    pub fn new() -> Self {
        let mut interpreter = Self {
            globals: HashMap::new(),
            locals: vec![HashMap::new()],
            function_bodies: HashMap::new(),
            control_flow: ControlFlow::None,
            monomorphizer: crate::typechecker::generics::Monomorphizer::new(),
            current_security: None,
            next_func_id: 0,
            library_loader: LibraryLoader::new(),
            extern_functions: HashMap::new(),
        };

        // Register builtin functions in globals
        // Core builtins
        interpreter.register_builtin("print", 1);
        interpreter.register_builtin("len", 1);
        interpreter.register_builtin("str", 1);

        // String functions
        interpreter.register_builtin("split", 2);
        interpreter.register_builtin("join", 2);
        interpreter.register_builtin("trim", 1);
        interpreter.register_builtin("trimStart", 1);
        interpreter.register_builtin("trimEnd", 1);
        interpreter.register_builtin("indexOf", 2);
        interpreter.register_builtin("lastIndexOf", 2);
        interpreter.register_builtin("includes", 2);
        interpreter.register_builtin("startsWith", 2);
        interpreter.register_builtin("endsWith", 2);
        interpreter.register_builtin("substring", 3);
        interpreter.register_builtin("charAt", 2);
        interpreter.register_builtin("toUpperCase", 1);
        interpreter.register_builtin("toLowerCase", 1);
        interpreter.register_builtin("repeat", 2);
        interpreter.register_builtin("replace", 3);
        interpreter.register_builtin("padStart", 3);
        interpreter.register_builtin("padEnd", 3);

        interpreter
    }

    /// Register a builtin function in globals
    fn register_builtin(&mut self, name: &str, arity: usize) {
        let func_value = Value::Function(FunctionRef {
            name: name.to_string(),
            arity,
            bytecode_offset: 0, // Not used in interpreter
            local_count: 0,     // Not used in interpreter
        });
        self.globals.insert(name.to_string(), func_value);
    }

    /// Evaluate a program
    pub fn eval(
        &mut self,
        program: &Program,
        security: &crate::security::SecurityContext,
    ) -> Result<Value, RuntimeError> {
        // Store security context for builtin calls
        self.current_security = Some(security as *const _);

        let mut last_value = Value::Null;

        for item in &program.items {
            match item {
                Item::Function(func) => {
                    // Store user-defined function body
                    self.function_bodies.insert(
                        func.name.name.clone(),
                        UserFunction {
                            name: func.name.name.clone(),
                            params: func.params.clone(),
                            body: func.body.clone(),
                        },
                    );

                    // Also store as a value for reference
                    let func_value = Value::Function(FunctionRef {
                        name: func.name.name.clone(),
                        arity: func.params.len(),
                        bytecode_offset: 0, // Not used in interpreter
                        local_count: 0,     // Not used in interpreter
                    });
                    self.globals.insert(func.name.name.clone(), func_value);
                }
                Item::Statement(stmt) => {
                    last_value = self.eval_statement(stmt)?;

                    // Check for early return at top level
                    if let ControlFlow::Return(val) = &self.control_flow {
                        last_value = val.clone();
                        self.control_flow = ControlFlow::None;
                        break;
                    }
                }
                Item::Import(_) => {
                    // Import execution handled in BLOCKER 04-D (module loading)
                    // For now, just skip - imports are syntactically valid but not yet functional
                }
                Item::Export(export_decl) => {
                    // Export wraps an item - evaluate the inner item
                    match &export_decl.item {
                        crate::ast::ExportItem::Function(func) => {
                            // Same as Function case above
                            self.function_bodies.insert(
                                func.name.name.clone(),
                                UserFunction {
                                    name: func.name.name.clone(),
                                    params: func.params.clone(),
                                    body: func.body.clone(),
                                },
                            );
                            let func_value = Value::Function(FunctionRef {
                                name: func.name.name.clone(),
                                arity: func.params.len(),
                                bytecode_offset: 0,
                                local_count: 0,
                            });
                            self.globals.insert(func.name.name.clone(), func_value);
                        }
                        crate::ast::ExportItem::Variable(var) => {
                            // Evaluate the variable declaration
                            let value = self.eval_expr(&var.init)?;
                            // Store in globals (not locals) so exports can be extracted
                            self.globals.insert(var.name.name.clone(), value);
                            last_value = Value::Null;
                        }
                    }
                }
                Item::Extern(extern_decl) => {
                    // Load the dynamic library
                    self.library_loader
                        .load(&extern_decl.library)
                        .map_err(|e| RuntimeError::TypeError {
                            msg: format!("Failed to load library '{}': {}", extern_decl.library, e),
                            span: extern_decl.span,
                        })?;

                    // Determine the symbol name (use 'as' name if provided, otherwise function name)
                    let symbol_name = extern_decl
                        .symbol
                        .as_ref()
                        .unwrap_or(&extern_decl.name);

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
                        .map(|(_, ty)| Self::convert_extern_type_annotation(ty))
                        .collect();

                    let return_type = Self::convert_extern_type_annotation(&extern_decl.return_type);

                    // Create ExternFunction
                    let extern_fn = unsafe {
                        ExternFunction::new(*fn_ptr, param_types, return_type)
                    };

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
                    last_value = Value::Null;
                }
            }
        }

        Ok(last_value)
    }

    /// Convert ExternTypeAnnotation (AST) to ExternType (FFI runtime)
    fn convert_extern_type_annotation(annotation: &crate::ast::ExternTypeAnnotation) -> crate::ffi::ExternType {
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

    /// Get a variable value
    pub(super) fn get_variable(
        &self,
        name: &str,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        // Check locals (innermost to outermost)
        for scope in self.locals.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }

        // Check globals
        if let Some(value) = self.globals.get(name) {
            return Ok(value.clone());
        }

        // Check builtins - return a function value for builtin functions
        if crate::stdlib::is_builtin(name) {
            return Ok(Value::Function(crate::value::FunctionRef {
                name: name.to_string(),
                arity: 0, // Arity not used for builtins
                bytecode_offset: 0,
                local_count: 0,
            }));
        }

        // Check array intrinsics - return a function value
        if crate::stdlib::is_array_intrinsic(name) {
            return Ok(Value::Function(crate::value::FunctionRef {
                name: name.to_string(),
                arity: 0, // Arity checked at call site
                bytecode_offset: 0,
                local_count: 0,
            }));
        }

        // Check math constants
        match name {
            "PI" => return Ok(Value::Number(crate::stdlib::math::PI)),
            "E" => return Ok(Value::Number(crate::stdlib::math::E)),
            "SQRT2" => return Ok(Value::Number(crate::stdlib::math::SQRT2)),
            "LN2" => return Ok(Value::Number(crate::stdlib::math::LN2)),
            "LN10" => return Ok(Value::Number(crate::stdlib::math::LN10)),
            _ => {}
        }

        Err(RuntimeError::UndefinedVariable {
            name: name.to_string(),
            span,
        })
    }

    /// Set a variable value
    pub(super) fn set_variable(
        &mut self,
        name: &str,
        value: Value,
        span: crate::span::Span,
    ) -> Result<(), RuntimeError> {
        // Find in locals (innermost to outermost)
        for scope in self.locals.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }

        // Check globals
        if self.globals.contains_key(name) {
            self.globals.insert(name.to_string(), value);
            return Ok(());
        }

        Err(RuntimeError::UndefinedVariable {
            name: name.to_string(),
            span,
        })
    }

    /// Get an array element by index
    pub(super) fn get_array_element(
        &self,
        arr: Value,
        idx: Value,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if let Value::Array(arr) = arr {
            if let Value::Number(n) = idx {
                let index_val = n as i64;
                if n.fract() != 0.0 || n < 0.0 {
                    return Err(RuntimeError::InvalidIndex { span });
                }

                let borrowed = arr.borrow();
                if index_val >= 0 && (index_val as usize) < borrowed.len() {
                    Ok(borrowed[index_val as usize].clone())
                } else {
                    Err(RuntimeError::OutOfBounds { span })
                }
            } else {
                Err(RuntimeError::InvalidIndex { span })
            }
        } else {
            Err(RuntimeError::TypeError {
                msg: "Cannot index non-array".to_string(),
                span,
            })
        }
    }

    /// Set an array element by index
    pub(super) fn set_array_element(
        &self,
        arr: Value,
        idx: Value,
        value: Value,
        span: crate::span::Span,
    ) -> Result<(), RuntimeError> {
        if let Value::Array(arr) = arr {
            if let Value::Number(n) = idx {
                let index_val = n as i64;
                if n.fract() != 0.0 || n < 0.0 {
                    return Err(RuntimeError::InvalidIndex { span });
                }

                let mut borrowed = arr.borrow_mut();
                if index_val >= 0 && (index_val as usize) < borrowed.len() {
                    borrowed[index_val as usize] = value;
                    Ok(())
                } else {
                    Err(RuntimeError::OutOfBounds { span })
                }
            } else {
                Err(RuntimeError::InvalidIndex { span })
            }
        } else {
            Err(RuntimeError::TypeError {
                msg: "Cannot index non-array".to_string(),
                span,
            })
        }
    }

    /// Push a new scope
    pub(super) fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    /// Pop the current scope
    pub(super) fn pop_scope(&mut self) {
        self.locals.pop();
    }

    /// Define a global variable (for testing/REPL)
    pub fn define_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_interpreter_creation() {
        let mut interp = Interpreter::new();
        interp.define_global("x".to_string(), Value::Number(42.0));
        assert!(interp.globals.contains_key("x"));
    }

    #[test]
    fn test_eval_literal() {
        let interp = Interpreter::new();
        assert_eq!(
            interp.eval_literal(&Literal::Number(42.0)),
            Value::Number(42.0)
        );
        assert_eq!(interp.eval_literal(&Literal::Bool(true)), Value::Bool(true));
        assert_eq!(interp.eval_literal(&Literal::Null), Value::Null);
    }

    #[test]
    fn test_scope_management() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.locals.len(), 1);

        interp.push_scope();
        assert_eq!(interp.locals.len(), 2);

        interp.pop_scope();
        assert_eq!(interp.locals.len(), 1);
    }
}
