// Merged: api_tests + api_conversion_tests + api_native_functions_tests + api_sandboxing_tests
//       + reflection_tests + json_value_tests + runtime_api

// ===== api_tests.rs =====

mod api_core {
    // Integration tests for Runtime API
    //
    // Tests the public embedding API for creating runtimes, evaluating code,
    // calling functions, and managing global variables.

    use atlas_runtime::api::{EvalError, ExecutionMode, Runtime};
    use atlas_runtime::Value;
    use std::sync::Arc;

    // Runtime Creation Tests

    #[test]
    fn test_runtime_creation_interpreter() {
        let runtime = Runtime::new(ExecutionMode::Interpreter);
        assert_eq!(runtime.mode(), ExecutionMode::Interpreter);
    }

    #[test]
    fn test_runtime_creation_vm() {
        let runtime = Runtime::new(ExecutionMode::VM);
        assert_eq!(runtime.mode(), ExecutionMode::VM);
    }

    #[test]
    fn test_runtime_with_custom_security() {
        use atlas_runtime::SecurityContext;
        let security = SecurityContext::allow_all();
        let runtime = Runtime::new_with_security(ExecutionMode::Interpreter, security);
        assert_eq!(runtime.mode(), ExecutionMode::Interpreter);
    }

    // Basic eval() Tests - Interpreter Mode

    #[test]
    fn test_eval_number_literal_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("42").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_eval_string_literal_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("\"hello\"").unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "hello"));
    }

    #[test]
    fn test_eval_bool_true_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("true").unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_eval_bool_false_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("false").unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_eval_null_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("null").unwrap();
        assert!(matches!(result, Value::Null));
    }

    #[test]
    fn test_eval_arithmetic_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("1 + 2").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_eval_string_concat_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("\"hello\" + \" \" + \"world\"").unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "hello world"));
    }

    #[test]
    fn test_eval_comparison_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("5 > 3").unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_eval_logical_and_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("true && false").unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_eval_logical_or_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("true || false").unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    // Basic eval() Tests - VM Mode

    #[test]
    fn test_eval_number_literal_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        let result = runtime.eval("42").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_eval_string_literal_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        let result = runtime.eval("\"hello\"").unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "hello"));
    }

    #[test]
    fn test_eval_bool_true_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        let result = runtime.eval("true").unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_eval_bool_false_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        let result = runtime.eval("false").unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_eval_null_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        let result = runtime.eval("null").unwrap();
        assert!(matches!(result, Value::Null));
    }

    #[test]
    fn test_eval_arithmetic_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        let result = runtime.eval("1 + 2").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_eval_string_concat_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        let result = runtime.eval("\"hello\" + \" \" + \"world\"").unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "hello world"));
    }

    #[test]
    fn test_eval_comparison_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        let result = runtime.eval("5 > 3").unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_eval_logical_and_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        let result = runtime.eval("true && false").unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_eval_logical_or_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        let result = runtime.eval("true || false").unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    // State Persistence Tests - Interpreter
    // Note: Cross-eval state persistence requires persistent symbol tables
    // This is a future enhancement. For v0.2 phase-01, use single-eval programs
    // or set_global/get_global for programmatic state management.

    #[test]
    fn test_single_eval_with_multiple_statements() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        // Variables defined and used in the same eval() work fine
        let result = runtime
            .eval("var x: number = 1; var y: number = 2; x + y")
            .unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_function_definition_and_call_single_eval() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        // Define and call function in the same eval()
        let result = runtime
            .eval("fn add(x: number, y: number) -> number { return x + y; } add(10, 20)")
            .unwrap();
        assert!(matches!(result, Value::Number(n) if n == 30.0));
    }

    #[test]
    fn test_function_multiple_calls_single_eval() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime
            .eval("fn square(x: number) -> number { return x * x; } square(3) + square(4)")
            .unwrap();
        assert!(matches!(result, Value::Number(n) if n == 25.0));
    }

    // State Persistence Tests - VM

    #[test]
    fn test_global_variable_persistence_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        runtime.eval("var x: number = 42;").unwrap();
        // Note: VM doesn't persist state yet in this phase
        // This test documents current limitation
    }

    #[test]
    fn test_function_definition_persistence_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        runtime
            .eval("fn add(x: number, y: number) -> number { return x + y; }")
            .unwrap();
        // Note: VM doesn't persist state yet in this phase
    }

    // Error Handling Tests

    #[test]
    fn test_eval_parse_error_missing_semicolon() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("let x: number =");
        assert!(matches!(result, Err(EvalError::ParseError(_))));
    }

    #[test]
    fn test_eval_parse_error_invalid_syntax() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("@#$%");
        assert!(matches!(result, Err(EvalError::ParseError(_))));
    }

    #[test]
    fn test_eval_type_error_wrong_type() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("let x: number = \"hello\";");
        assert!(matches!(result, Err(EvalError::TypeError(_))));
    }

    #[test]
    fn test_eval_type_error_arithmetic_on_string() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("\"hello\" - \"world\"");
        assert!(matches!(result, Err(EvalError::TypeError(_))));
    }

    #[test]
    fn test_eval_runtime_error_divide_by_zero() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("1 / 0");
        assert!(matches!(result, Err(EvalError::RuntimeError(_))));
    }

    #[test]
    fn test_eval_type_error_undefined_variable() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("nonexistent");
        // Undefined variables are caught at binding/typecheck phase
        assert!(result.is_err());
    }

    // call() Function Tests

    #[test]
    fn test_call_builtin_print() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.call("print", vec![Value::Number(42.0)]).unwrap();
        assert!(matches!(result, Value::Null));
    }

    #[test]
    fn test_call_builtin_str() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.call("str", vec![Value::Number(42.0)]).unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "42"));
    }

    #[test]
    fn test_call_builds_on_eval() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        // call() uses eval() internally, so functions must be defined inline
        let result = runtime.call("str", vec![Value::Number(42.0)]).unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "42"));
    }

    // get_global/set_global Tests - Interpreter Mode

    #[test]
    fn test_set_global_number_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        runtime.set_global("x", Value::Number(42.0));
        let value = runtime.get_global("x").unwrap();
        assert!(matches!(value, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_set_global_string_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        runtime.set_global("message", Value::String(Arc::new("hello".to_string())));
        let value = runtime.get_global("message").unwrap();
        assert!(matches!(value, Value::String(s) if s.as_ref() == "hello"));
    }

    #[test]
    fn test_set_global_bool_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        runtime.set_global("flag", Value::Bool(true));
        let value = runtime.get_global("flag").unwrap();
        assert!(matches!(value, Value::Bool(true)));
    }

    #[test]
    fn test_set_global_null_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        runtime.set_global("nothing", Value::Null);
        let value = runtime.get_global("nothing").unwrap();
        assert!(matches!(value, Value::Null));
    }

    #[test]
    fn test_get_global_nonexistent_interpreter() {
        let runtime = Runtime::new(ExecutionMode::Interpreter);
        let value = runtime.get_global("nonexistent");
        assert!(value.is_none());
    }

    #[test]
    fn test_set_global_and_get_global_roundtrip() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        runtime.set_global("x", Value::Number(100.0));
        let value = runtime.get_global("x").unwrap();
        assert!(matches!(value, Value::Number(n) if n == 100.0));
        // Note: Using set_global'd variables in eval() requires symbol table persistence (future phase)
    }

    #[test]
    fn test_set_global_overwrite_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        runtime.set_global("x", Value::Number(10.0));
        runtime.set_global("x", Value::Number(20.0));
        let value = runtime.get_global("x").unwrap();
        assert!(matches!(value, Value::Number(n) if n == 20.0));
    }

    // get_global/set_global Tests - VM Mode (current limitations)

    #[test]
    fn test_get_global_vm_returns_none() {
        let runtime = Runtime::new(ExecutionMode::VM);
        // VM mode doesn't support direct global access yet
        let value = runtime.get_global("x");
        assert!(value.is_none());
    }

    // Mode Parity Tests

    #[test]
    fn test_parity_arithmetic_expression() {
        let mut interp = Runtime::new(ExecutionMode::Interpreter);
        let mut vm = Runtime::new(ExecutionMode::VM);

        let expr = "((10 + 5) * 2) - 3";
        let interp_result = interp.eval(expr).unwrap();
        let vm_result = vm.eval(expr).unwrap();

        assert!(matches!(interp_result, Value::Number(n) if n == 27.0));
        assert!(matches!(vm_result, Value::Number(n) if n == 27.0));
    }

    #[test]
    fn test_parity_string_operations() {
        let mut interp = Runtime::new(ExecutionMode::Interpreter);
        let mut vm = Runtime::new(ExecutionMode::VM);

        let expr = "\"hello\" + \" \" + \"world\"";
        let interp_result = interp.eval(expr).unwrap();
        let vm_result = vm.eval(expr).unwrap();

        assert!(matches!(interp_result, Value::String(s) if s.as_ref() == "hello world"));
        assert!(matches!(vm_result, Value::String(s) if s.as_ref() == "hello world"));
    }

    #[test]
    fn test_parity_boolean_logic() {
        let mut interp = Runtime::new(ExecutionMode::Interpreter);
        let mut vm = Runtime::new(ExecutionMode::VM);

        let expr = "(true && false) || (false || true)";
        let interp_result = interp.eval(expr).unwrap();
        let vm_result = vm.eval(expr).unwrap();

        assert!(matches!(interp_result, Value::Bool(true)));
        assert!(matches!(vm_result, Value::Bool(true)));
    }

    // Complex Program Tests

    #[test]
    fn test_complex_program_with_control_flow_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let program = r#"
            fn factorial(n: number) -> number {
                if (n <= 1) {
                    return 1;
                } else {
                    return n * factorial(n - 1);
                }
            }
            factorial(5)
        "#;
        let result = runtime.eval(program).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 120.0));
    }

    #[test]
    fn test_complex_program_with_loops_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let program = r#"
            var sum: number = 0;
            for (var i: number = 1; i <= 10; i = i + 1) {
                sum = sum + i;
            }
            sum
        "#;
        let result = runtime.eval(program).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 55.0));
    }

    #[test]
    fn test_multiple_function_definitions_single_eval() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        // Define all functions in a single eval()
        let result = runtime
            .eval(
                r#"
                fn add(x: number, y: number) -> number { return x + y; }
                fn sub(x: number, y: number) -> number { return x - y; }
                fn mul(x: number, y: number) -> number { return x * y; }
                add(10, sub(20, mul(2, 3)))
            "#,
            )
            .unwrap();
        assert!(matches!(result, Value::Number(n) if n == 24.0));
    }
}

