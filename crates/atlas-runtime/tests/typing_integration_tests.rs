use atlas_runtime::repl::ReplCore;
use rstest::rstest;

fn type_name(result: &ReplCore, input: &str) -> (Option<String>, Vec<atlas_runtime::Diagnostic>) {
    let type_result = result.type_of_expression(input);
    let name = type_result.ty.map(|t| t.display_name());
    (name, type_result.diagnostics)
}

#[rstest(
    input,
    expected,
    case("1 + 2;", "number"),
    case("\"a\" + \"b\";", "string"),
    case("true && false;", "bool"),
    case("[1,2,3];", "number[]"),
    case("let arr = [1,2]; arr[0];", "number"),
    case("len(\"atlas\");", "number"),
    case("let s: string = \"x\"; s;", "string"),
    case("let n: number = 4; n;", "number"),
    case("let flag: bool = true; flag;", "bool"),
    case("match 1 { 1 => 2, _ => 0 };", "number"),
    case("let add = 1 + len([1,2]); add;", "number"),
    case("let val = len([1,2,3]); val;", "number"),
    case("let nested = [[1],[2]]; nested[0];", "number[]"),
    case("let nested = [[1],[2]]; nested[0][0];", "number"),
    case("let maybe = null; maybe;", "null"),
    case("let joined = \"a\" + \"b\"; joined;", "string"),
    case("let num = -1 + 2; num;", "number"),
    case("let cmp = 1 < 2; cmp;", "bool"),
    case("let logical = true || false; logical;", "bool"),
    case("let array_bool = [true, false]; array_bool[1];", "bool")
)]
fn typing_integration_infers_types(input: &str, expected: &str) {
    let repl = ReplCore::new();
    let (ty, diagnostics) = type_name(&repl, input);
    assert!(diagnostics.is_empty(), "Diagnostics: {:?}", diagnostics);
    assert_eq!(ty.expect("type"), expected);
}

#[rstest(
    input,
    case("1 + \"a\";"),
    case("if (1) { 1; }"),
    case("let check: bool = 1;"),
    case("let x: string = 1;"),
    case("let arr: number[] = [1, \"b\"];"),
    case("var flag: bool = 2;"),
    case("fn add(a: number, b: number) -> number { return \"x\"; };"),
    case("match true { 1 => 2 };"),
    case("let mismatch: number = true;"),
    case("while (\"no\") { let a = 1; };"),
    case("return 1;"),
    case("break;"),
    case("continue;"),
    case("let x = [1]; x[\"0\"];"),
    case("let x = true; x + 1;"),
    case("let x = [1,2]; x + 1;"),
    case("if (true) { let x: number = \"bad\"; };"),
    case("let s: string = len([1,2]);"),
    case("var arr = [1,2]; arr[0] = \"x\";"),
    case("let bools: bool[] = [true, 1];")
)]
fn typing_integration_reports_errors(input: &str) {
    let repl = ReplCore::new();
    let result = repl.type_of_expression(input);
    assert!(
        !result.diagnostics.is_empty(),
        "Expected diagnostics for input: {input}"
    );
}

#[rstest(
    input,
    case("let x = 1; let y = x + 2; y;"),
    case("let s = \"a\"; let t = s + \"b\"; t;"),
    case("let arr = [1,2]; arr[1];"),
    case("var n = 0; n = n + 1; n;"),
    case("let cond = true && false; cond;"),
    case("let cmp = 2 > 1; cmp;"),
    case("let nested = [[1,2], [3,4]]; nested[1][0];"),
    case("let lenVal = len(\"abc\"); lenVal;"),
    case("let square = 2 * 2; square;"),
    case("let mix = [1,2,3]; len(mix);"),
    case("let bools = [true, false]; bools[0];"),
    case("let mutableArr = [1,2]; mutableArr[0] = 3; mutableArr[0];"),
    case("let math = (1 + 2) * 3; math;"),
    case("var assign = 1; assign = assign + 1; assign;"),
    case("let zero = 0; let check = zero == 0; check;"),
    case("let arr = [1,2,3]; let idx = 1; arr[idx];"),
    case("let s = \"hi\"; let l = len(s); l;"),
    case("let sum = [1,2]; sum[0] + sum[1];"),
    case("let arr = [true]; len(arr);"),
    case("let chain = len([1,2]) + len(\"hi\"); chain;")
)]
fn typing_integration_regressions_remain_valid(input: &str) {
    let mut repl = ReplCore::new();
    let result = repl.eval_line(input);
    assert!(
        result.diagnostics.is_empty(),
        "Diagnostics: {:?}",
        result.diagnostics
    );
}
