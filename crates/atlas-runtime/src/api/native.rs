//! Native function registration and builder
//!
//! This module provides infrastructure for registering Rust closures as callable
//! native functions in Atlas code. Native functions can be registered with fixed
//! arity (specific argument count) or variadic (any argument count).
//!
//! # Examples
//!
//! ```rust,no_run
//! # use atlas_runtime::value::{Value, RuntimeError};
//! # use atlas_runtime::span::Span;
//! use atlas_runtime::api::native::NativeFunctionBuilder;
//!
//! // Fixed arity function (2 arguments)
//! let add_fn = NativeFunctionBuilder::new("add")
//!     .with_arity(2)
//!     .with_implementation(|args| {
//!         let a = match &args[0] {
//!             Value::Number(n) => *n,
//!             _ => return Err(RuntimeError::TypeError {
//!                 msg: "Expected number".to_string(),
//!                 span: Span::dummy()
//!             }),
//!         };
//!         let b = match &args[1] {
//!             Value::Number(n) => *n,
//!             _ => return Err(RuntimeError::TypeError {
//!                 msg: "Expected number".to_string(),
//!                 span: Span::dummy()
//!             }),
//!         };
//!         Ok(Value::Number(a + b))
//!     })
//!     .build();
//!
//! // Variadic function (any number of arguments)
//! let sum_fn = NativeFunctionBuilder::new("sum")
//!     .variadic()
//!     .with_implementation(|args| {
//!         let mut total = 0.0;
//!         for arg in args {
//!             match arg {
//!                 Value::Number(n) => total += n,
//!                 _ => return Err(RuntimeError::TypeError {
//!                     msg: "All arguments must be numbers".to_string(),
//!                     span: Span::dummy()
//!                 }),
//!             }
//!         }
//!         Ok(Value::Number(total))
//!     })
//!     .build();
//! ```

use crate::value::{NativeFn, RuntimeError, Value};
use std::sync::Arc;

/// Type alias for native function implementation
type NativeFnImpl = Box<dyn Fn(&[Value]) -> Result<Value, RuntimeError> + Send + Sync>;

/// Builder for constructing native functions with arity validation
///
/// Provides a fluent API for creating native functions that can be called from Atlas code.
/// Supports both fixed-arity functions (must be called with exact argument count) and
/// variadic functions (can be called with any number of arguments).
pub struct NativeFunctionBuilder {
    name: String,
    arity: Option<usize>,
    is_variadic: bool,
    implementation: Option<NativeFnImpl>,
}

impl NativeFunctionBuilder {
    /// Create a new native function builder with the given name
    ///
    /// # Arguments
    ///
    /// * `name` - Function name (for error messages and debugging)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use atlas_runtime::api::native::NativeFunctionBuilder;
    /// # use atlas_runtime::value::{Value, RuntimeError};
    /// # use atlas_runtime::span::Span;
    /// let builder = NativeFunctionBuilder::new("my_function");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arity: None,
            is_variadic: false,
            implementation: None,
        }
    }

    /// Set the function's arity (required argument count)
    ///
    /// When arity is set, the function will automatically validate that it receives
    /// exactly this many arguments. Calls with too few or too many arguments will
    /// result in a runtime error.
    ///
    /// Cannot be combined with `variadic()`.
    ///
    /// # Arguments
    ///
    /// * `arity` - Required number of arguments
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use atlas_runtime::api::native::NativeFunctionBuilder;
    /// # use atlas_runtime::value::{Value, RuntimeError};
    /// # use atlas_runtime::span::Span;
    /// let builder = NativeFunctionBuilder::new("add")
    ///     .with_arity(2);  // Requires exactly 2 arguments
    /// ```
    pub fn with_arity(mut self, arity: usize) -> Self {
        self.arity = Some(arity);
        self.is_variadic = false;
        self
    }

    /// Mark this function as variadic (accepts any number of arguments)
    ///
    /// Variadic functions receive an array of arguments and are responsible for
    /// validating the argument count and types themselves.
    ///
    /// Cannot be combined with `with_arity()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use atlas_runtime::api::native::NativeFunctionBuilder;
    /// # use atlas_runtime::value::{Value, RuntimeError};
    /// # use atlas_runtime::span::Span;
    /// let builder = NativeFunctionBuilder::new("sum")
    ///     .variadic();  // Accepts any number of arguments
    /// ```
    pub fn variadic(mut self) -> Self {
        self.is_variadic = true;
        self.arity = None;
        self
    }

    /// Set the function implementation
    ///
    /// The implementation is a closure that receives an array of Values and returns
    /// either a Value or a RuntimeError. For fixed-arity functions, the argument
    /// count has already been validated when this closure is called.
    ///
    /// # Arguments
    ///
    /// * `implementation` - Closure implementing the function logic
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use atlas_runtime::api::native::NativeFunctionBuilder;
    /// # use atlas_runtime::value::{Value, RuntimeError};
    /// # use atlas_runtime::span::Span;
    /// let builder = NativeFunctionBuilder::new("negate")
    ///     .with_arity(1)
    ///     .with_implementation(|args| {
    ///         match &args[0] {
    ///             Value::Number(n) => Ok(Value::Number(-n)),
    ///             _ => Err(RuntimeError::TypeError {
    ///                 msg: "Expected number".to_string(),
    ///                 span: Span::dummy()
    ///             }),
    ///         }
    ///     });
    /// ```
    pub fn with_implementation<F>(mut self, implementation: F) -> Self
    where
        F: Fn(&[Value]) -> Result<Value, RuntimeError> + Send + Sync + 'static,
    {
        self.implementation = Some(Box::new(implementation));
        self
    }

    /// Build the native function value
    ///
    /// Wraps the implementation with arity validation (if specified) and returns
    /// a Value::NativeFunction that can be registered in the runtime.
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - Native function value ready to register
    /// * `Err(BuildError)` - If implementation was not provided
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use atlas_runtime::api::native::NativeFunctionBuilder;
    /// # use atlas_runtime::value::{Value, RuntimeError};
    /// # use atlas_runtime::span::Span;
    /// let native_fn = NativeFunctionBuilder::new("identity")
    ///     .with_arity(1)
    ///     .with_implementation(|args| Ok(args[0].clone()))
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(self) -> Result<Value, BuildError> {
        let implementation = self
            .implementation
            .ok_or_else(|| BuildError::MissingImplementation(self.name.clone()))?;

        let name = self.name.clone();

        // Wrap implementation with arity validation if fixed arity
        let wrapped_fn: NativeFn = if let Some(expected_arity) = self.arity {
            Arc::new(move |args: &[Value]| {
                // Validate argument count
                if args.len() != expected_arity {
                    return Err(RuntimeError::TypeError {
                        msg: format!(
                            "Function '{}' expects {} argument{}, got {}",
                            name,
                            expected_arity,
                            if expected_arity == 1 { "" } else { "s" },
                            args.len()
                        ),
                        span: crate::span::Span::dummy(),
                    });
                }

                // Call implementation
                implementation(args)
            })
        } else {
            // Variadic function - no arity validation
            Arc::new(move |args: &[Value]| implementation(args))
        };

        Ok(Value::NativeFunction(wrapped_fn))
    }
}

