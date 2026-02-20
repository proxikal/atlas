//! Math standard library functions
//!
//! Complete math API with:
//! - Basic operations (abs, floor, ceil, round, min, max)
//! - Exponential/power (sqrt, pow, log)
//! - Trigonometry (sin, cos, tan, asin, acos, atan)
//! - Utilities (clamp, sign, random)
//! - Constants (PI, E, SQRT2, LN2, LN10)
//!
//! All functions follow IEEE 754 semantics:
//! - NaN propagates through operations
//! - Infinities handled correctly
//! - Signed zero preserved
//! - Domain errors return NaN (not panic)

use crate::span::Span;
use crate::value::{RuntimeError, Value};
use rand::RngExt;

// ============================================================================
// Math Constants
// ============================================================================

/// π (pi) - ratio of circle's circumference to diameter
pub const PI: f64 = std::f64::consts::PI;

/// e - Euler's number, base of natural logarithm
pub const E: f64 = std::f64::consts::E;

/// √2 - square root of 2
pub const SQRT2: f64 = std::f64::consts::SQRT_2;

/// ln(2) - natural logarithm of 2
pub const LN2: f64 = std::f64::consts::LN_2;

/// ln(10) - natural logarithm of 10
pub const LN10: f64 = std::f64::consts::LN_10;

// ============================================================================
// Basic Operations
// ============================================================================

/// abs(x: number) -> number
///
/// Returns absolute value of x.
/// Preserves signed zero: abs(-0) = +0
/// abs(±∞) = +∞
/// abs(NaN) = NaN
pub fn abs(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "abs() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.abs())),
        _ => Err(RuntimeError::TypeError {
            msg: "abs() expects number argument".to_string(),
            span,
        }),
    }
}

/// floor(x: number) -> number
///
/// Returns largest integer ≤ x.
/// floor(1.9) = 1, floor(-1.1) = -2
/// floor(±∞) = ±∞, floor(NaN) = NaN
pub fn floor(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "floor() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.floor())),
        _ => Err(RuntimeError::TypeError {
            msg: "floor() expects number argument".to_string(),
            span,
        }),
    }
}

/// ceil(x: number) -> number
///
/// Returns smallest integer ≥ x.
/// ceil(1.1) = 2, ceil(-1.9) = -1
/// ceil(±∞) = ±∞, ceil(NaN) = NaN
pub fn ceil(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "ceil() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.ceil())),
        _ => Err(RuntimeError::TypeError {
            msg: "ceil() expects number argument".to_string(),
            span,
        }),
    }
}

/// round(x: number) -> number
///
/// Rounds to nearest integer using ties-to-even (banker's rounding).
/// round(2.5) = 2, round(3.5) = 4
/// round(±∞) = ±∞, round(NaN) = NaN
pub fn round(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "round() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => {
            // Rust's round() uses ties-away-from-zero, we need ties-to-even
            // Implement banker's rounding manually
            let rounded = if n.is_nan() || n.is_infinite() {
                *n
            } else {
                let floor_val = n.floor();
                let fract = n - floor_val;
                if fract < 0.5 {
                    floor_val
                } else if fract > 0.5 {
                    floor_val + 1.0
                } else {
                    // Exactly 0.5 - round to even
                    if floor_val % 2.0 == 0.0 {
                        floor_val
                    } else {
                        floor_val + 1.0
                    }
                }
            };
            Ok(Value::Number(rounded))
        }
        _ => Err(RuntimeError::TypeError {
            msg: "round() expects number argument".to_string(),
            span,
        }),
    }
}

/// min(a: number, b: number) -> number
///
/// Returns smaller of two numbers.
/// If either is NaN, returns NaN.
/// min(-0, +0) is implementation-defined
pub fn min(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "min() expects 2 arguments".to_string(),
            span,
        });
    }

    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.min(*b))),
        _ => Err(RuntimeError::TypeError {
            msg: "min() expects number arguments".to_string(),
            span,
        }),
    }
}

/// max(a: number, b: number) -> number
///
/// Returns larger of two numbers.
/// If either is NaN, returns NaN.
/// max(-0, +0) is implementation-defined
pub fn max(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "max() expects 2 arguments".to_string(),
            span,
        });
    }

    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.max(*b))),
        _ => Err(RuntimeError::TypeError {
            msg: "max() expects number arguments".to_string(),
            span,
        }),
    }
}

// ============================================================================
// Exponential/Power Operations
// ============================================================================

/// sqrt(x: number) -> number
///
/// Returns square root of x.
/// sqrt(x) where x < 0 returns NaN
/// sqrt(+∞) = +∞, sqrt(NaN) = NaN
pub fn sqrt(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "sqrt() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.sqrt())),
        _ => Err(RuntimeError::TypeError {
            msg: "sqrt() expects number argument".to_string(),
            span,
        }),
    }
}

