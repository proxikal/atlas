//! HTTP client standard library functions
//!
//! Provides HTTP request building, execution, and response handling.

use crate::json_value::JsonValue;
use crate::span::Span;
use crate::stdlib::collections::hash::HashKey;
use crate::stdlib::collections::hashmap::AtlasHashMap;
use crate::value::{RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

/// HTTP Request configuration
#[derive(Debug, Clone)]
pub struct HttpRequest {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Option<String>,
    timeout_secs: u64,
    follow_redirects: bool,
    max_redirects: u32,
    query_params: Vec<(String, String)>,
}

impl HttpRequest {
    /// Create new HTTP request
    pub fn new(method: String, url: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "Atlas/0.1".to_string());

        Self {
            method,
            url,
            headers,
            body: None,
            timeout_secs: 30,
            follow_redirects: true,
            max_redirects: 10,
            query_params: Vec::new(),
        }
    }

    /// Set header (returns new request)
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Set body (returns new request)
    pub fn with_body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    /// Set timeout (returns new request)
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_secs = seconds;
        self
    }

    /// Get method
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Get URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Get body
    pub fn body(&self) -> Option<&String> {
        self.body.as_ref()
    }

    /// Get timeout
    pub fn timeout_secs(&self) -> u64 {
        self.timeout_secs
    }

    /// Set follow redirects (returns new request)
    pub fn with_follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    /// Set max redirects (returns new request)
    pub fn with_max_redirects(mut self, max: u32) -> Self {
        self.max_redirects = max;
        self
    }

    /// Add query parameter (returns new request)
    pub fn with_query_param(mut self, key: String, value: String) -> Self {
        self.query_params.push((key, value));
        self
    }

    /// Get query parameters
    pub fn query_params(&self) -> &[(String, String)] {
        &self.query_params
    }

    /// Get follow redirects setting
    pub fn follow_redirects(&self) -> bool {
        self.follow_redirects
    }

    /// Get max redirects
    pub fn max_redirects(&self) -> u32 {
        self.max_redirects
    }

    /// Build full URL with query parameters
    pub fn build_url(&self) -> String {
        if self.query_params.is_empty() {
            self.url.clone()
        } else {
            let query_string: Vec<String> = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect();
            format!("{}?{}", self.url, query_string.join("&"))
        }
    }
}

/// HTTP Response data
#[derive(Debug, Clone)]
pub struct HttpResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
    url: String,
}

impl HttpResponse {
    /// Create new HTTP response
    pub fn new(status: u16, headers: HashMap<String, String>, body: String, url: String) -> Self {
        Self {
            status,
            headers,
            body,
            url,
        }
    }

    /// Get status code
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Get headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Get body
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Get URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Check if response is success (200-299)
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }
}

// ============================================================================
// Request Building Functions
// ============================================================================

/// Create HTTP request with method and URL
///
/// Args:
/// - method: string (HTTP method: GET, POST)
/// - url: string (URL starting with http:// or https://)
///
/// Returns: HttpRequest
///
/// Example:
/// ```
/// let req = httpRequest("GET", "https://httpbin.org/get");
/// ```
pub fn http_request(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpRequest: expected 2 arguments (method, url)".to_string(),
            span,
        });
    }

    let method = expect_string(&args[0], "method", span)?;
    let url = expect_string(&args[1], "url", span)?;

    // Validate URL starts with http:// or https://
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(RuntimeError::TypeError {
            msg: format!(
                "httpRequest: URL must start with http:// or https://, got: {}",
                url
            ),
            span,
        });
    }

    // Validate method (GET, POST, PUT, DELETE, PATCH)
    let method_upper = method.to_uppercase();
    if method_upper != "GET"
        && method_upper != "POST"
        && method_upper != "PUT"
        && method_upper != "DELETE"
        && method_upper != "PATCH"
    {
        return Err(RuntimeError::TypeError {
            msg: format!(
                "httpRequest: method must be GET, POST, PUT, DELETE, or PATCH, got: {}",
                method
            ),
            span,
        });
    }

    let request = HttpRequest::new(method_upper, url);
    Ok(Value::HttpRequest(Rc::new(request)))
}

/// Create GET request (convenience function)
///
/// Args:
/// - url: string
///
/// Returns: HttpRequest
///
/// Example:
/// ```
/// let req = httpRequestGet("https://httpbin.org/get");
/// ```
pub fn http_request_get(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpRequestGet: expected 1 argument (url)".to_string(),
            span,
        });
    }

    http_request(&[Value::string("GET".to_string()), args[0].clone()], span)
}

