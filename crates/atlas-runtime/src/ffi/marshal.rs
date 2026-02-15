//! Type marshaling - Atlas ↔ C type conversions
//!
//! Provides bidirectional marshaling between Atlas values and C types:
//! - `MarshalContext::atlas_to_c()`: Convert Atlas values to C representations
//! - `MarshalContext::c_to_atlas()`: Convert C values to Atlas representations
//!
//! # Memory Safety
//!
//! - All allocated C strings are tracked in `MarshalContext`
//! - Automatic cleanup on `Drop`
//! - Null pointer checks for C pointers
//! - Range validation for numeric conversions

use crate::ffi::types::CType;
use crate::ffi::ExternType;
use crate::value::Value;
use std::ffi::{CStr, CString};

/// Marshal error types
#[derive(Debug, Clone, PartialEq)]
pub enum MarshalError {
    /// Type mismatch between Atlas value and target extern type
    TypeMismatch { expected: String, got: String },
    /// Null pointer encountered
    NullPointer,
    /// Invalid string (contains null byte or invalid UTF-8)
    InvalidString(String),
    /// Number out of range for target C type
    NumberOutOfRange { value: f64, target: String },
}

impl std::fmt::Display for MarshalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarshalError::TypeMismatch { expected, got } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, got)
            }
            MarshalError::NullPointer => write!(f, "Null pointer"),
            MarshalError::InvalidString(msg) => write!(f, "Invalid string: {}", msg),
            MarshalError::NumberOutOfRange { value, target } => {
                write!(f, "Number {} out of range for {}", value, target)
            }
        }
    }
}

impl std::error::Error for MarshalError {}

/// Marshal context for Atlas ↔ C conversions
///
/// Tracks allocated C strings for proper cleanup.
///
/// # Example
///
/// ```
/// # use atlas_runtime::ffi::{MarshalContext, ExternType};
/// # use atlas_runtime::value::Value;
/// let mut ctx = MarshalContext::new();
///
/// // Marshal Atlas number to C int
/// let c_value = ctx.atlas_to_c(&Value::Number(42.0), &ExternType::CInt).unwrap();
///
/// // Marshal C value back to Atlas
/// let atlas_value = ctx.c_to_atlas(&c_value).unwrap();
/// // ctx automatically cleans up on drop
/// ```
pub struct MarshalContext {
    /// Track allocated C strings for cleanup
    allocated_strings: Vec<CString>,
}

impl MarshalContext {
    /// Create a new marshal context
    pub fn new() -> Self {
        Self {
            allocated_strings: Vec::new(),
        }
    }

    /// Marshal Atlas value to C type
    ///
    /// # Arguments
    ///
    /// * `value` - Atlas value to marshal
    /// * `target` - Target C type
    ///
    /// # Returns
    ///
    /// C-compatible representation or marshal error
    ///
    /// # Examples
    ///
    /// ```
    /// # use atlas_runtime::ffi::{MarshalContext, ExternType, CType};
    /// # use atlas_runtime::value::Value;
    /// let mut ctx = MarshalContext::new();
    ///
    /// // Number to CInt
    /// let c_int = ctx.atlas_to_c(&Value::Number(42.0), &ExternType::CInt).unwrap();
    /// assert_eq!(c_int, CType::Int(42));
    ///
    /// // String to CCharPtr
    /// let c_str = ctx.atlas_to_c(
    ///     &Value::string("hello"),
    ///     &ExternType::CCharPtr
    /// ).unwrap();
    /// ```
    pub fn atlas_to_c(
        &mut self,
        value: &Value,
        target: &ExternType,
    ) -> Result<CType, MarshalError> {
        match (value, target) {
            (Value::Number(n), ExternType::CInt) => {
                // Validate range for i32
                if *n >= i32::MIN as f64 && *n <= i32::MAX as f64 && n.fract() == 0.0 {
                    Ok(CType::Int(*n as i32))
                } else {
                    Err(MarshalError::NumberOutOfRange {
                        value: *n,
                        target: "c_int".to_string(),
                    })
                }
            }

            (Value::Number(n), ExternType::CLong) => {
                // CLong can hold larger range
                if *n >= i64::MIN as f64 && *n <= i64::MAX as f64 && n.fract() == 0.0 {
                    Ok(CType::Long(*n as i64))
                } else {
                    Err(MarshalError::NumberOutOfRange {
                        value: *n,
                        target: "c_long".to_string(),
                    })
                }
            }

            (Value::Number(n), ExternType::CDouble) => Ok(CType::Double(*n)),

            (Value::String(s), ExternType::CCharPtr) => {
                // Create null-terminated C string
                let c_string = CString::new(s.as_str()).map_err(|e| {
                    MarshalError::InvalidString(format!("String contains null byte: {}", e))
                })?;

                // Get pointer before moving CString into storage
                let ptr = c_string.as_ptr();

                // Store CString to keep it alive
                self.allocated_strings.push(c_string);

                Ok(CType::CharPtr(ptr))
            }

            (Value::Bool(b), ExternType::CBool) => Ok(CType::Bool(if *b { 1 } else { 0 })),

            // Void is represented as Null at runtime
            (Value::Null, ExternType::CVoid) => Ok(CType::Void),

            _ => Err(MarshalError::TypeMismatch {
                expected: format!("{:?}", target),
                got: value.type_name().to_string(),
            }),
        }
    }

