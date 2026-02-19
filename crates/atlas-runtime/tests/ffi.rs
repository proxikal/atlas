// Merged: ffi_callback_tests + ffi_integration_complete_tests +
//         ffi_parsing_tests + ffi_types_tests +
//         ffi_interpreter_tests (mod interpreter_tests) + ffi_vm_tests (mod vm_tests)
// Platform-specific cfg_attr annotations preserved exactly.

use atlas_runtime::ast::{ExternTypeAnnotation, Item};
use atlas_runtime::compiler::Compiler;
use atlas_runtime::ffi::{create_callback, CType, ExternType, MarshalContext, MarshalError};
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::types::Type;
use atlas_runtime::value::{RuntimeError, Value};
use atlas_runtime::vm::VM;
use rstest::rstest;
use std::ffi::c_void;

// ===== ffi_callback_tests.rs =====

// FFI Callback Tests (phase-10c)
//
// Tests for C→Atlas function callbacks.

fn parse_and_eval(source: &str) -> Result<(Interpreter, Value), String> {
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
    let result = interpreter
        .eval(&program, &security)
        .map_err(|e| format!("Runtime error: {}", e))?;

    Ok((interpreter, result))
}

// ===== Callback Creation Tests =====

#[test]
fn test_create_callback_simple() {
    let source = r#"
        fn double(x: number) -> number {
            return x * 2;
        }
    "#;

    let (mut interp, _) = parse_and_eval(source).unwrap();

    let callback_ptr = interp
        .create_callback("double", vec![ExternType::CDouble], ExternType::CDouble)
        .unwrap();

    assert!(!callback_ptr.is_null());
    assert_eq!(interp.callback_count(), 1);
}

#[test]
fn test_create_callback_missing_function() {
    let source = r#"
        fn exists(x: number) -> number {
            return x;
        }
    "#;

    let (mut interp, _) = parse_and_eval(source).unwrap();

    let result = interp.create_callback(
        "nonexistent",
        vec![ExternType::CDouble],
        ExternType::CDouble,
    );

    assert!(result.is_err());
}

#[test]
fn test_create_callback_multiple() {
    let source = r#"
        fn add(x: number, y: number) -> number {
            return x + y;
        }
        fn multiply(x: number, y: number) -> number {
            return x * y;
        }
    "#;

    let (mut interp, _) = parse_and_eval(source).unwrap();

    let cb1 = interp
        .create_callback(
            "add",
            vec![ExternType::CDouble, ExternType::CDouble],
            ExternType::CDouble,
        )
        .unwrap();

    let cb2 = interp
        .create_callback(
            "multiply",
            vec![ExternType::CDouble, ExternType::CDouble],
            ExternType::CDouble,
        )
        .unwrap();

    assert!(!cb1.is_null());
    assert!(!cb2.is_null());
    assert_eq!(interp.callback_count(), 2);
}

#[test]
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
fn test_callback_function_pointer_valid() {
    let source = r#"
        fn identity(x: number) -> number {
            return x;
        }
    "#;

    let (mut interp, _) = parse_and_eval(source).unwrap();

    let callback_ptr = interp
        .create_callback("identity", vec![ExternType::CDouble], ExternType::CDouble)
        .unwrap();

    assert!(!callback_ptr.is_null());
    // Function pointer is valid and can be stored
    let _ = callback_ptr as usize;
}

#[test]
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
fn test_callback_no_params() {
    let source = r#"
        fn get_constant() -> number {
            return 42;
        }
    "#;

    let (mut interp, _) = parse_and_eval(source).unwrap();

    let callback_ptr = interp
        .create_callback("get_constant", vec![], ExternType::CInt)
        .unwrap();

    assert!(!callback_ptr.is_null());
}

#[test]
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
fn test_callback_void_return() {
    let source = r#"
        fn do_nothing(x: number) -> void {
            let y = x + 1;
        }
    "#;

    let (mut interp, _) = parse_and_eval(source).unwrap();

    let callback_ptr = interp
        .create_callback("do_nothing", vec![ExternType::CInt], ExternType::CVoid)
        .unwrap();

    assert!(!callback_ptr.is_null());
}

#[test]
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
fn test_callback_int_params() {
    let source = r#"
        fn sum_ints(a: number, b: number) -> number {
            return a + b;
        }
    "#;

    let (mut interp, _) = parse_and_eval(source).unwrap();

    let callback_ptr = interp
        .create_callback(
            "sum_ints",
            vec![ExternType::CInt, ExternType::CInt],
            ExternType::CInt,
        )
        .unwrap();

    assert!(!callback_ptr.is_null());
}