/// Create POST request with body (convenience function)
///
/// Args:
/// - url: string
/// - body: string
///
/// Returns: HttpRequest
///
/// Example:
/// ```
/// let req = httpRequestPost("https://httpbin.org/post", "data");
/// ```
pub fn http_request_post(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpRequestPost: expected 2 arguments (url, body)".to_string(),
            span,
        });
    }

    let request = http_request(&[Value::string("POST".to_string()), args[0].clone()], span)?;
    http_set_body(&[request, args[1].clone()], span)
}

/// Set header on request
///
/// Args:
/// - request: HttpRequest
/// - key: string (header name)
/// - value: string (header value)
///
/// Returns: HttpRequest (new request with header set)
///
/// Example:
/// ```
/// let req = httpSetHeader(req, "Content-Type", "application/json");
/// ```
pub fn http_set_header(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::TypeError {
            msg: "httpSetHeader: expected 3 arguments (request, key, value)".to_string(),
            span,
        });
    }

    let request = expect_http_request(&args[0], "request", span)?;
    let key = expect_string(&args[1], "key", span)?;
    let value = expect_string(&args[2], "value", span)?;

    let new_request = request.clone().with_header(key, value);
    Ok(Value::HttpRequest(Rc::new(new_request)))
}

/// Set body on request
///
/// Args:
/// - request: HttpRequest
/// - body: string
///
/// Returns: HttpRequest (new request with body set)
///
/// Example:
/// ```
/// let req = httpSetBody(req, "request data");
/// ```
pub fn http_set_body(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpSetBody: expected 2 arguments (request, body)".to_string(),
            span,
        });
    }

    let request = expect_http_request(&args[0], "request", span)?;
    let body = expect_string(&args[1], "body", span)?;

    let new_request = request.clone().with_body(body);
    Ok(Value::HttpRequest(Rc::new(new_request)))
}

/// Set timeout on request
///
/// Args:
/// - request: HttpRequest
/// - seconds: number
///
/// Returns: HttpRequest (new request with timeout set)
///
/// Example:
/// ```
/// let req = httpSetTimeout(req, 60);
/// ```
pub fn http_set_timeout(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpSetTimeout: expected 2 arguments (request, seconds)".to_string(),
            span,
        });
    }

    let request = expect_http_request(&args[0], "request", span)?;
    let seconds = expect_number(&args[1], "seconds", span)?;

    if seconds < 0.0 {
        return Err(RuntimeError::TypeError {
            msg: "httpSetTimeout: seconds must be non-negative".to_string(),
            span,
        });
    }

    let new_request = request.clone().with_timeout(seconds as u64);
    Ok(Value::HttpRequest(Rc::new(new_request)))
}

// ============================================================================
// Response Handling Functions
// ============================================================================

/// Get status code from response
///
/// Args:
/// - response: HttpResponse
///
/// Returns: number (status code)
///
/// Example:
/// ```
/// let status = httpStatus(response);
/// ```
pub fn http_status(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpStatus: expected 1 argument (response)".to_string(),
            span,
        });
    }

    let response = expect_http_response(&args[0], "response", span)?;
    Ok(Value::Number(response.status() as f64))
}

/// Get body from response as string
///
/// Args:
/// - response: HttpResponse
///
/// Returns: string (response body)
///
/// Example:
/// ```
/// let body = httpBody(response);
/// ```
pub fn http_body(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpBody: expected 1 argument (response)".to_string(),
            span,
        });
    }

    let response = expect_http_response(&args[0], "response", span)?;
    Ok(Value::string(response.body().to_string()))
}

/// Get header value from response
///
/// Args:
/// - response: HttpResponse
/// - key: string (header name, case-insensitive)
///
/// Returns: Option<string> (header value or None)
///
/// Example:
/// ```
/// let content_type = httpHeader(response, "Content-Type");
/// ```
pub fn http_header(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpHeader: expected 2 arguments (response, key)".to_string(),
            span,
        });
    }

    let response = expect_http_response(&args[0], "response", span)?;
    let key = expect_string(&args[1], "key", span)?;

    // Case-insensitive header lookup
    let value = response
        .headers()
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(&key))
        .map(|(_, v)| v.clone());

    match value {
        Some(v) => Ok(Value::Option(Some(Box::new(Value::string(v))))),
        None => Ok(Value::Option(None)),
    }
}

