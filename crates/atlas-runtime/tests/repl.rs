// repl.rs — REPL integration tests
// Covers: state persistence, type tracking

use atlas_runtime::repl::ReplCore;
use atlas_runtime::types::Type;
use atlas_runtime::Value;
use rstest::rstest;

// --- REPL state persistence ---

// Modern REPL State Tests
//
// Converted from repl_state_tests.rs (373 lines → ~240 lines = 36% reduction)

fn eval_ok(repl: &mut ReplCore, input: &str) -> Value {
    let result = repl.eval_line(input);
    if !result.diagnostics.is_empty() {
        panic!(
            "Expected success for '{}'\nGot: {:?}",
            input, result.diagnostics
        );
    }
    result.value.unwrap_or(Value::Null)
}

fn eval_err(repl: &mut ReplCore, input: &str) {
    let result = repl.eval_line(input);
    if result.diagnostics.is_empty() {
        panic!("Expected error for '{}', but succeeded", input);
    }
}

fn assert_value(repl: &mut ReplCore, expr: &str, expected: Value) {
    let value = eval_ok(repl, expr);
    assert_eq!(value, expected, "Expression '{}' failed", expr);
}

// ============================================================================
// Variable Persistence
// ============================================================================

#[test]
fn test_variable_persistence() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let x = 42;");
    assert_value(&mut repl, "x;", Value::Number(42.0));
    assert_value(&mut repl, "x + 8;", Value::Number(50.0));
}

#[test]
fn test_mutable_variable_reassignment() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "var count = 0;");
    eval_ok(&mut repl, "count = count + 1;");
    assert_value(&mut repl, "count;", Value::Number(1.0));
    eval_ok(&mut repl, "count = count + 10;");
    assert_value(&mut repl, "count;", Value::Number(11.0));
}

#[test]
fn test_multiple_variables_persist() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let a = 1;");
    eval_ok(&mut repl, "let b = 2;");
    eval_ok(&mut repl, "let c = 3;");
    assert_value(&mut repl, "a + b + c;", Value::Number(6.0));
}

// ============================================================================
// Function Persistence
// ============================================================================

#[test]
fn test_function_persistence() {
    let mut repl = ReplCore::new();
    eval_ok(
        &mut repl,
        "fn double(x: number) -> number { return x * 2; }",
    );
    assert_value(&mut repl, "double(21);", Value::Number(42.0));
}

#[test]
fn test_multiple_functions_persist() {
    let mut repl = ReplCore::new();
    eval_ok(
        &mut repl,
        "fn add(a: number, b: number) -> number { return a + b; }",
    );
    eval_ok(
        &mut repl,
        "fn mul(a: number, b: number) -> number { return a * b; }",
    );
    assert_value(&mut repl, "add(5, 3);", Value::Number(8.0));
    assert_value(&mut repl, "mul(6, 7);", Value::Number(42.0));
}

#[test]
fn test_functions_call_other_functions() {
    let mut repl = ReplCore::new();
    eval_ok(
        &mut repl,
        "fn square(x: number) -> number { return x * x; }",
    );
    eval_ok(
        &mut repl,
        "fn sum_of_squares(a: number, b: number) -> number { return square(a) + square(b); }",
    );
    assert_value(&mut repl, "sum_of_squares(3, 4);", Value::Number(25.0));
}

// ============================================================================
// Mixed Variables and Functions
// ============================================================================

#[test]
fn test_functions_use_global_variables() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let multiplier = 10;");
    eval_ok(
        &mut repl,
        "fn scale(x: number) -> number { return x * multiplier; }",
    );
    assert_value(&mut repl, "scale(5);", Value::Number(50.0));
}

#[test]
fn test_variables_use_functions() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "fn getValue() -> number { return 42; }");
    eval_ok(&mut repl, "let result = getValue();");
    assert_value(&mut repl, "result;", Value::Number(42.0));
}

// ============================================================================
// Error Recovery
// ============================================================================

#[test]
fn test_errors_do_not_reset_state() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let x = 100;");
    eval_err(&mut repl, "let y: number = \"bad\";"); // Type error
    assert_value(&mut repl, "x;", Value::Number(100.0)); // x still exists
}

#[test]
fn test_parse_error_does_not_reset_state() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let x = 50;");
    eval_err(&mut repl, "let = ;"); // Parse error
    assert_value(&mut repl, "x;", Value::Number(50.0));
}

#[test]
fn test_runtime_error_does_not_reset_state() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let x = 25;");
    eval_err(&mut repl, "let y = x / 0;"); // Runtime error
    assert_value(&mut repl, "x;", Value::Number(25.0));
}

// ============================================================================
// Redefinition Rules
// ============================================================================

#[test]
fn test_cannot_redeclare_variable() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let x = 1;");
    eval_err(&mut repl, "let x = 2;"); // Should error: already declared
}

#[test]
fn test_cannot_redeclare_function() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "fn foo() -> number { return 1; }");
    eval_err(&mut repl, "fn foo() -> number { return 2; }"); // Should error
}

// ============================================================================
// Complex State Interactions
// ============================================================================

#[test]
fn test_nested_function_calls_with_state() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let base = 10;");
    eval_ok(
        &mut repl,
        "fn add_base(x: number) -> number { return x + base; }",
    );
    eval_ok(
        &mut repl,
        "fn double_and_add(x: number) -> number { return add_base(x * 2); }",
    );
    assert_value(&mut repl, "double_and_add(5);", Value::Number(20.0));
}

#[test]
fn test_repl_can_use_arrays() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let arr = [1, 2, 3];");
    assert_value(&mut repl, "arr[1];", Value::Number(2.0));
    eval_ok(&mut repl, "arr[1] = 42;");
    assert_value(&mut repl, "arr[1];", Value::Number(42.0));
}

