//! pattern_matching.rs — Comprehensive integration tests for match / Option / Result
//!
//! Tests runtime correctness across both interpreter and VM engines.
//! All tests verify parity: identical output from interpreter and bytecode VM.

mod common;

use atlas_runtime::binder::Binder;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::value::Value;
use atlas_runtime::vm::VM;
use pretty_assertions::assert_eq;

// ============================================================================
// Helpers
// ============================================================================

fn interp_eval(source: &str) -> Value {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    // Run binder and typechecker for consistency with VM
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);

    let mut interpreter = Interpreter::new();
    interpreter
        .eval(&program, &SecurityContext::allow_all())
        .expect("Interpreter failed")
}

fn vm_eval(source: &str) -> Option<Value> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    // Run binder and typechecker (required for proper compilation)
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).expect("Compilation failed");
    let mut vm = VM::new(bytecode);
    vm.run(&SecurityContext::allow_all()).expect("VM failed")
}

/// Assert both engines produce identical results.
fn assert_parity(source: &str) {
    let interp = interp_eval(source);
    let vm = vm_eval(source).unwrap_or(Value::Null);
    assert_eq!(
        interp, vm,
        "Parity mismatch:\n{}\nInterp: {:?}\nVM:     {:?}",
        source, interp, vm
    );
}

/// Assert both engines produce a specific number.
fn assert_parity_number(source: &str, expected: f64) {
    let interp = interp_eval(source);
    let vm = vm_eval(source).unwrap_or(Value::Null);
    assert_eq!(
        interp,
        Value::Number(expected),
        "Interpreter wrong: {:?}",
        interp
    );
    assert_eq!(vm, Value::Number(expected), "VM wrong: {:?}", vm);
}

/// Assert both engines produce a specific string.
fn assert_parity_string(source: &str, expected: &str) {
    let interp = interp_eval(source);
    let vm = vm_eval(source).unwrap_or(Value::Null);
    assert_eq!(
        interp,
        Value::string(expected.to_string()),
        "Interpreter wrong: {:?}",
        interp
    );
    assert_eq!(
        vm,
        Value::string(expected.to_string()),
        "VM wrong: {:?}",
        vm
    );
}

/// Assert both engines produce a specific bool.
fn assert_parity_bool(source: &str, expected: bool) {
    let interp = interp_eval(source);
    let vm = vm_eval(source).unwrap_or(Value::Null);
    assert_eq!(
        interp,
        Value::Bool(expected),
        "Interpreter wrong: {:?}",
        interp
    );
    assert_eq!(vm, Value::Bool(expected), "VM wrong: {:?}", vm);
}

fn typecheck(source: &str) -> (bool, Vec<String>) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return (false, lex_diags.iter().map(|d| d.message.clone()).collect());
    }
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        return (
            false,
            parse_diags.iter().map(|d| d.message.clone()).collect(),
        );
    }
    let mut binder = Binder::new();
    let (mut symbol_table, bind_diags) = binder.bind(&program);
    if !bind_diags.is_empty() {
        return (
            false,
            bind_diags.iter().map(|d| d.message.clone()).collect(),
        );
    }
    let mut checker = TypeChecker::new(&mut symbol_table);
    let diags = checker.check(&program);
    let messages: Vec<String> = diags.iter().map(|d| d.message.clone()).collect();
    (diags.is_empty(), messages)
}

// ============================================================================
// Literal Patterns
// ============================================================================

#[test]
fn test_literal_number_match() {
    assert_parity_string(
        r#"fn run() -> string {
            return match 42 {
                42 => "yes",
                _ => "no"
            };
        }
        run();"#,
        "yes",
    );
}

#[test]
fn test_literal_number_no_match_falls_to_wildcard() {
    assert_parity_string(
        r#"fn run() -> string {
            return match 99 {
                1 => "one",
                2 => "two",
                _ => "other"
            };
        }
        run();"#,
        "other",
    );
}

#[test]
fn test_literal_string_match() {
    assert_parity_number(
        r#"fn run() -> number {
            return match "hello" {
                "hello" => 1,
                _ => 0
            };
        }
        run();"#,
        1.0,
    );
}

