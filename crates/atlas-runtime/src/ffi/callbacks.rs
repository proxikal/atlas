//! FFI Callbacks - Enable C code to call Atlas functions (phase-10c)
//!
//! Provides trampoline generation for Atlas→C function pointer conversion,
//! enabling bidirectional FFI (Atlas↔C).

use crate::ffi::types::ExternType;
use crate::value::{RuntimeError, Value};
use std::os::raw::{c_double, c_int, c_long};

/// Errors that can occur during callback creation or execution
#[derive(Debug)]
pub enum CallbackError {
    /// Marshaling error during argument/return conversion
    MarshalError(String),
    /// Execution error when calling Atlas function
    ExecutionError(String),
    /// Signature validation error
    InvalidSignature(String),
    /// Unsupported callback signature
    UnsupportedSignature(String),
}

impl From<crate::ffi::marshal::MarshalError> for CallbackError {
    fn from(e: crate::ffi::marshal::MarshalError) -> Self {
        CallbackError::MarshalError(format!("{:?}", e))
    }
}

impl std::fmt::Display for CallbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallbackError::MarshalError(msg) => write!(f, "Marshal error: {}", msg),
            CallbackError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            CallbackError::InvalidSignature(msg) => write!(f, "Invalid signature: {}", msg),
            CallbackError::UnsupportedSignature(msg) => {
                write!(f, "Unsupported signature: {}", msg)
            }
        }
    }
}

impl std::error::Error for CallbackError {}

/// Represents an Atlas function wrapped for C calling
///
/// Keeps the callback alive and provides the function pointer for C code.
pub struct CallbackHandle {
    /// Function pointer that C code can call
    fn_ptr: usize,
    /// Keep closure alive (Drop will invalidate fn_ptr)
    _closure: Box<dyn std::any::Any>,
    /// Signature for validation
    param_types: Vec<ExternType>,
    return_type: ExternType,
}

impl CallbackHandle {
    /// Get the function pointer for C code
    pub fn fn_ptr(&self) -> *const () {
        self.fn_ptr as *const ()
    }

    /// Get the callback signature
    pub fn signature(&self) -> (&[ExternType], &ExternType) {
        (&self.param_types, &self.return_type)
    }
}

/// Signature string for callback identification
fn signature_string(param_types: &[ExternType], return_type: &ExternType) -> String {
    let params = param_types
        .iter()
        .map(|t| format!("{:?}", t))
        .collect::<Vec<_>>()
        .join(",");
    format!("({:?})->({:?})", params, return_type)
}