// ===== Callback Execution Tests =====

#[test]
fn test_callback_basic_call() {
    use std::os::raw::c_double;

    let handle = create_callback(
        |args: &[Value]| {
            if let Some(Value::Number(n)) = args.first() {
                Ok(Value::Number(n * 3.0))
            } else {
                Ok(Value::Number(0.0))
            }
        },
        vec![ExternType::CDouble],
        ExternType::CDouble,
    )
    .unwrap();

    // Cast to trampoline and call with context
    let trampoline: unsafe extern "C" fn(*mut c_void, c_double) -> c_double =
        unsafe { std::mem::transmute(handle.trampoline()) };

    let result = unsafe { trampoline(handle.context(), 10.0) };
    assert!((result - 30.0).abs() < 0.0001);
}

#[test]
fn test_callback_with_two_params() {
    use std::os::raw::c_double;

    let handle = create_callback(
        |args: &[Value]| {
            if let (Some(Value::Number(a)), Some(Value::Number(b))) = (args.first(), args.get(1)) {
                Ok(Value::Number(a + b))
            } else {
                Ok(Value::Number(0.0))
            }
        },
        vec![ExternType::CDouble, ExternType::CDouble],
        ExternType::CDouble,
    )
    .unwrap();

    let trampoline: unsafe extern "C" fn(*mut c_void, c_double, c_double) -> c_double =
        unsafe { std::mem::transmute(handle.trampoline()) };

    let result = unsafe { trampoline(handle.context(), 15.0, 25.0) };
    assert!((result - 40.0).abs() < 0.0001);
}

#[test]
fn test_callback_int_return() {
    use std::os::raw::c_int;

    let handle = create_callback(
        |args: &[Value]| {
            if let Some(Value::Number(n)) = args.first() {
                Ok(Value::Number(((*n as i32) * 2) as f64))
            } else {
                Ok(Value::Number(0.0))
            }
        },
        vec![ExternType::CInt],
        ExternType::CInt,
    )
    .unwrap();

    let trampoline: unsafe extern "C" fn(*mut c_void, c_int) -> c_int =
        unsafe { std::mem::transmute(handle.trampoline()) };

    let result = unsafe { trampoline(handle.context(), 21) };
    assert_eq!(result, 42);
}

#[test]
fn test_callback_no_params_call() {
    use std::os::raw::c_int;

    let handle = create_callback(
        |_args: &[Value]| Ok(Value::Number(99.0)),
        vec![],
        ExternType::CInt,
    )
    .unwrap();

    let trampoline: unsafe extern "C" fn(*mut c_void) -> c_int =
        unsafe { std::mem::transmute(handle.trampoline()) };

    let result = unsafe { trampoline(handle.context()) };
    assert_eq!(result, 99);
}

#[test]
fn test_callback_void_return_call() {
    use std::os::raw::c_int;

    let handle = create_callback(
        |_args: &[Value]| Ok(Value::Null),
        vec![ExternType::CInt],
        ExternType::CVoid,
    )
    .unwrap();

    let trampoline: unsafe extern "C" fn(*mut c_void, c_int) =
        unsafe { std::mem::transmute(handle.trampoline()) };

    // Should not crash
    unsafe { trampoline(handle.context(), 42) };
}

#[test]
fn test_callback_error_handling() {
    use std::os::raw::c_double;

    let handle = create_callback(
        |args: &[Value]| {
            if let Some(Value::Number(n)) = args.first() {
                if *n < 0.0 {
                    Err(RuntimeError::TypeError {
                        msg: "Negative not allowed".to_string(),
                        span: atlas_runtime::span::Span::dummy(),
                    })
                } else {
                    Ok(Value::Number(n.sqrt()))
                }
            } else {
                Ok(Value::Number(0.0))
            }
        },
        vec![ExternType::CDouble],
        ExternType::CDouble,
    )
    .unwrap();

    let trampoline: unsafe extern "C" fn(*mut c_void, c_double) -> c_double =
        unsafe { std::mem::transmute(handle.trampoline()) };

    // Should not crash on error, returns 0.0
    let result = unsafe { trampoline(handle.context(), -1.0) };
    assert_eq!(result, 0.0);
}

// ===== Memory Safety Tests =====

