//! FFI function calling using direct function pointers
//!
//! Since Atlas extern declarations are statically typed (signature known at parse time),
//! we can use direct function pointer casts instead of dynamic FFI libraries like libffi.
//!
//! This approach is simpler, faster, and avoids native compilation issues.

use crate::ffi::marshal::{MarshalContext, MarshalError};
use crate::ffi::types::CType;
use crate::ffi::ExternType;
use crate::value::Value;
use std::os::raw::{c_char, c_double, c_int, c_long};

/// FFI call errors
#[derive(Debug, Clone, PartialEq)]
pub enum CallError {
    /// Marshaling error (Atlas↔C conversion failed)
    MarshalError(MarshalError),
    /// Wrong number of arguments
    ArityMismatch { expected: usize, got: usize },
    /// Unsupported signature (future: arrays, structs, etc.)
    UnsupportedSignature(String),
}

impl std::fmt::Display for CallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallError::MarshalError(e) => write!(f, "Marshal error: {}", e),
            CallError::ArityMismatch { expected, got } => {
                write!(f, "Expected {} arguments, got {}", expected, got)
            }
            CallError::UnsupportedSignature(sig) => {
                write!(f, "Unsupported FFI signature: {}", sig)
            }
        }
    }
}

impl std::error::Error for CallError {}

impl From<MarshalError> for CallError {
    fn from(e: MarshalError) -> Self {
        CallError::MarshalError(e)
    }
}

/// Extern function metadata and call handler
///
/// Stores the function pointer and signature for type-safe FFI calls.
#[derive(Clone)]
pub struct ExternFunction {
    /// Raw function pointer (type-erased)
    fn_ptr: *const (),
    /// Parameter types for marshaling
    param_types: Vec<ExternType>,
    /// Return type for marshaling
    return_type: ExternType,
}

// Safety: ExternFunction only stores a function pointer, which is thread-safe to share
unsafe impl Send for ExternFunction {}
unsafe impl Sync for ExternFunction {}

impl ExternFunction {
    /// Create a new extern function with its signature
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - `fn_ptr` points to a valid function
    /// - The function's actual signature matches `param_types` and `return_type`
    /// - The function remains valid for the lifetime of this ExternFunction
    pub unsafe fn new(
        fn_ptr: *const (),
        param_types: Vec<ExternType>,
        return_type: ExternType,
    ) -> Self {
        Self {
            fn_ptr,
            param_types,
            return_type,
        }
    }

    /// Call the extern function with Atlas values
    ///
    /// Marshals arguments to C types, calls the function, and marshals the result back.
    ///
    /// # Safety
    ///
    /// This is unsafe because it calls foreign code with potentially incorrect signatures.
    pub unsafe fn call(&self, args: &[Value]) -> Result<Value, CallError> {
        // Validate argument count
        if args.len() != self.param_types.len() {
            return Err(CallError::ArityMismatch {
                expected: self.param_types.len(),
                got: args.len(),
            });
        }

        // Marshal arguments
        let mut ctx = MarshalContext::new();
        let c_args: Vec<CType> = args
            .iter()
            .zip(self.param_types.iter())
            .map(|(arg, ty)| ctx.atlas_to_c(arg, ty))
            .collect::<Result<Vec<_>, _>>()?;

        // Call the function based on signature
        let c_result = self.call_with_signature(&c_args)?;

        // Marshal result back to Atlas
        let atlas_result = ctx.c_to_atlas(&c_result)?;
        Ok(atlas_result)
    }

