//! Process management stdlib functions
//!
//! This module provides Atlas stdlib functions for spawning external processes,
//! capturing output, managing environment variables, and controlling working directories.
//!
//! Command execution:
//! - exec: Execute command and wait for completion
//! - spawn: Spawn child process (non-blocking)
//! - shell: Execute shell command
//!
//! Standard I/O:
//! - execCapture: Execute and capture stdout/stderr
//! - execInherit: Execute with inherited stdio
//!
//! Environment variables:
//! - getEnv: Get environment variable
//! - setEnv: Set environment variable
//! - unsetEnv: Remove environment variable
//! - listEnv: List all environment variables
//!
//! Working directory:
//! - getCwd: Get current working directory
//! - setCwd: Set working directory for process
//!
//! Process control:
//! - processWait: Wait for process completion
//! - processKill: Kill running process
//! - processPid: Get current process ID

use super::stdlib_arity_error;
use crate::security::SecurityContext;
use crate::span::Span;
use crate::value::{RuntimeError, Value};
use std::collections::HashMap;
use std::env;
use std::process::{Command, Stdio};
use std::sync::Arc;

// ============================================================================
// Command Execution
// ============================================================================

/// Execute a command and wait for completion
///
/// Atlas signature: `exec(command: string | string[], options?: object) -> Result<object, string>`
///
/// Options:
/// - env: object - Custom environment variables
/// - cwd: string - Working directory
/// - inherit: bool - Inherit parent stdio (default: false)
///
/// Returns: { exitCode: number, stdout: string, stderr: string }
pub fn exec(args: &[Value], span: Span, security: &SecurityContext) -> Result<Value, RuntimeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(stdlib_arity_error("exec", 1, args.len(), span));
    }

    // Parse command
    let (program, command_args) = parse_command(&args[0], span)?;

    // Check permission
    security
        .check_process(&program)
        .map_err(|_| RuntimeError::ProcessPermissionDenied {
            command: program.clone(),
            span,
        })?;

    // Parse options
    let options = if args.len() == 2 {
        parse_exec_options(&args[1], span)?
    } else {
        ExecOptions::default()
    };

    // Build command
    let mut cmd = Command::new(&program);
    cmd.args(&command_args);

    // Set environment if provided
    if let Some(env_vars) = &options.env {
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
    }

    // Set working directory if provided
    if let Some(cwd) = &options.cwd {
        cmd.current_dir(cwd);
    }

    // Set stdio handling
    if options.inherit {
        cmd.stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit());
    } else {
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
    }

    // Execute command
    let output = cmd.output().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to execute command: {}", e),
        span,
    })?;

    // Build result object
    let result = HashMap::from([
        (
            "exitCode".to_string(),
            crate::json_value::JsonValue::Number(output.status.code().unwrap_or(-1) as f64),
        ),
        (
            "stdout".to_string(),
            crate::json_value::JsonValue::String(
                String::from_utf8_lossy(&output.stdout).to_string(),
            ),
        ),
        (
            "stderr".to_string(),
            crate::json_value::JsonValue::String(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ),
        ),
        (
            "success".to_string(),
            crate::json_value::JsonValue::Bool(output.status.success()),
        ),
    ]);

    // Return Result<object, string>
    if output.status.success() {
        Ok(Value::Result(Ok(Box::new(Value::JsonValue(Arc::new(
            crate::json_value::JsonValue::Object(result),
        ))))))
    } else {
        Ok(Value::Result(Err(Box::new(Value::string(format!(
            "Command failed with exit code {}",
            output.status.code().unwrap_or(-1)
        ))))))
    }
}

