//! Public embedding API for Atlas runtime
//!
//! This module provides a comprehensive API for embedding Atlas in Rust applications.
//! It includes:
//! - Runtime execution with mode selection (Interpreter or VM)
//! - Value conversion between Rust and Atlas types
//! - Function calling and global variable management
//! - Comprehensive error handling
//!
//! # Examples
//!
//! ```
//! use atlas_runtime::api::{Runtime, ExecutionMode};
//!
//! // Create a runtime with interpreter mode
//! let mut runtime = Runtime::new(ExecutionMode::Interpreter);
//!
//! // Evaluate code
//! let result = runtime.eval("1 + 2").unwrap();
//!
//! // Call Atlas functions from Rust
//! runtime.eval("fn add(x: number, y: number) -> number { x + y }").unwrap();
//! let result = runtime.call("add", vec![1.0.into(), 2.0.into()]).unwrap();
//! ```

pub mod conversion;
pub mod runtime;

// Re-export main types for convenience
pub use conversion::{ConversionError, FromAtlas, ToAtlas};
pub use runtime::{EvalError, ExecutionMode, Runtime};
