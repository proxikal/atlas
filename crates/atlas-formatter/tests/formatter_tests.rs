//! Formatter tests - 70+ tests for code formatting

use atlas_formatter::{
    check_formatted, format_source, format_source_with_config, FormatConfig, FormatResult,
};
use pretty_assertions::assert_eq;
use rstest::rstest;

fn fmt(source: &str) -> String {
    match format_source(source) {
        FormatResult::Ok(s) => s,
        FormatResult::ParseError(e) => panic!("Parse error: {:?}", e),
    }
}

fn fmt_with(source: &str, config: &FormatConfig) -> String {
    match format_source_with_config(source, config) {
        FormatResult::Ok(s) => s,
        FormatResult::ParseError(e) => panic!("Parse error: {:?}", e),
    }
}

// === Basic Statement Formatting ===

#[test]
fn test_let_declaration() {
    assert_eq!(fmt("let x = 5;"), "let x = 5;\n");
}

#[test]
fn test_var_declaration() {
    assert_eq!(fmt("var x = 10;"), "var x = 10;\n");
}

#[test]
fn test_let_with_type() {
    assert_eq!(fmt("let x: number = 42;"), "let x: number = 42;\n");
}

#[test]
fn test_string_literal() {
    assert_eq!(
        fmt("let s = \"hello world\";"),
        "let s = \"hello world\";\n"
    );
}

#[test]
fn test_boolean_literals() {
    assert_eq!(
        fmt("let a = true;\nlet b = false;"),
        "let a = true;\nlet b = false;\n"
    );
}

#[test]
fn test_null_literal() {
    assert_eq!(fmt("let n = null;"), "let n = null;\n");
}

#[test]
fn test_assignment() {
    assert_eq!(fmt("x = 42;"), "x = 42;\n");
}

#[test]
fn test_compound_assignment_add() {
    assert_eq!(fmt("x += 1;"), "x += 1;\n");
}

#[test]
fn test_compound_assignment_sub() {
    assert_eq!(fmt("x -= 5;"), "x -= 5;\n");
}

#[test]
fn test_compound_assignment_mul() {
    assert_eq!(fmt("x *= 2;"), "x *= 2;\n");
}

#[test]
fn test_compound_assignment_div() {
    assert_eq!(fmt("x /= 3;"), "x /= 3;\n");
}

#[test]
fn test_compound_assignment_mod() {
    assert_eq!(fmt("x %= 4;"), "x %= 4;\n");
}

#[test]
fn test_increment() {
    assert_eq!(fmt("x++;"), "x++;\n");
}

#[test]
fn test_decrement() {
    assert_eq!(fmt("x--;"), "x--;\n");
}

#[test]
fn test_break_statement() {
    assert_eq!(
        fmt("while (true) { break; }"),
        "while (true) {\n    break;\n}\n"
    );
}

#[test]
fn test_continue_statement() {
    assert_eq!(
        fmt("while (true) { continue; }"),
        "while (true) {\n    continue;\n}\n"
    );
}

#[test]
fn test_return_void() {
    assert_eq!(fmt("fn foo() { return; }"), "fn foo() {\n    return;\n}\n");
}

#[test]
fn test_return_value() {
    assert_eq!(
        fmt("fn foo() -> number { return 42; }"),
        "fn foo() -> number {\n    return 42;\n}\n"
    );
}

// === Function Declarations ===

#[test]
fn test_simple_function() {
    assert_eq!(
        fmt("fn hello() { print(\"hi\"); }"),
        "fn hello() {\n    print(\"hi\");\n}\n"
    );
}

#[test]
fn test_function_with_params() {
    assert_eq!(
        fmt("fn add(a: number, b: number) -> number { return a + b; }"),
        "fn add(a: number, b: number) -> number {\n    return a + b;\n}\n"
    );
}

#[test]
fn test_function_no_return_type() {
    assert_eq!(
        fmt("fn greet(name: string) { print(name); }"),
        "fn greet(name: string) {\n    print(name);\n}\n"
    );
}

#[test]
fn test_function_with_type_params() {
    assert_eq!(
        fmt("fn identity<T>(x: T) -> T { return x; }"),
        "fn identity<T>(x: T) -> T {\n    return x;\n}\n"
    );
}

#[test]
fn test_empty_function() {
    assert_eq!(fmt("fn noop() {}"), "fn noop() {}\n");
}

// === If Statements ===