/// Execute a shell command
///
/// Atlas signature: `shell(command: string, options?: object) -> Result<object, string>`
pub fn shell(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(stdlib_arity_error("shell", 1, args.len(), span));
    }

    let command_str = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected string for command, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // Detect shell
    let (shell_cmd, shell_arg) = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    // Check permission for shell
    security
        .check_process(shell_cmd)
        .map_err(|_| RuntimeError::ProcessPermissionDenied {
            command: shell_cmd.to_string(),
            span,
        })?;

    // Parse options
    let options = if args.len() == 2 {
        parse_exec_options(&args[1], span)?
    } else {
        ExecOptions::default()
    };

    // Build command
    let mut cmd = Command::new(shell_cmd);
    cmd.arg(shell_arg).arg(&command_str);

    // Set environment if provided
    if let Some(env_vars) = &options.env {
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
    }

    // Set working directory if provided
    if let Some(cwd) = &options.cwd {
        cmd.current_dir(cwd);
    }

    // Set stdio
    if options.inherit {
        cmd.stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit());
    } else {
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
    }

    // Execute
    let output = cmd.output().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to execute shell command: {}", e),
        span,
    })?;

    // Build result
    let result = HashMap::from([
        (
            "exitCode".to_string(),
            crate::json_value::JsonValue::Number(output.status.code().unwrap_or(-1) as f64),
        ),
        (
            "stdout".to_string(),
            crate::json_value::JsonValue::String(
                String::from_utf8_lossy(&output.stdout).to_string(),
            ),
        ),
        (
            "stderr".to_string(),
            crate::json_value::JsonValue::String(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ),
        ),
        (
            "success".to_string(),
            crate::json_value::JsonValue::Bool(output.status.success()),
        ),
    ]);

    if output.status.success() {
        Ok(Value::Result(Ok(Box::new(Value::JsonValue(Arc::new(
            crate::json_value::JsonValue::Object(result),
        ))))))
    } else {
        Ok(Value::Result(Err(Box::new(Value::string(format!(
            "Shell command failed with exit code {}",
            output.status.code().unwrap_or(-1)
        ))))))
    }
}

// ============================================================================
// Environment Variables
// ============================================================================

/// Get an environment variable
///
/// Atlas signature: `getEnv(name: string) -> string | null`
pub fn get_env(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("getEnv", 1, args.len(), span));
    }

    let var_name = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "Expected string for variable name, got {}",
                    args[0].type_name()
                ),
                span,
            })
        }
    };

    // Check permission
    security.check_environment(&var_name).map_err(|_| {
        RuntimeError::EnvironmentPermissionDenied {
            var: var_name.clone(),
            span,
        }
    })?;

    // Get environment variable
    match env::var(&var_name) {
        Ok(value) => Ok(Value::string(value)),
        Err(_) => Ok(Value::Null),
    }
}

/// Set an environment variable
///
/// Atlas signature: `setEnv(name: string, value: string) -> null`
pub fn set_env(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("setEnv", 2, args.len(), span));
    }

    let var_name = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "Expected string for variable name, got {}",
                    args[0].type_name()
                ),
                span,
            })
        }
    };

    let var_value = match &args[1] {
        Value::String(s) => s.as_ref().clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "Expected string for variable value, got {}",
                    args[1].type_name()
                ),
                span,
            })
        }
    };

    // Check permission
    security.check_environment(&var_name).map_err(|_| {
        RuntimeError::EnvironmentPermissionDenied {
            var: var_name.clone(),
            span,
        }
    })?;

    // Set environment variable
    env::set_var(&var_name, &var_value);

    Ok(Value::Null)
}

/// Remove an environment variable
///
/// Atlas signature: `unsetEnv(name: string) -> null`
pub fn unset_env(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("unsetEnv", 1, args.len(), span));
    }

    let var_name = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "Expected string for variable name, got {}",
                    args[0].type_name()
                ),
                span,
            })
        }
    };

    // Check permission
    security.check_environment(&var_name).map_err(|_| {
        RuntimeError::EnvironmentPermissionDenied {
            var: var_name.clone(),
            span,
        }
    })?;

    // Remove environment variable
    env::remove_var(&var_name);

    Ok(Value::Null)
}