#[test]
fn test_literal_bool_true() {
    assert_parity_string(
        r#"fn run() -> string {
            return match true {
                true => "yes",
                false => "no"
            };
        }
        run();"#,
        "yes",
    );
}

#[test]
fn test_literal_bool_false() {
    assert_parity_string(
        r#"fn run() -> string {
            return match false {
                true => "yes",
                false => "no"
            };
        }
        run();"#,
        "no",
    );
}

#[test]
fn test_literal_null_match() {
    assert_parity_string(
        r#"fn run() -> string {
            return match null {
                null => "null",
                _ => "not null"
            };
        }
        run();"#,
        "null",
    );
}

#[test]
fn test_literal_multiple_arms_first_wins() {
    // When scrutinee matches multiple arms, first arm wins
    assert_parity_string(
        r#"fn run() -> string {
            return match 5 {
                5 => "first",
                _ => "second"
            };
        }
        run();"#,
        "first",
    );
}

#[test]
fn test_literal_match_exhaustive_via_wildcard() {
    // Wildcard makes any match exhaustive — type checker should accept
    let (success, msgs) = typecheck(
        r#"fn run(x: number) -> string {
            return match x {
                1 => "one",
                _ => "other"
            };
        }"#,
    );
    assert!(success, "Should type check: {:?}", msgs);
}

// ============================================================================
// Variable Binding Patterns
// ============================================================================

#[test]
fn test_variable_binding_basic() {
    assert_parity_number(
        r#"fn run() -> number {
            return match 5 {
                x => x + 1
            };
        }
        run();"#,
        6.0,
    );
}

#[test]
fn test_variable_binding_scoped_to_arm() {
    // Variable bound in arm should not leak to outer scope
    assert_parity_number(
        r#"let outer: number = 100;
        fn run() -> number {
            match 42 { x => x + 1 };
            return outer;
        }
        run();"#,
        100.0,
    );
}

#[test]
fn test_wildcard_matches_anything_no_binding() {
    assert_parity_string(
        r#"fn run() -> string {
            return match 999 {
                _ => "matched"
            };
        }
        run();"#,
        "matched",
    );
}

#[test]
fn test_variable_binding_string() {
    assert_parity_string(
        r#"fn run() -> string {
            return match "hello" {
                s => s + " world"
            };
        }
        run();"#,
        "hello world",
    );
}

#[test]
fn test_variable_shadows_outer() {
    // Binding in arm pattern shadows outer var of same name
    assert_parity_number(
        r#"let x: number = 10;
        fn run() -> number {
            return match 42 {
                x => x
            };
        }
        run();"#,
        42.0,
    );
}

#[test]
fn test_variable_binding_combined_with_literals() {
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                0 => "zero",
                x => "got: " + str(x)
            };
        }
        run(7);"#,
        "got: 7",
    );
}

// ============================================================================
// Constructor Patterns — Option
// ============================================================================

#[test]
fn test_option_some_binds_value() {
    assert_parity_number(
        r#"fn run(opt: Option<number>) -> number {
            return match opt {
                Some(x) => x,
                None => 0
            };
        }
        run(Some(42));"#,
        42.0,
    );
}

#[test]
fn test_option_none_arm() {
    assert_parity_number(
        r#"fn run(opt: Option<number>) -> number {
            return match opt {
                Some(x) => x,
                None => 0
            };
        }
        run(None);"#,
        0.0,
    );
}

#[test]
fn test_option_some_with_expression_body() {
    assert_parity_number(
        r#"fn run(opt: Option<number>) -> number {
            return match opt {
                Some(x) => x * 2 + 1,
                None => -1
            };
        }
        run(Some(5));"#,
        11.0,
    );
}

#[test]
fn test_option_nested_some_some() {
    assert_parity_number(
        r#"fn run(opt: Option<Option<number>>) -> number {
            return match opt {
                Some(Some(x)) => x,
                Some(None) => -1,
                None => -2
            };
        }
        run(Some(Some(99)));"#,
        99.0,
    );
}

