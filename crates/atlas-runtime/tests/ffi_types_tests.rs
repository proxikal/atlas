//! Integration tests for FFI type system and marshaling
//!
//! Phase 10a: FFI Core Types + Type Marshaling
//!
//! Tests the complete FFI type system including:
//! - ExternType enum and type conversions
//! - Atlas ↔ C type marshaling
//! - MarshalContext memory management
//! - Type compatibility and validation

use atlas_runtime::ffi::{CType, ExternType, MarshalContext, MarshalError};
use atlas_runtime::types::Type;
use atlas_runtime::value::Value;
use rstest::rstest;

// ====================
// Extern Type Tests (8 tests)
// ====================

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
fn test_type_enum_extern_variant() {
    let extern_type = Type::Extern(ExternType::CInt);
    assert_eq!(extern_type.display_name(), "c_int");

    let extern_double = Type::Extern(ExternType::CDouble);
    assert_eq!(extern_double.display_name(), "c_double");
}

#[test]
fn test_type_assignability_with_extern() {
    let c_int1 = Type::Extern(ExternType::CInt);
    let c_int2 = Type::Extern(ExternType::CInt);
    let c_double = Type::Extern(ExternType::CDouble);

    // Same extern types are assignable
    assert!(c_int1.is_assignable_to(&c_int2));

    // Different extern types are not assignable
    assert!(!c_int1.is_assignable_to(&c_double));

    // Extern types don't assign to regular types
    assert!(!c_int1.is_assignable_to(&Type::Number));
    assert!(!Type::Number.is_assignable_to(&c_int1));
}

#[test]
fn test_extern_type_equality() {
    assert_eq!(ExternType::CInt, ExternType::CInt);
    assert_ne!(ExternType::CInt, ExternType::CLong);
    assert_ne!(ExternType::CDouble, ExternType::CBool);
}

