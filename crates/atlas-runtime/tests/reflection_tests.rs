//! Reflection API integration tests
//!
//! Tests reflection and introspection functionality with both
//! interpreter and VM execution engines (100% parity required).

use atlas_runtime::reflect::{get_value_type_info, TypeInfo, TypeKind, ValueInfo};
use atlas_runtime::types::Type;
use atlas_runtime::value::Value;
use atlas_runtime::Atlas;
use rstest::rstest;

// ============================================================================
// Type Information Tests
// ============================================================================

#[test]
fn test_type_info_from_primitive_types() {
    let num_info = TypeInfo::from_type(&Type::Number);
    assert_eq!(num_info.name, "number");
    assert_eq!(num_info.kind, TypeKind::Number);
    assert!(num_info.is_primitive());

    let str_info = TypeInfo::from_type(&Type::String);
    assert_eq!(str_info.name, "string");
    assert_eq!(str_info.kind, TypeKind::String);
    assert!(str_info.is_primitive());

    let bool_info = TypeInfo::from_type(&Type::Bool);
    assert_eq!(bool_info.name, "bool");
    assert_eq!(bool_info.kind, TypeKind::Bool);
    assert!(bool_info.is_primitive());

    let null_info = TypeInfo::from_type(&Type::Null);
    assert_eq!(null_info.name, "null");
    assert_eq!(null_info.kind, TypeKind::Null);
    assert!(null_info.is_primitive());
}

#[test]
fn test_type_info_from_array_type() {
    let arr_type = Type::Array(Box::new(Type::Number));
    let info = TypeInfo::from_type(&arr_type);

    assert_eq!(info.name, "number[]");
    assert_eq!(info.kind, TypeKind::Array);
    assert!(info.is_array());
    assert!(!info.is_primitive());

    assert!(info.element_type.is_some());
    let elem = info.element_type.as_ref().unwrap();
    assert_eq!(elem.name, "number");
    assert_eq!(elem.kind, TypeKind::Number);
}

#[test]
fn test_type_info_from_function_type() {
    let func_type = Type::Function {
        type_params: vec![],
        params: vec![Type::Number, Type::String],
        return_type: Box::new(Type::Bool),
    };

    let info = TypeInfo::from_type(&func_type);

    assert_eq!(info.name, "function");
    assert_eq!(info.kind, TypeKind::Function);
    assert!(info.is_function());
    assert!(!info.is_primitive());

    assert_eq!(info.parameters.len(), 2);
    assert_eq!(info.parameters[0].name, "number");
    assert_eq!(info.parameters[1].name, "string");

    assert!(info.return_type.is_some());
    let ret = info.return_type.as_ref().unwrap();
    assert_eq!(ret.name, "bool");
}

#[test]
fn test_type_info_from_generic_type() {
    let gen_type = Type::Generic {
        name: "Result".to_string(),
        type_args: vec![Type::Number, Type::String],
    };

    let info = TypeInfo::from_type(&gen_type);

    assert_eq!(info.name, "Result<number, string>");
    assert_eq!(info.kind, TypeKind::Generic);
    assert!(info.is_generic());

    assert_eq!(info.type_args.len(), 2);
    assert_eq!(info.type_args[0].name, "number");
    assert_eq!(info.type_args[1].name, "string");
}

#[test]
fn test_type_info_function_signature() {
    let func_type = Type::Function {
        type_params: vec![],
        params: vec![Type::Number, Type::String],
        return_type: Box::new(Type::Bool),
    };

    let info = TypeInfo::from_type(&func_type);
    let sig = info.function_signature().unwrap();

    assert_eq!(sig, "(number, string) -> bool");
}