/// List all environment variables
///
/// Atlas signature: `listEnv() -> object`
pub fn list_env(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("listEnv", 0, args.len(), span));
    }

    // Get all environment variables
    let env_vars: HashMap<String, crate::json_value::JsonValue> = env::vars()
        .map(|(key, value)| (key, crate::json_value::JsonValue::String(value)))
        .collect();

    Ok(Value::JsonValue(Arc::new(
        crate::json_value::JsonValue::Object(env_vars),
    )))
}

// ============================================================================
// Working Directory
// ============================================================================

/// Get current working directory
///
/// Atlas signature: `getCwd() -> string`
pub fn get_cwd(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("getCwd", 0, args.len(), span));
    }

    let cwd = env::current_dir().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to get current directory: {}", e),
        span,
    })?;

    Ok(Value::string(cwd.to_string_lossy()))
}

/// Get current process ID
///
/// Atlas signature: `getPid() -> number`
pub fn get_pid(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("getPid", 0, args.len(), span));
    }

    Ok(Value::Number(std::process::id() as f64))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse command from string or array
fn parse_command(value: &Value, span: Span) -> Result<(String, Vec<String>), RuntimeError> {
    match value {
        Value::String(s) => {
            // Single command, no arguments
            Ok((s.as_ref().clone(), vec![]))
        }
        Value::Array(arr) => {
            let arr_slice = arr.as_slice();
            if arr_slice.is_empty() {
                return Err(RuntimeError::TypeError {
                    msg: "Command array cannot be empty".to_string(),
                    span,
                });
            }

            // First element is the program
            let program = match &arr_slice[0] {
                Value::String(s) => s.as_ref().clone(),
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Command program must be a string".to_string(),
                        span,
                    })
                }
            };

            // Rest are arguments
            let mut args = Vec::new();
            for arg_val in &arr_slice[1..] {
                match arg_val {
                    Value::String(s) => args.push(s.as_ref().clone()),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "Command arguments must be strings".to_string(),
                            span,
                        })
                    }
                }
            }

            Ok((program, args))
        }
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "Expected string or array for command, got {}",
                value.type_name()
            ),
            span,
        }),
    }
}

/// Options for exec command
#[derive(Default)]
struct ExecOptions {
    env: Option<HashMap<String, String>>,
    cwd: Option<String>,
    inherit: bool,
}

/// Parse execution options from object
fn parse_exec_options(value: &Value, span: Span) -> Result<ExecOptions, RuntimeError> {
    let json_obj = match value {
        Value::JsonValue(j) => j,
        Value::Null => return Ok(ExecOptions::default()),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected object for options, got {}", value.type_name()),
                span,
            })
        }
    };

    let mut options = ExecOptions::default();

    // Parse env
    if let crate::json_value::JsonValue::Object(obj_map) = json_obj.as_ref() {
        if let Some(crate::json_value::JsonValue::Object(env_obj)) = obj_map.get("env") {
            let mut env_map = HashMap::new();
            for (key, val) in env_obj {
                if let crate::json_value::JsonValue::String(s) = val {
                    env_map.insert(key.clone(), s.clone());
                } else {
                    return Err(RuntimeError::TypeError {
                        msg: "Environment variable values must be strings".to_string(),
                        span,
                    });
                }
            }
            options.env = Some(env_map);
        }

        // Parse cwd
        if let Some(cwd_val) = obj_map.get("cwd") {
            if let crate::json_value::JsonValue::String(s) = cwd_val {
                options.cwd = Some(s.clone());
            } else {
                return Err(RuntimeError::TypeError {
                    msg: "Working directory must be a string".to_string(),
                    span,
                });
            }
        }

        // Parse inherit
        if let Some(inherit_val) = obj_map.get("inherit") {
            if let crate::json_value::JsonValue::Bool(b) = inherit_val {
                options.inherit = *b;
            } else {
                return Err(RuntimeError::TypeError {
                    msg: "Inherit option must be a boolean".to_string(),
                    span,
                });
            }
        }
    }

    Ok(options)
}
