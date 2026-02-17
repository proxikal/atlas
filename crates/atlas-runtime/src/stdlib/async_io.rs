//! Async I/O operations for Atlas
//!
//! Provides async file and network I/O operations that return Futures.
//! All operations are non-blocking and can be composed using Future combinators.

use crate::async_runtime::{block_on, AtlasFuture};
use crate::security::SecurityContext;
use crate::span::Span;
use crate::stdlib::http::{HttpRequest, HttpResponse};
use crate::value::{RuntimeError, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;

// ============================================================================
// Async File Operations
// ============================================================================

/// Read file asynchronously
///
/// Args:
/// - path: string (file path)
///
/// Returns: Future<string> (file contents)
///
/// Checks read permission before starting async operation.
/// UTF-8 validation is performed. Returns rejected future on error.
///
/// Example:
/// ```atlas
/// let future = readFileAsync("data.txt");
/// let content = await(future); // hypothetical await
/// ```
pub fn read_file_async(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let path_str = match &args[0] {
        Value::String(s) => s.to_string(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
    };

    // Permission check must happen synchronously before spawning task
    let path = PathBuf::from(&path_str);
    let abs_path = path.canonicalize().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to resolve path '{}': {}", path_str, e),
        span,
    })?;

    security.check_filesystem_read(&abs_path).map_err(|_| {
        RuntimeError::FilesystemPermissionDenied {
            operation: "file read".to_string(),
            path: abs_path.display().to_string(),
            span,
        }
    })?;

    // Spawn async task and block until complete (current_thread runtime limitation)
    let future = AtlasFuture::new_pending();
    let future_clone = future.clone();

    let task = async move {
        match fs::read_to_string(&abs_path).await {
            Ok(contents) => future_clone.resolve(Value::string(contents)),
            Err(e) => future_clone.reject(Value::string(format!(
                "Failed to read file '{}': {}",
                abs_path.display(),
                e
            ))),
        }
    };

    // Execute the async task
    block_on(task);

    Ok(Value::Future(Arc::new(future)))
}

/// Write file asynchronously
///
/// Args:
/// - path: string (file path)
/// - content: string (file content)
///
/// Returns: Future<null> (completes when done)
///
/// Checks write permission before starting async operation.
/// Creates parent directories if needed.
pub fn write_file_async(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let path_str = match &args[0] {
        Value::String(s) => s.to_string(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
    };

    let contents = match &args[1] {
        Value::String(s) => s.to_string(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
    };

    // Permission check
    let path = PathBuf::from(&path_str);
    let check_path = if path.exists() {
        path.canonicalize().map_err(|e| RuntimeError::IoError {
            message: format!("Failed to resolve path '{}': {}", path_str, e),
            span,
        })?
    } else {
        // Find first existing ancestor for permission check
        let mut check_parent = path.parent().unwrap_or_else(|| Path::new("."));
        while !check_parent.exists() && check_parent.parent().is_some() {
            check_parent = check_parent.parent().unwrap();
        }
        if !check_parent.exists() {
            check_parent = Path::new(".");
        }
        check_parent
            .canonicalize()
            .map_err(|e| RuntimeError::IoError {
                message: format!("Failed to resolve parent path: {}", e),
                span,
            })?
    };

    security.check_filesystem_write(&check_path).map_err(|_| {
        RuntimeError::FilesystemPermissionDenied {
            operation: "file write".to_string(),
            path: check_path.display().to_string(),
            span,
        }
    })?;

    // Spawn async task
    let future = AtlasFuture::new_pending();
    let future_clone = future.clone();
    let write_path = path.clone();

    let task = async move {
        // Create parent directory if needed
        if let Some(parent) = write_path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent).await {
                    future_clone.reject(Value::string(format!(
                        "Failed to create parent directory: {}",
                        e
                    )));
                    return;
                }
            }
        }

        match fs::write(&write_path, contents).await {
            Ok(_) => future_clone.resolve(Value::Null),
            Err(e) => future_clone.reject(Value::string(format!(
                "Failed to write file '{}': {}",
                write_path.display(),
                e
            ))),
        }
    };

    block_on(task);
    Ok(Value::Future(Arc::new(future)))
}

