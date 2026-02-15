//! Minimal embedding example
//!
//! Demonstrates the simplest possible use of Atlas runtime.
//!
//! Run with: cargo run --example 01_hello_world -p atlas-runtime

use atlas_runtime::api::{ExecutionMode, Runtime};

fn main() {
    // Create a runtime with interpreter mode
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);

    // Evaluate a simple expression
    let result = runtime.eval("1 + 2").expect("Failed to evaluate");

    println!("Result: {}", result);
    // Output: Result: 3

    // Evaluate a string expression
    let result = runtime
        .eval(r#""Hello, " + "World!""#)
        .expect("Failed to evaluate");

    println!("Result: {}", result);
    // Output: Result: Hello, World!

    // Define and call a function
    runtime
        .eval("fn greet(name: string) -> string { return \"Hello, \" + name + \"!\"; }")
        .expect("Failed to define function");

    let result = runtime
        .eval(r#"greet("Atlas")"#)
        .expect("Failed to call function");

    println!("Result: {}", result);
    // Output: Result: Hello, Atlas!
}