    /// Marshal C type to Atlas value
    ///
    /// # Arguments
    ///
    /// * `c_value` - C value to marshal
    ///
    /// # Returns
    ///
    /// Atlas value or marshal error
    ///
    /// # Safety
    ///
    /// For `CType::CharPtr`, the pointer must be valid and point to a
    /// null-terminated string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use atlas_runtime::ffi::{MarshalContext, CType};
    /// # use atlas_runtime::value::Value;
    /// let ctx = MarshalContext::new();
    ///
    /// // CInt to Number
    /// let atlas_value = ctx.c_to_atlas(&CType::Int(42)).unwrap();
    /// assert_eq!(atlas_value, Value::Number(42.0));
    ///
    /// // CBool to Bool
    /// let atlas_bool = ctx.c_to_atlas(&CType::Bool(1)).unwrap();
    /// assert_eq!(atlas_bool, Value::Bool(true));
    /// ```
    pub fn c_to_atlas(&self, c_value: &CType) -> Result<Value, MarshalError> {
        match c_value {
            CType::Int(i) => Ok(Value::Number(*i as f64)),

            CType::Long(l) => Ok(Value::Number(*l as f64)),

            CType::Double(d) => Ok(Value::Number(*d)),

            CType::CharPtr(ptr) => {
                if ptr.is_null() {
                    return Err(MarshalError::NullPointer);
                }

                unsafe {
                    let c_str = CStr::from_ptr(*ptr);
                    let s = c_str.to_str().map_err(|e| {
                        MarshalError::InvalidString(format!("Invalid UTF-8: {}", e))
                    })?;
                    Ok(Value::string(s))
                }
            }

            CType::Bool(b) => Ok(Value::Bool(*b != 0)),

            // Void is represented as Null at runtime
            CType::Void => Ok(Value::Null),
        }
    }
}

