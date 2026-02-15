//! FFI type system - C-compatible types for FFI boundary
//!
//! Defines:
//! - `ExternType`: Atlas type system representation of C types
//! - `CType`: Runtime representation of C values
//!
//! Type mapping:
//! - ExternType::CInt → CType::Int(i32)
//! - ExternType::CLong → CType::Long(i64)
//! - ExternType::CDouble → CType::Double(f64)
//! - ExternType::CCharPtr → CType::CharPtr(*const i8)
//! - ExternType::CVoid → CType::Void
//! - ExternType::CBool → CType::Bool(u8)

use crate::types::Type;
use serde::{Deserialize, Serialize};
use std::os::raw::c_char;

/// C-compatible extern types for FFI
///
/// These types represent C types in Atlas's type system.
/// Used for type checking extern function declarations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExternType {
    /// C int (platform-specific, typically i32)
    CInt,
    /// C long (platform-specific, i32 on 32-bit, i64 on 64-bit)
    CLong,
    /// C double (f64)
    CDouble,
    /// C char* (null-terminated string pointer)
    CCharPtr,
    /// C void (for void* or void return)
    CVoid,
    /// C bool (u8: 0 or 1)
    CBool,
}

impl ExternType {
    /// Check if an Atlas type can be marshaled to this extern type
    ///
    /// # Examples
    ///
    /// ```
    /// # use atlas_runtime::ffi::ExternType;
    /// # use atlas_runtime::Type;
    /// assert!(ExternType::CInt.accepts_atlas_type(&Type::Number));
    /// assert!(ExternType::CCharPtr.accepts_atlas_type(&Type::String));
    /// assert!(!ExternType::CInt.accepts_atlas_type(&Type::String));
    /// ```
    pub fn accepts_atlas_type(&self, atlas_type: &Type) -> bool {
        matches!(
            (self, atlas_type),
            (ExternType::CInt, Type::Number)
                | (ExternType::CLong, Type::Number)
                | (ExternType::CDouble, Type::Number)
                | (ExternType::CCharPtr, Type::String)
                | (ExternType::CVoid, Type::Void)
                | (ExternType::CBool, Type::Bool)
        )
    }

    /// Get the Atlas type this extern type maps to
    ///
    /// # Examples
    ///
    /// ```
    /// # use atlas_runtime::ffi::ExternType;
    /// # use atlas_runtime::Type;
    /// assert_eq!(ExternType::CInt.to_atlas_type(), Type::Number);
    /// assert_eq!(ExternType::CCharPtr.to_atlas_type(), Type::String);
    /// ```
    pub fn to_atlas_type(&self) -> Type {
        match self {
            ExternType::CInt | ExternType::CLong | ExternType::CDouble => Type::Number,
            ExternType::CCharPtr => Type::String,
            ExternType::CVoid => Type::Void,
            ExternType::CBool => Type::Bool,
        }
    }

    /// Get a display name for this extern type
    pub fn display_name(&self) -> &'static str {
        match self {
            ExternType::CInt => "c_int",
            ExternType::CLong => "c_long",
            ExternType::CDouble => "c_double",
            ExternType::CCharPtr => "c_char_ptr",
            ExternType::CVoid => "c_void",
            ExternType::CBool => "c_bool",
        }
    }
}

/// C type representation for FFI boundary
///
/// Runtime representation of C values during marshaling.
/// These are the actual C-compatible values passed across the FFI boundary.
#[derive(Debug, Clone)]
pub enum CType {
    /// C int value
    Int(i32),
    /// C long value
    Long(i64),
    /// C double value
    Double(f64),
    /// C char* (null-terminated string pointer)
    ///
    /// # Safety
    ///
    /// The pointer must be valid and point to a null-terminated string.
    /// Lifetime managed by MarshalContext.
    CharPtr(*const c_char),
    /// C void (no value)
    Void,
    /// C bool (0 or 1)
    Bool(u8),
}

// Manual PartialEq because we can't derive it for raw pointers
impl PartialEq for CType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CType::Int(a), CType::Int(b)) => a == b,
            (CType::Long(a), CType::Long(b)) => a == b,
            (CType::Double(a), CType::Double(b)) => a == b,
            (CType::CharPtr(a), CType::CharPtr(b)) => a == b,
            (CType::Void, CType::Void) => true,
            (CType::Bool(a), CType::Bool(b)) => a == b,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extern_type_accepts_atlas_type_valid() {
        assert!(ExternType::CInt.accepts_atlas_type(&Type::Number));
        assert!(ExternType::CLong.accepts_atlas_type(&Type::Number));
        assert!(ExternType::CDouble.accepts_atlas_type(&Type::Number));
        assert!(ExternType::CCharPtr.accepts_atlas_type(&Type::String));
        assert!(ExternType::CVoid.accepts_atlas_type(&Type::Void));
        assert!(ExternType::CBool.accepts_atlas_type(&Type::Bool));
    }

    #[test]
    fn test_extern_type_accepts_atlas_type_invalid() {
        assert!(!ExternType::CInt.accepts_atlas_type(&Type::String));
        assert!(!ExternType::CInt.accepts_atlas_type(&Type::Bool));
        assert!(!ExternType::CCharPtr.accepts_atlas_type(&Type::Number));
        assert!(!ExternType::CBool.accepts_atlas_type(&Type::Number));
    }

    #[test]
    fn test_extern_type_to_atlas_type() {
        assert_eq!(ExternType::CInt.to_atlas_type(), Type::Number);
        assert_eq!(ExternType::CLong.to_atlas_type(), Type::Number);
        assert_eq!(ExternType::CDouble.to_atlas_type(), Type::Number);
        assert_eq!(ExternType::CCharPtr.to_atlas_type(), Type::String);
        assert_eq!(ExternType::CVoid.to_atlas_type(), Type::Void);
        assert_eq!(ExternType::CBool.to_atlas_type(), Type::Bool);
    }

    #[test]
    fn test_extern_type_display_names() {
        assert_eq!(ExternType::CInt.display_name(), "c_int");
        assert_eq!(ExternType::CLong.display_name(), "c_long");
        assert_eq!(ExternType::CDouble.display_name(), "c_double");
        assert_eq!(ExternType::CCharPtr.display_name(), "c_char_ptr");
        assert_eq!(ExternType::CVoid.display_name(), "c_void");
        assert_eq!(ExternType::CBool.display_name(), "c_bool");
    }

    #[test]
    fn test_extern_type_equality() {
        assert_eq!(ExternType::CInt, ExternType::CInt);
        assert_ne!(ExternType::CInt, ExternType::CLong);
        assert_ne!(ExternType::CDouble, ExternType::CBool);
    }

    #[test]
    fn test_ctype_equality() {
        assert_eq!(CType::Int(42), CType::Int(42));
        assert_ne!(CType::Int(42), CType::Int(43));
        assert_eq!(CType::Double(3.14), CType::Double(3.14));
        assert_eq!(CType::Bool(1), CType::Bool(1));
        assert_ne!(CType::Bool(0), CType::Bool(1));
        assert_eq!(CType::Void, CType::Void);
    }

    #[test]
    fn test_all_extern_types_exist() {
        // Verify all 6 extern types are defined
        let types = vec![
            ExternType::CInt,
            ExternType::CLong,
            ExternType::CDouble,
            ExternType::CCharPtr,
            ExternType::CVoid,
            ExternType::CBool,
        ];
        assert_eq!(types.len(), 6);
    }
}