/// Create a C-callable callback for an Atlas function
///
/// This generates a trampoline function that:
/// 1. Receives C arguments
/// 2. Marshals them to Atlas values
/// 3. Calls the Atlas function
/// 4. Marshals the result back to C
/// 5. Returns to C code
///
/// # Safety
///
/// The callback_fn must correctly handle the provided arguments and return
/// a value compatible with the return_type.
pub fn create_callback<F>(
    callback_fn: F,
    param_types: Vec<ExternType>,
    return_type: ExternType,
) -> Result<CallbackHandle, CallbackError>
where
    F: Fn(&[Value]) -> Result<Value, RuntimeError> + 'static,
{
    // Build signature string
    let sig = signature_string(&param_types, &return_type);

    // Create appropriate trampoline based on signature
    match (param_types.as_slice(), &return_type) {
        // No parameters -> CInt
        ([], ExternType::CInt) => {
            let closure = Box::new(move || -> c_int {
                match callback_fn(&[]) {
                    Ok(Value::Number(n)) => n as c_int,
                    Ok(_) => 0,
                    Err(_) => 0,
                }
            });

            let fn_ptr = closure.as_ref() as *const _ as *const ();
            Ok(CallbackHandle {
                fn_ptr: fn_ptr as usize,
                _closure: Box::new(closure) as Box<dyn std::any::Any>,
                param_types,
                return_type,
            })
        }

        // CDouble -> CDouble (common math callback)
        ([ExternType::CDouble], ExternType::CDouble) => {
            let closure = Box::new(move |x: c_double| -> c_double {
                match callback_fn(&[Value::Number(x)]) {
                    Ok(Value::Number(n)) => n,
                    Ok(_) => 0.0,
                    Err(_) => 0.0,
                }
            });

            let fn_ptr = closure.as_ref() as *const _ as *const ();
            Ok(CallbackHandle {
                fn_ptr: fn_ptr as usize,
                _closure: Box::new(closure) as Box<dyn std::any::Any>,
                param_types,
                return_type,
            })
        }

        // CDouble, CDouble -> CDouble (binary math callback)
        ([ExternType::CDouble, ExternType::CDouble], ExternType::CDouble) => {
            let closure = Box::new(move |x: c_double, y: c_double| -> c_double {
                match callback_fn(&[Value::Number(x), Value::Number(y)]) {
                    Ok(Value::Number(n)) => n,
                    Ok(_) => 0.0,
                    Err(_) => 0.0,
                }
            });

            let fn_ptr = closure.as_ref() as *const _ as *const ();
            Ok(CallbackHandle {
                fn_ptr: fn_ptr as usize,
                _closure: Box::new(closure) as Box<dyn std::any::Any>,
                param_types,
                return_type,
            })
        }

        // CInt -> CInt
        ([ExternType::CInt], ExternType::CInt) => {
            let closure = Box::new(move |x: c_int| -> c_int {
                match callback_fn(&[Value::Number(x as f64)]) {
                    Ok(Value::Number(n)) => n as c_int,
                    Ok(_) => 0,
                    Err(_) => 0,
                }
            });

            let fn_ptr = closure.as_ref() as *const _ as *const ();
            Ok(CallbackHandle {
                fn_ptr: fn_ptr as usize,
                _closure: Box::new(closure) as Box<dyn std::any::Any>,
                param_types,
                return_type,
            })
        }

        // CInt, CInt -> CInt
        ([ExternType::CInt, ExternType::CInt], ExternType::CInt) => {
            let closure = Box::new(move |x: c_int, y: c_int| -> c_int {
                match callback_fn(&[Value::Number(x as f64), Value::Number(y as f64)]) {
                    Ok(Value::Number(n)) => n as c_int,
                    Ok(_) => 0,
                    Err(_) => 0,
                }
            });

            let fn_ptr = closure.as_ref() as *const _ as *const ();
            Ok(CallbackHandle {
                fn_ptr: fn_ptr as usize,
                _closure: Box::new(closure) as Box<dyn std::any::Any>,
                param_types,
                return_type,
            })
        }

        // CLong -> CLong
        ([ExternType::CLong], ExternType::CLong) => {
            let closure = Box::new(move |x: c_long| -> c_long {
                match callback_fn(&[Value::Number(x as f64)]) {
                    Ok(Value::Number(n)) => n as c_long,
                    Ok(_) => 0,
                    Err(_) => 0,
                }
            });

            let fn_ptr = closure.as_ref() as *const _ as *const ();
            Ok(CallbackHandle {
                fn_ptr: fn_ptr as usize,
                _closure: Box::new(closure) as Box<dyn std::any::Any>,
                param_types,
                return_type,
            })
        }

        // () -> CVoid
        ([], ExternType::CVoid) => {
            let closure = Box::new(move || {
                let _ = callback_fn(&[]);
            });

            let fn_ptr = closure.as_ref() as *const _ as *const ();
            Ok(CallbackHandle {
                fn_ptr: fn_ptr as usize,
                _closure: Box::new(closure) as Box<dyn std::any::Any>,
                param_types,
                return_type,
            })
        }

        // CInt -> CVoid
        ([ExternType::CInt], ExternType::CVoid) => {
            let closure = Box::new(move |x: c_int| {
                let _ = callback_fn(&[Value::Number(x as f64)]);
            });

            let fn_ptr = closure.as_ref() as *const _ as *const ();
            Ok(CallbackHandle {
                fn_ptr: fn_ptr as usize,
                _closure: Box::new(closure) as Box<dyn std::any::Any>,
                param_types,
                return_type,
            })
        }

        // Unsupported signature
        _ => Err(CallbackError::UnsupportedSignature(sig)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_callback_no_params_int_return() {
        let handle =
            create_callback(|_args| Ok(Value::Number(42.0)), vec![], ExternType::CInt).unwrap();

        assert_eq!(handle.param_types.len(), 0);
        assert!(matches!(handle.return_type, ExternType::CInt));
        assert!(!handle.fn_ptr().is_null());
    }

    #[test]
    fn test_callback_double_param_double_return() {
        let handle = create_callback(
            |args| {
                if let Some(Value::Number(n)) = args.first() {
                    Ok(Value::Number(n * 2.0))
                } else {
                    Ok(Value::Number(0.0))
                }
            },
            vec![ExternType::CDouble],
            ExternType::CDouble,
        )
        .unwrap();

        assert_eq!(handle.param_types.len(), 1);
        assert!(matches!(handle.return_type, ExternType::CDouble));
    }

    #[test]
    fn test_callback_unsupported_signature() {
        let result = create_callback(
            |_args| Ok(Value::Null),
            vec![ExternType::CCharPtr],
            ExternType::CCharPtr,
        );

        assert!(matches!(
            result,
            Err(CallbackError::UnsupportedSignature(_))
        ));
    }

    #[test]
    fn test_callback_signature_string() {
        let sig = signature_string(&[ExternType::CInt, ExternType::CDouble], &ExternType::CLong);
        assert!(sig.contains("CInt"));
        assert!(sig.contains("CDouble"));
        assert!(sig.contains("CLong"));
    }
}
