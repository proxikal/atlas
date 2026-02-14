//! VM tests for method call syntax (Phase 17) - mirrors interpreter tests for 100% parity

use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};
use rstest::rstest;

fn run_vm(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);
    let mut compiler = Compiler::new();
    match compiler.compile(&program) {
        Ok(bytecode) => {
            let mut vm = VM::new(bytecode);
            match vm.run(&SecurityContext::allow_all()) {
                Ok(opt_value) => match opt_value {
                    Some(value) => Ok(format!("{:?}", value)),
                    None => Ok("None".to_string()),
                },
                Err(e) => Err(format!("{:?}", e)),
            }
        }
        Err(e) => Err(format!("Compile error: {:?}", e)),
    }
}

// JSON as_string() Tests
#[rstest]
#[case(
    r#"let data: json = parseJSON("{\"name\":\"Alice\"}"); data["name"].as_string();"#,
    r#"String("Alice")"#
)]
#[case(r#"let data: json = parseJSON("{\"user\":{\"name\":\"Bob\"}}"); data["user"]["name"].as_string();"#, r#"String("Bob")"#)]
fn test_json_as_string(#[case] source: &str, #[case] expected: &str) {
    let result = run_vm(source).expect("Should succeed");
    assert_eq!(result, expected);
}

#[test]
fn test_json_as_string_error() {
    let result = run_vm(r#"let data: json = parseJSON("{\"age\":30}"); data["age"].as_string();"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot extract string"));
}

// JSON as_number() Tests
#[rstest]
#[case(
    r#"let data: json = parseJSON("{\"age\":30}"); data["age"].as_number();"#,
    "Number(30.0)"
)]
#[case(
    r#"let data: json = parseJSON("{\"price\":19.99}"); data["price"].as_number();"#,
    "Number(19.99)"
)]
fn test_json_as_number(#[case] source: &str, #[case] expected: &str) {
    let result = run_vm(source).expect("Should succeed");
    assert_eq!(result, expected);
}

#[test]
fn test_json_as_number_error() {
    let result =
        run_vm(r#"let data: json = parseJSON("{\"name\":\"Alice\"}"); data["name"].as_number();"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot extract number"));
}

// JSON as_bool() Tests
#[rstest]
#[case(
    r#"let data: json = parseJSON("{\"active\":true}"); data["active"].as_bool();"#,
    "Bool(true)"
)]
#[case(
    r#"let data: json = parseJSON("{\"disabled\":false}"); data["disabled"].as_bool();"#,
    "Bool(false)"
)]
fn test_json_as_bool(#[case] source: &str, #[case] expected: &str) {
    let result = run_vm(source).expect("Should succeed");
    assert_eq!(result, expected);
}

#[test]
fn test_json_as_bool_error() {
    let result = run_vm(r#"let data: json = parseJSON("{\"count\":5}"); data["count"].as_bool();"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot extract bool"));
}

// JSON is_null() Tests
#[rstest]
#[case(
    r#"let data: json = parseJSON("{\"value\":null}"); data["value"].is_null();"#,
    "Bool(true)"
)]
#[case(
    r#"let data: json = parseJSON("{\"value\":\"text\"}"); data["value"].is_null();"#,
    "Bool(false)"
)]
#[case(
    r#"let data: json = parseJSON("{\"value\":42}"); data["value"].is_null();"#,
    "Bool(false)"
)]
fn test_json_is_null(#[case] source: &str, #[case] expected: &str) {
    let result = run_vm(source).expect("Should succeed");
    assert_eq!(result, expected);
}

// Complex Tests
#[test]
fn test_chained_methods() {
    let result = run_vm(
        r#"
        let data: json = parseJSON("{\"user\":{\"name\":\"Charlie\"}}");
        data["user"]["name"].as_string();
    "#,
    )
    .expect("Should succeed");
    assert_eq!(result, r#"String("Charlie")"#);
}

#[test]
fn test_method_in_expression() {
    let result = run_vm(
        r#"
        let data: json = parseJSON("{\"count\":5}");
        data["count"].as_number() + 10;
    "#,
    )
    .expect("Should succeed");
    assert_eq!(result, "Number(15.0)");
}

#[test]
fn test_method_in_conditional() {
    let result = run_vm(
        r#"
        let data: json = parseJSON("{\"enabled\":true}");
        var result: string = "no";
        if (data["enabled"].as_bool()) {
            result = "yes";
        };
        result;
    "#,
    )
    .expect("Should succeed");
    assert_eq!(result, r#"String("yes")"#);
}

#[test]
fn test_multiple_methods() {
    let result = run_vm(
        r#"
        let data: json = parseJSON("{\"a\":5,\"b\":10}");
        data["a"].as_number() + data["b"].as_number();
    "#,
    )
    .expect("Should succeed");
    assert_eq!(result, "Number(15.0)");
}

// Error Cases
#[test]
fn test_as_string_on_null() {
    let result = run_vm(r#"let data: json = parseJSON("{\"v\":null}"); data["v"].as_string();"#);
    assert!(result.is_err());
}

#[test]
fn test_as_number_on_null() {
    let result = run_vm(r#"let data: json = parseJSON("{\"v\":null}"); data["v"].as_number();"#);
    assert!(result.is_err());
}

#[test]
fn test_as_bool_on_null() {
    let result = run_vm(r#"let data: json = parseJSON("{\"v\":null}"); data["v"].as_bool();"#);
    assert!(result.is_err());
}

#[test]
fn test_as_string_on_object() {
    let result =
        run_vm(r#"let data: json = parseJSON("{\"obj\":{\"a\":1}}"); data["obj"].as_string();"#);
    assert!(result.is_err());
}

#[test]
fn test_as_number_on_array() {
    let result =
        run_vm(r#"let data: json = parseJSON("{\"arr\":[1,2,3]}"); data["arr"].as_number();"#);
    assert!(result.is_err());
}