#[test]
fn test_if_simple() {
    assert_eq!(
        fmt("if (x > 0) { print(x); }"),
        "if (x > 0) {\n    print(x);\n}\n"
    );
}

#[test]
fn test_if_else() {
    assert_eq!(
        fmt("if (x > 0) { print(\"pos\"); } else { print(\"neg\"); }"),
        "if (x > 0) {\n    print(\"pos\");\n} else {\n    print(\"neg\");\n}\n"
    );
}

#[test]
fn test_nested_if() {
    assert_eq!(
        fmt("if (a) { if (b) { print(1); } }"),
        "if (a) {\n    if (b) {\n        print(1);\n    }\n}\n"
    );
}

// === Loop Formatting ===

#[test]
fn test_while_loop() {
    assert_eq!(
        fmt("while (x > 0) { x -= 1; }"),
        "while (x > 0) {\n    x -= 1;\n}\n"
    );
}

#[test]
fn test_for_loop() {
    assert_eq!(
        fmt("for (let i = 0; i < 10; i++) { print(i); }"),
        "for (let i = 0; i < 10; i++) {\n    print(i);\n}\n"
    );
}

#[test]
fn test_for_in_loop() {
    assert_eq!(
        fmt("for item in items { print(item); }"),
        "for item in items {\n    print(item);\n}\n"
    );
}

// === Expression Formatting ===

#[test]
fn test_binary_operators() {
    assert_eq!(fmt("let r = a + b * c;"), "let r = a + b * c;\n");
}

#[test]
fn test_comparison_operators() {
    assert_eq!(fmt("let r = a == b;"), "let r = a == b;\n");
}

#[test]
fn test_logical_operators() {
    assert_eq!(fmt("let r = a && b || c;"), "let r = a && b || c;\n");
}

#[test]
fn test_unary_negate() {
    assert_eq!(fmt("let x = -5;"), "let x = -5;\n");
}

#[test]
fn test_unary_not() {
    assert_eq!(fmt("let x = !true;"), "let x = !true;\n");
}

#[test]
fn test_grouped_expression() {
    assert_eq!(fmt("let x = (a + b) * c;"), "let x = (a + b) * c;\n");
}

#[test]
fn test_function_call() {
    assert_eq!(fmt("print(\"hello\");"), "print(\"hello\");\n");
}

#[test]
fn test_function_call_multiple_args() {
    assert_eq!(fmt("add(1, 2, 3);"), "add(1, 2, 3);\n");
}

#[test]
fn test_index_expression() {
    assert_eq!(fmt("let x = arr[0];"), "let x = arr[0];\n");
}

#[test]
fn test_member_access() {
    assert_eq!(fmt("let x = obj.field;"), "let x = obj.field;\n");
}

#[test]
fn test_method_call() {
    assert_eq!(fmt("arr.push(42);"), "arr.push(42);\n");
}

#[test]
fn test_method_chain() {
    assert_eq!(fmt("arr.push(1).push(2);"), "arr.push(1).push(2);\n");
}

#[test]
fn test_try_expression() {
    assert_eq!(
        fmt("fn foo() { let x = bar()?; }"),
        "fn foo() {\n    let x = bar()?;\n}\n"
    );
}

// === Array Literal Formatting ===

#[test]
fn test_empty_array() {
    assert_eq!(fmt("let a = [];"), "let a = [];\n");
}

#[test]
fn test_array_literal() {
    assert_eq!(fmt("let a = [1, 2, 3];"), "let a = [1, 2, 3];\n");
}

#[test]
fn test_nested_array() {
    assert_eq!(
        fmt("let a = [[1, 2], [3, 4]];"),
        "let a = [[1, 2], [3, 4]];\n"
    );
}

// === Match Expression ===

#[test]
fn test_match_expression() {
    assert_eq!(
        fmt("let r = match x { 1 => \"one\", 2 => \"two\", _ => \"other\", };"),
        "let r = match x {\n    1 => \"one\",\n    2 => \"two\",\n    _ => \"other\",\n};\n"
    );
}

#[test]
fn test_match_constructor_patterns() {
    assert_eq!(
        fmt("let r = match x { Ok(v) => v, Err(e) => 0, };"),
        "let r = match x {\n    Ok(v) => v,\n    Err(e) => 0,\n};\n"
    );
}

// === Indentation Configuration ===