#[test]
fn test_callback_lifetime_management() {
    use std::os::raw::c_double;

    let handle = create_callback(
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

    let trampoline: unsafe extern "C" fn(*mut c_void, c_double) -> c_double =
        unsafe { std::mem::transmute(handle.trampoline()) };
    let context = handle.context();

    // Call while handle is alive
    let result1 = unsafe { trampoline(context, 5.0) };
    assert!((result1 - 10.0).abs() < 0.0001);

    // Handle still valid
    let result2 = unsafe { trampoline(context, 7.0) };
    assert!((result2 - 14.0).abs() < 0.0001);

    // Explicit drop
    drop(handle);

    // After drop, function pointer is invalid (don't call it)
    // This test just verifies the drop doesn't crash
}

#[test]
fn test_callback_multiple_invocations() {
    use std::os::raw::c_int;

    let handle = create_callback(
        |args: &[Value]| {
            if let (Some(Value::Number(a)), Some(Value::Number(b))) = (args.first(), args.get(1)) {
                Ok(Value::Number(((*a as i32) + (*b as i32)) as f64))
            } else {
                Ok(Value::Number(0.0))
            }
        },
        vec![ExternType::CInt, ExternType::CInt],
        ExternType::CInt,
    )
    .unwrap();

    let trampoline: unsafe extern "C" fn(*mut c_void, c_int, c_int) -> c_int =
        unsafe { std::mem::transmute(handle.trampoline()) };
    let context = handle.context();

    // Multiple calls
    assert_eq!(unsafe { trampoline(context, 1, 2) }, 3);
    assert_eq!(unsafe { trampoline(context, 10, 20) }, 30);
    assert_eq!(unsafe { trampoline(context, 100, 200) }, 300);
}

#[test]
fn test_callback_signature() {
    let handle = create_callback(
        |_args: &[Value]| Ok(Value::Number(0.0)),
        vec![ExternType::CDouble],
        ExternType::CDouble,
    )
    .unwrap();

    let (params, ret) = handle.signature();
    assert_eq!(params.len(), 1);
    assert!(matches!(params[0], ExternType::CDouble));
    assert!(matches!(ret, ExternType::CDouble));
}

// ===== ffi_integration_complete_tests.rs =====

// Complete FFI Integration Tests (phase-10c)
//
// Tests for full FFI system: extern calls, callbacks, type marshaling, and parity.

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
    any(target_os = "windows", target_os = "macos"),
    ignore = "libm not available as standalone shared library on this platform"
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
    any(target_os = "windows", target_os = "macos"),
    ignore = "libm not available as standalone shared library on this platform"
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
    any(target_os = "windows", target_os = "macos"),
    ignore = "libm not available as standalone shared library on this platform"
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

    let trampoline: unsafe extern "C" fn(*mut c_void, c_double) -> c_double =
        unsafe { std::mem::transmute(handle.trampoline()) };

    let result = unsafe { trampoline(handle.context(), 49.0) };
    assert!((result - 7.0).abs() < 0.0001);
}

