//! Custom native functions example
//!
//! Demonstrates registering Rust functions callable from Atlas code.
//!
//! Run with: cargo run --example 02_custom_functions -p atlas-runtime

use atlas_runtime::api::{ExecutionMode, Runtime};
use atlas_runtime::span::Span;
use atlas_runtime::value::{RuntimeError, Value};

fn main() {
    let mut runtime = Runtime::new(ExecutionMode::VM);

    // Register a simple native function
    runtime.register_function("double", 1, |args| {
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n * 2.0)),
            _ => Err(RuntimeError::TypeError {
                msg: "Expected number".to_string(),
                span: Span::dummy(),
            }),
        }
    });

    let result = runtime.eval("double(21)").expect("Failed");
    println!("double(21) = {}", result);
    // Output: double(21) = 42

    // Register a function with multiple arguments
    runtime.register_function("add", 2, |args| {
        let a = match &args[0] {
            Value::Number(n) => *n,
            _ => return Err(RuntimeError::TypeError {
                msg: "Expected number".to_string(),
                span: Span::dummy(),
            }),
        };
        let b = match &args[1] {
            Value::Number(n) => *n,
            _ => return Err(RuntimeError::TypeError {
                msg: "Expected number".to_string(),
                span: Span::dummy(),
            }),
        };
        Ok(Value::Number(a + b))
    });

    let result = runtime.eval("add(10, 20)").expect("Failed");
    println!("add(10, 20) = {}", result);
    // Output: add(10, 20) = 30

    // Register a variadic function
    runtime.register_variadic("sum", |args| {
        let mut total = 0.0;
        for arg in args {
            match arg {
                Value::Number(n) => total += n,
                _ => return Err(RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: Span::dummy(),
                }),
            }
        }
        Ok(Value::Number(total))
    });

    let result = runtime.eval("sum(1, 2, 3, 4, 5)").expect("Failed");
    println!("sum(1, 2, 3, 4, 5) = {}", result);
    // Output: sum(1, 2, 3, 4, 5) = 15

    // Call native from Atlas function
    runtime
        .eval("fn square(x: number) -> number { return double(x) * x; }")
        .expect("Failed");

    let result = runtime.eval("square(5)").expect("Failed");
    println!("square(5) = {}", result);
    // Output: square(5) = 50
}
