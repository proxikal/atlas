//! AST interpreter (tree-walking)
//!
//! Direct AST evaluation with environment-based variable storage.
//! Supports:
//! - Expression evaluation (literals, binary/unary ops, calls, indexing)
//! - Statement execution (declarations, assignments, control flow)
//! - Function calls and stack frames
//! - Block scoping with shadowing

pub mod cache;
pub mod debugger;
mod expr;
mod stmt;

use crate::ast::{Block, ImportDecl, ImportSpecifier, Item, Param, Program};
use crate::ffi::{CallbackHandle, ExternFunction, LibraryLoader};
use crate::module_loader::ModuleLoader;
use crate::resolver::ModuleResolver;
use crate::value::{FunctionRef, RuntimeError, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

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
pub struct UserFunction {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Block,
}

/// Interpreter state
pub struct Interpreter {
    /// Global variables (value, is_mutable)
    pub(super) globals: HashMap<String, (Value, bool)>,
    /// Local scopes (stack of environments) - each entry is (value, is_mutable)
    pub(super) locals: Vec<HashMap<String, (Value, bool)>>,
    /// User-defined function bodies (accessed via Value::Function references)
    pub(super) function_bodies: HashMap<String, UserFunction>,
    /// Current control flow state
    pub(super) control_flow: ControlFlow,
    /// Monomorphizer for generic functions (tracks type substitutions)
    #[allow(dead_code)] // Will be used when generic runtime support is fully integrated
    pub(super) monomorphizer: crate::typechecker::generics::Monomorphizer,
    /// Security context for current evaluation (set during eval())
    pub(super) current_security: Option<std::sync::Arc<crate::security::SecurityContext>>,
    /// Output writer for print() (defaults to stdout)
    pub(super) output_writer: crate::stdlib::OutputWriter,
    /// Counter for generating unique nested function names
    next_func_id: usize,
    /// FFI library loader (phase-10b)
    library_loader: LibraryLoader,
    /// Loaded extern functions (phase-10b)
    extern_functions: HashMap<String, ExternFunction>,
    /// Registered callbacks for C→Atlas calls (phase-10c)
    callbacks: Vec<CallbackHandle>,
    /// Current module path for resolving relative imports (set by ModuleExecutor or eval_file)
    current_module_path: Option<PathBuf>,
    /// Cache of module exports (path -> exports map) for inline import processing
    module_exports_cache: HashMap<PathBuf, HashMap<String, Value>>,
    /// Lookup cache for optimized variable resolution (infrastructure for future optimization)
    #[allow(dead_code)]
    lookup_cache: cache::InterpreterCache,
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
            output_writer: crate::stdlib::stdout_writer(),
            next_func_id: 0,
            library_loader: LibraryLoader::new(),
            extern_functions: HashMap::new(),
            callbacks: Vec::new(),
            current_module_path: None,
            module_exports_cache: HashMap::new(),
            lookup_cache: cache::InterpreterCache::new(),
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

    /// Set the output writer (used by Runtime to redirect print() output)
    pub fn set_output_writer(&mut self, writer: crate::stdlib::OutputWriter) {
        self.output_writer = writer;
    }

    /// Register a builtin function in globals
    /// Builtins are immutable - they cannot be reassigned
    fn register_builtin(&mut self, name: &str, _arity: usize) {
        self.globals
            .insert(name.to_string(), (Value::Builtin(Arc::from(name)), false));
    }

    /// Get a cloned variable by name (search locals then globals).
    pub fn get_binding(&self, name: &str) -> Option<Value> {
        for scope in self.locals.iter().rev() {
            if let Some((value, _mutable)) = scope.get(name) {
                return Some(value.clone());
            }
        }
        self.globals.get(name).map(|(v, _)| v.clone())
    }

    /// Snapshot of current bindings (locals + globals) sorted by name.
    pub fn bindings_snapshot(&self) -> Vec<(String, Value)> {
        let mut entries: Vec<(String, Value)> = Vec::new();

        if let Some(scope) = self.locals.first() {
            for (k, (v, _mutable)) in scope {
                entries.push((k.clone(), v.clone()));
            }
        }

        for (k, (v, _mutable)) in &self.globals {
            entries.push((k.clone(), v.clone()));
        }

        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries
    }

    /// Evaluate a program
    pub fn eval(
        &mut self,
        program: &Program,
        security: &crate::security::SecurityContext,
    ) -> Result<Value, RuntimeError> {
        // Store security context for builtin calls
        self.current_security = Some(std::sync::Arc::new(security.clone()));

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
                    // Functions are immutable bindings
                    let func_value = Value::Function(FunctionRef {
                        name: func.name.name.clone(),
                        arity: func.params.len(),
                        bytecode_offset: 0, // Not used in interpreter
                        local_count: 0,     // Not used in interpreter
                    });
                    self.globals
                        .insert(func.name.name.clone(), (func_value, false));
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
                Item::Import(import_decl) => {
                    // Process the import: load module, evaluate, bind exports
                    // When called via ModuleExecutor, imports are pre-processed.
                    // When called directly (e.g., eval_file), we process inline.
                    self.process_import(import_decl)?;
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
                            // Functions are immutable bindings
                            let func_value = Value::Function(FunctionRef {
                                name: func.name.name.clone(),
                                arity: func.params.len(),
                                bytecode_offset: 0,
                                local_count: 0,
                            });
                            self.globals
                                .insert(func.name.name.clone(), (func_value, false));
                        }
                        crate::ast::ExportItem::Variable(var) => {
                            // Evaluate the variable declaration
                            let value = self.eval_expr(&var.init)?;
                            // Store in globals (not locals) so exports can be extracted
                            // Respect the variable's mutability
                            self.globals
                                .insert(var.name.name.clone(), (value, var.mutable));
                            last_value = Value::Null;
                        }
                        crate::ast::ExportItem::TypeAlias(_) => {
                            // Type aliases are compile-time only
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
                        .map(|(_, ty)| Self::convert_extern_type_annotation(ty))
                        .collect();

                    let return_type =
                        Self::convert_extern_type_annotation(&extern_decl.return_type);

                    // Create ExternFunction
                    let extern_fn =
                        unsafe { ExternFunction::new(*fn_ptr, param_types, return_type) };

                    // Store the extern function
                    self.extern_functions
                        .insert(extern_decl.name.clone(), extern_fn);

                    // Register as a callable global (extern functions are immutable)
                    let func_value = Value::Function(FunctionRef {
                        name: extern_decl.name.clone(),
                        arity: extern_decl.params.len(),
                        bytecode_offset: 0, // Not used for extern functions
                        local_count: 0,     // Not used for extern functions
                    });
                    self.globals
                        .insert(extern_decl.name.clone(), (func_value, false));
                    last_value = Value::Null;
                }
                Item::TypeAlias(_) => {
                    // Type aliases are compile-time only
                }
            }
        }

        Ok(last_value)
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

    /// Get a variable value
    pub(super) fn get_variable(
        &self,
        name: &str,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        // Check locals (innermost to outermost)
        for scope in self.locals.iter().rev() {
            if let Some((value, _mutable)) = scope.get(name) {
                return Ok(value.clone());
            }
        }

        // Check globals
        if let Some((value, _mutable)) = self.globals.get(name) {
            return Ok(value.clone());
        }

        // Constructor literals: None evaluates directly to Option::None
        if name == "None" {
            return Ok(Value::Option(None));
        }

        // Check builtins - return a builtin value
        if crate::stdlib::is_builtin(name) {
            return Ok(Value::Builtin(Arc::from(name)));
        }

        // Check array intrinsics - return a builtin value
        if crate::stdlib::is_array_intrinsic(name) {
            return Ok(Value::Builtin(Arc::from(name)));
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
    /// Returns an error if the variable is immutable (declared with 'let')
    pub(super) fn set_variable(
        &mut self,
        name: &str,
        value: Value,
        span: crate::span::Span,
    ) -> Result<(), RuntimeError> {
        // Find in locals (innermost to outermost)
        for scope in self.locals.iter_mut().rev() {
            if let Some((_, mutable)) = scope.get(name) {
                if !mutable {
                    return Err(RuntimeError::TypeError {
                        msg: format!("Cannot assign to immutable variable '{}'", name),
                        span,
                    });
                }
                scope.insert(name.to_string(), (value, true));
                return Ok(());
            }
        }

        // Check globals
        if let Some((_, mutable)) = self.globals.get(name) {
            if !mutable {
                return Err(RuntimeError::TypeError {
                    msg: format!("Cannot assign to immutable variable '{}'", name),
                    span,
                });
            }
            self.globals.insert(name.to_string(), (value, true));
            return Ok(());
        }

        Err(RuntimeError::UndefinedVariable {
            name: name.to_string(),
            span,
        })
    }

    /// Force-update a variable's value, bypassing the mutability check.
    ///
    /// Used by CoW write-back for collection mutation builtins: the binding
    /// itself isn't being rebound — the container's contents are being updated.
    /// This models "mutable container, immutable binding" semantics (like `const` in JS).
    pub(super) fn force_set_collection(&mut self, name: &str, value: Value) {
        for scope in self.locals.iter_mut().rev() {
            if scope.contains_key(name) {
                // Preserve existing mutability flag
                let mutable = scope.get(name).map(|(_, m)| *m).unwrap_or(false);
                scope.insert(name.to_string(), (value, mutable));
                return;
            }
        }
        if self.globals.contains_key(name) {
            let mutable = self.globals.get(name).map(|(_, m)| *m).unwrap_or(false);
            self.globals.insert(name.to_string(), (value, mutable));
        }
        // If variable doesn't exist, silently do nothing (best-effort write-back)
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

                if index_val >= 0 && (index_val as usize) < arr.len() {
                    Ok(arr[index_val as usize].clone())
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

    /// Assign `value` to the index position `idx` of the container named by `target_expr`,
    /// using Copy-on-Write semantics with full write-back.
    ///
    /// Pattern: clone container from env → mutate (CoW triggers if shared) → write back.
    /// Handles both simple variable targets (`arr[i] = val`) and nested index targets
    /// (`matrix[i][j] = val`) via recursive descent.
    pub(super) fn assign_at_index(
        &mut self,
        target_expr: &crate::ast::Expr,
        idx: Value,
        value: Value,
        span: crate::span::Span,
    ) -> Result<(), RuntimeError> {
        match target_expr {
            crate::ast::Expr::Identifier(id) => {
                // Base case: simple variable — clone → mutate → write back
                // Use force_set_collection: element mutation is not a variable rebinding.
                let mut container = self.get_variable(&id.name, span)?;
                Self::apply_index_mutation(&mut container, idx, value, span)?;
                self.force_set_collection(&id.name, container);
                Ok(())
            }
            crate::ast::Expr::Index(inner) => {
                // Nested: outer[inner_idx][idx] = value
                // 1. Evaluate inner index expression
                let inner_idx = self.eval_expr(&inner.index)?;
                // 2. Clone the intermediate element (eval_expr reads outer[inner_idx])
                let mut elem = self.eval_expr(target_expr)?;
                // 3. Mutate the element at idx (CoW triggers if shared)
                Self::apply_index_mutation(&mut elem, idx, value, span)?;
                // 4. Recursively write elem back to outer[inner_idx]
                self.assign_at_index(&inner.target, inner_idx, elem, span)
            }
            _ => Err(RuntimeError::TypeError {
                msg: "Invalid assignment target".to_string(),
                span,
            }),
        }
    }

    /// Apply a single index mutation to a container value (CoW semantics).
    ///
    /// `container` is mutated in-place via the CoW API — if the container's
    /// inner data is shared, `Arc::make_mut` clones it first. If exclusively
    /// owned, mutation is in-place with no allocation.
    fn apply_index_mutation(
        container: &mut Value,
        idx: Value,
        value: Value,
        span: crate::span::Span,
    ) -> Result<(), RuntimeError> {
        match (container, &idx) {
            (Value::Array(arr), Value::Number(n)) => {
                if n.fract() != 0.0 || *n < 0.0 {
                    return Err(RuntimeError::InvalidIndex { span });
                }
                let i = *n as usize;
                if i >= arr.len() {
                    return Err(RuntimeError::OutOfBounds { span });
                }
                arr.set(i, value);
                Ok(())
            }
            (container, _) => Err(RuntimeError::TypeError {
                msg: format!("Cannot index-assign to type '{}'", container.type_name()),
                span,
            }),
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
    /// Defaults to mutable for flexibility in interactive contexts
    pub fn define_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, (value, true));
    }

    /// Create a C-callable callback for an Atlas function (phase-10c)
    ///
    /// Returns a function pointer that C code can call. The callback will:
    /// 1. Receive C arguments
    /// 2. Marshal them to Atlas values
    /// 3. Call the Atlas function
    /// 4. Marshal the result back to C
    /// 5. Return to C code
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Function doesn't exist
    /// - Signature is invalid or unsupported
    /// - Marshaling fails
    pub fn create_callback(
        &mut self,
        function_name: &str,
        param_types: Vec<crate::ffi::ExternType>,
        return_type: crate::ffi::ExternType,
    ) -> Result<*const (), RuntimeError> {
        use crate::ffi::create_callback;

        // Verify function exists
        if !self.function_bodies.contains_key(function_name) {
            return Err(RuntimeError::TypeError {
                msg: format!("Function '{}' not found", function_name),
                span: crate::span::Span::dummy(),
            });
        }

        // Get function for closure
        let fn_name = function_name.to_string();
        let function_bodies = self.function_bodies.clone();
        let globals = self.globals.clone();
        let output_writer = self.output_writer.clone();

        // Create callback that calls interpreter
        let callback_fn = move |args: &[Value]| -> Result<Value, RuntimeError> {
            // Create a temporary interpreter for callback execution.
            // Inherit the parent's output_writer so print() inside callbacks
            // writes to the same destination as the parent interpreter.
            let mut temp_interp = Interpreter {
                globals: globals.clone(),
                locals: vec![HashMap::new()],
                function_bodies: function_bodies.clone(),
                control_flow: ControlFlow::None,
                monomorphizer: crate::typechecker::generics::Monomorphizer::new(),
                current_security: None,
                output_writer: output_writer.clone(),
                next_func_id: 0,
                library_loader: LibraryLoader::new(),
                extern_functions: HashMap::new(),
                callbacks: Vec::new(),
                current_module_path: None,
                module_exports_cache: HashMap::new(),
                lookup_cache: cache::InterpreterCache::new(),
            };

            // Get function body
            let func = temp_interp
                .function_bodies
                .get(&fn_name)
                .ok_or_else(|| RuntimeError::TypeError {
                    msg: format!("Function '{}' not found", fn_name),
                    span: crate::span::Span::dummy(),
                })?
                .clone();

            // Create new scope for function call
            temp_interp.push_scope();

            // Bind parameters (parameters are mutable)
            for (i, param) in func.params.iter().enumerate() {
                if let Some(arg) = args.get(i) {
                    temp_interp
                        .locals
                        .last_mut()
                        .unwrap()
                        .insert(param.name.name.clone(), (arg.clone(), true));
                }
            }

            // Execute function body
            let result = temp_interp.eval_block(&func.body)?;

            // Pop scope
            temp_interp.pop_scope();

            // Handle return value
            match temp_interp.control_flow {
                ControlFlow::Return(val) => Ok(val),
                _ => Ok(result),
            }
        };

        // Create callback handle
        let handle = create_callback(callback_fn, param_types, return_type).map_err(|e| {
            RuntimeError::TypeError {
                msg: format!("Failed to create callback: {}", e),
                span: crate::span::Span::dummy(),
            }
        })?;

        let trampoline = handle.trampoline();
        self.callbacks.push(handle);

        Ok(trampoline)
    }

    /// Get the number of registered callbacks
    pub fn callback_count(&self) -> usize {
        self.callbacks.len()
    }

    /// Set the current module path for import resolution
    ///
    /// Called by ModuleExecutor before eval() to enable relative import resolution.
    /// Also used by Runtime.eval_file() for file-based execution.
    pub fn set_module_path(&mut self, path: Option<PathBuf>) {
        self.current_module_path = path;
    }

    /// Get the current module path
    pub fn module_path(&self) -> Option<&PathBuf> {
        self.current_module_path.as_ref()
    }

    /// Process an import declaration
    ///
    /// Loads the imported module, evaluates it (if not cached), and binds
    /// the imported symbols into the current interpreter's globals.
    ///
    /// # Arguments
    /// * `import` - The import declaration to process
    ///
    /// # Returns
    /// * `Ok(())` if imports were processed successfully
    /// * `Err(RuntimeError)` if module not found, circular import, or export not found
    fn process_import(&mut self, import: &ImportDecl) -> Result<(), RuntimeError> {
        // Get current module path - required for relative import resolution
        let current_path = self.current_module_path.clone().ok_or_else(|| {
            RuntimeError::TypeError {
                msg: "Cannot process imports without a module context. Use eval_file() for file-based execution.".to_string(),
                span: import.span,
            }
        })?;

        // Determine project root from current path (parent directory)
        let project_root = current_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        // Resolve the import path
        let mut resolver = ModuleResolver::new(project_root.clone());
        let import_path = resolver
            .resolve_path(&import.source, &current_path, import.span)
            .map_err(|diag| RuntimeError::TypeError {
                msg: diag.message,
                span: import.span,
            })?;

        // Check cache first
        if let Some(exports) = self.module_exports_cache.get(&import_path) {
            return self.bind_imports(import, exports.clone());
        }

        // Load and evaluate the module
        let mut loader = ModuleLoader::new(project_root);
        let modules = loader.load_module(&import_path).map_err(|diags| {
            let msg = diags
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("; ");
            RuntimeError::TypeError {
                msg,
                span: import.span,
            }
        })?;

        // Get security context (required for eval)
        let security = self
            .current_security
            .clone()
            .unwrap_or_else(|| Arc::new(crate::security::SecurityContext::allow_all()));

        // Evaluate each module in dependency order
        for module in &modules {
            // Skip if already cached
            if self.module_exports_cache.contains_key(&module.path) {
                continue;
            }

            // Set module path for recursive imports
            let prev_path = self.current_module_path.take();
            self.current_module_path = Some(module.path.clone());

            // Evaluate the module
            self.eval(&module.ast, &security)?;

            // Extract and cache exports
            let exports = self.extract_module_exports(module);
            self.module_exports_cache
                .insert(module.path.clone(), exports);

            // Restore previous path
            self.current_module_path = prev_path;
        }

        // Get the requested module's exports and bind imports
        let exports = self
            .module_exports_cache
            .get(&import_path)
            .cloned()
            .unwrap_or_default();
        self.bind_imports(import, exports)
    }

    /// Bind imported symbols to globals
    fn bind_imports(
        &mut self,
        import: &ImportDecl,
        exports: HashMap<String, Value>,
    ) -> Result<(), RuntimeError> {
        for specifier in &import.specifiers {
            match specifier {
                ImportSpecifier::Named { name, span } => {
                    let value = exports
                        .get(&name.name)
                        .ok_or_else(|| RuntimeError::TypeError {
                            msg: format!(
                                "'{}' is not exported from module '{}'",
                                name.name, import.source
                            ),
                            span: *span,
                        })?;
                    // Imported values are immutable bindings
                    self.globals
                        .insert(name.name.clone(), (value.clone(), false));
                }
                ImportSpecifier::Namespace { alias: _, span } => {
                    return Err(RuntimeError::TypeError {
                        msg: "Namespace imports (import * as) not yet implemented".to_string(),
                        span: *span,
                    });
                }
            }
        }
        Ok(())
    }

    /// Extract exports from an evaluated module
    fn extract_module_exports(
        &self,
        module: &crate::module_loader::LoadedModule,
    ) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        for item in &module.ast.items {
            if let Item::Export(export_decl) = item {
                match &export_decl.item {
                    crate::ast::ExportItem::Function(func) => {
                        if let Some((value, _)) = self.globals.get(&func.name.name) {
                            exports.insert(func.name.name.clone(), value.clone());
                        }
                    }
                    crate::ast::ExportItem::Variable(var) => {
                        if let Some((value, _)) = self.globals.get(&var.name.name) {
                            exports.insert(var.name.name.clone(), value.clone());
                        }
                    }
                    crate::ast::ExportItem::TypeAlias(_) => {
                        // Type aliases are compile-time only
                    }
                }
            }
        }
        exports
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