#[test]
fn test_all_extern_types_exist() {
    // Verify all 6 extern types are defined and accessible
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

#[rstest]
#[case(ExternType::CInt, Type::Number, true)]
#[case(ExternType::CLong, Type::Number, true)]
#[case(ExternType::CDouble, Type::Number, true)]
#[case(ExternType::CCharPtr, Type::String, true)]
#[case(ExternType::CVoid, Type::Void, true)]
#[case(ExternType::CBool, Type::Bool, true)]
#[case(ExternType::CInt, Type::String, false)]
#[case(ExternType::CCharPtr, Type::Number, false)]
fn test_extern_type_accepts_atlas_type(
    #[case] extern_type: ExternType,
    #[case] atlas_type: Type,
    #[case] expected: bool,
) {
    assert_eq!(extern_type.accepts_atlas_type(&atlas_type), expected);
}

#[rstest]
#[case(ExternType::CInt, Type::Number)]
#[case(ExternType::CLong, Type::Number)]
#[case(ExternType::CDouble, Type::Number)]
#[case(ExternType::CCharPtr, Type::String)]
#[case(ExternType::CVoid, Type::Void)]
#[case(ExternType::CBool, Type::Bool)]
fn test_extern_type_to_atlas_type_mapping(
    #[case] extern_type: ExternType,
    #[case] expected_atlas_type: Type,
) {
    assert_eq!(extern_type.to_atlas_type(), expected_atlas_type);
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

// ====================
// Atlas→C Marshaling Tests (10 tests)
// ====================

#[rstest]
#[case(Value::Number(42.0), ExternType::CInt, CType::Int(42))]
#[case(Value::Number(0.0), ExternType::CInt, CType::Int(0))]
#[case(Value::Number(-100.0), ExternType::CInt, CType::Int(-100))]
fn test_marshal_number_to_cint_valid(
    #[case] value: Value,
    #[case] target: ExternType,
    #[case] expected: CType,
) {
    let mut ctx = MarshalContext::new();
    let result = ctx.atlas_to_c(&value, &target).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_marshal_number_to_cint_out_of_range() {
    let mut ctx = MarshalContext::new();
    // i32::MAX + 1 is out of range
    let result = ctx.atlas_to_c(&Value::Number(3e9), &ExternType::CInt);
    assert!(matches!(result, Err(MarshalError::NumberOutOfRange { .. })));
}

#[test]
fn test_marshal_number_to_cint_non_integer() {
    let mut ctx = MarshalContext::new();
    // Non-integer values should fail for CInt
    let result = ctx.atlas_to_c(&Value::Number(3.14), &ExternType::CInt);
    assert!(matches!(result, Err(MarshalError::NumberOutOfRange { .. })));
}

#[rstest]
#[case(Value::Number(1000.0), CType::Long(1000))]
#[case(Value::Number(0.0), CType::Long(0))]
#[case(Value::Number(-999.0), CType::Long(-999))]
fn test_marshal_number_to_clong(#[case] value: Value, #[case] expected: CType) {
    let mut ctx = MarshalContext::new();
    let result = ctx.atlas_to_c(&value, &ExternType::CLong).unwrap();
    assert_eq!(result, expected);
}

#[rstest]
#[case(Value::Number(3.14), CType::Double(3.14))]
#[case(Value::Number(0.0), CType::Double(0.0))]
#[case(Value::Number(-2.5), CType::Double(-2.5))]
fn test_marshal_number_to_cdouble(#[case] value: Value, #[case] expected: CType) {
    let mut ctx = MarshalContext::new();
    let result = ctx.atlas_to_c(&value, &ExternType::CDouble).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_marshal_string_to_char_ptr_valid() {
    let mut ctx = MarshalContext::new();
    let result = ctx.atlas_to_c(&Value::string("hello world"), &ExternType::CCharPtr);

    assert!(result.is_ok());
    if let Ok(CType::CharPtr(ptr)) = result {
        assert!(!ptr.is_null());
        unsafe {
            let c_str = std::ffi::CStr::from_ptr(ptr);
            assert_eq!(c_str.to_str().unwrap(), "hello world");
        }
    }
}

#[test]
fn test_marshal_string_with_null_byte() {
    let mut ctx = MarshalContext::new();
    // Strings containing null bytes should fail
    let result = ctx.atlas_to_c(&Value::string("hello\0world"), &ExternType::CCharPtr);
    assert!(matches!(result, Err(MarshalError::InvalidString(_))));
}

#[rstest]
#[case(Value::Bool(true), CType::Bool(1))]
#[case(Value::Bool(false), CType::Bool(0))]
fn test_marshal_bool_to_cbool(#[case] value: Value, #[case] expected: CType) {
    let mut ctx = MarshalContext::new();
    let result = ctx.atlas_to_c(&value, &ExternType::CBool).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_marshal_null_to_cvoid() {
    let mut ctx = MarshalContext::new();
    let result = ctx.atlas_to_c(&Value::Null, &ExternType::CVoid);
    assert_eq!(result, Ok(CType::Void));
}

#[rstest]
#[case(Value::string("hello"), ExternType::CInt)]
#[case(Value::Number(42.0), ExternType::CCharPtr)]
#[case(Value::Bool(true), ExternType::CDouble)]
fn test_marshal_type_mismatch(#[case] value: Value, #[case] target: ExternType) {
    let mut ctx = MarshalContext::new();
    let result = ctx.atlas_to_c(&value, &target);
    assert!(matches!(result, Err(MarshalError::TypeMismatch { .. })));
}

// ====================
// C→Atlas Marshaling Tests (7 tests)
// ====================

#[rstest]
#[case(CType::Int(42), Value::Number(42.0))]
#[case(CType::Int(0), Value::Number(0.0))]
#[case(CType::Int(-100), Value::Number(-100.0))]
fn test_unmarshal_cint_to_number(#[case] c_value: CType, #[case] expected: Value) {
    let ctx = MarshalContext::new();
    let result = ctx.c_to_atlas(&c_value).unwrap();
    assert_eq!(result, expected);
}

#[rstest]
#[case(CType::Long(1000), Value::Number(1000.0))]
#[case(CType::Long(0), Value::Number(0.0))]
#[case(CType::Long(-999), Value::Number(-999.0))]
fn test_unmarshal_clong_to_number(#[case] c_value: CType, #[case] expected: Value) {
    let ctx = MarshalContext::new();
    let result = ctx.c_to_atlas(&c_value).unwrap();
    assert_eq!(result, expected);
}

#[rstest]
#[case(CType::Double(3.14), Value::Number(3.14))]
#[case(CType::Double(0.0), Value::Number(0.0))]
#[case(CType::Double(-2.5), Value::Number(-2.5))]
fn test_unmarshal_cdouble_to_number(#[case] c_value: CType, #[case] expected: Value) {
    let ctx = MarshalContext::new();
    let result = ctx.c_to_atlas(&c_value).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_unmarshal_char_ptr_to_string() {
    let c_string = std::ffi::CString::new("hello").unwrap();
    let ctx = MarshalContext::new();
    let result = ctx.c_to_atlas(&CType::CharPtr(c_string.as_ptr())).unwrap();
    assert_eq!(result, Value::string("hello"));
}

#[rstest]
#[case(CType::Bool(1), Value::Bool(true))]
#[case(CType::Bool(0), Value::Bool(false))]
#[case(CType::Bool(255), Value::Bool(true))] // Non-zero is true
#[case(CType::Bool(100), Value::Bool(true))]
fn test_unmarshal_cbool_to_bool(#[case] c_value: CType, #[case] expected: Value) {
    let ctx = MarshalContext::new();
    let result = ctx.c_to_atlas(&c_value).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_unmarshal_cvoid_to_null() {
    let ctx = MarshalContext::new();
    let result = ctx.c_to_atlas(&CType::Void).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn test_unmarshal_null_pointer() {
    let ctx = MarshalContext::new();
    let result = ctx.c_to_atlas(&CType::CharPtr(std::ptr::null()));
    assert!(matches!(result, Err(MarshalError::NullPointer)));
}

// ====================
// MarshalContext Tests (5 tests)
// ====================

#[test]
fn test_marshal_context_tracks_strings() {
    let mut ctx = MarshalContext::new();

    // Allocate multiple C strings and verify they remain valid
    let s1 = ctx
        .atlas_to_c(&Value::string("first"), &ExternType::CCharPtr)
        .unwrap();
    let s2 = ctx
        .atlas_to_c(&Value::string("second"), &ExternType::CCharPtr)
        .unwrap();
    let s3 = ctx
        .atlas_to_c(&Value::string("third"), &ExternType::CCharPtr)
        .unwrap();

    // Verify all strings are still accessible (they're being tracked)
    if let (CType::CharPtr(p1), CType::CharPtr(p2), CType::CharPtr(p3)) = (s1, s2, s3) {
        unsafe {
            assert_eq!(std::ffi::CStr::from_ptr(p1).to_str().unwrap(), "first");
            assert_eq!(std::ffi::CStr::from_ptr(p2).to_str().unwrap(), "second");
            assert_eq!(std::ffi::CStr::from_ptr(p3).to_str().unwrap(), "third");
        }
    }
}

#[test]
fn test_marshal_context_cleanup() {
    // Create context in inner scope
    {
        let mut ctx = MarshalContext::new();
        ctx.atlas_to_c(&Value::string("test"), &ExternType::CCharPtr)
            .unwrap();
        // ctx drops here, cleaning up allocated strings
    }
    // If we reach here without crash, cleanup worked
}

#[test]
fn test_marshal_context_multiple_conversions() {
    let mut ctx = MarshalContext::new();

    // Different types in same context - all should succeed
    let int_result = ctx.atlas_to_c(&Value::Number(42.0), &ExternType::CInt);
    let str_result = ctx.atlas_to_c(&Value::string("hello"), &ExternType::CCharPtr);
    let bool_result = ctx.atlas_to_c(&Value::Bool(true), &ExternType::CBool);
    let double_result = ctx.atlas_to_c(&Value::Number(3.14), &ExternType::CDouble);

    assert!(int_result.is_ok());
    assert!(str_result.is_ok());
    assert!(bool_result.is_ok());
    assert!(double_result.is_ok());
}

#[test]
fn test_marshal_context_reuse() {
    let mut ctx = MarshalContext::new();

    // Use context multiple times
    for i in 0..10 {
        let result = ctx.atlas_to_c(&Value::Number(i as f64), &ExternType::CInt);
        assert_eq!(result, Ok(CType::Int(i)));
    }

    // Context can be reused without issues
}

#[test]
fn test_marshal_context_concurrent_conversions() {
    let mut ctx = MarshalContext::new();

    // Perform multiple conversions that allocate and deallocate
    let str1 = ctx
        .atlas_to_c(&Value::string("first"), &ExternType::CCharPtr)
        .unwrap();
    let str2 = ctx
        .atlas_to_c(&Value::string("second"), &ExternType::CCharPtr)
        .unwrap();

    // Both strings should remain valid
    if let CType::CharPtr(ptr1) = str1 {
        if let CType::CharPtr(ptr2) = str2 {
            unsafe {
                assert_eq!(std::ffi::CStr::from_ptr(ptr1).to_str().unwrap(), "first");
                assert_eq!(std::ffi::CStr::from_ptr(ptr2).to_str().unwrap(), "second");
            }
        }
    }
}

// ====================
// Integration Tests
// ====================

#[test]
fn test_roundtrip_marshaling() {
    // Test Atlas → C → Atlas roundtrip for all types
    let mut ctx = MarshalContext::new();

    // Number (via CInt)
    let num = Value::Number(42.0);
    let c_num = ctx.atlas_to_c(&num, &ExternType::CInt).unwrap();
    let num_back = ctx.c_to_atlas(&c_num).unwrap();
    assert_eq!(num, num_back);

    // Number (via CDouble)
    let num_f = Value::Number(3.14);
    let c_num_f = ctx.atlas_to_c(&num_f, &ExternType::CDouble).unwrap();
    let num_f_back = ctx.c_to_atlas(&c_num_f).unwrap();
    assert_eq!(num_f, num_f_back);

    // Bool
    let b = Value::Bool(true);
    let c_b = ctx.atlas_to_c(&b, &ExternType::CBool).unwrap();
    let b_back = ctx.c_to_atlas(&c_b).unwrap();
    assert_eq!(b, b_back);

    // Void (represented as Null)
    let v = Value::Null;
    let c_v = ctx.atlas_to_c(&v, &ExternType::CVoid).unwrap();
    let v_back = ctx.c_to_atlas(&c_v).unwrap();
    assert_eq!(v, v_back);
}
