//! Error handling example
//!
//! Demonstrates handling all error types from Atlas runtime.
//!
//! Run with: cargo run --example 05_error_handling -p atlas-runtime

use atlas_runtime::api::{EvalError, ExecutionMode, Runtime};

fn main() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);

    println!("Demonstrating error handling:\n");

    // Parse error
    println!("1. Parse error:");
    match runtime.eval("let x = ") {
        Ok(_) => println!("   Unexpected success"),
        Err(EvalError::ParseError(diags)) => {
            println!("   Caught parse error:");
            for diag in diags {
                println!("     - {}", diag.message);
            }
        }
        Err(e) => println!("   Unexpected error: {:?}", e),
    }

    // Type error
    println!("\n2. Type error:");
    match runtime.eval(r#"let x: number = "not a number";"#) {
        Ok(_) => println!("   Unexpected success"),
        Err(EvalError::TypeError(diags)) => {
            println!("   Caught type error:");
            for diag in diags {
                println!("     - {}", diag.message);
            }
        }
        Err(e) => println!("   Unexpected error: {:?}", e),
    }

    // Runtime error (division by zero)
    println!("\n3. Runtime error:");
    match runtime.eval("10 / 0") {
        Ok(val) => println!("   Result: {}", val), // May succeed with infinity
        Err(EvalError::RuntimeError(err)) => {
            println!("   Caught runtime error: {:?}", err);
        }
        Err(e) => println!("   Unexpected error: {:?}", e),
    }

    // Successful execution
    println!("\n4. Successful execution:");
    match runtime.eval("21 * 2") {
        Ok(val) => println!("   Result: {}", val),
        Err(e) => println!("   Unexpected error: {:?}", e),
    }
}