/// Errors that can occur when building a native function
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    /// No implementation was provided
    MissingImplementation(String),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::MissingImplementation(name) => {
                write!(f, "Native function '{}' missing implementation", name)
            }
        }
    }
}

impl std::error::Error for BuildError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    #[test]
    fn test_builder_fixed_arity() {
        let add_fn = NativeFunctionBuilder::new("add")
            .with_arity(2)
            .with_implementation(|args| {
                let a = match &args[0] {
                    Value::Number(n) => *n,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "Expected number".to_string(),
                            span: Span::dummy(),
                        })
                    }
                };
                let b = match &args[1] {
                    Value::Number(n) => *n,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "Expected number".to_string(),
                            span: Span::dummy(),
                        })
                    }
                };
                Ok(Value::Number(a + b))
            })
            .build()
            .unwrap();

        // Extract the native function
        if let Value::NativeFunction(func) = add_fn {
            // Test correct arity
            let result = func(&[Value::Number(10.0), Value::Number(20.0)]).unwrap();
            assert_eq!(result, Value::Number(30.0));

            // Test wrong arity (too few)
            let err = func(&[Value::Number(10.0)]).unwrap_err();
            match err {
                RuntimeError::TypeError { msg, .. } => {
                    assert!(msg.contains("expects 2 arguments, got 1"));
                }
                _ => panic!("Expected TypeError"),
            }

            // Test wrong arity (too many)
            let err = func(&[
                Value::Number(10.0),
                Value::Number(20.0),
                Value::Number(30.0),
            ])
            .unwrap_err();
            match err {
                RuntimeError::TypeError { msg, .. } => {
                    assert!(msg.contains("expects 2 arguments, got 3"));
                }
                _ => panic!("Expected TypeError"),
            }
        } else {
            panic!("Expected NativeFunction value");
        }
    }

    #[test]
    fn test_builder_variadic() {
        let sum_fn = NativeFunctionBuilder::new("sum")
            .variadic()
            .with_implementation(|args| {
                let mut total = 0.0;
                for arg in args {
                    match arg {
                        Value::Number(n) => total += n,
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "All arguments must be numbers".to_string(),
                                span: Span::dummy(),
                            })
                        }
                    }
                }
                Ok(Value::Number(total))
            })
            .build()
            .unwrap();

        if let Value::NativeFunction(func) = sum_fn {
            // Test 0 arguments
            let result = func(&[]).unwrap();
            assert_eq!(result, Value::Number(0.0));

            // Test 1 argument
            let result = func(&[Value::Number(42.0)]).unwrap();
            assert_eq!(result, Value::Number(42.0));

            // Test multiple arguments
            let result = func(&[
                Value::Number(10.0),
                Value::Number(20.0),
                Value::Number(30.0),
            ])
            .unwrap();
            assert_eq!(result, Value::Number(60.0));
        } else {
            panic!("Expected NativeFunction value");
        }
    }

    #[test]
    fn test_builder_missing_implementation() {
        let result = NativeFunctionBuilder::new("test").with_arity(1).build();

        assert!(result.is_err());
        match result.unwrap_err() {
            BuildError::MissingImplementation(name) => {
                assert_eq!(name, "test");
            }
        }
    }
}
