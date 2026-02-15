//! Sandboxing example
//!
//! Demonstrates running untrusted code with restrictions.
//!
//! Run with: cargo run --example 06_sandboxing -p atlas-runtime

use atlas_runtime::api::{ExecutionMode, Runtime, RuntimeConfig};
use std::time::Duration;

fn main() {
    println!("Demonstrating sandboxing:\n");

    // Create a sandboxed runtime with restrictive defaults
    println!("1. Sandboxed runtime (default restrictions):");
    let mut sandboxed = Runtime::sandboxed(ExecutionMode::VM);

    // Basic operations still work
    let result = sandboxed.eval("1 + 2 * 3").expect("Failed");
    println!("   1 + 2 * 3 = {}", result);

    // Functions work
    sandboxed
        .eval("fn add(a: number, b: number) -> number { return a + b; }")
        .expect("Failed");
    let result = sandboxed.eval("add(10, 20)").expect("Failed");
    println!("   add(10, 20) = {}", result);

    // Create a custom sandboxed configuration
    println!("\n2. Custom sandboxed runtime:");
    let config = RuntimeConfig::new()
        .with_max_execution_time(Duration::from_secs(10))
        .with_max_memory_bytes(50_000_000) // 50MB
        .with_io_allowed(false)
        .with_network_allowed(false);

    let mut custom_sandboxed = Runtime::with_config(ExecutionMode::Interpreter, config);

    let result = custom_sandboxed
        .eval("fn factorial(n: number) -> number { if (n <= 1) { return 1; } else { return n * factorial(n - 1); } } factorial(10)")
        .expect("Failed");
    println!("   factorial(10) = {}", result);

    // Permissive runtime for comparison
    println!("\n3. Permissive runtime (default config):");
    let mut permissive = Runtime::new(ExecutionMode::VM);

    let result = permissive.eval("10 * 5").expect("Failed");
    println!("   10 * 5 = {}", result);

    println!("\nNote: Timeout and memory limits are configured but not yet enforced.");
    println!("IO/network restrictions control the SecurityContext.");
}
