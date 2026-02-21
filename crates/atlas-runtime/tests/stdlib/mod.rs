//! Common utilities for stdlib interpreter tests

pub mod array_intrinsics;
pub mod array_pure;
pub mod math_basic;
pub mod math_trig;
pub mod math_utils_constants;
use atlas_runtime::runtime::Atlas;
use atlas_runtime::value::Value;

/// Helper to evaluate Atlas source code using interpreter
pub fn eval(source: &str) -> Result<Value, atlas_runtime::value::RuntimeError> {
    let runtime = Atlas::new();
    runtime.eval(source)
}

/// Helper to evaluate and unwrap result
pub fn eval_ok(source: &str) -> Value {
    eval(source).unwrap()
}

/// Helper to evaluate and expect error
pub fn eval_err(source: &str) -> atlas_runtime::value::RuntimeError {
    eval(source).unwrap_err()
}
