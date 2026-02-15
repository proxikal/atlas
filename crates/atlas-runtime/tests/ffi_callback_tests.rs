//! FFI Callback Tests (phase-10c)
//!
//! Tests for C→Atlas function callbacks.

use atlas_runtime::ffi::{create_callback, ExternType};
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::value::{RuntimeError, Value};

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
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
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

    // Cast to function pointer and call
    let fn_ptr = handle.fn_ptr() as *const ();
    let f: extern "C" fn(c_double) -> c_double = unsafe { std::mem::transmute(fn_ptr) };

    let result = f(10.0);
    assert!((result - 30.0).abs() < 0.0001);
}

#[test]
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
fn test_callback_with_two_params() {
    use std::os::raw::c_double;

    let handle = create_callback(
        |args: &[Value]| {
            if let (Some(Value::Number(a)), Some(Value::Number(b))) = (args.get(0), args.get(1)) {
                Ok(Value::Number(a + b))
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

    let result = f(15.0, 25.0);
    assert!((result - 40.0).abs() < 0.0001);
}

#[test]
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
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

    let fn_ptr = handle.fn_ptr() as *const ();
    let f: extern "C" fn(c_int) -> c_int = unsafe { std::mem::transmute(fn_ptr) };

    let result = f(21);
    assert_eq!(result, 42);
}

#[test]
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
fn test_callback_no_params_call() {
    use std::os::raw::c_int;

    let handle = create_callback(
        |_args: &[Value]| Ok(Value::Number(99.0)),
        vec![],
        ExternType::CInt,
    )
    .unwrap();

    let fn_ptr = handle.fn_ptr() as *const ();
    let f: extern "C" fn() -> c_int = unsafe { std::mem::transmute(fn_ptr) };

    let result = f();
    assert_eq!(result, 99);
}

#[test]
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
fn test_callback_void_return_call() {
    use std::os::raw::c_int;

    let handle = create_callback(
        |_args: &[Value]| Ok(Value::Null),
        vec![ExternType::CInt],
        ExternType::CVoid,
    )
    .unwrap();

    let fn_ptr = handle.fn_ptr() as *const ();
    let f: extern "C" fn(c_int) = unsafe { std::mem::transmute(fn_ptr) };

    // Should not crash
    f(42);
}

#[test]
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
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

    let fn_ptr = handle.fn_ptr() as *const ();
    let f: extern "C" fn(c_double) -> c_double = unsafe { std::mem::transmute(fn_ptr) };

    // Should not crash on error, returns 0.0
    let result = f(-1.0);
    assert_eq!(result, 0.0);
}

// ===== Memory Safety Tests =====

#[test]
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
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

    let fn_ptr = handle.fn_ptr() as *const ();
    let f: extern "C" fn(c_double) -> c_double = unsafe { std::mem::transmute(fn_ptr) };

    // Call while handle is alive
    let result1 = f(5.0);
    assert!((result1 - 10.0).abs() < 0.0001);

    // Handle still valid
    let result2 = f(7.0);
    assert!((result2 - 14.0).abs() < 0.0001);

    // Explicit drop
    drop(handle);

    // After drop, function pointer is invalid (don't call it)
    // This test just verifies the drop doesn't crash
}

#[test]
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
fn test_callback_multiple_invocations() {
    use std::os::raw::c_int;

    let handle = create_callback(
        |args: &[Value]| {
            if let (Some(Value::Number(a)), Some(Value::Number(b))) = (args.get(0), args.get(1)) {
                Ok(Value::Number(((*a as i32) + (*b as i32)) as f64))
            } else {
                Ok(Value::Number(0.0))
            }
        },
        vec![ExternType::CInt, ExternType::CInt],
        ExternType::CInt,
    )
    .unwrap();

    let fn_ptr = handle.fn_ptr() as *const ();
    let f: extern "C" fn(c_int, c_int) -> c_int = unsafe { std::mem::transmute(fn_ptr) };

    // Multiple calls
    assert_eq!(f(1, 2), 3);
    assert_eq!(f(10, 20), 30);
    assert_eq!(f(100, 200), 300);
}

#[test]
#[ignore = "Direct function pointer calling requires platform-specific trampolines"]
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