#[test]
fn test_option_nested_some_none() {
    assert_parity_number(
        r#"fn run(opt: Option<Option<number>>) -> number {
            return match opt {
                Some(Some(x)) => x,
                Some(None) => -1,
                None => -2
            };
        }
        run(Some(None));"#,
        -1.0,
    );
}

#[test]
fn test_option_wildcard_exhaustive() {
    // Wildcard should make Option match exhaustive
    let (success, msgs) = typecheck(
        r#"fn run(opt: Option<number>) -> string {
            return match opt {
                Some(x) => "some",
                _ => "none or other"
            };
        }"#,
    );
    assert!(success, "Wildcard makes Option exhaustive: {:?}", msgs);
}

#[test]
fn test_option_type_bound_value_string() {
    assert_parity_string(
        r#"fn run(opt: Option<string>) -> string {
            return match opt {
                Some(s) => s,
                None => "empty"
            };
        }
        run(Some("hello"));"#,
        "hello",
    );
}

#[test]
fn test_option_some_arm_executes_some_value() {
    assert_parity_bool(
        r#"fn run() -> bool {
            let opt: Option<number> = Some(10);
            return match opt {
                Some(x) => true,
                None => false
            };
        }
        run();"#,
        true,
    );
}

#[test]
fn test_option_none_arm_executes_none_value() {
    assert_parity_bool(
        r#"fn run() -> bool {
            let opt: Option<number> = None;
            return match opt {
                Some(x) => true,
                None => false
            };
        }
        run();"#,
        false,
    );
}

#[test]
fn test_option_as_function_return() {
    assert_parity_string(
        r#"fn maybe_string(flag: bool) -> Option<string> {
            if (flag) {
                return Some("yes");
            }
            return None;
        }
        fn run() -> string {
            return match maybe_string(true) {
                Some(s) => s,
                None => "nothing"
            };
        }
        run();"#,
        "yes",
    );
}

// ============================================================================
// Constructor Patterns — Result
// ============================================================================

#[test]
fn test_result_ok_binds_value() {
    assert_parity_number(
        r#"fn run(res: Result<number, string>) -> number {
            return match res {
                Ok(x) => x,
                Err(e) => -1
            };
        }
        run(Ok(42));"#,
        42.0,
    );
}

#[test]
fn test_result_err_binds_value() {
    assert_parity_string(
        r#"fn run(res: Result<number, string>) -> string {
            return match res {
                Ok(x) => "ok",
                Err(e) => e
            };
        }
        run(Err("oops"));"#,
        "oops",
    );
}

#[test]
fn test_result_ok_nested_ok() {
    assert_parity_number(
        r#"fn run(res: Result<Result<number, string>, string>) -> number {
            return match res {
                Ok(Ok(x)) => x,
                Ok(Err(e)) => -1,
                Err(e) => -2
            };
        }
        run(Ok(Ok(1)));"#,
        1.0,
    );
}

#[test]
fn test_result_wildcard_exhaustive() {
    let (success, msgs) = typecheck(
        r#"fn run(res: Result<number, string>) -> string {
            return match res {
                Ok(x) => "ok",
                _ => "err"
            };
        }"#,
    );
    assert!(success, "Wildcard makes Result exhaustive: {:?}", msgs);
}

#[test]
fn test_result_error_type_number() {
    assert_parity_number(
        r#"fn run(res: Result<string, number>) -> number {
            return match res {
                Ok(s) => 0,
                Err(code) => code
            };
        }
        run(Err(404));"#,
        404.0,
    );
}

#[test]
fn test_result_real_use_case_divide() {
    assert_parity_string(
        r#"fn divide(a: number, b: number) -> Result<number, string> {
            if (b == 0) {
                return Err("division by zero");
            }
            return Ok(a / b);
        }
        fn run() -> string {
            return match divide(10, 2) {
                Ok(x) => str(x),
                Err(e) => e
            };
        }
        run();"#,
        "5",
    );
}