/// Append to file asynchronously
///
/// Args:
/// - path: string (file path)
/// - content: string (content to append)
///
/// Returns: Future<null> (completes when done)
///
/// Creates file if it doesn't exist.
pub fn append_file_async(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let path_str = match &args[0] {
        Value::String(s) => s.to_string(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
    };

    let contents = match &args[1] {
        Value::String(s) => s.to_string(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
    };

    // Permission check
    let path = PathBuf::from(&path_str);
    let check_path = if path.exists() {
        path.canonicalize().map_err(|e| RuntimeError::IoError {
            message: format!("Failed to resolve path '{}': {}", path_str, e),
            span,
        })?
    } else {
        // Find first existing ancestor for permission check
        let mut check_parent = path.parent().unwrap_or_else(|| Path::new("."));
        while !check_parent.exists() && check_parent.parent().is_some() {
            check_parent = check_parent.parent().unwrap();
        }
        if !check_parent.exists() {
            check_parent = Path::new(".");
        }
        check_parent
            .canonicalize()
            .map_err(|e| RuntimeError::IoError {
                message: format!("Failed to resolve parent path: {}", e),
                span,
            })?
    };

    security.check_filesystem_write(&check_path).map_err(|_| {
        RuntimeError::FilesystemPermissionDenied {
            operation: "file write".to_string(),
            path: check_path.display().to_string(),
            span,
        }
    })?;

    // Spawn async task
    let future = AtlasFuture::new_pending();
    let future_clone = future.clone();
    let append_path = path.clone();

    let task = async move {
        match fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&append_path)
            .await
        {
            Ok(mut file) => {
                match file.write_all(contents.as_bytes()).await {
                    Ok(_) => {
                        // Flush to ensure data is written
                        if let Err(e) = file.flush().await {
                            future_clone.reject(Value::string(format!(
                                "Failed to flush file '{}': {}",
                                append_path.display(),
                                e
                            )));
                        } else {
                            future_clone.resolve(Value::Null);
                        }
                    }
                    Err(e) => future_clone.reject(Value::string(format!(
                        "Failed to append to file '{}': {}",
                        append_path.display(),
                        e
                    ))),
                }
            }
            Err(e) => future_clone.reject(Value::string(format!(
                "Failed to open file '{}': {}",
                append_path.display(),
                e
            ))),
        }
    };

    block_on(task);
    Ok(Value::Future(Arc::new(future)))
}

// ============================================================================
// Async HTTP Operations
// ============================================================================

/// Execute HTTP request asynchronously
///
/// Args:
/// - request: HttpRequest
///
/// Returns: Future<HttpResponse> (resolves to response or rejects with error)
///
/// Uses reqwest's async client for non-blocking network I/O.
/// Supports all HTTP methods, headers, body, timeout, and redirects.
pub fn http_send_async(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpSendAsync: expected 1 argument (request)".to_string(),
            span,
        });
    }

    let request = match &args[0] {
        Value::HttpRequest(req) => req.clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "httpSendAsync: argument must be HttpRequest".to_string(),
                span,
            })
        }
    };

    let future = AtlasFuture::new_pending();
    let future_clone = future.clone();

    let task = async move {
        // Build reqwest client (async version)
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(request.timeout_secs()))
            .redirect(if request.follow_redirects() {
                reqwest::redirect::Policy::limited(request.max_redirects() as usize)
            } else {
                reqwest::redirect::Policy::none()
            })
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                future_clone.reject(Value::string(format!(
                    "Failed to create HTTP client: {}",
                    e
                )));
                return;
            }
        };

        // Build URL with query parameters
        let url = request.build_url();

        // Build request with proper method
        let mut req_builder = match request.method() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            "PATCH" => client.patch(&url),
            method => {
                future_clone.reject(Value::string(format!(
                    "Unsupported HTTP method: {}",
                    method
                )));
                return;
            }
        };

        // Add headers
        for (key, value) in request.headers() {
            req_builder = req_builder.header(key, value);
        }

        // Add body if present
        if let Some(body) = request.body() {
            req_builder = req_builder.body(body.clone());
        }

        // Execute request
        let response = match req_builder.send().await {
            Ok(r) => r,
            Err(e) => {
                let error_msg = if e.is_timeout() {
                    format!("Request timeout after {} seconds", request.timeout_secs())
                } else if e.is_connect() {
                    format!("Connection error: {}", e)
                } else {
                    format!("Network error: {}", e)
                };
                future_clone.reject(Value::string(error_msg));
                return;
            }
        };

        // Extract response data
        let status = response.status().as_u16();
        let final_url = response.url().to_string();

        let mut headers_map = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                headers_map.insert(key.to_string(), value_str.to_string());
            }
        }

        let body = match response.text().await {
            Ok(b) => b,
            Err(e) => {
                future_clone.reject(Value::string(format!(
                    "Failed to read response body: {}",
                    e
                )));
                return;
            }
        };

        let http_response = HttpResponse::new(status, headers_map, body, final_url);
        future_clone.resolve(Value::HttpResponse(Arc::new(http_response)));
    };

    block_on(task);
    Ok(Value::Future(Arc::new(future)))
}

