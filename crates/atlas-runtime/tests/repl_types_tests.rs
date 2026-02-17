use atlas_runtime::repl::{ReplCore};
use atlas_runtime::types::Type;
use rstest::rstest;

fn type_name(ty: &Type) -> String {
    ty.display_name()
}

#[rstest(input, expected,
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

#[rstest(input, expected_type,
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
    assert!(result.diagnostics.is_empty(), "Diagnostics: {:?}", result.diagnostics);
    assert!(
        !result.bindings.is_empty(),
        "expected binding info for {input}"
    );
    let binding = result.bindings.first().unwrap();
    assert_eq!(binding.name.starts_with("var"), false);
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
        assert!(res.diagnostics.is_empty(), "Diagnostics: {:?}", res.diagnostics);
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