#[test]
fn test_type_info_describe() {
    let num_info = TypeInfo::from_type(&Type::Number);
    assert_eq!(num_info.describe(), "primitive number type");

    let arr_info = TypeInfo::from_type(&Type::Array(Box::new(Type::String)));
    assert_eq!(arr_info.describe(), "array of string");

    let func_type = Type::Function {
        type_params: vec![],
        params: vec![Type::Number],
        return_type: Box::new(Type::Void),
    };
    let func_info = TypeInfo::from_type(&func_type);
    assert_eq!(func_info.describe(), "function (number) -> void");
}

#[test]
fn test_type_info_nested_arrays() {
    // number[][]
    let nested = Type::Array(Box::new(Type::Array(Box::new(Type::Number))));
    let info = TypeInfo::from_type(&nested);

    assert_eq!(info.name, "number[][]");
    assert!(info.is_array());

    let outer_elem = info.element_type.as_ref().unwrap();
    assert_eq!(outer_elem.name, "number[]");
    assert!(outer_elem.is_array());

    let inner_elem = outer_elem.element_type.as_ref().unwrap();
    assert_eq!(inner_elem.name, "number");
    assert!(inner_elem.is_primitive());
}

#[test]
fn test_type_info_equality() {
    let info1 = TypeInfo::from_type(&Type::Number);
    let info2 = TypeInfo::from_type(&Type::Number);
    let info3 = TypeInfo::from_type(&Type::String);

    assert_eq!(info1, info2);
    assert_ne!(info1, info3);
}

// ============================================================================
// Value Information Tests
// ============================================================================

#[test]
fn test_value_info_type_name() {
    let num_info = ValueInfo::new(Value::Number(42.0));
    assert_eq!(num_info.type_name(), "number");

    let str_info = ValueInfo::new(Value::string("test"));
    assert_eq!(str_info.type_name(), "string");

    let arr_info = ValueInfo::new(Value::array(vec![]));
    assert_eq!(arr_info.type_name(), "array");
}

#[test]
fn test_value_info_get_length() {
    let arr = Value::array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    let info = ValueInfo::new(arr);
    assert_eq!(info.get_length(), Some(3));

    let str_val = Value::string("hello");
    let info = ValueInfo::new(str_val);
    assert_eq!(info.get_length(), Some(5));

    let num = Value::Number(42.0);
    let info = ValueInfo::new(num);
    assert_eq!(info.get_length(), None);
}

#[test]
fn test_value_info_is_empty() {
    let empty_arr = Value::array(vec![]);
    assert!(ValueInfo::new(empty_arr).is_empty());

    let empty_str = Value::string("");
    assert!(ValueInfo::new(empty_str).is_empty());

    let non_empty = Value::array(vec![Value::Number(1.0)]);
    assert!(!ValueInfo::new(non_empty).is_empty());
}

#[test]
fn test_value_info_type_checks() {
    let num_info = ValueInfo::new(Value::Number(42.0));
    assert!(num_info.is_number());
    assert!(!num_info.is_string());
    assert!(!num_info.is_bool());
    assert!(!num_info.is_null());

    let str_info = ValueInfo::new(Value::string("test"));
    assert!(str_info.is_string());
    assert!(!str_info.is_number());

    let bool_info = ValueInfo::new(Value::Bool(true));
    assert!(bool_info.is_bool());
    assert!(!bool_info.is_number());

    let null_info = ValueInfo::new(Value::Null);
    assert!(null_info.is_null());
    assert!(!null_info.is_number());
}

#[test]
fn test_value_info_get_values() {
    let num = Value::Number(42.5);
    let info = ValueInfo::new(num);
    assert_eq!(info.get_number(), Some(42.5));
    assert_eq!(info.get_string(), None);

    let bool_val = Value::Bool(false);
    let info = ValueInfo::new(bool_val);
    assert_eq!(info.get_bool(), Some(false));
    assert_eq!(info.get_number(), None);
}

#[test]
fn test_value_info_array_elements() {
    let arr = Value::array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    let info = ValueInfo::new(arr);

    let elements = info.get_array_elements().unwrap();
    assert_eq!(elements.len(), 3);
    assert_eq!(elements[0], Value::Number(1.0));
    assert_eq!(elements[1], Value::Number(2.0));
    assert_eq!(elements[2], Value::Number(3.0));
}