#[test]
fn test_indent_2_spaces() {
    let config = FormatConfig::default().with_indent_size(2);
    assert_eq!(
        fmt_with("if (true) { print(1); }", &config),
        "if (true) {\n  print(1);\n}\n"
    );
}

#[test]
fn test_indent_8_spaces() {
    let config = FormatConfig::default().with_indent_size(8);
    assert_eq!(
        fmt_with("if (true) { print(1); }", &config),
        "if (true) {\n        print(1);\n}\n"
    );
}

// === Trailing Commas ===

#[test]
fn test_no_trailing_commas() {
    let config = FormatConfig::default().with_trailing_commas(false);
    let result = fmt_with("let r = match x { 1 => \"a\", _ => \"b\", };", &config);
    assert!(result.contains("\"a\",\n"));
}

// === Line Breaking ===

#[test]
fn test_long_function_params_break() {
    let config = FormatConfig::default().with_max_width(40);
    let result = fmt_with(
        "fn long_function_name(first_parameter: string, second_parameter: number) {}",
        &config,
    );
    assert!(result.contains('\n'));
    assert!(result.contains("first_parameter"));
    assert!(result.contains("second_parameter"));
}

#[test]
fn test_short_params_no_break() {
    assert_eq!(fmt("fn f(a: number) {}"), "fn f(a: number) {}\n");
}

// === Import/Export ===

#[test]
fn test_import_named() {
    assert_eq!(
        fmt("import { foo, bar } from \"./mod\";"),
        "import { foo, bar } from \"./mod\";\n"
    );
}

#[test]
fn test_import_namespace() {
    assert_eq!(
        fmt("import * as ns from \"./lib\";"),
        "import * as ns from \"./lib\";\n"
    );
}

#[test]
fn test_export_function() {
    assert_eq!(
        fmt("export fn add(a: number, b: number) -> number { return a + b; }"),
        "export fn add(a: number, b: number) -> number {\n    return a + b;\n}\n"
    );
}

#[test]
fn test_export_variable() {
    assert_eq!(fmt("export let VERSION = 1;"), "export let VERSION = 1;\n");
}

// === Spacing Between Items ===

#[test]
fn test_blank_line_between_functions() {
    let result = fmt("fn a() {}\nfn b() {}");
    assert!(
        result.contains("}\n\nfn b"),
        "Should have blank line between functions, got:\n{}",
        result
    );
}

#[test]
fn test_no_extra_blank_lines_between_statements() {
    assert_eq!(fmt("let x = 1;\nlet y = 2;"), "let x = 1;\nlet y = 2;\n");
}

// === Empty Input ===

#[test]
fn test_empty_source() {
    assert_eq!(fmt(""), "");
}

// === Parse Error Handling ===

#[test]
fn test_parse_error_returns_error() {
    assert!(matches!(
        format_source("let x = ;"),
        FormatResult::ParseError(_)
    ));
}

// === Check Mode ===

#[test]
fn test_check_formatted_already_formatted() {
    assert!(check_formatted("let x = 5;\n"));
}

#[test]
fn test_check_formatted_needs_formatting() {
    assert!(!check_formatted("let x = 5;"));
}

// === Idempotency Tests ===

#[rstest]
#[case("let x = 5;")]
#[case("fn foo(a: number, b: string) -> number { return a; }")]
#[case("if (true) { print(1); } else { print(2); }")]
#[case("while (x > 0) { x -= 1; }")]
#[case("for (let i = 0; i < 10; i++) { print(i); }")]
#[case("let a = [1, 2, 3];")]
#[case("import { foo } from \"./bar\";")]
#[case("let r = match x { 1 => \"one\", _ => \"other\", };")]
fn test_idempotency(#[case] source: &str) {
    let first = fmt(source);
    let second = fmt(&first);
    assert_eq!(
        first, second,
        "Formatting is not idempotent for: {}",
        source
    );
}