/// Get all headers from response
///
/// Args:
/// - response: HttpResponse
///
/// Returns: HashMap (all headers)
///
/// Example:
/// ```
/// let headers = httpHeaders(response);
/// ```
pub fn http_headers(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpHeaders: expected 1 argument (response)".to_string(),
            span,
        });
    }

    let response = expect_http_response(&args[0], "response", span)?;

    let mut atlas_map = AtlasHashMap::new();
    for (key, value) in response.headers() {
        atlas_map.insert(
            HashKey::String(Rc::new(key.clone())),
            Value::string(value.clone()),
        );
    }

    Ok(Value::HashMap(Rc::new(RefCell::new(atlas_map))))
}

/// Get final URL from response (after redirects)
///
/// Args:
/// - response: HttpResponse
///
/// Returns: string (final URL)
///
/// Example:
/// ```
/// let url = httpUrl(response);
/// ```
pub fn http_url(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpUrl: expected 1 argument (response)".to_string(),
            span,
        });
    }

    let response = expect_http_response(&args[0], "response", span)?;
    Ok(Value::string(response.url().to_string()))
}

/// Check if response is success (status 200-299)
///
/// Args:
/// - response: HttpResponse
///
/// Returns: bool
///
/// Example:
/// ```
/// if httpIsSuccess(response) { ... }
/// ```
pub fn http_is_success(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpIsSuccess: expected 1 argument (response)".to_string(),
            span,
        });
    }

    let response = expect_http_response(&args[0], "response", span)?;
    Ok(Value::Bool(response.is_success()))
}

// ============================================================================
// HTTP Client Functions
// ============================================================================

/// Execute HTTP request
///
/// Args:
/// - request: HttpRequest
///
/// Returns: Result<HttpResponse> (Ok with response or Err with error message)
///
/// Example:
/// ```
/// let result = httpSend(request);
/// ```
pub fn http_send(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpSend: expected 1 argument (request)".to_string(),
            span,
        });
    }

    let request = expect_http_request(&args[0], "request", span)?;

    // Build reqwest client
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(request.timeout_secs()))
        .redirect(if request.follow_redirects {
            reqwest::redirect::Policy::limited(request.max_redirects as usize)
        } else {
            reqwest::redirect::Policy::none()
        })
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return Ok(Value::Result(Err(Box::new(Value::string(format!(
                "httpSend: failed to create client: {}",
                e
            ))))));
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
            return Ok(Value::Result(Err(Box::new(Value::string(format!(
                "httpSend: unsupported method: {}",
                method
            ))))));
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
    let response = match req_builder.send() {
        Ok(r) => r,
        Err(e) => {
            let error_msg = if e.is_timeout() {
                format!(
                    "httpSend: request timeout after {} seconds",
                    request.timeout_secs()
                )
            } else if e.is_connect() {
                format!("httpSend: connection error: {}", e)
            } else {
                format!("httpSend: network error: {}", e)
            };
            return Ok(Value::Result(Err(Box::new(Value::string(error_msg)))));
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

    let body = match response.text() {
        Ok(b) => b,
        Err(e) => {
            return Ok(Value::Result(Err(Box::new(Value::string(format!(
                "httpSend: failed to read response body: {}",
                e
            ))))));
        }
    };

    let http_response = HttpResponse::new(status, headers_map, body, final_url);
    Ok(Value::Result(Ok(Box::new(Value::HttpResponse(Rc::new(
        http_response,
    ))))))
}

/// Simple GET request
///
/// Args:
/// - url: string
///
/// Returns: Result<HttpResponse>
///
/// Example:
/// ```
/// let result = httpGet("https://httpbin.org/get");
/// ```
pub fn http_get(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpGet: expected 1 argument (url)".to_string(),
            span,
        });
    }

    let request = http_request_get(args, span)?;
    http_send(&[request], span)
}

/// Simple POST request
///
/// Args:
/// - url: string
/// - body: string
///
/// Returns: Result<HttpResponse>
///
/// Example:
/// ```
/// let result = httpPost("https://httpbin.org/post", "data");
/// ```
pub fn http_post(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpPost: expected 2 arguments (url, body)".to_string(),
            span,
        });
    }

    let request = http_request_post(args, span)?;
    http_send(&[request], span)
}