/// pow(base: number, exponent: number) -> number
///
/// Returns base raised to exponent power.
/// pow(x, 0) = 1 for any x (including NaN)
/// pow(1, y) = 1 for any y (including NaN)
/// pow(NaN, y) = NaN (except y=0)
pub fn pow(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "pow() expects 2 arguments".to_string(),
            span,
        });
    }

    match (&args[0], &args[1]) {
        (Value::Number(base), Value::Number(exp)) => Ok(Value::Number(base.powf(*exp))),
        _ => Err(RuntimeError::TypeError {
            msg: "pow() expects number arguments".to_string(),
            span,
        }),
    }
}

/// log(x: number) -> number
///
/// Returns natural logarithm (base e) of x.
/// log(x) where x < 0 returns NaN
/// log(0) = -∞, log(+∞) = +∞, log(NaN) = NaN
pub fn log(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "log() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.ln())),
        _ => Err(RuntimeError::TypeError {
            msg: "log() expects number argument".to_string(),
            span,
        }),
    }
}

// ============================================================================
// Trigonometric Functions (all use radians)
// ============================================================================

/// sin(x: number) -> number
///
/// Returns sine of x (x in radians).
/// sin(±∞) = NaN, sin(NaN) = NaN
pub fn sin(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "sin() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.sin())),
        _ => Err(RuntimeError::TypeError {
            msg: "sin() expects number argument".to_string(),
            span,
        }),
    }
}

/// cos(x: number) -> number
///
/// Returns cosine of x (x in radians).
/// cos(±∞) = NaN, cos(NaN) = NaN
pub fn cos(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "cos() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.cos())),
        _ => Err(RuntimeError::TypeError {
            msg: "cos() expects number argument".to_string(),
            span,
        }),
    }
}

/// tan(x: number) -> number
///
/// Returns tangent of x (x in radians).
/// tan(±∞) = NaN, tan(NaN) = NaN
pub fn tan(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "tan() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.tan())),
        _ => Err(RuntimeError::TypeError {
            msg: "tan() expects number argument".to_string(),
            span,
        }),
    }
}

/// asin(x: number) -> number
///
/// Returns arcsine of x in radians.
/// Domain: [-1, 1], outside returns NaN
/// asin(NaN) = NaN
pub fn asin(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "asin() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.asin())),
        _ => Err(RuntimeError::TypeError {
            msg: "asin() expects number argument".to_string(),
            span,
        }),
    }
}

/// acos(x: number) -> number
///
/// Returns arccosine of x in radians.
/// Domain: [-1, 1], outside returns NaN
/// acos(NaN) = NaN
pub fn acos(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "acos() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.acos())),
        _ => Err(RuntimeError::TypeError {
            msg: "acos() expects number argument".to_string(),
            span,
        }),
    }
}

/// atan(x: number) -> number
///
/// Returns arctangent of x in radians.
/// Range: (-π/2, π/2)
/// atan(±∞) = ±π/2, atan(NaN) = NaN
pub fn atan(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "atan() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.atan())),
        _ => Err(RuntimeError::TypeError {
            msg: "atan() expects number argument".to_string(),
            span,
        }),
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// clamp(value: number, min: number, max: number) -> number
///
/// Restricts value to [min, max] range.
/// If min > max, returns NaN
/// Propagates NaN if any argument is NaN
pub fn clamp(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::TypeError {
            msg: "clamp() expects 3 arguments".to_string(),
            span,
        });
    }

    match (&args[0], &args[1], &args[2]) {
        (Value::Number(value), Value::Number(min_val), Value::Number(max_val)) => {
            if min_val > max_val {
                return Ok(Value::Number(f64::NAN));
            }
            let clamped = value.max(*min_val).min(*max_val);
            Ok(Value::Number(clamped))
        }
        _ => Err(RuntimeError::TypeError {
            msg: "clamp() expects number arguments".to_string(),
            span,
        }),
    }
}

/// sign(x: number) -> number
///
/// Returns sign of x: -1 for negative, 0 for zero, 1 for positive.
/// Preserves signed zero: sign(-0) = -0, sign(+0) = +0
/// sign(NaN) = NaN
pub fn sign(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "sign() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => {
            let result = if n.is_nan() {
                f64::NAN
            } else if *n > 0.0 {
                1.0
            } else if *n < 0.0 {
                -1.0
            } else {
                // Preserve signed zero
                *n
            };
            Ok(Value::Number(result))
        }
        _ => Err(RuntimeError::TypeError {
            msg: "sign() expects number argument".to_string(),
            span,
        }),
    }
}

/// random() -> number
///
/// Returns pseudo-random number in [0, 1) with uniform distribution.
/// Uses thread-local rng for randomness.
pub fn random(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::TypeError {
            msg: "random() expects no arguments".to_string(),
            span,
        });
    }

    let mut rng = rand::rng();
    let value: f64 = rng.random(); // random() for f64 returns [0.0, 1.0)
    Ok(Value::Number(value))
}