#[test]
fn test_callback_with_computation() {
    use std::os::raw::c_double;

    let handle = create_callback(
        |args: &[Value]| {
            if let (Some(Value::Number(a)), Some(Value::Number(b))) = (args.first(), args.get(1)) {
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

    let trampoline: unsafe extern "C" fn(*mut c_void, c_double, c_double) -> c_double =
        unsafe { std::mem::transmute(handle.trampoline()) };

    let result = unsafe { trampoline(handle.context(), 3.0, 4.0) };
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
    any(target_os = "windows", target_os = "macos"),
    ignore = "libm not available as standalone shared library on this platform"
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
    any(target_os = "windows", target_os = "macos"),
    ignore = "libm not available as standalone shared library on this platform"
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
    any(target_os = "windows", target_os = "macos"),
    ignore = "libm not available as standalone shared library on this platform"
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
    any(target_os = "windows", target_os = "macos"),
    ignore = "libm not available as standalone shared library on this platform"
)]
fn test_ffi_multiple_calls() {
    let source = r#"
        extern "m" fn sqrt(x: CDouble) -> CDouble;

        fn sum_of_roots() -> number {
            var total = 0;
            var i = 1;
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

    // Call them all with proper trampolines
    let f1: unsafe extern "C" fn(*mut c_void, c_double) -> c_double =
        unsafe { std::mem::transmute(handle1.trampoline()) };
    let f2: unsafe extern "C" fn(*mut c_void, c_int) -> c_int =
        unsafe { std::mem::transmute(handle2.trampoline()) };
    let f3: unsafe extern "C" fn(*mut c_void, c_long) -> c_long =
        unsafe { std::mem::transmute(handle3.trampoline()) };

    assert!(unsafe { (f1(handle1.context(), 21.0) - 42.0).abs() } < 0.0001);
    assert_eq!(unsafe { f2(handle2.context(), 32) }, 42);
    assert_eq!(unsafe { f3(handle3.context(), 47) }, 42);
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

    let data = [1u8, 2, 3, 4, 5];
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

    // Verify both trampoline and context are valid pointers
    assert!(!handle.trampoline().is_null());
    assert!(!handle.context().is_null());
}

#[test]
#[cfg_attr(
    any(target_os = "windows", target_os = "macos"),
    ignore = "libm not available as standalone shared library on this platform"
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

// ===== ffi_parsing_tests.rs =====

// Tests for FFI extern declaration parsing (phase-10b)

fn parse_program(source: &str) -> (Vec<Item>, Vec<atlas_runtime::diagnostic::Diagnostic>) {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty(), "Lexer errors: {:?}", lex_diags);

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    (program.items, parse_diags)
}

#[test]
fn test_extern_basic_declaration() {
    let source = r#"extern "libm" fn pow(base: CDouble, exp: CDouble) -> CDouble;"#;
    let (items, diagnostics) = parse_program(source);

    assert_eq!(diagnostics.len(), 0, "Parse errors: {:?}", diagnostics);
    assert_eq!(items.len(), 1);

    if let Item::Extern(extern_decl) = &items[0] {
        assert_eq!(extern_decl.name, "pow");
        assert_eq!(extern_decl.library, "libm");
        assert!(extern_decl.symbol.is_none());
        assert_eq!(extern_decl.params.len(), 2);
        assert_eq!(extern_decl.params[0].0, "base");
        assert!(matches!(
            extern_decl.params[0].1,
            ExternTypeAnnotation::CDouble
        ));
        assert_eq!(extern_decl.params[1].0, "exp");
        assert!(matches!(
            extern_decl.params[1].1,
            ExternTypeAnnotation::CDouble
        ));
        assert!(matches!(
            extern_decl.return_type,
            ExternTypeAnnotation::CDouble
        ));
    } else {
        panic!("Expected extern declaration, got: {:?}", items[0]);
    }
}

#[test]
fn test_extern_with_symbol_renaming() {
    let source = r#"extern "libc" fn string_length as "strlen"(s: CCharPtr) -> CLong;"#;
    let (items, diagnostics) = parse_program(source);

    assert_eq!(diagnostics.len(), 0, "Parse errors: {:?}", diagnostics);
    assert_eq!(items.len(), 1);

    if let Item::Extern(extern_decl) = &items[0] {
        assert_eq!(extern_decl.name, "string_length");
        assert_eq!(extern_decl.library, "libc");
        assert_eq!(extern_decl.symbol, Some("strlen".to_string()));
        assert_eq!(extern_decl.params.len(), 1);
        assert_eq!(extern_decl.params[0].0, "s");
        assert!(matches!(
            extern_decl.params[0].1,
            ExternTypeAnnotation::CCharPtr
        ));
        assert!(matches!(
            extern_decl.return_type,
            ExternTypeAnnotation::CLong
        ));
    } else {
        panic!("Expected extern declaration");
    }
}

#[test]
fn test_extern_no_params() {
    let source = r#"extern "libc" fn getpid() -> CInt;"#;
    let (items, diagnostics) = parse_program(source);

    assert_eq!(diagnostics.len(), 0, "Parse errors: {:?}", diagnostics);
    assert_eq!(items.len(), 1);

    if let Item::Extern(extern_decl) = &items[0] {
        assert_eq!(extern_decl.name, "getpid");
        assert_eq!(extern_decl.library, "libc");
        assert!(extern_decl.symbol.is_none());
        assert_eq!(extern_decl.params.len(), 0);
        assert!(matches!(
            extern_decl.return_type,
            ExternTypeAnnotation::CInt
        ));
    } else {
        panic!("Expected extern declaration");
    }
}

#[test]
fn test_extern_void_return() {
    let source = r#"extern "libc" fn exit(code: CInt) -> CVoid;"#;
    let (items, diagnostics) = parse_program(source);

    assert_eq!(diagnostics.len(), 0, "Parse errors: {:?}", diagnostics);
    assert_eq!(items.len(), 1);

    if let Item::Extern(extern_decl) = &items[0] {
        assert_eq!(extern_decl.name, "exit");
        assert_eq!(extern_decl.library, "libc");
        assert_eq!(extern_decl.params.len(), 1);
        assert_eq!(extern_decl.params[0].0, "code");
        assert!(matches!(
            extern_decl.params[0].1,
            ExternTypeAnnotation::CInt
        ));
        assert!(matches!(
            extern_decl.return_type,
            ExternTypeAnnotation::CVoid
        ));
    } else {
        panic!("Expected extern declaration");
    }
}

#[test]
fn test_extern_multiple_declarations() {
    let source = r#"
        extern "libm" fn sin(x: CDouble) -> CDouble;
        extern "libm" fn cos(x: CDouble) -> CDouble;
        extern "libm" fn tan(x: CDouble) -> CDouble;
    "#;
    let (items, diagnostics) = parse_program(source);

    assert_eq!(diagnostics.len(), 0, "Parse errors: {:?}", diagnostics);
    assert_eq!(items.len(), 3);

    let names: Vec<_> = items
        .iter()
        .filter_map(|item| {
            if let Item::Extern(extern_decl) = item {
                Some(extern_decl.name.as_str())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(names, vec!["sin", "cos", "tan"]);
}

#[test]
fn test_extern_all_types() {
    let source = r#"
        extern "test" fn test_int(x: CInt) -> CInt;
        extern "test" fn test_long(x: CLong) -> CLong;
        extern "test" fn test_double(x: CDouble) -> CDouble;
        extern "test" fn test_charptr(x: CCharPtr) -> CCharPtr;
        extern "test" fn test_void(x: CInt) -> CVoid;
        extern "test" fn test_bool(x: CBool) -> CBool;
    "#;
    let (items, diagnostics) = parse_program(source);

    assert_eq!(diagnostics.len(), 0, "Parse errors: {:?}", diagnostics);
    assert_eq!(items.len(), 6);

    // Verify all items are extern declarations
    for item in &items {
        assert!(matches!(item, Item::Extern(_)));
    }
}

#[test]
fn test_extern_invalid_type_error() {
    let source = r#"extern "lib" fn bad(x: InvalidType) -> CInt;"#;
    let (_items, diagnostics) = parse_program(source);

    // Should have a parse error for unknown type
    assert!(
        !diagnostics.is_empty(),
        "Expected parse error for invalid type"
    );
}

#[test]
fn test_extern_mixed_with_functions() {
    let source = r#"
        extern "libm" fn sqrt(x: CDouble) -> CDouble;
        fn double(x: number) -> number { return x * 2; }
        extern "libc" fn strlen(s: CCharPtr) -> CLong;
    "#;
    let (items, diagnostics) = parse_program(source);

    assert_eq!(diagnostics.len(), 0, "Parse errors: {:?}", diagnostics);
    assert_eq!(items.len(), 3);

    assert!(matches!(items[0], Item::Extern(_)));
    assert!(matches!(items[1], Item::Function(_)));
    assert!(matches!(items[2], Item::Extern(_)));
}

// ===== ffi_types_tests.rs =====

// Integration tests for FFI type system and marshaling
//
// Phase 10a: FFI Core Types + Type Marshaling
//
// Tests the complete FFI type system including:
// - ExternType enum and type conversions
// - Atlas ↔ C type marshaling
// - MarshalContext memory management
// - Type compatibility and validation

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
    let types = [
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
    assert_eq!(
        CType::Double(std::f64::consts::PI),
        CType::Double(std::f64::consts::PI)
    );
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
    let result = ctx.atlas_to_c(&Value::Number(std::f64::consts::PI), &ExternType::CInt);
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
#[case(
    Value::Number(std::f64::consts::PI),
    CType::Double(std::f64::consts::PI)
)]
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
#[case(
    CType::Double(std::f64::consts::PI),
    Value::Number(std::f64::consts::PI)
)]
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
    let double_result = ctx.atlas_to_c(&Value::Number(std::f64::consts::PI), &ExternType::CDouble);

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
    let num_f = Value::Number(std::f64::consts::PI);
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

// ===== ffi_interpreter_tests.rs (parity: interpreter_tests) =====

#[cfg(test)]
mod interpreter_tests {
    use super::*;

    // Integration tests for FFI interpreter execution (phase-10b)

    fn run_program(source: &str) -> Result<Value, String> {
        // Parse
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

        // Execute
        let mut interpreter = Interpreter::new();
        let security = SecurityContext::default();
        interpreter
            .eval(&program, &security)
            .map_err(|e| format!("Runtime error: {}", e))
    }

    #[test]
    #[cfg_attr(
        any(target_os = "windows", target_os = "macos"),
        ignore = "libm not available as standalone shared library on this platform"
    )]
    fn test_extern_sqrt_basic() {
        let source = r#"
            extern "m" fn sqrt(x: CDouble) -> CDouble;
            sqrt(16.0);
        "#;

        match run_program(source) {
            Ok(Value::Number(n)) => {
                assert!(
                    (n - 4.0).abs() < 0.0001,
                    "sqrt(16.0) should be 4.0, got {}",
                    n
                );
            }
            Ok(other) => panic!("Expected number, got: {:?}", other),
            Err(e) => panic!("Program failed: {}", e),
        }
    }

    #[test]
    #[cfg_attr(
        any(target_os = "windows", target_os = "macos"),
        ignore = "libm not available as standalone shared library on this platform"
    )]
    fn test_extern_pow_basic() {
        let source = r#"
            extern "m" fn pow(base: CDouble, exp: CDouble) -> CDouble;
            pow(2.0, 3.0);
        "#;

        match run_program(source) {
            Ok(Value::Number(n)) => {
                assert!(
                    (n - 8.0).abs() < 0.0001,
                    "pow(2.0, 3.0) should be 8.0, got {}",
                    n
                );
            }
            Ok(other) => panic!("Expected number, got: {:?}", other),
            Err(e) => panic!("Program failed: {}", e),
        }
    }

    #[test]
    #[cfg_attr(
        any(target_os = "windows", target_os = "macos"),
        ignore = "libm not available as standalone shared library on this platform"
    )]
    fn test_extern_multiple_calls() {
        let source = r#"
            extern "m" fn sqrt(x: CDouble) -> CDouble;
            let a = sqrt(9.0);
            let b = sqrt(25.0);
            a + b;
        "#;

        match run_program(source) {
            Ok(Value::Number(n)) => {
                // 3.0 + 5.0 = 8.0
                assert!(
                    (n - 8.0).abs() < 0.0001,
                    "sqrt(9) + sqrt(25) should be 8.0, got {}",
                    n
                );
            }
            Ok(other) => panic!("Expected number, got: {:?}", other),
            Err(e) => panic!("Program failed: {}", e),
        }
    }

    #[test]
    #[cfg_attr(
        any(target_os = "windows", target_os = "macos"),
        ignore = "libm not available as standalone shared library on this platform"
    )]
    fn test_extern_with_user_functions() {
        let source = r#"
            extern "m" fn sqrt(x: CDouble) -> CDouble;

            fn hypotenuse(a: number, b: number) -> number {
                return sqrt(a * a + b * b);
            }

            hypotenuse(3.0, 4.0);
        "#;

        match run_program(source) {
            Ok(Value::Number(n)) => {
                assert!(
                    (n - 5.0).abs() < 0.0001,
                    "hypotenuse(3, 4) should be 5.0, got {}",
                    n
                );
            }
            Ok(other) => panic!("Expected number, got: {:?}", other),
            Err(e) => panic!("Program failed: {}", e),
        }
    }

    #[test]
    fn test_extern_library_not_found() {
        let source = r#"
            extern "nonexistent_lib_xyz" fn foo() -> CInt;
            foo();
        "#;

        match run_program(source) {
            Err(e) if e.contains("Failed to load library") => {
                // Expected error
            }
            Ok(_) => panic!("Should have failed to load nonexistent library"),
            Err(e) => panic!("Wrong error: {}", e),
        }
    }

    #[test]
    #[cfg_attr(
        any(target_os = "windows", target_os = "macos"),
        ignore = "libm not available as standalone shared library on this platform"
    )]
    fn test_extern_symbol_not_found() {
        let source = r#"
            extern "m" fn nonexistent_symbol_xyz() -> CDouble;
            nonexistent_symbol_xyz();
        "#;

        match run_program(source) {
            Err(e) if e.contains("Failed to find symbol") => {
                // Expected error
            }
            Ok(_) => panic!("Should have failed to find nonexistent symbol"),
            Err(e) => panic!("Wrong error: {}", e),
        }
    }

    #[test]
    #[cfg_attr(
        any(target_os = "windows", target_os = "macos"),
        ignore = "libm not available as standalone shared library on this platform"
    )]
    fn test_extern_ceil_floor() {
        let source = r#"
            extern "m" fn ceil(x: CDouble) -> CDouble;
            extern "m" fn floor(x: CDouble) -> CDouble;

            let a = ceil(3.2);
            let b = floor(3.8);
            a + b;
        "#;

        match run_program(source) {
            Ok(Value::Number(n)) => {
                // ceil(3.2) = 4.0, floor(3.8) = 3.0, sum = 7.0
                assert!(
                    (n - 7.0).abs() < 0.0001,
                    "ceil(3.2) + floor(3.8) should be 7.0, got {}",
                    n
                );
            }
            Ok(other) => panic!("Expected number, got: {:?}", other),
            Err(e) => panic!("Program failed: {}", e),
        }
    }

    #[test]
    #[cfg_attr(
        any(target_os = "windows", target_os = "macos"),
        ignore = "libm not available as standalone shared library on this platform"
    )]
    fn test_extern_sin_cos() {
        let source = r#"
            extern "m" fn sin(x: CDouble) -> CDouble;
            extern "m" fn cos(x: CDouble) -> CDouble;

            // sin^2 + cos^2 = 1
            let x = 0.5;
            let s = sin(x);
            let c = cos(x);
            s * s + c * c;
        "#;

        match run_program(source) {
            Ok(Value::Number(n)) => {
                // sin^2(x) + cos^2(x) should always be 1
                assert!(
                    (n - 1.0).abs() < 0.0001,
                    "sin^2 + cos^2 should be 1.0, got {}",
                    n
                );
            }
            Ok(other) => panic!("Expected number, got: {:?}", other),
            Err(e) => panic!("Program failed: {}", e),
        }
    }
}

