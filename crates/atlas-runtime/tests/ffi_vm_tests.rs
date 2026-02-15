//! Integration tests for FFI VM execution (phase-10b)

use atlas_runtime::compiler::Compiler;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::value::Value;
use atlas_runtime::vm::VM;

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
    target_os = "windows",
    ignore = "libm not available as .dll on Windows"
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
    target_os = "windows",
    ignore = "libm not available as .dll on Windows"
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
    target_os = "windows",
    ignore = "libm not available as .dll on Windows"
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
    target_os = "windows",
    ignore = "libm not available as .dll on Windows"
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
    target_os = "windows",
    ignore = "libm not available as .dll on Windows"
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
    target_os = "windows",
    ignore = "libm not available as .dll on Windows"
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
    target_os = "windows",
    ignore = "libm not available as .dll on Windows"
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
