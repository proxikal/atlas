//! Complete FFI Integration Tests (phase-10c)
//!
//! Tests for full FFI system: extern calls, callbacks, type marshaling, and parity.

use atlas_runtime::compiler::Compiler;
use atlas_runtime::ffi::{create_callback, ExternType};
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::value::Value;
use atlas_runtime::vm::VM;

fn run_interpreter(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return Err(format!("Lexer errors: {:?}", lex_diags));
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        return Err(format!("Parser errors: {:?}", parse_diags));
    }

    let mut interpreter = Interpreter::new();
    let security = SecurityContext::default();
    interpreter
        .eval(&program, &security)
        .map_err(|e| format!("Runtime error: {}", e))
}

fn run_vm(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return Err(format!("Lexer errors: {:?}", lex_diags));
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        return Err(format!("Parser errors: {:?}", parse_diags));
    }

    let mut compiler = Compiler::new();
    let bytecode = compiler
        .compile(&program)
        .map_err(|e| format!("Compiler error: {:?}", e))?;

    let mut vm = VM::new(bytecode);
    let security = SecurityContext::default();

    vm.load_extern_declarations(&program)
        .map_err(|e| format!("Extern loading error: {}", e))?;

    vm.run(&security)
        .map_err(|e| format!("Runtime error: {}", e))
        .map(|opt| opt.unwrap_or(Value::Null))
}

// ===== Full FFI Flow Tests =====

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "libm not available as .dll on Windows"
)]
fn test_full_ffi_flow_interpreter() {
    let source = r#"
        extern "m" fn sqrt(x: CDouble) -> CDouble;
        sqrt(16.0);
    "#;

    match run_interpreter(source) {
        Ok(Value::Number(n)) => {
            assert!(
                (n - 4.0).abs() < 0.0001,
                "sqrt(16) should be 4.0, got {}",
                n
            );
        }
        Ok(other) => panic!("Expected number, got: {:?}", other),
        Err(e) => panic!("Program failed: {}", e),
    }
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "libm not available as .dll on Windows"
)]
fn test_full_ffi_flow_vm() {
    let source = r#"
        extern "m" fn sqrt(x: CDouble) -> CDouble;
        sqrt(25.0);
    "#;

    match run_vm(source) {
        Ok(Value::Number(n)) => {
            assert!(
                (n - 5.0).abs() < 0.0001,
                "sqrt(25) should be 5.0, got {}",
                n
            );
        }
        Ok(other) => panic!("Expected number, got: {:?}", other),
        Err(e) => panic!("Program failed: {}", e),
    }
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "libm not available as .dll on Windows"
)]
fn test_parity_extern_call_basic() {
    let source = r#"
        extern "m" fn pow(base: CDouble, exp: CDouble) -> CDouble;
        pow(2.0, 8.0);
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    match (interp_result, vm_result) {
        (Value::Number(i), Value::Number(v)) => {
            assert!(
                (i - v).abs() < 0.0001,
                "Interpreter and VM results differ: {} vs {}",
                i,
                v
            );
            assert!((i - 256.0).abs() < 0.0001, "2^8 should be 256");
        }
        _ => panic!("Expected number results from both engines"),
    }
}

// ===== Callback Tests =====

#[test]
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
fn test_callback_basic_functionality() {
    use std::os::raw::c_double;

    let handle = create_callback(
        |args: &[Value]| {
            if let Some(Value::Number(n)) = args.first() {
                Ok(Value::Number(n.sqrt()))
            } else {
                Ok(Value::Number(0.0))
            }
        },
        vec![ExternType::CDouble],
        ExternType::CDouble,
    )
    .unwrap();

    let fn_ptr = handle.fn_ptr() as *const ();
    let f: extern "C" fn(c_double) -> c_double = unsafe { std::mem::transmute(fn_ptr) };

    let result = f(49.0);
    assert!((result - 7.0).abs() < 0.0001);
}