impl Default for MarshalContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MarshalContext {
    fn drop(&mut self) {
        // CStrings automatically cleaned up when vec is dropped
        self.allocated_strings.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marshal_number_to_cint() {
        let mut ctx = MarshalContext::new();
        let result = ctx.atlas_to_c(&Value::Number(42.0), &ExternType::CInt);
        assert_eq!(result, Ok(CType::Int(42)));
    }

    #[test]
    fn test_marshal_number_to_clong() {
        let mut ctx = MarshalContext::new();
        let result = ctx.atlas_to_c(&Value::Number(1000.0), &ExternType::CLong);
        assert_eq!(result, Ok(CType::Long(1000)));
    }

    #[test]
    fn test_marshal_number_to_cdouble() {
        let mut ctx = MarshalContext::new();
        let result = ctx.atlas_to_c(&Value::Number(3.14), &ExternType::CDouble);
        assert_eq!(result, Ok(CType::Double(3.14)));
    }

    #[test]
    fn test_marshal_string_to_char_ptr() {
        let mut ctx = MarshalContext::new();
        let result = ctx.atlas_to_c(&Value::string("hello"), &ExternType::CCharPtr);
        assert!(result.is_ok());

        if let Ok(CType::CharPtr(ptr)) = result {
            assert!(!ptr.is_null());
            unsafe {
                let c_str = CStr::from_ptr(ptr);
                assert_eq!(c_str.to_str().unwrap(), "hello");
            }
        }
    }

    #[test]
    fn test_marshal_bool_to_cbool_true() {
        let mut ctx = MarshalContext::new();
        let result = ctx.atlas_to_c(&Value::Bool(true), &ExternType::CBool);
        assert_eq!(result, Ok(CType::Bool(1)));
    }

    #[test]
    fn test_marshal_bool_to_cbool_false() {
        let mut ctx = MarshalContext::new();
        let result = ctx.atlas_to_c(&Value::Bool(false), &ExternType::CBool);
        assert_eq!(result, Ok(CType::Bool(0)));
    }

    #[test]
    fn test_marshal_void_to_cvoid() {
        let mut ctx = MarshalContext::new();
        let result = ctx.atlas_to_c(&Value::Null, &ExternType::CVoid);
        assert_eq!(result, Ok(CType::Void));
    }

    #[test]
    fn test_marshal_type_mismatch() {
        let mut ctx = MarshalContext::new();
        let result = ctx.atlas_to_c(&Value::string("hello"), &ExternType::CInt);
        assert!(matches!(result, Err(MarshalError::TypeMismatch { .. })));
    }

    #[test]
    fn test_marshal_number_out_of_range() {
        let mut ctx = MarshalContext::new();
        // i32::MAX + 1 is out of range
        let result = ctx.atlas_to_c(&Value::Number(3e9), &ExternType::CInt);
        assert!(matches!(result, Err(MarshalError::NumberOutOfRange { .. })));
    }

    #[test]
    fn test_marshal_string_with_null_byte() {
        let mut ctx = MarshalContext::new();
        // CString::new will fail if string contains null byte
        let s = "hello\0world";
        let result = ctx.atlas_to_c(&Value::string(s), &ExternType::CCharPtr);
        assert!(matches!(result, Err(MarshalError::InvalidString(_))));
    }

    #[test]
    fn test_unmarshal_cint_to_number() {
        let ctx = MarshalContext::new();
        let result = ctx.c_to_atlas(&CType::Int(42));
        assert_eq!(result, Ok(Value::Number(42.0)));
    }

    #[test]
    fn test_unmarshal_clong_to_number() {
        let ctx = MarshalContext::new();
        let result = ctx.c_to_atlas(&CType::Long(1000));
        assert_eq!(result, Ok(Value::Number(1000.0)));
    }

    #[test]
    fn test_unmarshal_cdouble_to_number() {
        let ctx = MarshalContext::new();
        let result = ctx.c_to_atlas(&CType::Double(3.14));
        assert_eq!(result, Ok(Value::Number(3.14)));
    }

    #[test]
    fn test_unmarshal_char_ptr_to_string() {
        let c_string = CString::new("hello").unwrap();
        let ctx = MarshalContext::new();
        let result = ctx.c_to_atlas(&CType::CharPtr(c_string.as_ptr()));
        assert_eq!(result, Ok(Value::string("hello")));
    }

    #[test]
    fn test_unmarshal_cbool_to_bool() {
        let ctx = MarshalContext::new();
        assert_eq!(ctx.c_to_atlas(&CType::Bool(1)), Ok(Value::Bool(true)));
        assert_eq!(ctx.c_to_atlas(&CType::Bool(0)), Ok(Value::Bool(false)));
        assert_eq!(ctx.c_to_atlas(&CType::Bool(255)), Ok(Value::Bool(true))); // Non-zero is true
    }

    #[test]
    fn test_unmarshal_cvoid_to_void() {
        let ctx = MarshalContext::new();
        let result = ctx.c_to_atlas(&CType::Void);
        assert_eq!(result, Ok(Value::Null));
    }

    #[test]
    fn test_unmarshal_null_pointer() {
        let ctx = MarshalContext::new();
        let result = ctx.c_to_atlas(&CType::CharPtr(std::ptr::null()));
        assert!(matches!(result, Err(MarshalError::NullPointer)));
    }

    #[test]
    fn test_marshal_context_tracks_strings() {
        let mut ctx = MarshalContext::new();
        ctx.atlas_to_c(&Value::string("hello"), &ExternType::CCharPtr)
            .unwrap();
        ctx.atlas_to_c(&Value::string("world"), &ExternType::CCharPtr)
            .unwrap();
        assert_eq!(ctx.allocated_strings.len(), 2);
    }

    #[test]
    fn test_marshal_context_cleanup() {
        {
            let mut ctx = MarshalContext::new();
            ctx.atlas_to_c(&Value::string("test"), &ExternType::CCharPtr)
                .unwrap();
            // ctx drops here, cleaning up allocated strings
        }
        // If we got here without crash, cleanup worked
    }

    #[test]
    fn test_marshal_context_reuse() {
        let mut ctx = MarshalContext::new();
        ctx.atlas_to_c(&Value::Number(42.0), &ExternType::CInt)
            .unwrap();
        ctx.atlas_to_c(&Value::Number(43.0), &ExternType::CInt)
            .unwrap();
        ctx.atlas_to_c(&Value::Number(44.0), &ExternType::CInt)
            .unwrap();
        // Context can be reused multiple times
    }
}