/// POST JSON data
///
/// Args:
/// - url: string
/// - json: JsonValue
///
/// Returns: Result<HttpResponse>
///
/// Example:
/// ```
/// let result = httpPostJson("https://httpbin.org/post", jsonParse("{\"key\":\"value\"}"));
/// ```
pub fn http_post_json(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpPostJson: expected 2 arguments (url, json)".to_string(),
            span,
        });
    }

    let url_value = args[0].clone();
    let json_value = expect_json_value(&args[1], "json", span)?;

    // Convert Atlas JsonValue to serde_json::Value, then to string
    let serde_json = atlas_json_to_serde(&json_value);
    let json_string = match serde_json::to_string(&serde_json) {
        Ok(s) => s,
        Err(e) => {
            return Err(RuntimeError::TypeError {
                msg: format!("httpPostJson: failed to serialize JSON: {}", e),
                span,
            })
        }
    };

    // Create POST request with JSON body and Content-Type header
    let request = http_request_post(&[url_value, Value::string(json_string)], span)?;
    let request_with_header = http_set_header(
        &[
            request,
            Value::string("Content-Type".to_string()),
            Value::string("application/json".to_string()),
        ],
        span,
    )?;

    http_send(&[request_with_header], span)
}

/// Parse JSON from response body
///
/// Args:
/// - response: HttpResponse
///
/// Returns: Result<JsonValue>
///
/// Example:
/// ```
/// let json_result = httpParseJson(response);
/// ```
pub fn http_parse_json(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpParseJson: expected 1 argument (response)".to_string(),
            span,
        });
    }

    let response = expect_http_response(&args[0], "response", span)?;
    let body = response.body();

    // Parse JSON using serde_json, then convert to AtlasJsonValue
    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(serde_json) => {
            let atlas_json = serde_to_atlas_json(serde_json);
            Ok(Value::Result(Ok(Box::new(Value::JsonValue(Rc::new(
                atlas_json,
            ))))))
        }
        Err(e) => Ok(Value::Result(Err(Box::new(Value::string(format!(
            "httpParseJson: failed to parse JSON: {}",
            e
        )))))),
    }
}

// ============================================================================
// Phase 10b: Advanced HTTP Features
// ============================================================================

/// Create PUT request (convenience function)
pub fn http_request_put(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpRequestPut: expected 2 arguments (url, body)".to_string(),
            span,
        });
    }

    let url = expect_string(&args[0], "url", span)?;
    let body = expect_string(&args[1], "body", span)?;

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(RuntimeError::TypeError {
            msg: format!(
                "httpRequestPut: URL must start with http:// or https://, got: {}",
                url
            ),
            span,
        });
    }

    let request = HttpRequest::new("PUT".to_string(), url).with_body(body);
    Ok(Value::HttpRequest(Rc::new(request)))
}

/// Create DELETE request (convenience function)
pub fn http_request_delete(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpRequestDelete: expected 1 argument (url)".to_string(),
            span,
        });
    }

    let url = expect_string(&args[0], "url", span)?;

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(RuntimeError::TypeError {
            msg: format!(
                "httpRequestDelete: URL must start with http:// or https://, got: {}",
                url
            ),
            span,
        });
    }

    let request = HttpRequest::new("DELETE".to_string(), url);
    Ok(Value::HttpRequest(Rc::new(request)))
}

/// Create PATCH request (convenience function)
pub fn http_request_patch(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpRequestPatch: expected 2 arguments (url, body)".to_string(),
            span,
        });
    }

    let url = expect_string(&args[0], "url", span)?;
    let body = expect_string(&args[1], "body", span)?;

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(RuntimeError::TypeError {
            msg: format!(
                "httpRequestPatch: URL must start with http:// or https://, got: {}",
                url
            ),
            span,
        });
    }

    let request = HttpRequest::new("PATCH".to_string(), url).with_body(body);
    Ok(Value::HttpRequest(Rc::new(request)))
}

/// Simple PUT request
pub fn http_put(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpPut: expected 2 arguments (url, body)".to_string(),
            span,
        });
    }

    let request = http_request_put(args, span)?;
    http_send(&[request], span)
}

/// Simple DELETE request
pub fn http_delete(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpDelete: expected 1 argument (url)".to_string(),
            span,
        });
    }

    let request = http_request_delete(args, span)?;
    http_send(&[request], span)
}

/// Simple PATCH request
pub fn http_patch(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpPatch: expected 2 arguments (url, body)".to_string(),
            span,
        });
    }

    let request = http_request_patch(args, span)?;
    http_send(&[request], span)
}

/// Set query parameter on request
pub fn http_set_query(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::TypeError {
            msg: "httpSetQuery: expected 3 arguments (request, key, value)".to_string(),
            span,
        });
    }

    let mut request = expect_http_request(&args[0], "request", span)?;
    let key = expect_string(&args[1], "key", span)?;
    let value = expect_string(&args[2], "value", span)?;

    request = request.with_query_param(key, value);
    Ok(Value::HttpRequest(Rc::new(request)))
}