    /// Call function using direct function pointer casts
    ///
    /// This matches the C calling convention based on the declared signature.
    unsafe fn call_with_signature(&self, args: &[CType]) -> Result<CType, CallError> {
        // Generate signature key for dispatch
        let sig = self.signature_key();

        match sig.as_str() {
            // No parameters, various return types
            "()->CInt" => {
                let f: extern "C" fn() -> c_int = std::mem::transmute(self.fn_ptr);
                Ok(CType::Int(f()))
            }
            "()->CLong" => {
                let f: extern "C" fn() -> c_long = std::mem::transmute(self.fn_ptr);
                Ok(CType::Long(f()))
            }
            "()->CDouble" => {
                let f: extern "C" fn() -> c_double = std::mem::transmute(self.fn_ptr);
                Ok(CType::Double(f()))
            }
            "()->CVoid" => {
                let f: extern "C" fn() = std::mem::transmute(self.fn_ptr);
                f();
                Ok(CType::Void)
            }

            // One parameter signatures
            "(CInt)->CInt" => {
                if let CType::Int(a) = &args[0] {
                    let f: extern "C" fn(c_int) -> c_int = std::mem::transmute(self.fn_ptr);
                    Ok(CType::Int(f(*a)))
                } else {
                    unreachable!()
                }
            }
            "(CDouble)->CDouble" => {
                if let CType::Double(a) = &args[0] {
                    let f: extern "C" fn(c_double) -> c_double = std::mem::transmute(self.fn_ptr);
                    Ok(CType::Double(f(*a)))
                } else {
                    unreachable!()
                }
            }
            "(CCharPtr)->CInt" => {
                if let CType::CharPtr(a) = &args[0] {
                    let f: extern "C" fn(*const c_char) -> c_int = std::mem::transmute(self.fn_ptr);
                    Ok(CType::Int(f(*a)))
                } else {
                    unreachable!()
                }
            }

            // Two parameter signatures
            "(CDouble,CDouble)->CDouble" => {
                if let (CType::Double(a), CType::Double(b)) = (&args[0], &args[1]) {
                    let f: extern "C" fn(c_double, c_double) -> c_double =
                        std::mem::transmute(self.fn_ptr);
                    Ok(CType::Double(f(*a, *b)))
                } else {
                    unreachable!()
                }
            }
            "(CInt,CInt)->CInt" => {
                if let (CType::Int(a), CType::Int(b)) = (&args[0], &args[1]) {
                    let f: extern "C" fn(c_int, c_int) -> c_int = std::mem::transmute(self.fn_ptr);
                    Ok(CType::Int(f(*a, *b)))
                } else {
                    unreachable!()
                }
            }

            // Unsupported signature
            _ => Err(CallError::UnsupportedSignature(sig)),
        }
    }

    /// Generate a signature key for dispatch
    fn signature_key(&self) -> String {
        let params: Vec<String> = self
            .param_types
            .iter()
            .map(|t| format!("{:?}", t))
            .collect();
        format!("({})->{:?}", params.join(","), self.return_type)
    }

    /// Get parameter types
    pub fn param_types(&self) -> &[ExternType] {
        &self.param_types
    }

    /// Get return type
    pub fn return_type(&self) -> &ExternType {
        &self.return_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple C functions for testing (defined here as Rust functions with C ABI)
    extern "C" fn test_add(a: c_int, b: c_int) -> c_int {
        a + b
    }

    extern "C" fn test_double(x: c_double) -> c_double {
        x * 2.0
    }

    extern "C" fn test_no_args() -> c_int {
        42
    }

    #[test]
    fn test_extern_function_call_add() {
        unsafe {
            let func = ExternFunction::new(
                test_add as *const (),
                vec![ExternType::CInt, ExternType::CInt],
                ExternType::CInt,
            );

            let result = func
                .call(&[Value::Number(10.0), Value::Number(20.0)])
                .unwrap();
            assert_eq!(result, Value::Number(30.0));
        }
    }

    #[test]
    fn test_extern_function_call_double() {
        unsafe {
            let func = ExternFunction::new(
                test_double as *const (),
                vec![ExternType::CDouble],
                ExternType::CDouble,
            );

            let result = func.call(&[Value::Number(21.0)]).unwrap();
            assert_eq!(result, Value::Number(42.0));
        }
    }

    #[test]
    fn test_extern_function_no_args() {
        unsafe {
            let func = ExternFunction::new(test_no_args as *const (), vec![], ExternType::CInt);

            let result = func.call(&[]).unwrap();
            assert_eq!(result, Value::Number(42.0));
        }
    }

    #[test]
    fn test_extern_function_arity_mismatch() {
        unsafe {
            let func = ExternFunction::new(
                test_add as *const (),
                vec![ExternType::CInt, ExternType::CInt],
                ExternType::CInt,
            );

            // Wrong number of arguments
            let result = func.call(&[Value::Number(10.0)]);
            assert!(matches!(result, Err(CallError::ArityMismatch { .. })));
        }
    }

    #[test]
    fn test_signature_key_generation() {
        unsafe {
            let func = ExternFunction::new(
                test_add as *const (),
                vec![ExternType::CInt, ExternType::CInt],
                ExternType::CInt,
            );

            assert_eq!(func.signature_key(), "(CInt,CInt)->CInt");
        }
    }
}