#[test]
fn test_result_error_propagation_pattern() {
    assert_parity_string(
        r#"fn divide(a: number, b: number) -> Result<number, string> {
            if (b == 0) {
                return Err("div by zero");
            }
            return Ok(a / b);
        }
        fn run() -> string {
            return match divide(10, 0) {
                Ok(x) => str(x),
                Err(e) => "Error: " + e
            };
        }
        run();"#,
        "Error: div by zero",
    );
}

#[test]
fn test_result_chained_match() {
    assert_parity_number(
        r#"fn run() -> number {
            let r1: Result<number, string> = Ok(5);
            let r2: Result<number, string> = Ok(3);
            let a: number = match r1 { Ok(x) => x, Err(e) => 0 };
            let b: number = match r2 { Ok(x) => x, Err(e) => 0 };
            return a + b;
        }
        run();"#,
        8.0,
    );
}

#[test]
fn test_result_as_function_return() {
    assert_parity_number(
        r#"fn safe_sqrt(x: number) -> Result<number, string> {
            if (x < 0) {
                return Err("negative");
            }
            return Ok(x * x);
        }
        fn run() -> number {
            return match safe_sqrt(4) {
                Ok(val) => val,
                Err(e) => -1
            };
        }
        run();"#,
        16.0,
    );
}

#[test]
fn test_result_ok_err_type_any() {
    // Both Ok and Err can hold any type
    assert_parity_string(
        r#"fn run(res: Result<bool, number>) -> string {
            return match res {
                Ok(b) => str(b),
                Err(n) => str(n)
            };
        }
        run(Ok(true));"#,
        "true",
    );
}

// ============================================================================
// Array Patterns
// ============================================================================

#[test]
fn test_array_empty_pattern() {
    assert_parity_string(
        r#"fn run(arr: number[]) -> string {
            return match arr {
                [] => "empty",
                _ => "not empty"
            };
        }
        run([]);"#,
        "empty",
    );
}

#[test]
fn test_array_single_element_pattern() {
    assert_parity_number(
        r#"fn run(arr: number[]) -> number {
            return match arr {
                [] => 0,
                [x] => x,
                _ => -1
            };
        }
        run([42]);"#,
        42.0,
    );
}

#[test]
fn test_array_two_element_pattern() {
    assert_parity_number(
        r#"fn run(arr: number[]) -> number {
            return match arr {
                [a, b] => a + b,
                _ => 0
            };
        }
        run([3, 7]);"#,
        10.0,
    );
}

#[test]
fn test_array_length_mismatch_tries_next_arm() {
    assert_parity_string(
        r#"fn run(arr: number[]) -> string {
            return match arr {
                [x] => "one",
                [x, y] => "two",
                _ => "other"
            };
        }
        run([1, 2, 3]);"#,
        "other",
    );
}

#[test]
fn test_array_wildcard_element() {
    assert_parity_number(
        r#"fn run(arr: number[]) -> number {
            return match arr {
                [_, x] => x,
                _ => 0
            };
        }
        run([10, 99]);"#,
        99.0,
    );
}

#[test]
fn test_array_empty_vs_nonempty() {
    // Empty array matches [], non-empty doesn't
    assert_parity_bool(
        r#"fn run(arr: number[]) -> bool {
            return match arr {
                [] => true,
                _ => false
            };
        }
        run([1]);"#,
        false,
    );
}

// ============================================================================
// Exhaustiveness and Type Checking
// ============================================================================

#[test]
fn test_arm_type_mismatch_rejected() {
    let (success, msgs) = typecheck(
        r#"fn run(x: number) -> string {
            return match x {
                0 => "zero",
                1 => 123,
                _ => "other"
            };
        }"#,
    );
    assert!(!success, "Should reject mismatched arm types");
    assert!(
        msgs.iter().any(|m| {
            m.contains("type mismatch")
                || m.contains("incompatible")
                || m.contains("Return type mismatch")
        }),
        "Should report arm type mismatch: {:?}",
        msgs
    );
}

#[test]
fn test_match_as_expression_assignable() {
    assert_parity_number(
        r#"fn run() -> number {
            let result: number = match 5 {
                5 => 10,
                _ => 0
            };
            return result;
        }
        run();"#,
        10.0,
    );
}