#[test]
fn test_multiple_statements_in_one_input() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "let x = 1; let y = 2;");
    assert_value(&mut repl, "x + y;", Value::Number(3.0));
}

#[test]
fn test_recursion_across_repl_inputs() {
    let mut repl = ReplCore::new();
    eval_ok(&mut repl, "fn factorial(n: number) -> number { if (n <= 1) { return 1; } return n * factorial(n - 1); }");
    assert_value(&mut repl, "factorial(5);", Value::Number(120.0));
}

// --- REPL type tracking ---

fn type_name(ty: &Type) -> String {
    ty.display_name()
}

#[rstest(
    input,
    expected,
    case("1 + 1;", "number"),
    case("\"hi\";", "string"),
    case("true;", "bool"),
    case("[1, 2, 3];", "number[]"),
    case("len([1,2,3]) + 1;", "number"),
    case("len([1,2,3]);", "number"),
    case("match 1 { 1 => 2, _ => 0 };", "number"),
    case("let x = 1; x;", "number"),
    case("let msg: string = \"ok\"; msg;", "string"),
    case("let arr = [true, false]; arr;", "bool[]"),
    case("let bools = [true, false]; bools[0];", "bool"),
    case("len(\"hello\");", "number")
)]
fn type_of_expression_matches_expected(input: &str, expected: &str) {
    let repl = ReplCore::new();
    let result = repl.type_of_expression(input);
    assert!(
        result.diagnostics.is_empty(),
        "Diagnostics: {:?}",
        result.diagnostics
    );
    let ty = result.ty.expect("expected type");
    assert_eq!(type_name(&ty), expected);
}

#[rstest(
    input,
    expected_type,
    case("let x = 42;", "number"),
    case("let name = \"atlas\";", "string"),
    case("var flag = true;", "bool"),
    case("let data = [1,2,3];", "number[]"),
    case("var nothing = null;", "null"),
    case("let combo = [\"a\", \"b\"];", "string[]"),
    case("var result = len(\"abc\");", "number"),
    case("let nested = [[1,2],[3,4]];", "number[][]"),
    case("var vector = [1, 2, 3];", "number[]")
)]
fn let_binding_records_type(input: &str, expected_type: &str) {
    let mut repl = ReplCore::new();
    let result = repl.eval_line(input);
    assert!(
        result.diagnostics.is_empty(),
        "Diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        !result.bindings.is_empty(),
        "expected binding info for {input}"
    );
    let binding = result.bindings.first().unwrap();
    assert!(!binding.name.starts_with("var"));
    assert_eq!(binding.ty.display_name(), expected_type);
}

#[rstest(
    commands,
    expected_names,
    case(vec!["let a = 1;", "let b = 2;"], vec!["a", "b"]),
    case(vec!["let z = 0;", "let y = z + 1;", "let x = y + z;"], vec!["x", "y", "z"]),
    case(vec!["var list = [1,2];", "list = [3,4];"], vec!["list"]),
    case(vec!["let msg = \"hi\";", "let num = 7;"], vec!["msg", "num"]),
    case(vec!["let first = true;", "let second = false;", "let third = first && second;"], vec!["first", "second", "third"])
)]
fn vars_snapshot_sorted(commands: Vec<&str>, expected_names: Vec<&str>) {
    let mut repl = ReplCore::new();
    for cmd in commands {
        let res = repl.eval_line(cmd);
        assert!(
            res.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            res.diagnostics
        );
    }

    let vars = repl.variables();
    let names: Vec<String> = vars.iter().map(|v| v.name.clone()).collect();
    let mut expected: Vec<String> = expected_names.iter().map(|s| s.to_string()).collect();
    expected.sort();
    assert_eq!(names, expected);
}

#[rstest(
    input,
    case("1 + \"a\";"),
    case("let x: number = \"no\";"),
    case("fn f(a: number) -> number { return a + \"bad\"; };"),
    case("let arr: string[] = [1,2];"),
    case("if (1) { let a = 1; };"),
    case("while (\"no\") { let a = 1; };"),
    case("let x = true; x + 1;"),
    case("match true { 1 => 2 };")
)]
fn type_errors_surface_in_type_query(input: &str) {
    let repl = ReplCore::new();
    let result = repl.type_of_expression(input);
    assert!(
        !result.diagnostics.is_empty(),
        "Expected diagnostics for input: {input}"
    );
}

#[test]
fn let_binding_captures_value_and_mutability() {
    let mut repl = ReplCore::new();
    let res = repl.eval_line("var counter = 3;");
    assert!(res.diagnostics.is_empty());
    let binding = res.bindings.first().expect("binding");
    assert!(binding.mutable);
    assert_eq!(binding.value.to_string(), "3");
    assert_eq!(binding.ty.display_name(), "number");
}

#[rstest(
    input,
    expected_value,
    expected_type,
    case("let greeting = \"hello\";", "hello", "string"),
    case("let total = 1 + 2 + 3;", "6", "number"),
    case("let nested = [1, 2, 3];", "[1, 2, 3]", "number[]"),
    case("let truthy = !false;", "true", "bool"),
    case("let pair = [\"a\", \"b\"];", "[a, b]", "string[]")
)]
fn bindings_capture_display_value(input: &str, expected_value: &str, expected_type: &str) {
    let mut repl = ReplCore::new();
    let res = repl.eval_line(input);
    assert!(res.diagnostics.is_empty());
    let binding = res.bindings.first().unwrap();
    assert_eq!(binding.value.to_string(), expected_value);
    assert_eq!(binding.ty.display_name(), expected_type);
}
