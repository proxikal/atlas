# Phase 10b: FFI Library Loading + Extern Calls

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** Phase-10a complete (FFI core types + marshaling)

**Verification Steps:**
1. Verify phase-10a complete:
   ```bash
   ls crates/atlas-runtime/src/ffi/mod.rs
   ls crates/atlas-runtime/src/ffi/types.rs
   ls crates/atlas-runtime/src/ffi/marshal.rs
   grep -n "Type::Extern" crates/atlas-runtime/src/types.rs
   ```
2. Verify marshaling tests pass:
   ```bash
   cargo test -p atlas-runtime test_marshal -- --exact 2>&1 | grep "test result"
   ```
3. Check dependencies not yet added:
   ```bash
   grep "libffi\|libloading" crates/atlas-runtime/Cargo.toml || echo "Not added yet - will add"
   ```

**Expected State:**
- ✅ Phase-10a complete (extern types + marshaling)
- ✅ 30+ marshaling tests passing
- ❌ No library loading yet (we're creating it)
- ❌ No extern syntax yet (we're creating it)

---

## Objective

Implement dynamic library loading and extern function calling, enabling Atlas code to invoke C functions from shared libraries (.so/.dylib/.dll). Adds `extern` keyword to grammar, library loading infrastructure, symbol resolution, and function invocation using phase-10a's type marshaling.

This phase brings FFI from type theory to actual C function calls.

---

## Files

**Add Dependencies:** `crates/atlas-runtime/Cargo.toml` (~10 lines)
**Create:** `crates/atlas-runtime/src/ffi/loader.rs` (~350 lines)
**Create:** `crates/atlas-runtime/src/ffi/caller.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/ffi/mod.rs` (~50 lines)
**Update:** `crates/atlas-parser/src/ast.rs` (~80 lines - extern decl AST)
**Update:** `crates/atlas-parser/src/parser.rs` (~120 lines - extern syntax)
**Update:** `crates/atlas-runtime/src/interpreter/mod.rs` (~100 lines - extern calls)
**Update:** `crates/atlas-runtime/src/compiler/mod.rs` (~120 lines - extern codegen)
**Tests:** `crates/atlas-runtime/tests/ffi_loading_tests.rs` (~500 lines)
**Tests:** `crates/atlas-runtime/tests/ffi_integration_tests.rs` (~400 lines)

**Total:** ~2130 lines (library loading + extern calls)

---

## Implementation

### 1. Add FFI Dependencies

**File:** `crates/atlas-runtime/Cargo.toml`

```toml
[dependencies]
# ... existing dependencies ...

# FFI support
libffi = "3.2"           # Dynamic FFI calls
libloading = "0.8"       # Cross-platform dynamic library loading
```

---

### 2. Library Loader

**File:** `crates/atlas-runtime/src/ffi/loader.rs`

```rust
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum LoadError {
    LibraryNotFound(String),
    SymbolNotFound { library: String, symbol: String },
    LoadFailed(String),
}

pub struct LibraryLoader {
    /// Cache of loaded libraries by path
    loaded: HashMap<PathBuf, Library>,
    /// Platform-specific library search paths
    search_paths: Vec<PathBuf>,
}

impl LibraryLoader {
    pub fn new() -> Self {
        Self {
            loaded: HashMap::new(),
            search_paths: Self::default_search_paths(),
        }
    }

    /// Get platform-specific default library search paths
    fn default_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Platform-specific standard paths
        #[cfg(target_os = "linux")]
        {
            paths.push(PathBuf::from("/usr/lib"));
            paths.push(PathBuf::from("/usr/local/lib"));
            paths.push(PathBuf::from("/lib"));
        }

        #[cfg(target_os = "macos")]
        {
            paths.push(PathBuf::from("/usr/lib"));
            paths.push(PathBuf::from("/usr/local/lib"));
            paths.push(PathBuf::from("/opt/homebrew/lib"));
        }

        #[cfg(target_os = "windows")]
        {
            paths.push(PathBuf::from("C:\\Windows\\System32"));
        }

        // Current directory
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd);
        }

        paths
    }

    /// Resolve library name to full path with platform extension
    fn resolve_library_path(&self, name: &str) -> Option<PathBuf> {
        // Platform-specific extensions
        let extensions = if cfg!(target_os = "windows") {
            vec!["dll"]
        } else if cfg!(target_os = "macos") {
            vec!["dylib", "so"]
        } else {
            vec!["so"]
        };

        // Platform-specific prefixes
        let prefixes = if cfg!(target_os = "windows") {
            vec![""]
        } else {
            vec!["lib", ""]
        };

        // Try each combination
        for search_path in &self.search_paths {
            for prefix in &prefixes {
                for ext in &extensions {
                    let filename = format!("{}{}.{}", prefix, name, ext);
                    let full_path = search_path.join(&filename);
                    if full_path.exists() {
                        return Some(full_path);
                    }
                }
            }
        }

        None
    }

    /// Load a library by name or path
    pub fn load(&mut self, name: &str) -> Result<&Library, LoadError> {
        // Check if already loaded
        let path = if Path::new(name).exists() {
            PathBuf::from(name)
        } else {
            self.resolve_library_path(name)
                .ok_or_else(|| LoadError::LibraryNotFound(name.to_string()))?
        };

        if self.loaded.contains_key(&path) {
            return Ok(&self.loaded[&path]);
        }

        // Load library
        let library = unsafe {
            Library::new(&path)
                .map_err(|e| LoadError::LoadFailed(e.to_string()))?
        };

        self.loaded.insert(path.clone(), library);
        Ok(&self.loaded[&path])
    }

    /// Lookup a symbol in a loaded library
    pub unsafe fn lookup_symbol<T>(
        &self,
        library_name: &str,
        symbol_name: &str,
    ) -> Result<Symbol<T>, LoadError> {
        let path = self.resolve_library_path(library_name)
            .ok_or_else(|| LoadError::LibraryNotFound(library_name.to_string()))?;

        let library = self.loaded.get(&path)
            .ok_or_else(|| LoadError::LibraryNotFound(library_name.to_string()))?;

        library.get(symbol_name.as_bytes())
            .map_err(|_| LoadError::SymbolNotFound {
                library: library_name.to_string(),
                symbol: symbol_name.to_string(),
            })
    }
}
```

---

### 3. Function Caller

**File:** `crates/atlas-runtime/src/ffi/caller.rs`

```rust
use crate::ffi::types::CType;
use crate::ffi::marshal::{MarshalContext, MarshalError};
use crate::types::ExternType;
use crate::value::Value;
use libffi::high::Closure;
use libffi::raw;

#[derive(Debug)]
pub enum CallError {
    MarshalError(MarshalError),
    CallFailed(String),
}

impl From<MarshalError> for CallError {
    fn from(e: MarshalError) -> Self {
        CallError::MarshalError(e)
    }
}

pub struct ExternCaller {
    context: MarshalContext,
}

impl ExternCaller {
    pub fn new() -> Self {
        Self {
            context: MarshalContext::new(),
        }
    }

    /// Call an extern function with Atlas values
    pub unsafe fn call(
        &mut self,
        fn_ptr: *const (),
        param_types: &[ExternType],
        return_type: &ExternType,
        args: &[Value],
    ) -> Result<Value, CallError> {
        // Validate argument count
        if args.len() != param_types.len() {
            return Err(CallError::CallFailed(format!(
                "Expected {} arguments, got {}",
                param_types.len(),
                args.len()
            )));
        }

        // Marshal Atlas args to C args
        let c_args: Result<Vec<CType>, MarshalError> = args
            .iter()
            .zip(param_types.iter())
            .map(|(arg, ty)| self.context.atlas_to_c(arg, ty))
            .collect();
        let c_args = c_args?;

        // Prepare libffi call
        let c_return = self.invoke_ffi(fn_ptr, &c_args, return_type)?;

        // Marshal C return to Atlas value
        let atlas_return = self.context.c_to_atlas(&c_return)?;
        Ok(atlas_return)
    }

    unsafe fn invoke_ffi(
        &self,
        fn_ptr: *const (),
        args: &[CType],
        return_type: &ExternType,
    ) -> Result<CType, CallError> {
        // Convert to libffi types
        let arg_types: Vec<raw::ffi_type> = args.iter()
            .map(|a| Self::ctype_to_ffi_type(a))
            .collect();

        let ret_ffi_type = Self::extern_type_to_ffi_type(return_type);

        // Build CIF (Call Interface)
        let mut cif: raw::ffi_cif = std::mem::zeroed();
        let status = raw::ffi_prep_cif(
            &mut cif,
            raw::FFI_DEFAULT_ABI,
            arg_types.len(),
            &ret_ffi_type as *const _ as *mut _,
            arg_types.as_ptr() as *mut _,
        );

        if status != raw::ffi_status_FFI_OK {
            return Err(CallError::CallFailed("ffi_prep_cif failed".to_string()));
        }

        // Prepare argument pointers
        let arg_ptrs: Vec<*mut std::ffi::c_void> = args.iter()
            .map(|a| Self::ctype_to_ptr(a))
            .collect();

        // Call function
        let mut result_storage: u64 = 0;
        raw::ffi_call(
            &mut cif,
            Some(std::mem::transmute(fn_ptr)),
            &mut result_storage as *mut _ as *mut _,
            arg_ptrs.as_ptr() as *mut _,
        );

        // Convert result back to CType
        Self::ffi_result_to_ctype(&result_storage, return_type)
    }

    fn ctype_to_ffi_type(c: &CType) -> raw::ffi_type {
        match c {
            CType::Int(_) => raw::ffi_type_sint,
            CType::Long(_) => raw::ffi_type_slong,
            CType::Double(_) => raw::ffi_type_double,
            CType::CharPtr(_) => raw::ffi_type_pointer,
            CType::Bool(_) => raw::ffi_type_uint8,
            CType::Void => raw::ffi_type_void,
        }
    }

    fn extern_type_to_ffi_type(et: &ExternType) -> raw::ffi_type {
        match et {
            ExternType::CInt => raw::ffi_type_sint,
            ExternType::CLong => raw::ffi_type_slong,
            ExternType::CDouble => raw::ffi_type_double,
            ExternType::CCharPtr => raw::ffi_type_pointer,
            ExternType::CBool => raw::ffi_type_uint8,
            ExternType::CVoid => raw::ffi_type_void,
        }
    }

    // Helper methods for pointer conversion and result extraction
    // ... (implementation details)
}
```

---

### 4. Extern Declaration Syntax

**File:** `crates/atlas-parser/src/ast.rs`

```rust
/// Extern function declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternDecl {
    pub name: String,
    pub library: String,  // Library name (e.g., "m" for libm)
    pub symbol: Option<String>,  // Symbol name if different from name
    pub params: Vec<(String, ExternTypeAnnotation)>,
    pub return_type: ExternTypeAnnotation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExternTypeAnnotation {
    CInt,
    CLong,
    CDouble,
    CCharPtr,
    CBool,
    CVoid,
}

/// Add to Statement enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    // ... existing variants ...

    /// Extern function declaration
    Extern(ExternDecl),
}
```

**Example Atlas syntax:**
```atlas
// Declare C math function
extern fn sqrt(x: c_double) -> c_double from "m";

// Declare with different symbol name
extern fn my_strlen(s: c_char_ptr) -> c_int from "c" as "strlen";

// Call it
let result = sqrt(16.0);  // 4.0
```

---

### 5. Parser Support

**File:** `crates/atlas-parser/src/parser.rs`

Add parsing for `extern fn` declarations:
- Parse `extern` keyword
- Parse function signature with extern types
- Parse `from "library"` clause
- Parse optional `as "symbol"` clause
- Build `Statement::Extern(ExternDecl)` AST node

---

### 6. Interpreter Integration

**File:** `crates/atlas-runtime/src/interpreter/mod.rs`

```rust
use crate::ffi::loader::LibraryLoader;
use crate::ffi::caller::ExternCaller;

pub struct Interpreter {
    // ... existing fields ...
    ffi_loader: LibraryLoader,
    ffi_caller: ExternCaller,
    extern_functions: HashMap<String, (*const (), Vec<ExternType>, ExternType)>,
}

impl Interpreter {
    fn execute_extern_decl(&mut self, decl: &ExternDecl) -> Result<(), RuntimeError> {
        // Load library
        let library = self.ffi_loader.load(&decl.library)?;

        // Lookup symbol
        let symbol_name = decl.symbol.as_ref().unwrap_or(&decl.name);
        let fn_ptr = unsafe {
            self.ffi_loader.lookup_symbol::<*const ()>(&decl.library, symbol_name)?
        };

        // Store function info
        let param_types = decl.params.iter()
            .map(|(_, ty)| ty.to_extern_type())
            .collect();
        let return_type = decl.return_type.to_extern_type();

        self.extern_functions.insert(
            decl.name.clone(),
            (*fn_ptr, param_types, return_type),
        );

        Ok(())
    }

    fn call_extern_function(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        let (fn_ptr, param_types, return_type) = self.extern_functions.get(name)
            .ok_or_else(|| RuntimeError::UndefinedFunction(name.to_string()))?;

        unsafe {
            self.ffi_caller.call(*fn_ptr, param_types, return_type, args)
                .map_err(|e| RuntimeError::FFIError(format!("{:?}", e)))
        }
    }
}
```

---

### 7. VM Integration

**File:** `crates/atlas-runtime/src/compiler/mod.rs`

Add bytecode instructions for extern calls:
- `OpExternDecl` - Register extern function
- `OpCallExtern` - Call extern function by name

Compile `Statement::Extern` to `OpExternDecl`.
Compile calls to extern functions using `OpCallExtern`.

---

## Tests (Use rstest + insta)

**File:** `crates/atlas-runtime/tests/ffi_loading_tests.rs`

### Library Loading Tests (12 tests)
1. `test_load_standard_c_library` - Load libc
2. `test_load_math_library` - Load libm
3. `test_library_path_resolution_linux` - Platform-specific paths
4. `test_library_path_resolution_macos` - .dylib extension
5. `test_library_path_resolution_windows` - .dll extension
6. `test_library_not_found_error` - Missing library
7. `test_library_cache_reuse` - Same library loaded once
8. `test_lookup_symbol_success` - Find "strlen" in libc
9. `test_lookup_symbol_not_found` - Missing symbol error
10. `test_multiple_libraries` - Load multiple libraries
11. `test_relative_path_library` - Load from ./
12. `test_absolute_path_library` - Load with full path

### Extern Declaration Tests (8 tests)
1. `test_parse_extern_declaration` - Basic extern syntax
2. `test_parse_extern_with_as_clause` - Symbol aliasing
3. `test_extern_all_param_types` - All 6 extern types
4. `test_extern_no_params` - Void params
5. `test_extern_void_return` - Void return
6. `test_extern_declaration_typechecks` - Type validation
7. `test_duplicate_extern_error` - Redeclaration error
8. `test_extern_invalid_library` - Library not found

### Function Calling Tests (10 tests)
1. `test_call_c_sqrt` - sqrt(16.0) → 4.0
2. `test_call_c_strlen` - strlen("hello") → 5
3. `test_call_c_abs` - abs(-42) → 42
4. `test_call_multiple_params` - pow(2.0, 3.0) → 8.0
5. `test_call_void_return` - Function returning void
6. `test_call_wrong_arg_count` - Error on mismatch
7. `test_call_type_mismatch` - String to CInt error
8. `test_call_undefined_extern` - Call before declaration
9. `test_call_extern_from_function` - Extern call inside Atlas function
10. `test_call_nested_externs` - Extern calls extern

**File:** `crates/atlas-runtime/tests/ffi_integration_tests.rs`

### Integration Tests (10 tests)
1. `test_interpreter_extern_sqrt` - Full interpreter flow
2. `test_vm_extern_sqrt` - Full VM flow
3. `test_parity_extern_math_functions` - Interpreter == VM
4. `test_extern_with_string_marshaling` - C string handling
5. `test_extern_error_propagation` - FFI errors surface correctly
6. `test_multiple_extern_libraries` - libm + libc together
7. `test_extern_performance_overhead` - Measure call overhead
8. `test_extern_in_loop` - Repeated extern calls
9. `test_extern_with_atlas_functions` - Mix extern and native
10. `test_compile_time_extern_validation` - Type errors caught early

**Minimum test count:** 40 tests (20 interpreter + 20 VM for parity)

---

## Integration Points

- **Uses:** Phase-10a marshaling infrastructure
- **Uses:** Parser for extern syntax
- **Uses:** Type system for validation
- **Updates:** Interpreter with extern execution
- **Updates:** VM with extern bytecode
- **Blocks:** Phase-10c (callbacks) depends on this
- **Output:** Working extern function calls

---

## Acceptance Criteria

- [ ] `libffi` and `libloading` dependencies added
- [ ] `LibraryLoader` loads .so/.dylib/.dll correctly
- [ ] Platform-specific path resolution works
- [ ] Library caching prevents duplicate loads
- [ ] Symbol lookup finds C functions
- [ ] `extern fn` syntax parses correctly
- [ ] Extern declarations type-check
- [ ] `ExternCaller` invokes C functions via libffi
- [ ] Interpreter executes extern calls
- [ ] VM compiles and executes extern calls
- [ ] Can call standard C functions (sqrt, strlen, abs, pow)
- [ ] String marshaling works (CCharPtr)
- [ ] Error handling for missing libraries/symbols
- [ ] Error handling for type mismatches
- [ ] 40+ tests pass (20 interpreter + 20 VM)
- [ ] 100% interpreter/VM parity
- [ ] No clippy warnings
- [ ] `cargo fmt` applied
- [ ] Works on Linux, macOS, Windows

---

## Notes

**Platform Compatibility:**
- Library extensions: .so (Linux), .dylib (macOS), .dll (Windows)
- Library prefixes: "lib" on Unix, none on Windows
- Search paths: /usr/lib, /usr/local/lib, /opt/homebrew/lib, C:\Windows\System32

**Safety:**
- All FFI calls are `unsafe` and isolated in caller.rs
- Validation before calling (arg count, types)
- Error handling prevents crashes

**Limitations (v0.2):**
- No array marshaling yet
- No struct marshaling yet
- No callbacks yet (phase-10c)
- No variadic functions
- No function pointers as parameters

**Next Phase:**
Phase-10c will add callbacks (C→Atlas calls), memory safety wrappers, and comprehensive integration tests + documentation.