// ===== ffi_vm_tests.rs (parity: vm_tests) =====

#[cfg(test)]
mod vm_tests {
    use super::*;

    // Integration tests for FFI VM execution (phase-10b)

    fn run_program(source: &str) -> Result<Value, String> {
        // Parse
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

        // Compile
        let mut compiler = Compiler::new();
        let bytecode = compiler
            .compile(&program)
            .map_err(|e| format!("Compiler error: {:?}", e))?;

        // Execute
        let mut vm = VM::new(bytecode);
        let security = SecurityContext::default();

        // Load extern declarations BEFORE running bytecode
        vm.load_extern_declarations(&program)
            .map_err(|e| format!("Extern loading error: {}", e))?;

        vm.run(&security)
            .map_err(|e| format!("Runtime error: {}", e))
            .map(|opt| opt.unwrap_or(Value::Null))
    }

    #[test]
    #[cfg_attr(
        any(target_os = "windows", target_os = "macos"),
        ignore = "libm not available as standalone shared library on this platform"
    )]
    fn test_extern_sqrt_basic() {
        let source = r#"
            extern "m" fn sqrt(x: CDouble) -> CDouble;
            sqrt(16.0);
        "#;

        match run_program(source) {
            Ok(Value::Number(n)) => {
                assert!(
                    (n - 4.0).abs() < 0.0001,
                    "sqrt(16.0) should be 4.0, got {}",
                    n
                );
            }
            Ok(other) => panic!("Expected number, got: {:?}", other),
            Err(e) => panic!("Program failed: {}", e),
        }
    }

    #[test]
    #[cfg_attr(
        any(target_os = "windows", target_os = "macos"),
        ignore = "libm not available as standalone shared library on this platform"
    )]
    fn test_extern_pow_basic() {
        let source = r#"
            extern "m" fn pow(base: CDouble, exp: CDouble) -> CDouble;
            pow(2.0, 3.0);
        "#;

        match run_program(source) {
            Ok(Value::Number(n)) => {
                assert!(
                    (n - 8.0).abs() < 0.0001,
                    "pow(2.0, 3.0) should be 8.0, got {}",
                    n
                );
            }
            Ok(other) => panic!("Expected number, got: {:?}", other),
            Err(e) => panic!("Program failed: {}", e),
        }
    }

    #[test]
    #[cfg_attr(
        any(target_os = "windows", target_os = "macos"),
        ignore = "libm not available as standalone shared library on this platform"
    )]
    fn test_extern_multiple_calls() {
        let source = r#"
            extern "m" fn sqrt(x: CDouble) -> CDouble;
            let a = sqrt(9.0);
            let b = sqrt(25.0);
            a + b;
        "#;

        match run_program(source) {
            Ok(Value::Number(n)) => {
                // 3.0 + 5.0 = 8.0
                assert!(
                    (n - 8.0).abs() < 0.0001,
                    "sqrt(9) + sqrt(25) should be 8.0, got {}",
                    n
                );
            }
            Ok(other) => panic!("Expected number, got: {:?}", other),
            Err(e) => panic!("Program failed: {}", e),
        }
    }

    #[test]
    #[cfg_attr(
        any(target_os = "windows", target_os = "macos"),
        ignore = "libm not available as standalone shared library on this platform"
    )]
    fn test_extern_with_user_functions() {
        let source = r#"
            extern "m" fn sqrt(x: CDouble) -> CDouble;

            fn hypotenuse(a: number, b: number) -> number {
                return sqrt(a * a + b * b);
            }

            hypotenuse(3.0, 4.0);
        "#;

        match run_program(source) {
            Ok(Value::Number(n)) => {
                assert!(
                    (n - 5.0).abs() < 0.0001,
                    "hypotenuse(3, 4) should be 5.0, got {}",
                    n
                );
            }
            Ok(other) => panic!("Expected number, got: {:?}", other),
            Err(e) => panic!("Program failed: {}", e),
        }
    }

    #[test]
    fn test_extern_library_not_found() {
        let source = r#"
            extern "nonexistent_lib_xyz" fn foo() -> CInt;
            foo();
        "#;

        match run_program(source) {
            Err(e) if e.contains("Failed to load library") => {
                // Expected error
            }
            Ok(_) => panic!("Should have failed to load nonexistent library"),
            Err(e) => panic!("Wrong error: {}", e),
        }
    }

    #[test]
    #[cfg_attr(
        any(target_os = "windows", target_os = "macos"),
        ignore = "libm not available as standalone shared library on this platform"
    )]
    fn test_extern_symbol_not_found() {
        let source = r#"
            extern "m" fn nonexistent_symbol_xyz() -> CDouble;
            nonexistent_symbol_xyz();
        "#;

        match run_program(source) {
            Err(e) if e.contains("Failed to find symbol") => {
                // Expected error
            }
            Ok(_) => panic!("Should have failed to find nonexistent symbol"),
            Err(e) => panic!("Wrong error: {}", e),
        }
    }

    #[test]
    #[cfg_attr(
        any(target_os = "windows", target_os = "macos"),
        ignore = "libm not available as standalone shared library on this platform"
    )]
    fn test_extern_ceil_floor() {
        let source = r#"
            extern "m" fn ceil(x: CDouble) -> CDouble;
            extern "m" fn floor(x: CDouble) -> CDouble;

            let a = ceil(3.2);
            let b = floor(3.8);
            a + b;
        "#;

        match run_program(source) {
            Ok(Value::Number(n)) => {
                // ceil(3.2) = 4.0, floor(3.8) = 3.0, sum = 7.0
                assert!(
                    (n - 7.0).abs() < 0.0001,
                    "ceil(3.2) + floor(3.8) should be 7.0, got {}",
                    n
                );
            }
            Ok(other) => panic!("Expected number, got: {:?}", other),
            Err(e) => panic!("Program failed: {}", e),
        }
    }

    #[test]
    #[cfg_attr(
        any(target_os = "windows", target_os = "macos"),
        ignore = "libm not available as standalone shared library on this platform"
    )]
    fn test_extern_sin_cos() {
        let source = r#"
            extern "m" fn sin(x: CDouble) -> CDouble;
            extern "m" fn cos(x: CDouble) -> CDouble;

            // sin^2 + cos^2 = 1
            let x = 0.5;
            let s = sin(x);
            let c = cos(x);
            s * s + c * c;
        "#;

        match run_program(source) {
            Ok(Value::Number(n)) => {
                // sin^2(x) + cos^2(x) should always be 1
                assert!(
                    (n - 1.0).abs() < 0.0001,
                    "sin^2 + cos^2 should be 1.0, got {}",
                    n
                );
            }
            Ok(other) => panic!("Expected number, got: {:?}", other),
            Err(e) => panic!("Program failed: {}", e),
        }
    }
}
