//! For-in loop execution tests (Phase-20c)
//!
//! Tests that for-in loops execute correctly in the interpreter.

use atlas_runtime::{Atlas, Value};

#[test]
fn test_for_in_basic_execution() {
    let source = r#"
        let arr: array = [1, 2, 3];
        var sum: number = 0;
        for item in arr {
            sum = sum + item;
        }
        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert!(result.is_ok(), "Should execute for-in loop: {:?}", result);
    assert_eq!(result.unwrap(), Value::Number(6.0), "Sum should be 6");
}

#[test]
fn test_for_in_empty_array() {
    let source = r#"
        let arr: array = [];
        var count: number = 0;
        for item in arr {
            count = count + 1;
        }
        count
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert!(result.is_ok(), "Should handle empty array: {:?}", result);
    assert_eq!(result.unwrap(), Value::Number(0.0), "Count should be 0");
}

#[test]
fn test_for_in_with_strings() {
    let source = r#"
        let words: array = ["hello", "world"];
        var result: string = "";
        for word in words {
            result = result + word + " ";
        }
        result
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert!(result.is_ok(), "Should work with strings: {:?}", result);
    match result.unwrap() {
        Value::String(s) => assert_eq!(&*s, "hello world "),
        other => panic!("Expected string, got {:?}", other),
    }
}

#[test]
fn test_for_in_nested() {
    let source = r#"
        let matrix: array = [[1, 2], [3, 4]];
        var sum: number = 0;
        for row in matrix {
            for item in row {
                sum = sum + item;
            }
        }
        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert!(result.is_ok(), "Should handle nested loops: {:?}", result);
    assert_eq!(result.unwrap(), Value::Number(10.0), "Sum should be 10");
}

#[test]
fn test_for_in_modifies_external_variable() {
    let source = r#"
        let arr: array = [10, 20, 30];
        var total: number = 0;
        for x in arr {
            total = total + x;
        }
        total
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(60.0));
}

#[test]
fn test_for_in_with_break() {
    let source = r#"
        let arr: array = [1, 2, 3, 4, 5];
        var sum: number = 0;
        for item in arr {
            if (item > 3) {
                break;
            }
            sum = sum + item;
        }
        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(6.0),
        "Should break at 4, sum 1+2+3=6"
    );
}

#[test]
fn test_for_in_with_continue() {
    let source = r#"
        let arr: array = [1, 2, 3, 4, 5];
        var sum: number = 0;
        for item in arr {
            if (item == 3) {
                continue;
            }
            sum = sum + item;
        }
        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(12.0),
        "Should skip 3, sum 1+2+4+5=12"
    );
}

#[test]
fn test_for_in_variable_shadowing() {
    let source = r#"
        let item: number = 100;
        let arr: array = [1, 2, 3];

        for item in arr {
            // 'item' here shadows outer 'item'
        }

        item
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(100.0),
        "Outer variable unchanged"
    );
}

#[test]
fn test_for_in_in_function() {
    let source = r#"
        fn sum_array(arr: array) -> number {
            var total: number = 0;
            for item in arr {
                total = total + item;
            }
            return total;
        }

        sum_array([10, 20, 30])
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(60.0));
}
