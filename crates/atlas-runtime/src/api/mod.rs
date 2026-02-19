//! Public embedding API for Atlas runtime
//!
//! This module provides a comprehensive API for embedding Atlas in Rust applications.
//! It includes:
//! - Runtime execution with mode selection (Interpreter or VM)
//! - Value conversion between Rust and Atlas types
//! - Native function registration
//! - Function calling and global variable management
//! - Comprehensive error handling
//!
//! # Examples
//!
//! ```rust,no_run
//! use atlas_runtime::api::{Runtime, ExecutionMode};
//! use atlas_runtime::value::Value;
//!
//! // Create a runtime with interpreter mode
//! let mut runtime = Runtime::new(ExecutionMode::Interpreter);
//!
//! // Evaluate code
//! let result = runtime.eval("1 + 2").unwrap();
//!
//! // Call Atlas functions from Rust
//! runtime.eval("fn add(x: number, y: number) -> number { x + y }").unwrap();
//! let result = runtime.call("add", vec![Value::Number(1.0), Value::Number(2.0)]).unwrap();
//! ```

pub mod config;
pub mod conversion;
pub mod native;
pub mod runtime;

// Re-export main types for convenience
pub use config::RuntimeConfig;
pub use conversion::{ConversionError, FromAtlas, ToAtlas};
pub use native::{BuildError, NativeFunctionBuilder};
pub use runtime::{EvalError, ExecutionMode, Runtime};
