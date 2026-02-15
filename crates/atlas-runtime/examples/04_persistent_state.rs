//! Persistent state example
//!
//! Demonstrates state persistence across multiple eval() calls.
//!
//! Run with: cargo run --example 04_persistent_state -p atlas-runtime

use atlas_runtime::api::{ExecutionMode, Runtime};

fn main() {
    let mut runtime = Runtime::new(ExecutionMode::VM);

    println!("Demonstrating persistent state across eval() calls:\n");

    // Define a function in first eval
    runtime
        .eval("fn factorial(n: number) -> number { if (n <= 1) { return 1; } else { return n * factorial(n - 1); } }")
        .expect("Failed to define function");

    println!("Defined factorial function");

    // Call it in subsequent evals
    let result1 = runtime.eval("factorial(5)").expect("Failed");
    println!("factorial(5) = {}", result1);

    let result2 = runtime.eval("factorial(10)").expect("Failed");
    println!("factorial(10) = {}", result2);

    // Functions can call each other across eval() boundaries
    runtime
        .eval("fn double(x: number) -> number { return x * 2; }")
        .expect("Failed");

    runtime
        .eval("fn quadruple(x: number) -> number { return double(double(x)); }")
        .expect("Failed");

    let result3 = runtime.eval("quadruple(7)").expect("Failed");
    println!("quadruple(7) = {}", result3);
}