#[test]
fn test_bool_exhaustive_both_arms() {
    let (success, msgs) = typecheck(
        r#"fn run(b: bool) -> string {
            return match b {
                true => "yes",
                false => "no"
            };
        }"#,
    );
    assert!(success, "true+false should be exhaustive: {:?}", msgs);
}

#[test]
fn test_number_match_requires_wildcard() {
    let (success, msgs) = typecheck(
        r#"fn run(n: number) -> string {
            return match n {
                1 => "one",
                _ => "other"
            };
        }"#,
    );
    assert!(
        success,
        "Number with wildcard should be exhaustive: {:?}",
        msgs
    );
}

#[test]
fn test_string_match_requires_wildcard() {
    let (success, msgs) = typecheck(
        r#"fn run(s: string) -> number {
            return match s {
                "a" => 1,
                _ => 0
            };
        }"#,
    );
    assert!(
        success,
        "String with wildcard should be exhaustive: {:?}",
        msgs
    );
}

#[test]
fn test_option_missing_none_rejected() {
    let (success, msgs) = typecheck(
        r#"fn run(opt: Option<number>) -> string {
            return match opt {
                Some(x) => "some"
            };
        }"#,
    );
    assert!(
        !success,
        "Should reject non-exhaustive Option (missing None)"
    );
    assert!(
        msgs.iter()
            .any(|m| m.contains("Non-exhaustive") || m.contains("None")),
        "Should mention missing None: {:?}",
        msgs
    );
}

#[test]
fn test_option_missing_some_rejected() {
    let (success, msgs) = typecheck(
        r#"fn run(opt: Option<number>) -> string {
            return match opt {
                None => "none"
            };
        }"#,
    );
    assert!(
        !success,
        "Should reject non-exhaustive Option (missing Some)"
    );
    assert!(
        msgs.iter()
            .any(|m| m.contains("Non-exhaustive") || m.contains("Some")),
        "Should mention missing Some: {:?}",
        msgs
    );
}

#[test]
fn test_result_missing_err_rejected() {
    let (success, msgs) = typecheck(
        r#"fn run(res: Result<number, string>) -> string {
            return match res {
                Ok(x) => "ok"
            };
        }"#,
    );
    assert!(
        !success,
        "Should reject non-exhaustive Result (missing Err)"
    );
    assert!(
        msgs.iter()
            .any(|m| m.contains("Non-exhaustive") || m.contains("Err")),
        "Should mention missing Err: {:?}",
        msgs
    );
}

#[test]
fn test_result_missing_ok_rejected() {
    let (success, msgs) = typecheck(
        r#"fn run(res: Result<number, string>) -> string {
            return match res {
                Err(e) => "err"
            };
        }"#,
    );
    assert!(!success, "Should reject non-exhaustive Result (missing Ok)");
    assert!(
        msgs.iter()
            .any(|m| m.contains("Non-exhaustive") || m.contains("Ok")),
        "Should mention missing Ok: {:?}",
        msgs
    );
}

// ============================================================================
// Parity — identical output in both engines
// ============================================================================

#[test]
fn test_parity_simple_literal_match() {
    assert_parity(
        r#"fn run() -> string {
            return match 3 {
                1 => "one",
                2 => "two",
                3 => "three",
                _ => "other"
            };
        }
        run();"#,
    );
}

#[test]
fn test_parity_constructor_option_match() {
    assert_parity(
        r#"fn run(opt: Option<number>) -> number {
            return match opt {
                Some(x) => x * 10,
                None => 0
            };
        }
        run(Some(7));"#,
    );
}

#[test]
fn test_parity_nested_constructor_match() {
    assert_parity(
        r#"fn run(res: Result<Option<number>, string>) -> number {
            return match res {
                Ok(Some(v)) => v,
                Ok(None) => 0,
                Err(e) => -1
            };
        }
        run(Ok(Some(55)));"#,
    );
}

#[test]
fn test_parity_variable_binding_in_arm() {
    assert_parity(
        r#"fn run() -> number {
            return match 42 {
                n => n + 8
            };
        }
        run();"#,
    );
}