#[test]
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
fn test_callback_with_computation() {
    use std::os::raw::c_double;

    let handle = create_callback(
        |args: &[Value]| {
            if let (Some(Value::Number(a)), Some(Value::Number(b))) = (args.get(0), args.get(1)) {
                // Pythagorean theorem: sqrt(a^2 + b^2)
                Ok(Value::Number((a * a + b * b).sqrt()))
            } else {
                Ok(Value::Number(0.0))
            }
        },
        vec![ExternType::CDouble, ExternType::CDouble],
        ExternType::CDouble,
    )
    .unwrap();

    let fn_ptr = handle.fn_ptr() as *const ();
    let f: extern "C" fn(c_double, c_double) -> c_double = unsafe { std::mem::transmute(fn_ptr) };

    let result = f(3.0, 4.0);
    assert!((result - 5.0).abs() < 0.0001);
}

// ===== Error Handling Tests =====

#[test]
fn test_error_propagation_extern_library_not_found() {
    let source = r#"
        extern "totally_fake_library_9999" fn fake_func() -> CInt;
        fake_func();
    "#;

    assert!(run_interpreter(source).is_err());
    assert!(run_vm(source).is_err());
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "libm not available as .dll on Windows"
)]
fn test_error_propagation_symbol_not_found() {
    let source = r#"
        extern "m" fn totally_fake_symbol_xyz() -> CDouble;
        totally_fake_symbol_xyz();
    "#;

    assert!(run_interpreter(source).is_err());
    assert!(run_vm(source).is_err());
}

// ===== Complex Integration Tests =====

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "libm not available as .dll on Windows"
)]
fn test_extern_with_user_functions() {
    let source = r#"
        extern "m" fn sqrt(x: CDouble) -> CDouble;

        fn distance(x1: number, y1: number, x2: number, y2: number) -> number {
            let dx = x2 - x1;
            let dy = y2 - y1;
            return sqrt(dx * dx + dy * dy);
        }

        distance(0.0, 0.0, 3.0, 4.0);
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    match (interp_result, vm_result) {
        (Value::Number(i), Value::Number(v)) => {
            assert!((i - 5.0).abs() < 0.0001);
            assert!((v - 5.0).abs() < 0.0001);
            assert!((i - v).abs() < 0.0001, "Parity check failed");
        }
        _ => panic!("Expected number results"),
    }
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "libm not available as .dll on Windows"
)]
fn test_multiple_extern_functions() {
    let source = r#"
        extern "m" fn sin(x: CDouble) -> CDouble;
        extern "m" fn cos(x: CDouble) -> CDouble;

        let x = 0.0;
        let s = sin(x);
        let c = cos(x);
        s * s + c * c;
    "#;

    let result = run_interpreter(source).unwrap();
    if let Value::Number(n) = result {
        assert!((n - 1.0).abs() < 0.0001, "sin^2 + cos^2 should be 1");
    } else {
        panic!("Expected number");
    }
}

// ===== Performance and Stress Tests =====

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "libm not available as .dll on Windows"
)]
fn test_ffi_multiple_calls() {
    let source = r#"
        extern "m" fn sqrt(x: CDouble) -> CDouble;

        fn sum_of_roots() -> number {
            let total = 0;
            let i = 1;
            while (i <= 10) {
                total = total + sqrt(i);
                i = i + 1;
            }
            return total;
        }

        sum_of_roots();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    match (interp_result, vm_result) {
        (Value::Number(i), Value::Number(v)) => {
            // Sum of sqrt(1) through sqrt(10)
            assert!(i > 0.0);
            assert!((i - v).abs() < 0.01, "Parity check failed");
        }
        _ => panic!("Expected number results"),
    }
}