#[test]
fn test_get_value_type_info_primitives() {
    let num = Value::Number(42.0);
    let info = get_value_type_info(&num);
    assert_eq!(info.name, "number");
    assert_eq!(info.kind, TypeKind::Number);

    let str_val = Value::string("hello");
    let info = get_value_type_info(&str_val);
    assert_eq!(info.name, "string");
    assert_eq!(info.kind, TypeKind::String);

    let bool_val = Value::Bool(true);
    let info = get_value_type_info(&bool_val);
    assert_eq!(info.name, "bool");
    assert_eq!(info.kind, TypeKind::Bool);

    let null_val = Value::Null;
    let info = get_value_type_info(&null_val);
    assert_eq!(info.name, "null");
    assert_eq!(info.kind, TypeKind::Null);
}

#[test]
fn test_get_value_type_info_array() {
    let arr = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);
    let info = get_value_type_info(&arr);
    assert_eq!(info.name, "array");
    assert_eq!(info.kind, TypeKind::Array);
}

#[test]
fn test_get_value_type_info_option() {
    let some_val = Value::Option(Some(Box::new(Value::Number(42.0))));
    let info = get_value_type_info(&some_val);
    assert_eq!(info.name, "Option");
    assert_eq!(info.kind, TypeKind::Option);

    let none_val = Value::Option(None);
    let info = get_value_type_info(&none_val);
    assert_eq!(info.name, "Option");
    assert_eq!(info.kind, TypeKind::Option);
}

#[test]
fn test_get_value_type_info_result() {
    let ok_val = Value::Result(Ok(Box::new(Value::Number(42.0))));
    let info = get_value_type_info(&ok_val);
    assert_eq!(info.name, "Result");
    assert_eq!(info.kind, TypeKind::Result);

    let err_val = Value::Result(Err(Box::new(Value::string("error"))));
    let info = get_value_type_info(&err_val);
    assert_eq!(info.name, "Result");
    assert_eq!(info.kind, TypeKind::Result);
}

// ============================================================================
// Stdlib Reflection Integration Tests (Interpreter)
// ============================================================================

fn run_interpreter(code: &str) -> Value {
    let runtime = Atlas::new();
    runtime.eval(code).expect("Interpreter execution failed")
}

#[rstest]
#[case("reflect_typeof(42)", "number")]
#[case("reflect_typeof(\"hello\")", "string")]
#[case("reflect_typeof(true)", "bool")]
#[case("reflect_typeof(null)", "null")]
#[case("reflect_typeof([1, 2, 3])", "array")]
fn test_interpreter_typeof(#[case] code: &str, #[case] expected: &str) {
    let result = run_interpreter(code);
    assert_eq!(result, Value::string(expected));
}

#[rstest]
#[case("reflect_is_primitive(42)", true)]
#[case("reflect_is_primitive(\"test\")", true)]
#[case("reflect_is_primitive(true)", true)]
#[case("reflect_is_primitive(null)", true)]
#[case("reflect_is_primitive([1, 2])", false)]
fn test_interpreter_is_primitive(#[case] code: &str, #[case] expected: bool) {
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(expected));
}

#[rstest]
#[case("reflect_same_type(42, 99)", true)]
#[case("reflect_same_type(42, \"test\")", false)]
#[case("reflect_same_type(\"a\", \"b\")", true)]
#[case("reflect_same_type(true, false)", true)]
fn test_interpreter_same_type(#[case] code: &str, #[case] expected: bool) {
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(expected));
}

#[rstest]
#[case("reflect_get_length([1, 2, 3])", 3.0)]
#[case("reflect_get_length(\"hello\")", 5.0)]
#[case("reflect_get_length([])", 0.0)]
#[case("reflect_get_length(\"\")", 0.0)]
fn test_interpreter_get_length(#[case] code: &str, #[case] expected: f64) {
    let result = run_interpreter(code);
    assert_eq!(result, Value::Number(expected));
}