#[test]
fn test_idempotency_complex_program() {
    let source = r#"import { math } from "./math";

fn fibonacci(n: number) -> number {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main() {
    let result = fibonacci(10);
    print(result);
}"#;
    let first = fmt(source);
    let second = fmt(&first);
    assert_eq!(first, second);
}

#[test]
fn test_already_formatted_unchanged() {
    let source = "let x = 5;\n";
    assert_eq!(fmt(source), source);
}

// === Various Code Styles Converge ===

#[rstest]
#[case("let   x   =   5;", "let x = 5;\n")]
#[case("let x=5;", "let x = 5;\n")]
fn test_styles_converge(#[case] input: &str, #[case] expected: &str) {
    assert_eq!(fmt(input), expected);
}

// === Complex Nesting ===

#[test]
fn test_deep_nesting() {
    assert_eq!(
        fmt("if (a) { if (b) { if (c) { print(1); } } }"),
        "if (a) {\n    if (b) {\n        if (c) {\n            print(1);\n        }\n    }\n}\n"
    );
}

#[test]
fn test_function_with_loops_and_conditions() {
    let result =
        fmt("fn process(items: string) { for item in items { if (item > 0) { print(item); } } }");
    assert!(result.contains("fn process"));
    assert!(result.contains("    for item in items"));
    assert!(result.contains("        if (item > 0)"));
    assert!(result.contains("            print(item)"));
}

// === Number Formatting ===

#[test]
fn test_integer_formatting() {
    assert!(fmt("let x = 42;").contains("42"));
}

#[test]
fn test_float_formatting() {
    assert!(fmt("let x = 3.14;").contains("3.14"));
}

// === Expression Statement ===

#[test]
fn test_expr_statement() {
    assert_eq!(fmt("foo();"), "foo();\n");
}

// === Formatted Output Parses Successfully ===

#[rstest]
#[case("let x = 5;")]
#[case("fn foo(a: number) -> number { return a + 1; }")]
#[case("if (true) { print(1); } else { print(2); }")]
#[case("for (let i = 0; i < 5; i++) { print(i); }")]
#[case("for item in [1, 2, 3] { print(item); }")]
#[case("let r = match x { 1 => true, _ => false, };")]
fn test_formatted_output_parses(#[case] source: &str) {
    let formatted = fmt(source);
    let mut lexer = atlas_runtime::lexer::Lexer::new(&formatted);
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::parser::Parser::new(tokens);
    let (_, diags) = parser.parse();
    assert!(
        diags.is_empty(),
        "Formatted output should parse without errors, got: {:?}\nFormatted:\n{}",
        diags,
        formatted
    );
}

// === Index Assignment ===

#[test]
fn test_index_assignment() {
    assert_eq!(fmt("arr[0] = 42;"), "arr[0] = 42;\n");
}

// === Multiple Statements ===

#[test]
fn test_multiple_statements() {
    assert_eq!(
        fmt("let x = 1;\nlet y = 2;\nlet z = x + y;"),
        "let x = 1;\nlet y = 2;\nlet z = x + y;\n"
    );
}

// === Additional tests for coverage ===

#[test]
fn test_nested_function_call() {
    assert_eq!(fmt("print(add(1, 2));"), "print(add(1, 2));\n");
}

#[test]
fn test_complex_binary_expression() {
    assert_eq!(
        fmt("let x = a + b - c * d / e % f;"),
        "let x = a + b - c * d / e % f;\n"
    );
}

#[test]
fn test_all_comparison_ops() {
    assert_eq!(fmt("let a = x < y;"), "let a = x < y;\n");
    assert_eq!(fmt("let a = x <= y;"), "let a = x <= y;\n");
    assert_eq!(fmt("let a = x > y;"), "let a = x > y;\n");
    assert_eq!(fmt("let a = x >= y;"), "let a = x >= y;\n");
    assert_eq!(fmt("let a = x != y;"), "let a = x != y;\n");
}

#[test]
fn test_empty_while_body() {
    assert_eq!(fmt("while (true) {}"), "while (true) {}\n");
}

#[test]
fn test_while_with_break() {
    let result = fmt("while (true) { if (done) { break; } }");
    assert!(result.contains("while (true)"));
    assert!(result.contains("if (done)"));
    assert!(result.contains("break;"));
}

#[test]
fn test_multiple_imports() {
    let result = fmt("import { a } from \"./a\";\nimport { b } from \"./b\";");
    assert!(result.contains("import { a }"));
    assert!(result.contains("import { b }"));
}

#[test]
fn test_function_returning_function_type() {
    assert_eq!(
        fmt("fn make() -> (number) -> number { return add; }"),
        "fn make() -> fn(number) -> number {\n    return add;\n}\n"
    );
}

#[test]
fn test_generic_type_annotation() {
    assert_eq!(
        fmt("let x: Result<number, string> = ok(42);"),
        "let x: Result<number, string> = ok(42);\n"
    );
}