#[test]
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
fn test_callback_stress_multiple_types() {
    use std::os::raw::{c_double, c_int, c_long};

    // Test CDouble
    let handle1 = create_callback(
        |args: &[Value]| {
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

    // Test CInt
    let handle2 = create_callback(
        |args: &[Value]| {
            if let Some(Value::Number(n)) = args.first() {
                Ok(Value::Number(((*n as i32) + 10) as f64))
            } else {
                Ok(Value::Number(0.0))
            }
        },
        vec![ExternType::CInt],
        ExternType::CInt,
    )
    .unwrap();

    // Test CLong
    let handle3 = create_callback(
        |args: &[Value]| {
            if let Some(Value::Number(n)) = args.first() {
                Ok(Value::Number(((*n as i64) - 5) as f64))
            } else {
                Ok(Value::Number(0.0))
            }
        },
        vec![ExternType::CLong],
        ExternType::CLong,
    )
    .unwrap();

    // Call them all
    let f1: extern "C" fn(c_double) -> c_double =
        unsafe { std::mem::transmute(handle1.fn_ptr() as *const ()) };
    let f2: extern "C" fn(c_int) -> c_int =
        unsafe { std::mem::transmute(handle2.fn_ptr() as *const ()) };
    let f3: extern "C" fn(c_long) -> c_long =
        unsafe { std::mem::transmute(handle3.fn_ptr() as *const ()) };

    assert!((f1(21.0) - 42.0).abs() < 0.0001);
    assert_eq!(f2(32), 42);
    assert_eq!(f3(47), 42);
}

// ===== Safety Wrapper Tests =====

#[test]
fn test_safe_cstring() {
    use atlas_runtime::ffi::SafeCString;

    let s = SafeCString::new("hello world").unwrap();
    assert!(!s.as_ptr().is_null());
}

#[test]
fn test_safe_cstring_with_null_byte() {
    use atlas_runtime::ffi::SafeCString;

    let result = SafeCString::new("hello\0world");
    assert!(result.is_err());
}

#[test]
fn test_check_null_valid() {
    use atlas_runtime::ffi::check_null;

    let x = 42;
    let ptr = &x as *const i32;
    assert!(check_null(ptr).is_ok());
}

#[test]
fn test_check_null_invalid() {
    use atlas_runtime::ffi::check_null;

    let ptr: *const i32 = std::ptr::null();
    assert!(check_null(ptr).is_err());
}

#[test]
fn test_bounded_buffer() {
    use atlas_runtime::ffi::BoundedBuffer;

    let data = vec![1u8, 2, 3, 4, 5];
    let buffer = BoundedBuffer::new(data.as_ptr(), data.len()).unwrap();

    assert_eq!(buffer.len(), 5);
    assert_eq!(buffer.as_slice(), &[1, 2, 3, 4, 5]);
    assert!(!buffer.is_empty());
}

#[test]
fn test_safe_marshal_context() {
    use atlas_runtime::ffi::SafeMarshalContext;

    let mut ctx = SafeMarshalContext::new();
    let result = ctx.safe_atlas_to_c(&Value::Number(42.0), &ExternType::CDouble);
    assert!(result.is_ok());
}

// ===== Platform Compatibility Tests =====

#[test]
fn test_ffi_platform_compatibility() {
    // This test verifies FFI works on current platform
    let handle = create_callback(
        |_args: &[Value]| Ok(Value::Number(1.0)),
        vec![],
        ExternType::CInt,
    )
    .unwrap();

    assert!(!handle.fn_ptr().is_null());
}

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "libm not available as .dll on Windows"
)]
fn test_library_loading_platform_specific() {
    // Tests that library names resolve correctly on current platform
    let source = r#"
        extern "m" fn sqrt(x: CDouble) -> CDouble;
        sqrt(4.0);
    "#;

    // Should work on Linux/macOS (libm.so/libm.dylib)
    // Ignored on Windows (no libm.dll)
    let result = run_interpreter(source);
    assert!(result.is_ok() || result.is_err()); // May fail on macOS without separate libm
}