#[test]
fn test_parity_match_as_expression_in_computation() {
    assert_parity(
        r#"fn run() -> number {
            let base: number = 10;
            let extra: number = match true {
                true => 5,
                false => 0
            };
            return base + extra;
        }
        run();"#,
    );
}

#[test]
fn test_parity_array_pattern() {
    assert_parity(
        r#"fn run(arr: number[]) -> number {
            return match arr {
                [] => 0,
                [x] => x,
                [a, b] => a + b,
                _ => -1
            };
        }
        run([4, 6]);"#,
    );
}

// ============================================================================
// Real-world Patterns
// ============================================================================

#[test]
fn test_match_multiple_in_same_function() {
    assert_parity_number(
        r#"fn run() -> number {
            let a: Option<number> = Some(3);
            let b: Option<number> = None;
            let x: number = match a {
                Some(v) => v,
                None => 0
            };
            let y: number = match b {
                Some(v) => v,
                None => 10
            };
            return x + y;
        }
        run();"#,
        13.0,
    );
}

#[test]
fn test_match_result_from_user_function() {
    assert_parity_string(
        r#"fn checked_add(a: number, b: number) -> Result<number, string> {
            if (a < 0 || b < 0) {
                return Err("negative input");
            }
            return Ok(a + b);
        }
        fn run() -> string {
            return match checked_add(5, 3) {
                Ok(sum) => "sum=" + str(sum),
                Err(e) => "error: " + e
            };
        }
        run();"#,
        "sum=8",
    );
}

#[test]
fn test_match_in_loop() {
    assert_parity_number(
        r#"fn run() -> number {
            var total: number = 0;
            var i: number = 0;
            while (i < 3) {
                total = total + match i {
                    0 => 1,
                    1 => 10,
                    _ => 100
                };
                i = i + 1;
            }
            return total;
        }
        run();"#,
        111.0,
    );
}

#[test]
fn test_match_nested_option_result() {
    assert_parity_number(
        r#"fn parse(s: string) -> Option<Result<number, string>> {
            if (s == "") {
                return None;
            }
            if (s == "bad") {
                return Some(Err("invalid"));
            }
            return Some(Ok(42));
        }
        fn run() -> number {
            return match parse("ok") {
                Some(Ok(n)) => n,
                Some(Err(e)) => -1,
                None => -2
            };
        }
        run();"#,
        42.0,
    );
}

#[test]
fn debug_direct_some_match() {
    // No function wrapper
    let src = r#"fn run(opt: Option<number>) -> number { return match opt { Some(x) => x, None => 0 }; } run(Some(42));"#;
    let interp = interp_eval(src);
    println!("Interp result: {:?}", interp);
    // Test VM separately, catch panics
    let result = std::panic::catch_unwind(|| vm_eval(src));
    println!("VM result: {:?}", result);
}

#[test]
fn debug_none_value() {
    let src = r#"fn run(opt: Option<number>) -> number { return match opt { Some(x) => x, None => 0 }; } run(None);"#;
    let interp = interp_eval(src);
    println!("Interp result: {:?}", interp);
}

// ============================================================================
// Guard Clauses
// ============================================================================

#[test]
fn test_guard_basic_positive() {
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                x if x > 0 => "pos",
                _ => "non-pos"
            };
        }
        run(5);"#,
        "pos",
    );
}

#[test]
fn test_guard_basic_negative() {
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                x if x > 0 => "pos",
                _ => "non-pos"
            };
        }
        run(-3);"#,
        "non-pos",
    );
}

#[test]
fn test_guard_false_tries_next_arm() {
    // guard fails → falls to wildcard
    assert_parity_number(
        r#"fn run(n: number) -> number {
            return match n {
                x if x > 10 => 1,
                _ => 2
            };
        }
        run(5);"#,
        2.0,
    );
}

#[test]
fn test_guard_uses_bound_variable() {
    assert_parity_number(
        r#"fn run(opt: Option<number>) -> number {
            return match opt {
                Some(x) if x > 10 => x,
                Some(x) => 0,
                None => -1
            };
        }
        run(Some(42));"#,
        42.0,
    );
}