#[rstest]
#[case("reflect_is_empty([])", true)]
#[case("reflect_is_empty(\"\")", true)]
#[case("reflect_is_empty([1])", false)]
#[case("reflect_is_empty(\"x\")", false)]
fn test_interpreter_is_empty(#[case] code: &str, #[case] expected: bool) {
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(expected));
}

#[test]
fn test_interpreter_type_describe() {
    let result = run_interpreter("reflect_type_describe(42)");
    assert_eq!(result, Value::string("primitive number type"));

    let result = run_interpreter("reflect_type_describe([1, 2])");
    // Just verify it returns a string
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_interpreter_clone() {
    let result = run_interpreter("reflect_clone(42)");
    assert_eq!(result, Value::Number(42.0));

    let result = run_interpreter("reflect_clone(\"test\")");
    assert_eq!(result, Value::string("test"));
}

#[test]
fn test_interpreter_value_to_string() {
    let result = run_interpreter("reflect_value_to_string(42)");
    assert_eq!(result, Value::string("42"));

    let result = run_interpreter("reflect_value_to_string([1, 2, 3])");
    assert_eq!(result, Value::string("[1, 2, 3]"));
}

#[test]
fn test_interpreter_deep_equals() {
    let code = r#"
        let a = [1, 2, 3];
        let b = [1, 2, 3];
        reflect_deep_equals(a, b)
    "#;
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(true));

    let code = r#"
        let a = [1, 2, 3];
        let b = [1, 2, 4];
        reflect_deep_equals(a, b)
    "#;
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_interpreter_nested_deep_equals() {
    let code = r#"
        let a = [[1, 2], [3, 4]];
        let b = [[1, 2], [3, 4]];
        reflect_deep_equals(a, b)
    "#;
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Stdlib Reflection Integration Tests (VM)
// ============================================================================

fn run_vm(code: &str) -> Value {
    use atlas_runtime::compiler::Compiler;
    use atlas_runtime::lexer::Lexer;
    use atlas_runtime::parser::Parser;
    use atlas_runtime::vm::VM;
    use atlas_runtime::SecurityContext;

    // Add semicolon if needed (like Atlas::eval() does)
    let code = code.trim();
    let code_with_semi = if !code.is_empty() && !code.ends_with(';') && !code.ends_with('}') {
        format!("{};", code)
    } else {
        code.to_string()
    };

    // Lex
    let mut lexer = Lexer::new(&code_with_semi);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        panic!("Lexer errors: {:?}", lex_diags);
    }

    // Parse
    let mut parser = Parser::new(tokens);
    let (ast, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        panic!("Parser errors: {:?}", parse_diags);
    }

    // Compile
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("Compilation failed");

    // Run in VM
    let mut vm = VM::new(bytecode);
    vm.run(&SecurityContext::allow_all())
        .expect("VM execution failed")
        .expect("VM returned None")
}

#[rstest]
#[case("reflect_typeof(42)", "number")]
#[case("reflect_typeof(\"hello\")", "string")]
#[case("reflect_typeof(true)", "bool")]
#[case("reflect_typeof(null)", "null")]
#[case("reflect_typeof([1, 2, 3])", "array")]
fn test_vm_typeof(#[case] code: &str, #[case] expected: &str) {
    let result = run_vm(code);
    assert_eq!(result, Value::string(expected));
}

#[rstest]
#[case("reflect_is_primitive(42)", true)]
#[case("reflect_is_primitive(\"test\")", true)]
#[case("reflect_is_primitive(true)", true)]
#[case("reflect_is_primitive(null)", true)]
#[case("reflect_is_primitive([1, 2])", false)]
fn test_vm_is_primitive(#[case] code: &str, #[case] expected: bool) {
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(expected));
}