/// Set follow redirects option
pub fn http_set_follow_redirects(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpSetFollowRedirects: expected 2 arguments (request, follow)".to_string(),
            span,
        });
    }

    let mut request = expect_http_request(&args[0], "request", span)?;
    let follow = expect_bool(&args[1], "follow", span)?;

    request = request.with_follow_redirects(follow);
    Ok(Value::HttpRequest(Rc::new(request)))
}

/// Set max redirects
pub fn http_set_max_redirects(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpSetMaxRedirects: expected 2 arguments (request, max)".to_string(),
            span,
        });
    }

    let mut request = expect_http_request(&args[0], "request", span)?;
    let max = expect_number(&args[1], "max", span)? as u32;

    request = request.with_max_redirects(max);
    Ok(Value::HttpRequest(Rc::new(request)))
}

/// Set User-Agent header
pub fn http_set_user_agent(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "httpSetUserAgent: expected 2 arguments (request, agent)".to_string(),
            span,
        });
    }

    let mut request = expect_http_request(&args[0], "request", span)?;
    let agent = expect_string(&args[1], "agent", span)?;

    request = request.with_header("User-Agent".to_string(), agent);
    Ok(Value::HttpRequest(Rc::new(request)))
}

/// Set basic authentication
pub fn http_set_auth(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::TypeError {
            msg: "httpSetAuth: expected 3 arguments (request, user, pass)".to_string(),
            span,
        });
    }

    let mut request = expect_http_request(&args[0], "request", span)?;
    let user = expect_string(&args[1], "user", span)?;
    let pass = expect_string(&args[2], "pass", span)?;

    // Create Basic auth header
    let auth = format!("{}:{}", user, pass);
    let encoded =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, auth.as_bytes());
    let auth_header = format!("Basic {}", encoded);

    request = request.with_header("Authorization".to_string(), auth_header);
    Ok(Value::HttpRequest(Rc::new(request)))
}

/// Get status text from status code
pub fn http_status_text(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpStatusText: expected 1 argument (response)".to_string(),
            span,
        });
    }

    let response = expect_http_response(&args[0], "response", span)?;
    let status_text = match response.status() {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Unknown",
    };

    Ok(Value::string(status_text.to_string()))
}

/// Get Content-Type header
pub fn http_content_type(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpContentType: expected 1 argument (response)".to_string(),
            span,
        });
    }

    let response = expect_http_response(&args[0], "response", span)?;
    let content_type = response
        .headers()
        .get("content-type")
        .or_else(|| response.headers().get("Content-Type"));

    match content_type {
        Some(ct) => Ok(Value::Option(Some(Box::new(Value::string(ct.clone()))))),
        None => Ok(Value::Option(None)),
    }
}

/// Get Content-Length header
pub fn http_content_length(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpContentLength: expected 1 argument (response)".to_string(),
            span,
        });
    }

    let response = expect_http_response(&args[0], "response", span)?;
    let content_length = response
        .headers()
        .get("content-length")
        .or_else(|| response.headers().get("Content-Length"));

    match content_length {
        Some(cl) => {
            if let Ok(len) = cl.parse::<f64>() {
                Ok(Value::Option(Some(Box::new(Value::Number(len)))))
            } else {
                Ok(Value::Option(None))
            }
        }
        None => Ok(Value::Option(None)),
    }
}

/// Check if response is redirect (300-399)
pub fn http_is_redirect(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpIsRedirect: expected 1 argument (response)".to_string(),
            span,
        });
    }

    let response = expect_http_response(&args[0], "response", span)?;
    let status = response.status();
    Ok(Value::Bool((300..400).contains(&status)))
}

/// Check if response is client error (400-499)
pub fn http_is_client_error(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpIsClientError: expected 1 argument (response)".to_string(),
            span,
        });
    }

    let response = expect_http_response(&args[0], "response", span)?;
    let status = response.status();
    Ok(Value::Bool((400..500).contains(&status)))
}

/// Check if response is server error (500-599)
pub fn http_is_server_error(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpIsServerError: expected 1 argument (response)".to_string(),
            span,
        });
    }

    let response = expect_http_response(&args[0], "response", span)?;
    let status = response.status();
    Ok(Value::Bool((500..600).contains(&status)))
}