#[test]
fn test_guard_bound_variable_fails() {
    // x = 5, guard fails, second Some arm matches
    assert_parity_number(
        r#"fn run(opt: Option<number>) -> number {
            return match opt {
                Some(x) if x > 10 => x,
                Some(x) => 0,
                None => -1
            };
        }
        run(Some(5));"#,
        0.0,
    );
}

#[test]
fn test_guard_accesses_outer_scope() {
    assert_parity_string(
        r#"fn run(val: number, limit: number) -> string {
            return match val {
                x if x == limit => "hit",
                _ => "miss"
            };
        }
        run(7, 7);"#,
        "hit",
    );
}

#[test]
fn test_guard_multiple_guarded_arms_first_wins() {
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                x if x < 0 => "neg",
                x if x == 0 => "zero",
                _ => "pos"
            };
        }
        run(0);"#,
        "zero",
    );
}

#[test]
fn test_guard_with_boolean_expression() {
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                x if x > 0 && x < 10 => "single digit",
                _ => "other"
            };
        }
        run(7);"#,
        "single digit",
    );
}

#[test]
fn test_guard_boolean_expression_fails() {
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                x if x > 0 && x < 10 => "single digit",
                _ => "other"
            };
        }
        run(15);"#,
        "other",
    );
}

#[test]
fn test_guard_on_result_constructor() {
    assert_parity_number(
        r#"fn run(r: Result<number, string>) -> number {
            return match r {
                Ok(v) if v != 0 => v,
                Ok(_) => 0,
                Err(e) => -1
            };
        }
        run(Ok(42));"#,
        42.0,
    );
}

#[test]
fn test_guard_on_result_constructor_zero() {
    assert_parity_number(
        r#"fn run(r: Result<number, string>) -> number {
            return match r {
                Ok(v) if v != 0 => v,
                Ok(_) => 0,
                Err(e) => -1
            };
        }
        run(Ok(0));"#,
        0.0,
    );
}

#[test]
fn test_guard_does_not_satisfy_exhaustiveness_alone() {
    // Guarded arm does NOT make match exhaustive — wildcard still required
    let (ok, diags) = typecheck(
        r#"fn run(n: number) -> string {
            return match n {
                x if x > 0 => "pos"
            };
        }
        run(1);"#,
    );
    assert!(!ok, "Expected type error for non-exhaustive guarded match");
    let has_exhaustive_error = diags
        .iter()
        .any(|d| d.contains("exhaustive") || d.contains("AT3027"));
    assert!(
        has_exhaustive_error,
        "Expected exhaustiveness error, got: {:?}",
        diags
    );
}

#[test]
fn test_guard_parity_interpreter_vm() {
    assert_parity_number(
        r#"fn run(n: number) -> number {
            return match n {
                x if x > 0 => x * 2,
                _ => 0
            };
        }
        run(5);"#,
        10.0,
    );
}

#[test]
fn test_guard_with_multiple_bound_variables() {
    // Guard with multiple bound variables from array pattern
    assert_parity_string(
        r#"fn run(a: number, b: number) -> string {
            return match [a, b] {
                [x, y] if x < y => "ascending",
                [x, y] => "not ascending",
                _ => "other"
            };
        }
        run(1, 5);"#,
        "ascending",
    );
}

#[test]
fn test_guard_zero_does_not_match() {
    // Explicit test: guard `x > 0` on value 0 should fall through
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                x if x > 0 => "positive",
                _ => "non-positive"
            };
        }
        run(0);"#,
        "non-positive",
    );
}

// ============================================================================
// OR Patterns
// ============================================================================

#[test]
fn test_or_basic_two_literals() {
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                0 | 1 => "small",
                _ => "big"
            };
        }
        run(0);"#,
        "small",
    );
}

#[test]
fn test_or_basic_second_alternative() {
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                0 | 1 => "small",
                _ => "big"
            };
        }
        run(1);"#,
        "small",
    );
}