#[rstest]
#[case("reflect_same_type(42, 99)", true)]
#[case("reflect_same_type(42, \"test\")", false)]
#[case("reflect_same_type(\"a\", \"b\")", true)]
#[case("reflect_same_type(true, false)", true)]
fn test_vm_same_type(#[case] code: &str, #[case] expected: bool) {
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(expected));
}

#[rstest]
#[case("reflect_get_length([1, 2, 3])", 3.0)]
#[case("reflect_get_length(\"hello\")", 5.0)]
#[case("reflect_get_length([])", 0.0)]
#[case("reflect_get_length(\"\")", 0.0)]
fn test_vm_get_length(#[case] code: &str, #[case] expected: f64) {
    let result = run_vm(code);
    assert_eq!(result, Value::Number(expected));
}

#[rstest]
#[case("reflect_is_empty([])", true)]
#[case("reflect_is_empty(\"\")", true)]
#[case("reflect_is_empty([1])", false)]
#[case("reflect_is_empty(\"x\")", false)]
fn test_vm_is_empty(#[case] code: &str, #[case] expected: bool) {
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(expected));
}

#[test]
fn test_vm_type_describe() {
    let result = run_vm("reflect_type_describe(42)");
    assert_eq!(result, Value::string("primitive number type"));

    let result = run_vm("reflect_type_describe([1, 2])");
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_vm_clone() {
    let result = run_vm("reflect_clone(42)");
    assert_eq!(result, Value::Number(42.0));

    let result = run_vm("reflect_clone(\"test\")");
    assert_eq!(result, Value::string("test"));
}

#[test]
fn test_vm_value_to_string() {
    let result = run_vm("reflect_value_to_string(42)");
    assert_eq!(result, Value::string("42"));

    let result = run_vm("reflect_value_to_string([1, 2, 3])");
    assert_eq!(result, Value::string("[1, 2, 3]"));
}

#[test]
fn test_vm_deep_equals() {
    let code = r#"
        let a = [1, 2, 3];
        let b = [1, 2, 3];
        reflect_deep_equals(a, b)
    "#;
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(true));

    let code = r#"
        let a = [1, 2, 3];
        let b = [1, 2, 4];
        reflect_deep_equals(a, b)
    "#;
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_vm_nested_deep_equals() {
    let code = r#"
        let a = [[1, 2], [3, 4]];
        let b = [[1, 2], [3, 4]];
        reflect_deep_equals(a, b)
    "#;
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Parity Verification Tests
// ============================================================================

#[rstest]
#[case("reflect_typeof(42)")]
#[case("reflect_typeof(\"test\")")]
#[case("reflect_typeof([1, 2, 3])")]
#[case("reflect_is_primitive(42)")]
#[case("reflect_is_primitive([1])")]
#[case("reflect_same_type(1, 2)")]
#[case("reflect_same_type(1, \"a\")")]
#[case("reflect_get_length([1, 2, 3])")]
#[case("reflect_get_length(\"hello\")")]
#[case("reflect_is_empty([])")]
#[case("reflect_is_empty([1])")]
#[case("reflect_clone(42)")]
#[case("reflect_value_to_string(42)")]
fn test_parity_reflection_functions(#[case] code: &str) {
    let interpreter_result = run_interpreter(code);
    let vm_result = run_vm(code);

    assert_eq!(
        interpreter_result, vm_result,
        "Parity violation for: {}",
        code
    );
}

#[test]
fn test_parity_deep_equals() {
    let cases = vec![
        "reflect_deep_equals([1, 2], [1, 2])",
        "reflect_deep_equals([1, 2], [1, 3])",
        "reflect_deep_equals([[1]], [[1]])",
        "reflect_deep_equals(42, 42)",
        "reflect_deep_equals(\"a\", \"a\")",
    ];

    for code in cases {
        let interpreter_result = run_interpreter(code);
        let vm_result = run_vm(code);

        assert_eq!(
            interpreter_result, vm_result,
            "Parity violation for: {}",
            code
        );
    }
}