/// GET and parse JSON in one call
pub fn http_get_json(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpGetJson: expected 1 argument (url)".to_string(),
            span,
        });
    }

    // Execute GET request
    let get_result = http_get(args, span)?;

    // Extract result
    match get_result {
        Value::Result(Ok(response)) => {
            // Parse JSON from response
            http_parse_json(&[*response], span)
        }
        Value::Result(Err(err)) => Ok(Value::Result(Err(err))),
        _ => Err(RuntimeError::TypeError {
            msg: "httpGetJson: unexpected return type".to_string(),
            span,
        }),
    }
}

/// Placeholder for permission checking
/// Note: Full permission integration requires architectural changes to pass SecurityContext to stdlib functions
pub fn http_check_permission(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "httpCheckPermission: expected 1 argument (url)".to_string(),
            span,
        });
    }

    let _url = expect_string(&args[0], "url", span)?;

    // TODO: Full implementation requires SecurityContext access
    // For now, always return true (allow all)
    // This should be updated when SecurityContext can be passed to stdlib functions
    Ok(Value::Bool(true))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Expect a string value
fn expect_string(value: &Value, arg_name: &str, span: Span) -> Result<String, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.as_ref().clone()),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "expected string for '{}', got {}",
                arg_name,
                value.type_name()
            ),
            span,
        }),
    }
}

/// Expect a number value
fn expect_number(value: &Value, arg_name: &str, span: Span) -> Result<f64, RuntimeError> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "expected number for '{}', got {}",
                arg_name,
                value.type_name()
            ),
            span,
        }),
    }
}

/// Expect a boolean value
fn expect_bool(value: &Value, arg_name: &str, span: Span) -> Result<bool, RuntimeError> {
    match value {
        Value::Bool(b) => Ok(*b),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "expected bool for '{}', got {}",
                arg_name,
                value.type_name()
            ),
            span,
        }),
    }
}

/// Expect an HttpRequest value
fn expect_http_request(
    value: &Value,
    arg_name: &str,
    span: Span,
) -> Result<HttpRequest, RuntimeError> {
    match value {
        Value::HttpRequest(req) => Ok(req.as_ref().clone()),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "expected HttpRequest for '{}', got {}",
                arg_name,
                value.type_name()
            ),
            span,
        }),
    }
}

/// Expect an HttpResponse value
fn expect_http_response(
    value: &Value,
    arg_name: &str,
    span: Span,
) -> Result<HttpResponse, RuntimeError> {
    match value {
        Value::HttpResponse(res) => Ok(res.as_ref().clone()),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "expected HttpResponse for '{}', got {}",
                arg_name,
                value.type_name()
            ),
            span,
        }),
    }
}

/// Expect a JsonValue
fn expect_json_value(
    value: &Value,
    arg_name: &str,
    span: Span,
) -> Result<Rc<JsonValue>, RuntimeError> {
    match value {
        Value::JsonValue(json) => Ok(json.clone()),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "expected JsonValue for '{}', got {}",
                arg_name,
                value.type_name()
            ),
            span,
        }),
    }
}

/// Convert serde_json::Value to Atlas JsonValue
fn serde_to_atlas_json(value: serde_json::Value) -> JsonValue {
    match value {
        serde_json::Value::Null => JsonValue::Null,
        serde_json::Value::Bool(b) => JsonValue::Bool(b),
        serde_json::Value::Number(n) => JsonValue::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => JsonValue::String(s),
        serde_json::Value::Array(arr) => {
            JsonValue::Array(arr.into_iter().map(serde_to_atlas_json).collect())
        }
        serde_json::Value::Object(obj) => JsonValue::Object(
            obj.into_iter()
                .map(|(k, v)| (k, serde_to_atlas_json(v)))
                .collect(),
        ),
    }
}

/// Convert Atlas JsonValue to serde_json::Value
fn atlas_json_to_serde(value: &JsonValue) -> serde_json::Value {
    match value {
        JsonValue::Null => serde_json::Value::Null,
        JsonValue::Bool(b) => serde_json::Value::Bool(*b),
        JsonValue::Number(n) => serde_json::Value::Number(
            serde_json::Number::from_f64(*n).unwrap_or_else(|| serde_json::Number::from(0)),
        ),
        JsonValue::String(s) => serde_json::Value::String(s.clone()),
        JsonValue::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(atlas_json_to_serde).collect())
        }
        JsonValue::Object(obj) => serde_json::Value::Object(
            obj.iter()
                .map(|(k, v)| (k.clone(), atlas_json_to_serde(v)))
                .collect(),
        ),
    }
}