/// GET request asynchronously
///
/// Args:
/// - url: string
///
/// Returns: Future<HttpResponse>
pub fn http_get_async(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpGetAsync: expected 1 argument (url)".to_string(),
            span,
        });
    }

    let url = match &args[0] {
        Value::String(s) => s.to_string(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "httpGetAsync: argument must be string".to_string(),
                span,
            })
        }
    };

    // Create GET request
    let request = HttpRequest::new("GET".to_string(), url);
    http_send_async(&[Value::HttpRequest(Arc::new(request))], span)
}

/// POST request asynchronously
///
/// Args:
/// - url: string
/// - body: string
///
/// Returns: Future<HttpResponse>
pub fn http_post_async(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpPostAsync: expected 2 arguments (url, body)".to_string(),
            span,
        });
    }

    let url = match &args[0] {
        Value::String(s) => s.to_string(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "httpPostAsync: url must be string".to_string(),
                span,
            })
        }
    };

    let body = match &args[1] {
        Value::String(s) => s.to_string(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "httpPostAsync: body must be string".to_string(),
                span,
            })
        }
    };

    let request = HttpRequest::new("POST".to_string(), url).with_body(body);
    http_send_async(&[Value::HttpRequest(Arc::new(request))], span)
}

/// PUT request asynchronously
///
/// Args:
/// - url: string
/// - body: string
///
/// Returns: Future<HttpResponse>
pub fn http_put_async(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpPutAsync: expected 2 arguments (url, body)".to_string(),
            span,
        });
    }

    let url = match &args[0] {
        Value::String(s) => s.to_string(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "httpPutAsync: url must be string".to_string(),
                span,
            })
        }
    };

    let body = match &args[1] {
        Value::String(s) => s.to_string(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "httpPutAsync: body must be string".to_string(),
                span,
            })
        }
    };

    let request = HttpRequest::new("PUT".to_string(), url).with_body(body);
    http_send_async(&[Value::HttpRequest(Arc::new(request))], span)
}

/// DELETE request asynchronously
///
/// Args:
/// - url: string
///
/// Returns: Future<HttpResponse>
pub fn http_delete_async(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpDeleteAsync: expected 1 argument (url)".to_string(),
            span,
        });
    }

    let url = match &args[0] {
        Value::String(s) => s.to_string(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "httpDeleteAsync: argument must be string".to_string(),
                span,
            })
        }
    };

    let request = HttpRequest::new("DELETE".to_string(), url);
    http_send_async(&[Value::HttpRequest(Arc::new(request))], span)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Poll a future to drive it to completion (for testing/development)
///
/// This is a utility function to help work with pending futures in the
/// current synchronous execution model. Future phases will provide proper
/// async/await syntax.
///
/// Args:
/// - future: Future
///
/// Returns: value (resolved value or error)
pub fn await_future(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "await: expected 1 argument (future)".to_string(),
            span,
        });
    }

    let future = match &args[0] {
        Value::Future(f) => f.clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "await: argument must be Future".to_string(),
                span,
            })
        }
    };

    // For now, futures are immediately resolved due to block_on usage
    // In future phases with true async, this would poll/wait for completion
    match future.get_state() {
        crate::async_runtime::FutureState::Resolved(value) => Ok(value),
        crate::async_runtime::FutureState::Rejected(error) => Err(RuntimeError::TypeError {
            msg: format!("Future rejected: {}", error),
            span,
        }),
        crate::async_runtime::FutureState::Pending => {
            // This shouldn't happen with current block_on implementation
            Err(RuntimeError::TypeError {
                msg: "Future is still pending".to_string(),
                span,
            })
        }
    }
}
