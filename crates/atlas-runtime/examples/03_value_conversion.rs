//! Value conversion example
//!
//! Demonstrates converting between Rust and Atlas types.
//!
//! Run with: cargo run --example 03_value_conversion -p atlas-runtime

use atlas_runtime::api::{ExecutionMode, FromAtlas, Runtime, ToAtlas};
use atlas_runtime::value::Value;

fn main() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);

    // Convert Rust values to Atlas
    let number: Value = 42.0.to_atlas();
    let string: Value = "Hello".to_atlas();
    let bool_val: Value = true.to_atlas();

    println!("Rust -> Atlas:");
    println!("  42.0 -> {}", number);
    println!("  \"Hello\" -> {}", string);
    println!("  true -> {}", bool_val);

    // Evaluate and convert back to Rust
    let result = runtime.eval("21 * 2").expect("Failed");
    let number: f64 = f64::from_atlas(&result).expect("Not a number");
    println!("\nAtlas -> Rust:");
    println!("  21 * 2 = {}", number);

    // Work with arrays
    let array: Value = vec![1.0, 2.0, 3.0].to_atlas();
    println!("\nArray: {}", array);

    // Work with options
    let some_val: Value = Some(42.0).to_atlas();
    let none_val: Value = Option::<f64>::None.to_atlas();
    println!("\nOptions:");
    println!("  Some(42.0) -> {}", some_val);
    println!("  None -> {}", none_val);
}