#[test]
fn test_or_three_way() {
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                0 | 1 | 2 => "tiny",
                _ => "big"
            };
        }
        run(2);"#,
        "tiny",
    );
}

#[test]
fn test_or_does_not_match_falls_to_next_arm() {
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                0 | 1 => "small",
                _ => "big"
            };
        }
        run(5);"#,
        "big",
    );
}

#[test]
fn test_or_string_patterns() {
    assert_parity_bool(
        r#"fn run(s: string) -> bool {
            return match s {
                "yes" | "y" | "true" => true,
                _ => false
            };
        }
        run("y");"#,
        true,
    );
}

#[test]
fn test_or_string_no_match() {
    assert_parity_bool(
        r#"fn run(s: string) -> bool {
            return match s {
                "yes" | "y" | "true" => true,
                _ => false
            };
        }
        run("no");"#,
        false,
    );
}

#[test]
fn test_or_bool_exhaustive() {
    // true | false covers all bools — exhaustive without wildcard
    let (ok, diags) = typecheck(
        r#"fn run(b: bool) -> string {
            return match b {
                true | false => "covered"
            };
        }
        run(true);"#,
    );
    assert!(
        ok,
        "Expected OR bool pattern to be exhaustive, got: {:?}",
        diags
    );
}

#[test]
fn test_or_bool_exhaustive_runtime() {
    assert_parity_string(
        r#"fn run(b: bool) -> string {
            return match b {
                true | false => "covered"
            };
        }
        run(false);"#,
        "covered",
    );
}

#[test]
fn test_or_option_exhaustive() {
    // Some(_) | None covers Option — exhaustive
    let (ok, diags) = typecheck(
        r#"fn run(opt: Option<number>) -> number {
            return match opt {
                Some(_) | None => 0
            };
        }
        run(None);"#,
    );
    assert!(
        ok,
        "Expected Some | None OR to be exhaustive, got: {:?}",
        diags
    );
}

#[test]
fn test_or_result_exhaustive() {
    // Ok(_) | Err(_) covers Result — exhaustive
    let (ok, diags) = typecheck(
        r#"fn run(r: Result<number, string>) -> number {
            return match r {
                Ok(_) | Err(_) => 0
            };
        }
        run(Ok(1));"#,
    );
    assert!(
        ok,
        "Expected Ok | Err OR to be exhaustive, got: {:?}",
        diags
    );
}

#[test]
fn test_or_parity_interpreter_vm() {
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                1 | 2 | 3 => "low",
                4 | 5 | 6 => "mid",
                _ => "high"
            };
        }
        run(5);"#,
        "mid",
    );
}

#[test]
fn test_or_first_match_wins() {
    // First arm has OR that matches; second arm not reached
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                1 | 2 => "first",
                2 | 3 => "second",
                _ => "other"
            };
        }
        run(2);"#,
        "first",
    );
}

#[test]
fn test_or_no_bindings_in_literal_alternatives() {
    // Pure literal OR — no variable binding, just result
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                0 | 1 => "zero or one",
                _ => "other"
            };
        }
        run(1);"#,
        "zero or one",
    );
}

#[test]
fn test_or_in_first_arm_wildcard_second() {
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                10 | 20 => "tens",
                _ => "other"
            };
        }
        run(10);"#,
        "tens",
    );
}

#[test]
fn test_or_with_wildcard_sub_pattern() {
    // `0 | _` — the wildcard sub-pattern always matches
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                0 | _ => "anything",
            };
        }
        run(99);"#,
        "anything",
    );
}

#[test]
fn test_guard_plus_unguarded_wildcard() {
    // Classic: guarded arm then unguarded wildcard
    assert_parity_string(
        r#"fn run(n: number) -> string {
            return match n {
                x if x > 100 => "big",
                _ => "small"
            };
        }
        run(50);"#,
        "small",
    );
}

#[test]
fn test_or_all_arms_same_type() {
    // OR pattern result type must be consistent with rest of match
    assert_parity_number(
        r#"fn run(n: number) -> number {
            return match n {
                0 | 1 | 2 => 99,
                _ => 0
            };
        }
        run(0);"#,
        99.0,
    );
}
