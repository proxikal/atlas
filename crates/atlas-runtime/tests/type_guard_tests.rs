//! Tests for type guard predicates and narrowing.

mod common;

use atlas_runtime::diagnostic::{Diagnostic, DiagnosticLevel};
use atlas_runtime::{Atlas, Binder, Lexer, Parser, TypeChecker, Value};
use rstest::rstest;

fn typecheck(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize_with_comments();
    if !lex_diags.is_empty() {
        return lex_diags;
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        return parse_diags;
    }

    let mut binder = Binder::new();
    let (mut table, mut bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&mut table);
    let mut type_diags = checker.check(&program);

    bind_diags.append(&mut type_diags);
    bind_diags
}

fn errors(source: &str) -> Vec<Diagnostic> {
    typecheck(source)
        .into_iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect()
}

fn eval(code: &str) -> Value {
    let runtime = Atlas::new();
    runtime.eval(code).expect("Interpretation failed")
}

// =============================================================================
// Predicate syntax + validation
// =============================================================================

#[rstest]
#[case(
    r#"
    fn isStr(x: number | string) -> bool is x: string { return isString(x); }
    fn test(x: number | string) -> number {
        if (isStr(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNum(x: number | string) -> bool is x: number { return isNumber(x); }
    fn test(x: number | string) -> number {
        if (isNum(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isBoolish(x: bool | null) -> bool is x: bool { return isBool(x); }
    fn test(x: bool | null) -> bool {
        if (isBoolish(x)) { let _y: bool = x; return _y; }
        else { return false; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn hasName(x: WithName | WithId) -> bool is x: WithName { return hasField(x, "name"); }
    fn test(x: WithName | WithId) -> number {
        if (hasName(x)) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithLen = { len: () -> number };
    type WithId = { id: number };
    fn hasLen(x: WithLen | WithId) -> bool is x: WithLen { return hasMethod(x, "len"); }
    fn test(x: WithLen | WithId) -> number {
        if (hasLen(x)) { let _y: WithLen = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type Ok = { tag: string, value: number };
    type Err = { tag: number, message: string };
    fn isOk(x: Ok | Err) -> bool is x: Ok { return hasTag(x, "ok"); }
    fn test(x: Ok | Err) -> number {
        if (isOk(x)) { let _y: Ok = x; return 1; }
        else { let _y: Err = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNullish(x: null | string) -> bool is x: null { return isNull(x); }
    fn test(x: null | string) -> number {
        if (isNullish(x)) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isObj(x: json | string) -> bool is x: json { return isObject(x); }
    fn test(x: json | string) -> number {
        if (isObj(x)) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isArr(x: number[] | string) -> bool is x: number[] { return isArray(x); }
    fn test(x: number[] | string) -> number {
        if (isArr(x)) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isFunc(x: ((number) -> number) | string) -> bool is x: (number) -> number { return isFunction(x); }
    fn test(x: ((number) -> number) | string) -> number {
        if (isFunc(x)) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
fn test_predicate_syntax_valid(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

#[rstest]
#[case(
    r#"
    fn isStr(x: number) -> number is x: number { return 1; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is y: number { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: string { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number {
        return 1; // return type mismatch
    }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) is x: number { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number { let _y: string = x; return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number | string) -> bool is x: bool { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number | string) -> bool is missing: string { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number { return false; }
    fn test(x: number | string) -> number { if (isStr(x)) { return 1; } return 2; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number { return true; }
    fn test(x: number) -> number { if (isStr(x)) { let _y: string = x; } return 1; }
    "#
)]
fn test_predicate_syntax_errors(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

// =============================================================================
// Built-in guard narrowing
// =============================================================================

#[rstest]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isNumber(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: bool | null) -> number {
        if (isBool(x)) { let _y: bool = x; return 1; }
        else { let _y: null = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: null | string) -> number {
        if (isNull(x)) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number[] | string) -> number {
        if (isArray(x)) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn f(x: number) -> number { return x; }
    fn test(x: ((number) -> number) | string) -> number {
        if (isFunction(x)) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: json | string) -> number {
        if (isObject(x)) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (!isString(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) || isNumber(x)) { let _y: number | string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) && isType(x, "string")) { let _y: string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "number")) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
fn test_builtin_guard_narrowing(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// User-defined guards
// =============================================================================

#[rstest]
#[case(
    r#"
    fn isText(x: number | string) -> bool is x: string { return isString(x); }
    fn test(x: number | string) -> number {
        if (isText(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn isNamed(x: WithName | WithId) -> bool is x: WithName { return hasField(x, "name"); }
    fn test(x: WithName | WithId) -> number {
        if (isNamed(x)) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithLen = { len: () -> number };
    type WithId = { id: number };
    fn isLen(x: WithLen | WithId) -> bool is x: WithLen { return hasMethod(x, "len"); }
    fn test(x: WithLen | WithId) -> number {
        if (isLen(x)) { let _y: WithLen = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNum(x: number | string) -> bool is x: number { return isNumber(x); }
    fn test(x: number | string) -> number {
        if (isNum(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isArr(x: number[] | string) -> bool is x: number[] { return isArray(x); }
    fn test(x: number[] | string) -> number {
        if (isArr(x)) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNullish(x: null | string) -> bool is x: null { return isNull(x); }
    fn test(x: null | string) -> number {
        if (isNullish(x)) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type Ok = { tag: string, value: number };
    type Err = { tag: number, message: string };
    fn isOk(x: Ok | Err) -> bool is x: Ok { return hasTag(x, "ok"); }
    fn test(x: Ok | Err) -> number {
        if (isOk(x)) { let _y: Ok = x; return 1; }
        else { let _y: Err = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isObj(x: json | string) -> bool is x: json { return isObject(x); }
    fn test(x: json | string) -> number {
        if (isObj(x)) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isFunc(x: ((number) -> number) | string) -> bool is x: (number) -> number { return isFunction(x); }
    fn test(x: ((number) -> number) | string) -> number {
        if (isFunc(x)) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isTypeString(x: number | string) -> bool is x: string { return isType(x, "string"); }
    fn test(x: number | string) -> number {
        if (isTypeString(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
fn test_user_defined_guards(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// Guard composition + control flow
// =============================================================================

#[rstest]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) || isNumber(x)) { let _y: number | string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) && isType(x, "string")) { let _y: string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (!isString(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) && !isNull(x)) { let _y: string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "string") || isType(x, "number")) { let _y: number | string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) && isNumber(x)) { return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x)) { let _y: string = x; }
        if (isNumber(x)) { let _y: number = x; }
        return 1;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x)) { let _y: string = x; return 1; }
        if (isNumber(x)) { let _y: number = x; return 2; }
        return 3;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        var result: number = 0;
        if (isString(x)) { result = 1; }
        if (isNumber(x)) { result = 2; }
        return result;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        while (isString(x)) { let _y: string = x; return 1; }
        return 2;
    }
    "#
)]
fn test_guard_composition_and_flow(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// Structural + discriminated guards
// =============================================================================

#[rstest]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name")) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithLen = { len: () -> number };
    type WithId = { id: number };
    fn test(x: WithLen | WithId) -> number {
        if (hasMethod(x, "len")) { let _y: WithLen = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithTag = { tag: string, value: number };
    type WithNumTag = { tag: number, message: string };
    fn test(x: WithTag | WithNumTag) -> number {
        if (hasTag(x, "ok")) { let _y: WithTag = x; return 1; }
        else { let _y: WithNumTag = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type One = { name: string, id: number };
    type Two = { id: number };
    fn test(x: One | Two) -> number {
        if (hasField(x, "name")) { let _y: One = x; return 1; }
        else { let _y: Two = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type One = { len: () -> number, id: number };
    type Two = { id: number };
    fn test(x: One | Two) -> number {
        if (hasMethod(x, "len")) { let _y: One = x; return 1; }
        else { let _y: Two = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type One = { tag: string, id: number };
    type Two = { tag: number, id: number };
    fn test(x: One | Two) -> number {
        if (hasTag(x, "one")) { let _y: One = x; return 1; }
        else { let _y: Two = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name") && hasField(x, "name")) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name") || hasField(x, "id")) { let _y: WithName | WithId = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (!hasField(x, "name")) { let _y: WithId = x; return 1; }
        else { let _y: WithName = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name")) { let _y: { name: string } = x; return 1; }
        else { let _y: { id: number } = x; return 2; }
    }
    "#
)]
fn test_structural_guards(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// isType guard tests
// =============================================================================

#[rstest]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "string")) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "number")) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: bool | null) -> number {
        if (isType(x, "bool")) { let _y: bool = x; return 1; }
        else { let _y: null = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: null | string) -> number {
        if (isType(x, "null")) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number[] | string) -> number {
        if (isType(x, "array")) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn f(x: number) -> number { return x; }
    fn test(x: ((number) -> number) | string) -> number {
        if (isType(x, "function")) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: json | string) -> number {
        if (isType(x, "json")) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: json | string) -> number {
        if (isType(x, "object")) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "number") || isType(x, "string")) { let _y: number | string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (!isType(x, "string")) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
fn test_is_type_guard(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// Runtime guard behavior
// =============================================================================

#[rstest]
#[case("isString(\"ok\")", Value::Bool(true))]
#[case("isString(1)", Value::Bool(false))]
#[case("isNumber(1)", Value::Bool(true))]
#[case("isBool(true)", Value::Bool(true))]
#[case("isNull(null)", Value::Bool(true))]
#[case("isArray([1, 2])", Value::Bool(true))]
#[case("isType(\"ok\", \"string\")", Value::Bool(true))]
#[case("isType(1, \"number\")", Value::Bool(true))]
#[case("isType([1, 2], \"array\")", Value::Bool(true))]
#[case("isType(null, \"null\")", Value::Bool(true))]
fn test_runtime_basic_guards(#[case] expr: &str, #[case] expected: Value) {
    let code = format!("{}", expr);
    let result = eval(&code);
    assert_eq!(result, expected);
}

#[rstest]
#[case(
    r#"
    let obj = parseJSON("{\"tag\":\"ok\", \"value\": 1}");
    hasTag(obj, "ok")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"tag\":\"bad\", \"value\": 1}");
    hasTag(obj, "ok")
    "#,
    Value::Bool(false)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasField(obj, "name")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasField(obj, "missing")
    "#,
    Value::Bool(false)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasMethod(obj, "name")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasMethod(obj, "missing")
    "#,
    Value::Bool(false)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    isObject(obj)
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    isType(obj, "object")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let hmap = hashMapNew();
    hashMapPut(hmap, "name", 1);
    hasField(hmap, "name")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let hmap = hashMapNew();
    hashMapPut(hmap, "tag", "ok");
    hasTag(hmap, "ok")
    "#,
    Value::Bool(true)
)]
fn test_runtime_structural_guards(#[case] code: &str, #[case] expected: Value) {
    let result = eval(code);
    assert_eq!(result, expected);
}