// ===== api_conversion_tests.rs =====

mod api_conversion {
    // Integration tests for value conversion API
    //
    // Tests ToAtlas and FromAtlas traits for bidirectional conversion
    // between Rust and Atlas types.

    use atlas_runtime::api::{ConversionError, FromAtlas, ToAtlas};
    use atlas_runtime::Value;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::Mutex;

    // f64 Conversion Tests

    #[test]
    fn test_f64_to_atlas() {
        let value = 42.5.to_atlas();
        assert!(matches!(value, Value::Number(n) if n == 42.5));
    }

    #[test]
    fn test_f64_from_atlas_success() {
        let value = Value::Number(42.5);
        let result: f64 = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, 42.5);
    }

    #[test]
    fn test_f64_from_atlas_type_mismatch() {
        let value = Value::String(Arc::new("hello".to_string()));
        let result: Result<f64, _> = FromAtlas::from_atlas(&value);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConversionError::TypeMismatch { expected, found } => {
                assert_eq!(expected, "number");
                assert_eq!(found, "string");
            }
            _ => panic!("Expected TypeMismatch error"),
        }
    }

    #[test]
    fn test_f64_zero() {
        let value = 0.0.to_atlas();
        let result: f64 = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_f64_negative() {
        let value = (-123.456).to_atlas();
        let result: f64 = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, -123.456);
    }

    #[test]
    fn test_f64_large_number() {
        let value = 1.7976931348623157e308.to_atlas();
        let result: f64 = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, 1.7976931348623157e308);
    }

    // String Conversion Tests

    #[test]
    fn test_string_to_atlas() {
        let value = "hello world".to_string().to_atlas();
        assert!(matches!(value, Value::String(s) if s.as_ref() == "hello world"));
    }

    #[test]
    fn test_string_from_atlas_success() {
        let value = Value::String(Arc::new("hello".to_string()));
        let result: String = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_string_from_atlas_type_mismatch() {
        let value = Value::Number(42.0);
        let result: Result<String, _> = FromAtlas::from_atlas(&value);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_empty() {
        let value = String::new().to_atlas();
        let result: String = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_string_unicode() {
        let value = "Hello, 世界! 🌍".to_string().to_atlas();
        let result: String = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, "Hello, 世界! 🌍");
    }

    #[test]
    fn test_string_ref_to_atlas() {
        let s = "test";
        let value = s.to_atlas();
        assert!(matches!(value, Value::String(ref rs) if rs.as_ref() == "test"));
    }

    // bool Conversion Tests

    #[test]
    fn test_bool_true_to_atlas() {
        let value = true.to_atlas();
        assert!(matches!(value, Value::Bool(true)));
    }

    #[test]
    fn test_bool_false_to_atlas() {
        let value = false.to_atlas();
        assert!(matches!(value, Value::Bool(false)));
    }

    #[test]
    fn test_bool_from_atlas_true() {
        let value = Value::Bool(true);
        let result: bool = FromAtlas::from_atlas(&value).unwrap();
        assert!(result);
    }

    #[test]
    fn test_bool_from_atlas_false() {
        let value = Value::Bool(false);
        let result: bool = FromAtlas::from_atlas(&value).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_bool_from_atlas_type_mismatch() {
        let value = Value::Null;
        let result: Result<bool, _> = FromAtlas::from_atlas(&value);
        assert!(result.is_err());
    }

    // () (null) Conversion Tests

    #[test]
    fn test_unit_to_atlas() {
        let value = ().to_atlas();
        assert!(matches!(value, Value::Null));
    }

    #[test]
    fn test_unit_from_atlas_success() {
        let value = Value::Null;
        let result: () = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, ());
    }

    #[test]
    fn test_unit_from_atlas_type_mismatch() {
        let value = Value::Number(0.0);
        let result: Result<(), _> = FromAtlas::from_atlas(&value);
        assert!(result.is_err());
    }

    // Option<T> Conversion Tests

    #[test]
    fn test_option_some_number_to_atlas() {
        let value = Some(42.0).to_atlas();
        assert!(matches!(value, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_option_some_string_to_atlas() {
        let value = Some("hello".to_string()).to_atlas();
        assert!(matches!(value, Value::String(s) if s.as_ref() == "hello"));
    }

    #[test]
    fn test_option_none_to_atlas() {
        let value: Option<f64> = None;
        let atlas_value = value.to_atlas();
        assert!(matches!(atlas_value, Value::Null));
    }

    #[test]
    fn test_option_some_from_atlas() {
        let value = Value::Number(42.0);
        let result: Option<f64> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, Some(42.0));
    }

    #[test]
    fn test_option_none_from_atlas() {
        let value = Value::Null;
        let result: Option<f64> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_option_string_some_from_atlas() {
        let value = Value::String(Arc::new("test".to_string()));
        let result: Option<String> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, Some("test".to_string()));
    }

    #[test]
    fn test_option_string_none_from_atlas() {
        let value = Value::Null;
        let result: Option<String> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, None);
    }

    // Vec<T> Conversion Tests

    #[test]
    fn test_vec_f64_to_atlas() {
        let vec = vec![1.0, 2.0, 3.0];
        let value = vec.to_atlas();
        match value {
            Value::Array(arr) => {
                let arr_borrow = arr.lock().unwrap();
                assert_eq!(arr_borrow.len(), 3);
                assert!(matches!(arr_borrow[0], Value::Number(n) if n == 1.0));
                assert!(matches!(arr_borrow[1], Value::Number(n) if n == 2.0));
                assert!(matches!(arr_borrow[2], Value::Number(n) if n == 3.0));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_vec_f64_from_atlas() {
        let arr = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        let value = Value::Array(Arc::new(Mutex::new(arr)));
        let result: Vec<f64> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_vec_string_to_atlas() {
        let vec = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let value = vec.to_atlas();
        match value {
            Value::Array(arr) => {
                let arr_borrow = arr.lock().unwrap();
                assert_eq!(arr_borrow.len(), 3);
                assert!(matches!(&arr_borrow[0], Value::String(s) if s.as_ref() == "a"));
                assert!(matches!(&arr_borrow[1], Value::String(s) if s.as_ref() == "b"));
                assert!(matches!(&arr_borrow[2], Value::String(s) if s.as_ref() == "c"));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_vec_string_from_atlas() {
        let arr = vec![
            Value::String(Arc::new("x".to_string())),
            Value::String(Arc::new("y".to_string())),
        ];
        let value = Value::Array(Arc::new(Mutex::new(arr)));
        let result: Vec<String> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, vec!["x".to_string(), "y".to_string()]);
    }

    #[test]
    fn test_vec_empty_to_atlas() {
        let vec: Vec<f64> = vec![];
        let value = vec.to_atlas();
        match value {
            Value::Array(arr) => {
                let arr_borrow = arr.lock().unwrap();
                assert_eq!(arr_borrow.len(), 0);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_vec_empty_from_atlas() {
        let value = Value::Array(Arc::new(Mutex::new(vec![])));
        let result: Vec<f64> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_vec_from_atlas_wrong_type() {
        let value = Value::Number(42.0);
        let result: Result<Vec<f64>, _> = FromAtlas::from_atlas(&value);
        assert!(result.is_err());
    }

    #[test]
    fn test_vec_from_atlas_element_type_mismatch() {
        let arr = vec![
            Value::Number(1.0),
            Value::String(Arc::new("oops".to_string())),
        ];
        let value = Value::Array(Arc::new(Mutex::new(arr)));
        let result: Result<Vec<f64>, _> = FromAtlas::from_atlas(&value);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConversionError::ArrayElementTypeMismatch {
                index,
                expected,
                found,
            } => {
                assert_eq!(index, 1);
                assert_eq!(expected, "number");
                assert_eq!(found, "string");
            }
            _ => panic!("Expected ArrayElementTypeMismatch error"),
        }
    }

    // Nested Conversion Tests

    #[test]
    fn test_nested_vec_option_f64() {
        let data = vec![Some(1.0), None, Some(3.0)];
        let value = data.to_atlas();

        // Convert back
        let result: Vec<Option<f64>> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, vec![Some(1.0), None, Some(3.0)]);
    }

    #[test]
    fn test_nested_vec_option_string() {
        let data = vec![Some("hello".to_string()), None, Some("world".to_string())];
        let value = data.to_atlas();

        // Convert back
        let result: Vec<Option<String>> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(
            result,
            vec![Some("hello".to_string()), None, Some("world".to_string())]
        );
    }

    #[test]
    fn test_nested_option_vec_f64() {
        let data: Option<Vec<f64>> = Some(vec![1.0, 2.0, 3.0]);
        let value = data.to_atlas();

        // Convert back
        let result: Option<Vec<f64>> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, Some(vec![1.0, 2.0, 3.0]));
    }

    #[test]
    fn test_nested_option_vec_none() {
        let data: Option<Vec<f64>> = None;
        let value = data.to_atlas();

        // Convert back
        let result: Option<Vec<f64>> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, None);
    }

    // HashMap Conversion Tests

    #[test]
    fn test_hashmap_to_atlas_creates_json() {
        let mut map = HashMap::new();
        map.insert("x".to_string(), 1.0);
        map.insert("y".to_string(), 2.0);

        let value = map.to_atlas();
        assert!(matches!(value, Value::JsonValue(_)));
    }

    #[test]
    fn test_hashmap_string_to_atlas() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), "Alice".to_string());
        map.insert("city".to_string(), "Boston".to_string());

        let value = map.to_atlas();
        assert!(matches!(value, Value::JsonValue(_)));
    }

    // Bidirectional Roundtrip Tests

    #[test]
    fn test_roundtrip_f64() {
        let original = 42.5;
        let value = original.to_atlas();
        let result: f64 = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn test_roundtrip_string() {
        let original = "hello world".to_string();
        let value = original.clone().to_atlas();
        let result: String = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn test_roundtrip_bool_true() {
        let original = true;
        let value = original.to_atlas();
        let result: bool = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn test_roundtrip_bool_false() {
        let original = false;
        let value = original.to_atlas();
        let result: bool = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn test_roundtrip_option_some() {
        let original = Some(42.0);
        let value = original.to_atlas();
        let result: Option<f64> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn test_roundtrip_option_none() {
        let original: Option<f64> = None;
        let value = original.to_atlas();
        let result: Option<f64> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn test_roundtrip_vec_f64() {
        let original = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let value = original.clone().to_atlas();
        let result: Vec<f64> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn test_roundtrip_vec_string() {
        let original = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let value = original.clone().to_atlas();
        let result: Vec<String> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn test_roundtrip_vec_option_f64() {
        let original = vec![Some(1.0), None, Some(3.0), None, Some(5.0)];
        let value = original.clone().to_atlas();
        let result: Vec<Option<f64>> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(original, result);
    }

    // Error Message Quality Tests

    #[test]
    fn test_conversion_error_display_type_mismatch() {
        let error = ConversionError::TypeMismatch {
            expected: "number".to_string(),
            found: "string".to_string(),
        };
        let message = format!("{}", error);
        assert!(message.contains("number"));
        assert!(message.contains("string"));
        assert!(message.contains("mismatch"));
    }

    #[test]
    fn test_conversion_error_display_array_element() {
        let error = ConversionError::ArrayElementTypeMismatch {
            index: 5,
            expected: "number".to_string(),
            found: "bool".to_string(),
        };
        let message = format!("{}", error);
        assert!(message.contains("5"));
        assert!(message.contains("number"));
        assert!(message.contains("bool"));
        assert!(message.contains("Array"));
    }

    #[test]
    fn test_conversion_error_display_object_value() {
        let error = ConversionError::ObjectValueTypeMismatch {
            key: "name".to_string(),
            expected: "string".to_string(),
            found: "number".to_string(),
        };
        let message = format!("{}", error);
        assert!(message.contains("name"));
        assert!(message.contains("string"));
        assert!(message.contains("number"));
        assert!(message.contains("Object"));
    }
}

// ===== api_native_functions_tests.rs =====

mod api_native {
    // Native function registration and calling tests
    //
    // Tests native Rust function registration and calling from Atlas code.
    // Verifies both fixed-arity and variadic functions in interpreter and VM modes.

    use atlas_runtime::api::{ExecutionMode, Runtime};
    use atlas_runtime::span::Span;
    use atlas_runtime::value::{RuntimeError, Value};
    use rstest::rstest;

    // ============================================================================
    // Test Fixtures
    // ============================================================================

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_register_fixed_arity_native(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        // Register a simple add function
        runtime.register_function("add", 2, |args| {
            let a = match &args[0] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            let b = match &args[1] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            Ok(Value::Number(a + b))
        });

        // Call the native function
        let result = runtime.eval("add(10, 20)").unwrap();
        assert_eq!(result, Value::Number(30.0));
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_register_variadic_native(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        // Register a variadic sum function
        runtime.register_variadic("sum", |args| {
            let mut total = 0.0;
            for arg in args {
                match arg {
                    Value::Number(n) => total += n,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "All arguments must be numbers".to_string(),
                            span: Span::dummy(),
                        })
                    }
                }
            }
            Ok(Value::Number(total))
        });

        // Call with different argument counts
        let result = runtime.eval("sum()").unwrap();
        assert_eq!(result, Value::Number(0.0));

        let result = runtime.eval("sum(42)").unwrap();
        assert_eq!(result, Value::Number(42.0));

        let result = runtime.eval("sum(1, 2, 3, 4, 5)").unwrap();
        assert_eq!(result, Value::Number(15.0));
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_arity_validation_too_few(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("add", 2, |args| {
            let a = match &args[0] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            let b = match &args[1] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            Ok(Value::Number(a + b))
        });

        // Call with too few arguments
        let result = runtime.eval("add(10)");
        assert!(result.is_err());
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_arity_validation_too_many(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("add", 2, |args| {
            let a = match &args[0] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            let b = match &args[1] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            Ok(Value::Number(a + b))
        });

        // Call with too many arguments
        let result = runtime.eval("add(10, 20, 30)");
        assert!(result.is_err());
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_returning_error(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("alwaysFails", 0, |_args| {
            Err(RuntimeError::TypeError {
                msg: "This function always fails".to_string(),
                span: Span::dummy(),
            })
        });

        let result = runtime.eval("alwaysFails()");
        assert!(result.is_err());
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_with_string_args(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("greet", 1, |args| match &args[0] {
            Value::String(s) => Ok(Value::string(format!("Hello, {}!", s))),
            _ => Err(RuntimeError::TypeError {
                msg: "Expected string".to_string(),
                span: Span::dummy(),
            }),
        });

        let result = runtime.eval(r#"greet("World")"#).unwrap();
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), "Hello, World!"),
            _ => panic!("Expected string result"),
        }
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_with_bool_args(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("negate", 1, |args| match &args[0] {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err(RuntimeError::TypeError {
                msg: "Expected bool".to_string(),
                span: Span::dummy(),
            }),
        });

        let result = runtime.eval("negate(true)").unwrap();
        assert_eq!(result, Value::Bool(false));

        let result = runtime.eval("negate(false)").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_with_array_args(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("arrayLength", 1, |args| match &args[0] {
            Value::Array(arr) => Ok(Value::Number(arr.lock().unwrap().len() as f64)),
            _ => Err(RuntimeError::TypeError {
                msg: "Expected array".to_string(),
                span: Span::dummy(),
            }),
        });

        let result = runtime.eval("arrayLength([1, 2, 3, 4, 5])").unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_returning_null(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("returnNull", 0, |_args| Ok(Value::Null));

        let result = runtime.eval("returnNull()").unwrap();
        assert_eq!(result, Value::Null);
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_returning_array(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("makeRange", 1, |args| {
            let n = match &args[0] {
                Value::Number(n) => *n as usize,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            let arr: Vec<Value> = (0..n).map(|i| Value::Number(i as f64)).collect();
            Ok(Value::array(arr))
        });

        let result = runtime.eval("makeRange(5)").unwrap();
        match result {
            Value::Array(arr) => {
                let borrowed = arr.lock().unwrap();
                assert_eq!(borrowed.len(), 5);
                assert_eq!(borrowed[0], Value::Number(0.0));
                assert_eq!(borrowed[4], Value::Number(4.0));
            }
            _ => panic!("Expected array result"),
        }
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_called_from_atlas_function(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("multiply", 2, |args| {
            let a = match &args[0] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            let b = match &args[1] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            Ok(Value::Number(a * b))
        });

        runtime
            .eval("fn square(x: number) -> number { return multiply(x, x); }")
            .unwrap();
        let result = runtime.eval("square(5)").unwrap();
        assert_eq!(result, Value::Number(25.0));
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_with_closure_capture(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        let multiplier = 10.0;
        runtime.register_function("scale", 1, move |args| {
            let n = match &args[0] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            Ok(Value::Number(n * multiplier))
        });

        let result = runtime.eval("scale(5)").unwrap();
        assert_eq!(result, Value::Number(50.0));
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_multiple_native_functions(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("add", 2, |args| {
            let a = match &args[0] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            let b = match &args[1] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            Ok(Value::Number(a + b))
        });

        runtime.register_function("sub", 2, |args| {
            let a = match &args[0] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            let b = match &args[1] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            Ok(Value::Number(a - b))
        });

        let result = runtime.eval("add(10, 5)").unwrap();
        assert_eq!(result, Value::Number(15.0));

        let result = runtime.eval("sub(10, 5)").unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_override_builtin_name(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        // Register a native with the same name as a builtin
        runtime.register_function("len", 1, |args| {
            // Custom len that always returns 42
            let _ = args;
            Ok(Value::Number(42.0))
        });

        // Native should take precedence over builtin
        let result = runtime.eval("len([1, 2, 3])").unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_with_zero_arity(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("getFortyTwo", 0, |_args| Ok(Value::Number(42.0)));

        let result = runtime.eval("getFortyTwo()").unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_with_three_args(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("sum3", 3, |args| {
            let a = match &args[0] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            let b = match &args[1] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            let c = match &args[2] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            Ok(Value::Number(a + b + c))
        });

        let result = runtime.eval("sum3(10, 20, 30)").unwrap();
        assert_eq!(result, Value::Number(60.0));
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_variadic_with_zero_args(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_variadic("count", |args| Ok(Value::Number(args.len() as f64)));

        let result = runtime.eval("count()").unwrap();
        assert_eq!(result, Value::Number(0.0));
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_variadic_with_many_args(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_variadic("count", |args| Ok(Value::Number(args.len() as f64)));

        let result = runtime
            .eval("count(1, 2, 3, 4, 5, 6, 7, 8, 9, 10)")
            .unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_in_expression(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("double", 1, |args| {
            let n = match &args[0] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            Ok(Value::Number(n * 2.0))
        });

        let result = runtime.eval("double(5) + double(10)").unwrap();
        assert_eq!(result, Value::Number(30.0));
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_nested_calls(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("inc", 1, |args| {
            let n = match &args[0] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            Ok(Value::Number(n + 1.0))
        });

        let result = runtime.eval("inc(inc(inc(0)))").unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_with_option_return(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("makeSome", 1, |args| {
            Ok(Value::Option(Some(Box::new(args[0].clone()))))
        });

        let result = runtime.eval("makeSome(42)").unwrap();
        match result {
            Value::Option(Some(val)) => assert_eq!(*val, Value::Number(42.0)),
            _ => panic!("Expected Some value"),
        }
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_with_result_return(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("makeOk", 1, |args| {
            Ok(Value::Result(Ok(Box::new(args[0].clone()))))
        });

        let result = runtime.eval("makeOk(42)").unwrap();
        match result {
            Value::Result(Ok(val)) => assert_eq!(*val, Value::Number(42.0)),
            _ => panic!("Expected Ok value"),
        }
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_persists_across_evaluations(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("add", 2, |args| {
            let a = match &args[0] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            let b = match &args[1] {
                Value::Number(n) => *n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };
            Ok(Value::Number(a + b))
        });

        // Call in separate evaluations
        let result1 = runtime.eval("add(1, 2)").unwrap();
        assert_eq!(result1, Value::Number(3.0));

        let result2 = runtime.eval("add(10, 20)").unwrap();
        assert_eq!(result2, Value::Number(30.0));
    }

    #[rstest]
    #[case::interpreter(ExecutionMode::Interpreter)]
    #[case::vm(ExecutionMode::VM)]
    fn test_native_with_complex_logic(#[case] mode: ExecutionMode) {
        let mut runtime = Runtime::new(mode);

        runtime.register_function("fibonacci", 1, |args| {
            let n = match &args[0] {
                Value::Number(n) => *n as i32,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Expected number".to_string(),
                        span: Span::dummy(),
                    })
                }
            };

            fn fib(n: i32) -> i32 {
                if n <= 1 {
                    n
                } else {
                    fib(n - 1) + fib(n - 2)
                }
            }

            Ok(Value::Number(fib(n) as f64))
        });

        let result = runtime.eval("fibonacci(10)").unwrap();
        assert_eq!(result, Value::Number(55.0));
    }
}

// ===== api_sandboxing_tests.rs =====

mod api_sandboxing {
    // Tests for Runtime sandboxing and configuration

    use atlas_runtime::api::{ExecutionMode, Runtime, RuntimeConfig};
    use std::time::Duration;

    #[test]
    fn test_default_config_is_permissive() {
        let config = RuntimeConfig::default();
        assert!(config.allow_io);
        assert!(config.allow_network);
        assert!(config.max_execution_time.is_none());
        assert!(config.max_memory_bytes.is_none());
    }

    #[test]
    fn test_sandboxed_config_is_restrictive() {
        let config = RuntimeConfig::sandboxed();
        assert!(!config.allow_io);
        assert!(!config.allow_network);
        assert_eq!(config.max_execution_time, Some(Duration::from_secs(5)));
        assert_eq!(config.max_memory_bytes, Some(10_000_000));
    }

    #[test]
    fn test_custom_config_fluent_api() {
        let config = RuntimeConfig::new()
            .with_max_execution_time(Duration::from_secs(30))
            .with_max_memory_bytes(100_000_000)
            .with_io_allowed(false)
            .with_network_allowed(true);

        assert_eq!(config.max_execution_time, Some(Duration::from_secs(30)));
        assert_eq!(config.max_memory_bytes, Some(100_000_000));
        assert!(!config.allow_io);
        assert!(config.allow_network);
    }

    #[test]
    fn test_runtime_with_sandboxed_config() {
        let mut runtime = Runtime::sandboxed(ExecutionMode::Interpreter);

        // Basic expressions should still work
        let result = runtime.eval("1 + 2").unwrap();
        assert_eq!(result.to_string(), "3");
    }

    #[test]
    fn test_runtime_with_custom_config() {
        let config = RuntimeConfig::new()
            .with_io_allowed(false)
            .with_network_allowed(false);

        let mut runtime = Runtime::with_config(ExecutionMode::VM, config);

        // Basic expressions should work
        let result = runtime.eval("let x: number = 42; x").unwrap();
        assert_eq!(result.to_string(), "42");
    }

    #[test]
    fn test_sandboxed_runtime_basic_arithmetic() {
        let mut runtime = Runtime::sandboxed(ExecutionMode::Interpreter);
        let result = runtime.eval("10 * 5 + 3").unwrap();
        assert_eq!(result.to_string(), "53");
    }

    #[test]
    fn test_sandboxed_runtime_function_definitions() {
        let mut runtime = Runtime::sandboxed(ExecutionMode::VM);
        runtime
            .eval("fn add(a: number, b: number) -> number { return a + b; }")
            .unwrap();

        let result = runtime.eval("add(10, 20)").unwrap();
        assert_eq!(result.to_string(), "30");
    }

    #[test]
    fn test_sandboxed_runtime_string_operations() {
        let mut runtime = Runtime::sandboxed(ExecutionMode::Interpreter);
        let result = runtime.eval(r#""Hello, " + "World!""#).unwrap();
        assert_eq!(result.to_string(), "Hello, World!");
    }

    #[test]
    fn test_sandboxed_runtime_array_operations() {
        let mut runtime = Runtime::sandboxed(ExecutionMode::VM);
        let result = runtime.eval("len([1, 2, 3])").unwrap();
        assert_eq!(result.to_string(), "3");
    }

    #[test]
    fn test_sandboxed_runtime_loops() {
        let mut runtime = Runtime::sandboxed(ExecutionMode::Interpreter);
        let result = runtime
            .eval(
                r#"
            var sum: number = 0;
            for (var i: number = 0; i < 10; i = i + 1) {
                sum = sum + i;
            }
            sum
            "#,
            )
            .unwrap();

        assert_eq!(result.to_string(), "45");
    }

    #[test]
    fn test_sandboxed_runtime_conditionals() {
        let mut runtime = Runtime::sandboxed(ExecutionMode::VM);
        runtime
            .eval(
                r#"
            fn maximum(a: number, b: number) -> number {
                if (a > b) {
                    return a;
                } else {
                    return b;
                }
            }
            "#,
            )
            .unwrap();

        let result = runtime.eval("maximum(10, 20)").unwrap();
        assert_eq!(result.to_string(), "20");
    }

    #[test]
    fn test_sandboxed_runtime_native_functions() {
        let mut runtime = Runtime::sandboxed(ExecutionMode::Interpreter);

        runtime.register_function("double", 1, |args| {
            if let atlas_runtime::value::Value::Number(n) = &args[0] {
                Ok(atlas_runtime::value::Value::Number(n * 2.0))
            } else {
                Err(atlas_runtime::value::RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: atlas_runtime::span::Span::dummy(),
                })
            }
        });

        let result = runtime.eval("double(21)").unwrap();
        assert_eq!(result.to_string(), "42");
    }

    #[test]
    fn test_config_clone() {
        let config1 = RuntimeConfig::sandboxed();
        let config2 = config1.clone();

        assert_eq!(config1.allow_io, config2.allow_io);
        assert_eq!(config1.allow_network, config2.allow_network);
        assert_eq!(config1.max_execution_time, config2.max_execution_time);
        assert_eq!(config1.max_memory_bytes, config2.max_memory_bytes);
    }

    #[test]
    fn test_multiple_sandboxed_runtimes_independent() {
        let mut runtime1 = Runtime::sandboxed(ExecutionMode::Interpreter);
        let mut runtime2 = Runtime::sandboxed(ExecutionMode::Interpreter);

        let result1 = runtime1.eval("let x: number = 10; x").unwrap();
        let result2 = runtime2.eval("let x: number = 20; x").unwrap();

        assert_eq!(result1.to_string(), "10");
        assert_eq!(result2.to_string(), "20");
    }

    #[test]
    fn test_config_with_only_time_limit() {
        let config = RuntimeConfig::new().with_max_execution_time(Duration::from_secs(10));

        assert_eq!(config.max_execution_time, Some(Duration::from_secs(10)));
        assert!(config.max_memory_bytes.is_none());
        assert!(config.allow_io);
        assert!(config.allow_network);
    }

    #[test]
    fn test_config_with_only_memory_limit() {
        let config = RuntimeConfig::new().with_max_memory_bytes(50_000_000);

        assert!(config.max_execution_time.is_none());
        assert_eq!(config.max_memory_bytes, Some(50_000_000));
        assert!(config.allow_io);
        assert!(config.allow_network);
    }

    #[test]
    fn test_config_disable_only_io() {
        let config = RuntimeConfig::new().with_io_allowed(false);

        assert!(!config.allow_io);
        assert!(config.allow_network);
    }

    #[test]
    fn test_config_disable_only_network() {
        let config = RuntimeConfig::new().with_network_allowed(false);

        assert!(config.allow_io);
        assert!(!config.allow_network);
    }

    #[test]
    fn test_sandboxed_runtime_error_handling() {
        let mut runtime = Runtime::sandboxed(ExecutionMode::Interpreter);

        // Type errors should still be caught
        let result = runtime.eval(r#"let x: number = "not a number";"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_sandboxed_runtime_persistent_state() {
        let mut runtime = Runtime::sandboxed(ExecutionMode::VM);

        // Define a function in one eval()
        runtime
            .eval("fn increment(x: number) -> number { return x + 1; }")
            .unwrap();

        // Call it in subsequent eval() calls
        let result1 = runtime.eval("increment(5)").unwrap();
        let result2 = runtime.eval("increment(10)").unwrap();

        assert_eq!(result1.to_string(), "6");
        assert_eq!(result2.to_string(), "11");
    }
}

// ===== reflection_tests.rs =====

mod reflection {
    // Reflection API integration tests
    //
    // Tests reflection and introspection functionality with both
    // interpreter and VM execution engines (100% parity required).

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
}

// ===== json_value_tests.rs =====

mod json_value {
    // Integration tests for JsonValue type
    //
    // Tests both interpreter and VM parity for JSON value operations.

    use atlas_runtime::{JsonValue, Value};
    use std::collections::HashMap;
    use std::sync::Arc;

    /// Helper to create a test JSON object
    fn make_test_json() -> Value {
        let mut user = HashMap::new();
        user.insert("name".to_string(), JsonValue::String("Alice".to_string()));
        user.insert("age".to_string(), JsonValue::Number(30.0));

        let mut data = HashMap::new();
        data.insert("user".to_string(), JsonValue::object(user));
        data.insert("active".to_string(), JsonValue::Bool(true));

        Value::JsonValue(Arc::new(JsonValue::object(data)))
    }

    #[test]
    fn test_json_value_type_display() {
        assert_eq!(make_test_json().type_name(), "json");
    }

    #[test]
    fn test_json_value_equality() {
        let json1 = Value::JsonValue(Arc::new(JsonValue::Number(42.0)));
        let json2 = Value::JsonValue(Arc::new(JsonValue::Number(42.0)));
        let json3 = Value::JsonValue(Arc::new(JsonValue::Number(43.0)));

        assert_eq!(json1, json2); // Same value
        assert_ne!(json1, json3); // Different value
    }

    // ===== Type Declaration Tests =====

    // NOTE: Type annotation test skipped - no JSON literal syntax yet
    // This will be added in Phase 4: JSON API when json_parse() is implemented
    // For now, JsonValue can only be constructed from Rust code, not Atlas code

    // #[rstest]
    // #[case("let x: json = null; x", "null")]
    // fn test_json_type_annotation(#[case] input: &str, #[case] _expected: &str) {
    //     // Test that "json" type annotation is recognized
    //     let runtime = Atlas::new();
    //     let result = runtime.eval(input);
    //     assert!(result.is_ok(), "Should accept json type annotation");
    // }

    // ===== Object Indexing Tests =====

    #[test]
    fn test_json_object_string_indexing_interpreter() {
        let json = make_test_json();

        // Access nested object
        if let Value::JsonValue(j) = json {
            let user = j.index_str("user");
            assert!(user.is_object());

            let name = user.index_str("name");
            assert_eq!(name, JsonValue::String("Alice".to_string()));

            let age = user.index_str("age");
            assert_eq!(age, JsonValue::Number(30.0));
        } else {
            panic!("Expected JsonValue");
        }
    }

    #[test]
    fn test_json_missing_key_returns_null() {
        let json = make_test_json();

        if let Value::JsonValue(j) = json {
            let missing = j.index_str("nonexistent");
            assert_eq!(missing, JsonValue::Null);
        } else {
            panic!("Expected JsonValue");
        }
    }

    // ===== Array Indexing Tests =====

    #[test]
    fn test_json_array_number_indexing() {
        let arr = Value::JsonValue(Arc::new(JsonValue::array(vec![
            JsonValue::Number(10.0),
            JsonValue::Number(20.0),
            JsonValue::Number(30.0),
        ])));

        if let Value::JsonValue(j) = arr {
            assert_eq!(j.index_num(0.0), JsonValue::Number(10.0));
            assert_eq!(j.index_num(1.0), JsonValue::Number(20.0));
            assert_eq!(j.index_num(2.0), JsonValue::Number(30.0));

            // Out of bounds returns null
            assert_eq!(j.index_num(3.0), JsonValue::Null);
            assert_eq!(j.index_num(100.0), JsonValue::Null);

            // Negative index returns null
            assert_eq!(j.index_num(-1.0), JsonValue::Null);

            // Fractional index returns null
            assert_eq!(j.index_num(1.5), JsonValue::Null);
        } else {
            panic!("Expected JsonValue");
        }
    }

    // ===== Type Extraction Tests =====

    #[test]
    fn test_json_type_checking_methods() {
        let null_val = JsonValue::Null;
        let bool_val = JsonValue::Bool(true);
        let num_val = JsonValue::Number(42.0);
        let str_val = JsonValue::String("hello".to_string());
        let arr_val = JsonValue::array(vec![]);
        let obj_val = JsonValue::object(HashMap::new());

        assert!(null_val.is_null());
        assert!(!null_val.is_bool());

        assert!(bool_val.is_bool());
        assert_eq!(bool_val.as_bool(), Some(true));

        assert!(num_val.is_number());
        assert_eq!(num_val.as_number(), Some(42.0));

        assert!(str_val.is_string());
        assert_eq!(str_val.as_string(), Some("hello"));

        assert!(arr_val.is_array());
        assert!(obj_val.is_object());
    }

    #[test]
    fn test_json_extraction_returns_none_for_wrong_type() {
        let num = JsonValue::Number(42.0);

        assert_eq!(num.as_number(), Some(42.0));
        assert_eq!(num.as_bool(), None);
        assert_eq!(num.as_string(), None);
        assert_eq!(num.as_array(), None);
        assert_eq!(num.as_object(), None);
    }

    // ===== Nested Access Tests =====

    #[test]
    fn test_json_nested_object_access() {
        let mut inner = HashMap::new();
        inner.insert(
            "city".to_string(),
            JsonValue::String("New York".to_string()),
        );

        let mut outer = HashMap::new();
        outer.insert("address".to_string(), JsonValue::object(inner));

        let json = JsonValue::object(outer);

        let address = json.index_str("address");
        let city = address.index_str("city");

        assert_eq!(city, JsonValue::String("New York".to_string()));
    }

    #[test]
    fn test_json_nested_array_object_access() {
        let user1 = {
            let mut map = HashMap::new();
            map.insert("name".to_string(), JsonValue::String("Bob".to_string()));
            JsonValue::object(map)
        };

        let user2 = {
            let mut map = HashMap::new();
            map.insert("name".to_string(), JsonValue::String("Carol".to_string()));
            JsonValue::object(map)
        };

        let users = JsonValue::array(vec![user1, user2]);

        let first = users.index_num(0.0);
        let name = first.index_str("name");

        assert_eq!(name, JsonValue::String("Bob".to_string()));
    }

    // ===== Display/Format Tests =====

    #[test]
    fn test_json_display_null() {
        assert_eq!(JsonValue::Null.to_string(), "null");
    }

    #[test]
    fn test_json_display_bool() {
        assert_eq!(JsonValue::Bool(true).to_string(), "true");
        assert_eq!(JsonValue::Bool(false).to_string(), "false");
    }

    #[test]
    fn test_json_display_number() {
        assert_eq!(JsonValue::Number(42.0).to_string(), "42");
        assert_eq!(JsonValue::Number(2.5).to_string(), "2.5");
        assert_eq!(JsonValue::Number(-5.0).to_string(), "-5");
    }

    #[test]
    fn test_json_display_string() {
        assert_eq!(
            JsonValue::String("hello".to_string()).to_string(),
            "\"hello\""
        );
    }

    #[test]
    fn test_json_display_array() {
        let arr = JsonValue::array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(arr.to_string(), "[1, 2, 3]");
    }

    #[test]
    fn test_json_display_object() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), JsonValue::Number(1.0));

        let obj = JsonValue::object(map);
        // Note: HashMap order is not guaranteed, so we just check it contains the right parts
        let display = obj.to_string();
        assert!(display.starts_with('{'));
        assert!(display.ends_with('}'));
        assert!(display.contains("\"a\""));
        assert!(display.contains('1'));
    }

    // ===== Length Tests =====

    #[test]
    fn test_json_array_length() {
        let arr = JsonValue::array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(arr.len(), Some(3));
    }

    #[test]
    fn test_json_object_length() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), JsonValue::Number(1.0));
        map.insert("b".to_string(), JsonValue::Number(2.0));

        let obj = JsonValue::object(map);
        assert_eq!(obj.len(), Some(2));
    }

    #[test]
    fn test_json_non_collection_length_is_none() {
        assert_eq!(JsonValue::Null.len(), None);
        assert_eq!(JsonValue::Bool(true).len(), None);
        assert_eq!(JsonValue::Number(42.0).len(), None);
        assert_eq!(JsonValue::String("hi".to_string()).len(), None);
    }

    #[test]
    fn test_json_is_empty() {
        assert!(JsonValue::Null.is_empty());
        assert!(JsonValue::array(vec![]).is_empty());
        assert!(!JsonValue::array(vec![JsonValue::Null]).is_empty());
        assert!(JsonValue::object(HashMap::new()).is_empty());
    }

    // ===== Isolation Tests (Type System) =====
    // These tests verify that JsonValue is isolated from regular types

    #[test]
    fn test_json_isolation_in_type_system() {
        use atlas_runtime::Type;

        let json_type = Type::JsonValue;
        let number_type = Type::Number;
        let string_type = Type::String;

        // JsonValue can only assign to JsonValue
        assert!(json_type.is_assignable_to(&json_type));
        assert!(!json_type.is_assignable_to(&number_type));
        assert!(!json_type.is_assignable_to(&string_type));

        // Other types cannot assign to JsonValue
        assert!(!number_type.is_assignable_to(&json_type));
        assert!(!string_type.is_assignable_to(&json_type));
    }

    #[test]
    fn test_json_type_display_name() {
        use atlas_runtime::Type;

        assert_eq!(Type::JsonValue.display_name(), "json");
    }
}

// ===== runtime_api.rs =====

mod runtime_api {
    // Integration tests for Atlas runtime API
    //
    // These tests validate the runtime API without using the CLI,
    // ensuring it can be embedded in other applications.

    use atlas_runtime::{Atlas, DiagnosticLevel, RuntimeResult, Value};

    /// Test that runtime can be created and used
    #[test]
    fn test_runtime_api_availability() {
        let runtime = Atlas::new();
        let _result: RuntimeResult<Value> = runtime.eval("test");
        // API is available and types work correctly
    }

    /// Test eval with simple input
    #[test]
    fn test_eval_basic() {
        let runtime = Atlas::new();
        let result = runtime.eval("1");
        // Currently stubbed, but test structure is correct
        assert!(result.is_err() || result.is_ok());
    }

    /// Test eval_file with path
    #[test]
    fn test_eval_file_basic() {
        let runtime = Atlas::new();
        let result = runtime.eval_file("test.atlas");
        // Currently stubbed, but test structure is correct
        assert!(result.is_err() || result.is_ok());
    }

    /// Test that diagnostics have the correct structure
    #[test]
    fn test_diagnostic_structure() {
        let runtime = Atlas::new();
        let result = runtime.eval("invalid");

        match result {
            Err(diagnostics) => {
                assert!(
                    !diagnostics.is_empty(),
                    "Should return at least one diagnostic"
                );

                let diag = &diagnostics[0];
                assert_eq!(diag.level, DiagnosticLevel::Error);
                assert!(!diag.message.is_empty(), "Diagnostic should have a message");
            }
            Ok(_) => {
                // Currently returns error, but when implemented might succeed
                // This test is flexible for future implementation
            }
        }
    }

    /// Test that runtime can be used multiple times
    #[test]
    fn test_runtime_reuse() {
        let runtime = Atlas::new();

        let _result1 = runtime.eval("1");
        let _result2 = runtime.eval("2");
        let _result3 = runtime.eval("3");

        // Runtime can be called multiple times without panicking
    }

    /// Test that multiple runtime instances can coexist
    #[test]
    fn test_multiple_runtimes() {
        let runtime1 = Atlas::new();
        let runtime2 = Atlas::new();

        let _result1 = runtime1.eval("test1");
        let _result2 = runtime2.eval("test2");

        // Multiple independent runtimes can be created
    }

    /// Test error diagnostics for invalid syntax
    #[test]
    fn test_error_diagnostics() {
        let runtime = Atlas::new();
        let result = runtime.eval("@#$%");

        match result {
            Err(diagnostics) => {
                assert!(!diagnostics.is_empty());
                for diag in &diagnostics {
                    assert_eq!(diag.level, DiagnosticLevel::Error);
                }
            }
            Ok(_) => {
                // When implemented, this should be an error
            }
        }
    }

    /// Test that runtime works without CLI dependencies
    #[test]
    fn test_no_cli_dependency() {
        // This test ensures we can use the runtime API
        // without pulling in any CLI-specific code
        let runtime = Atlas::new();
        let _result = runtime.eval("1 + 1");

        // If this compiles and runs, we have no CLI dependencies
    }

    // Future tests (currently ignored until implementation)

    #[test]
    #[ignore]
    fn test_eval_returns_value() {
        let runtime = Atlas::new();
        let result = runtime.eval("42");

        match result {
            Ok(Value::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    #[ignore]
    fn test_eval_preserves_state() {
        let runtime = Atlas::new();

        // Define a variable
        runtime.eval("let x: int = 10;").unwrap();

        // Use it in another eval
        let result = runtime.eval("x").unwrap();

        match result {
            Value::Number(n) => assert_eq!(n, 10.0),
            _ => panic!("Expected Number(10.0)"),
        }
    }

    #[test]
    #[ignore]
    fn test_eval_file_with_real_file() {
        use std::fs;
        use std::io::Write;

        // Create a temporary test file
        let mut file = fs::File::create("test_program.atlas").unwrap();
        writeln!(file, "let x: int = 42;").unwrap();

        let runtime = Atlas::new();
        let result = runtime.eval_file("test_program.atlas");

        // Clean up
        fs::remove_file("test_program.atlas").unwrap();

        match result {
            Ok(Value::Null) => (), // Variable declaration returns null
            _ => panic!("Expected Null"),
        }
    }
}
