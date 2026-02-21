//! Runtime value representation
//!
//! Shared value representation for interpreter and VM.
//! - Numbers, Bools, Null: Immediate values (stack-allocated)
//! - Strings: Heap-allocated, reference-counted (Arc<String>), immutable
//! - Arrays: Copy-on-write (ValueArray wrapping Arc<Vec<Value>>), value semantics
//! - Functions: Reference to bytecode or builtin
//! - NativeFunction: Rust closures callable from Atlas
//! - JsonValue: Isolated dynamic type for JSON interop (Arc<JsonValue>)

use crate::json_value::JsonValue;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Copy-on-write array. Cheap to clone (refcount bump).
/// Mutations on a shared array clone the inner Vec first (Arc::make_mut).
#[derive(Clone, Debug)]
pub struct ValueArray(Arc<Vec<Value>>);

impl ValueArray {
    pub fn new() -> Self {
        ValueArray(Arc::new(Vec::new()))
    }

    pub fn from_vec(v: Vec<Value>) -> Self {
        ValueArray(Arc::new(v))
    }

    /// Read access — no clone needed.
    pub fn as_slice(&self) -> &[Value] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get element by index — returns reference into inner Vec.
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.0.get(index)
    }

    /// Mutating access — triggers CoW if Arc is shared.
    pub fn push(&mut self, value: Value) {
        Arc::make_mut(&mut self.0).push(value);
    }

    pub fn pop(&mut self) -> Option<Value> {
        Arc::make_mut(&mut self.0).pop()
    }

    pub fn set(&mut self, index: usize, value: Value) -> bool {
        let inner = Arc::make_mut(&mut self.0);
        if index < inner.len() {
            inner[index] = value;
            true
        } else {
            false
        }
    }

    pub fn insert(&mut self, index: usize, value: Value) {
        Arc::make_mut(&mut self.0).insert(index, value);
    }

    pub fn remove(&mut self, index: usize) -> Value {
        Arc::make_mut(&mut self.0).remove(index)
    }

    pub fn truncate(&mut self, len: usize) {
        Arc::make_mut(&mut self.0).truncate(len);
    }

    pub fn extend(&mut self, iter: impl IntoIterator<Item = Value>) {
        Arc::make_mut(&mut self.0).extend(iter);
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Value> {
        self.0.iter()
    }

    /// Returns true if this array is the sole owner (no other clones).
    /// Used by the VM to decide whether to mutate in-place or CoW-copy.
    pub fn is_exclusively_owned(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }

    /// Convert to owned Vec — clones only if shared.
    pub fn into_vec(self) -> Vec<Value> {
        Arc::try_unwrap(self.0).unwrap_or_else(|arc| (*arc).clone())
    }

    /// Expose inner Arc for cases that need to check sharing (e.g., equality).
    pub fn arc(&self) -> &Arc<Vec<Value>> {
        &self.0
    }
}

impl Default for ValueArray {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for ValueArray {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_slice() == other.0.as_slice()
    }
}

impl std::ops::Index<usize> for ValueArray {
    type Output = Value;
    fn index(&self, index: usize) -> &Value {
        &self.0[index]
    }
}

impl From<Vec<Value>> for ValueArray {
    fn from(v: Vec<Value>) -> Self {
        ValueArray::from_vec(v)
    }
}

impl FromIterator<Value> for ValueArray {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Self {
        ValueArray(Arc::new(iter.into_iter().collect()))
    }
}

/// Copy-on-write string-keyed map. Cheap to clone (refcount bump).
/// Mutations clone the inner HashMap if shared (Arc::make_mut).
#[derive(Clone, Debug, Default)]
pub struct ValueMap(Arc<HashMap<String, Value>>);

impl ValueMap {
    pub fn new() -> Self {
        ValueMap(Arc::new(HashMap::new()))
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn insert(&mut self, key: String, value: Value) {
        Arc::make_mut(&mut self.0).insert(key, value);
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        Arc::make_mut(&mut self.0).remove(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, Value> {
        self.0.iter()
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, String, Value> {
        self.0.keys()
    }

    pub fn values(&self) -> std::collections::hash_map::Values<'_, String, Value> {
        self.0.values()
    }

    pub fn is_exclusively_owned(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }
}

impl PartialEq for ValueMap {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

impl From<HashMap<String, Value>> for ValueMap {
    fn from(m: HashMap<String, Value>) -> Self {
        ValueMap(Arc::new(m))
    }
}

/// Copy-on-write wrapper for AtlasHashMap
#[derive(Clone, Debug, Default)]
pub struct ValueHashMap(Arc<crate::stdlib::collections::hashmap::AtlasHashMap>);

impl ValueHashMap {
    pub fn new() -> Self {
        ValueHashMap(Arc::new(crate::stdlib::collections::hashmap::AtlasHashMap::new()))
    }

    pub fn inner(&self) -> &crate::stdlib::collections::hashmap::AtlasHashMap {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut crate::stdlib::collections::hashmap::AtlasHashMap {
        Arc::make_mut(&mut self.0)
    }

    pub fn is_exclusively_owned(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }
}

impl PartialEq for ValueHashMap {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

/// Copy-on-write wrapper for AtlasHashSet
#[derive(Clone, Debug, Default)]
pub struct ValueHashSet(Arc<crate::stdlib::collections::hashset::AtlasHashSet>);

impl ValueHashSet {
    pub fn new() -> Self {
        ValueHashSet(Arc::new(crate::stdlib::collections::hashset::AtlasHashSet::new()))
    }

    pub fn inner(&self) -> &crate::stdlib::collections::hashset::AtlasHashSet {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut crate::stdlib::collections::hashset::AtlasHashSet {
        Arc::make_mut(&mut self.0)
    }

    pub fn is_exclusively_owned(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }
}

impl PartialEq for ValueHashSet {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

/// Copy-on-write wrapper for AtlasQueue
#[derive(Clone, Debug, Default)]
pub struct ValueQueue(Arc<crate::stdlib::collections::queue::AtlasQueue>);

impl ValueQueue {
    pub fn new() -> Self {
        ValueQueue(Arc::new(crate::stdlib::collections::queue::AtlasQueue::new()))
    }

    pub fn inner(&self) -> &crate::stdlib::collections::queue::AtlasQueue {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut crate::stdlib::collections::queue::AtlasQueue {
        Arc::make_mut(&mut self.0)
    }

    pub fn is_exclusively_owned(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }
}

impl PartialEq for ValueQueue {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

/// Copy-on-write wrapper for AtlasStack
#[derive(Clone, Debug, Default)]
pub struct ValueStack(Arc<crate::stdlib::collections::stack::AtlasStack>);

impl ValueStack {
    pub fn new() -> Self {
        ValueStack(Arc::new(crate::stdlib::collections::stack::AtlasStack::new()))
    }

    pub fn inner(&self) -> &crate::stdlib::collections::stack::AtlasStack {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut crate::stdlib::collections::stack::AtlasStack {
        Arc::make_mut(&mut self.0)
    }

    pub fn is_exclusively_owned(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }
}

impl PartialEq for ValueStack {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

/// Explicit reference semantics wrapper.
///
/// `Shared<T>` opts into reference semantics: all clones point to the same underlying
/// value. Mutation through any clone is visible to all other clones. This is the
/// intentional escape hatch from CoW — used when the program explicitly requests
/// shared mutable state (e.g., `shared<Buffer>`).
///
/// Contrast with `ValueArray` which uses `Arc<Vec<Value>>` + CoW: mutations on a
/// `ValueArray` clone never affect the original. Mutations on a `Shared<T>` always do.
#[derive(Clone, Debug)]
pub struct Shared<T>(Arc<Mutex<T>>);

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Shared(Arc::new(Mutex::new(value)))
    }

    /// Acquire the lock and apply a read function.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.0.lock().expect("Shared<T> lock poisoned");
        f(&*guard)
    }

    /// Acquire the lock and apply a mutation function.
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.0.lock().expect("Shared<T> lock poisoned");
        f(&mut *guard)
    }

    /// Returns true if this is the only reference to the inner value.
    pub fn is_exclusively_owned(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }
}

impl<T: PartialEq> PartialEq for Shared<T> {
    fn eq(&self, other: &Self) -> bool {
        // Pointer equality — two Shared<T> are equal only if they are the same allocation.
        // This matches reference semantics: two different `shared<T>` variables with the
        // same contents are NOT equal unless they are the same reference.
        Arc::ptr_eq(&self.0, &other.0)
    }
}

/// Native function type - Rust closure callable from Atlas
///
/// Native functions receive an array of Atlas values and return either a value or a runtime error.
/// Arc provides thread safety and cheap cloning for sharing natives across execution contexts.
pub type NativeFn = Arc<dyn Fn(&[Value]) -> Result<Value, RuntimeError> + Send + Sync>;

/// Runtime value type
#[derive(Clone)]
pub enum Value {
    /// Numeric value (IEEE 754 double-precision)
    Number(f64),
    /// String value (reference-counted, immutable)
    String(Arc<String>),
    /// Boolean value
    Bool(bool),
    /// Null value
    Null,
    /// Array value (copy-on-write, value semantics)
    Array(ValueArray),
    /// Function reference (bytecode or builtin)
    Function(FunctionRef),
    /// Builtin stdlib function (dispatched through the registry by name)
    Builtin(Arc<str>),
    /// Native function (Rust closure callable from Atlas)
    NativeFunction(NativeFn),
    /// JSON value (isolated dynamic type for JSON interop)
    JsonValue(Arc<JsonValue>),
    /// Option value (Some(value) or None)
    Option(Option<Box<Value>>),
    /// Result value (Ok(value) or Err(error))
    Result(Result<Box<Value>, Box<Value>>),
    /// HashMap collection (key-value pairs)
    HashMap(ValueHashMap),
    /// HashSet collection (unique values)
    HashSet(ValueHashSet),
    /// Queue collection (FIFO)
    Queue(ValueQueue),
    /// Stack collection (LIFO)
    Stack(ValueStack),
    /// Regular expression pattern
    Regex(Arc<regex::Regex>),
    /// DateTime value (UTC timezone)
    DateTime(Arc<chrono::DateTime<chrono::Utc>>),
    /// HTTP Request configuration
    HttpRequest(Arc<crate::stdlib::http::HttpRequest>),
    /// HTTP Response data
    HttpResponse(Arc<crate::stdlib::http::HttpResponse>),
    /// Future value (async computation)
    Future(Arc<crate::async_runtime::AtlasFuture>),
    /// Task handle (spawned async task)
    TaskHandle(Arc<Mutex<crate::async_runtime::task::TaskHandle>>),
    /// Channel sender (for message passing)
    ChannelSender(Arc<Mutex<crate::async_runtime::channel::ChannelSender>>),
    /// Channel receiver (for message passing)
    ChannelReceiver(Arc<Mutex<crate::async_runtime::channel::ChannelReceiver>>),
    /// Async mutex (for async synchronization)
    AsyncMutex(Arc<tokio::sync::Mutex<Value>>),
    /// Closure (function + captured upvalue environment)
    Closure(ClosureRef),
    /// Explicitly shared reference — reference semantics (see Shared<T>).
    /// Mutations are visible to all aliases. Used for `shared<T>` annotated values.
    SharedValue(Shared<Box<Value>>),
}

/// Function reference
#[derive(Debug, Clone)]
pub struct FunctionRef {
    /// Function name
    pub name: String,
    /// Number of parameters
    pub arity: usize,
    /// Bytecode offset (for VM) or builtin ID
    pub bytecode_offset: usize,
    /// Total number of local variables (parameters + locals)
    /// Used by VM to properly allocate stack space
    pub local_count: usize,
}

/// Closure reference — a function with a captured upvalue environment
#[derive(Debug, Clone)]
pub struct ClosureRef {
    /// The underlying compiled function
    pub func: FunctionRef,
    /// Captured outer-scope values (by value, at closure creation time)
    pub upvalues: Arc<Vec<Value>>,
}

impl Value {
    /// Create a new string value
    pub fn string(s: impl Into<String>) -> Self {
        Value::String(Arc::new(s.into()))
    }

    /// Create a new array value
    pub fn array(values: Vec<Value>) -> Self {
        Value::Array(ValueArray::from_vec(values))
    }

    /// Get the type name of this value
    pub fn type_name(&self) -> &str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::Null => "null",
            Value::Array(_) => "array",
            Value::Function(_) => "function",
            Value::Builtin(_) => "builtin",
            Value::NativeFunction(_) => "function",
            Value::JsonValue(_) => "json",
            Value::Option(_) => "Option",
            Value::Result(_) => "Result",
            Value::HashMap(_) => "hashmap",
            Value::HashSet(_) => "hashset",
            Value::Queue(_) => "queue",
            Value::Stack(_) => "stack",
            Value::Regex(_) => "regex",
            Value::DateTime(_) => "datetime",
            Value::HttpRequest(_) => "HttpRequest",
            Value::HttpResponse(_) => "HttpResponse",
            Value::Future(_) => "future",
            Value::TaskHandle(_) => "TaskHandle",
            Value::ChannelSender(_) => "ChannelSender",
            Value::ChannelReceiver(_) => "ChannelReceiver",
            Value::AsyncMutex(_) => "AsyncMutex",
            Value::Closure(_) => "function",
            Value::SharedValue(_) => "shared",
        }
    }

    /// Check if this value is truthy
    /// In Atlas, only `true` is truthy - no implicit conversions
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            _ => false, // Only bool values are truthy in Atlas
        }
    }

    /// Get a display string representation (alias for to_string for backwards compatibility)
    pub fn to_display_string(&self) -> String {
        self.to_string()
    }
}

impl PartialEq for Value {
    /// Equality contract:
    ///
    /// **Value types** (content equality — two equal values may be different allocations):
    /// - Number, String, Bool, Null: primitive equality
    /// - Array, HashMap, HashSet, Queue, Stack: CoW wrappers compare by content
    /// - Regex: compare by pattern string
    /// - DateTime: compare timestamps
    /// - HttpRequest, HttpResponse: compare by field content
    /// - Option, Result, JsonValue: deep structural equality
    /// - Function, Builtin: compare by name
    /// - Closure: compare by function name
    ///
    /// **Reference types** (identity equality — only the same allocation is equal):
    /// - NativeFunction: closures have no meaningful content equality
    /// - SharedValue: Shared<T> uses Arc::ptr_eq (reference semantics by design)
    /// - Future, TaskHandle, ChannelSender, ChannelReceiver, AsyncMutex:
    ///   live runtime objects — identity is the only meaningful equality
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // --- Value types: content equality ---
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::HashMap(a), Value::HashMap(b)) => a == b,
            (Value::HashSet(a), Value::HashSet(b)) => a == b,
            (Value::Queue(a), Value::Queue(b)) => a == b,
            (Value::Stack(a), Value::Stack(b)) => a == b,
            (Value::Regex(a), Value::Regex(b)) => a.as_str() == b.as_str(),
            (Value::DateTime(a), Value::DateTime(b)) => a == b,
            (Value::HttpRequest(a), Value::HttpRequest(b)) => a.as_ref() == b.as_ref(),
            (Value::HttpResponse(a), Value::HttpResponse(b)) => a.as_ref() == b.as_ref(),
            (Value::JsonValue(a), Value::JsonValue(b)) => a == b,
            (Value::Option(a), Value::Option(b)) => a == b,
            (Value::Result(a), Value::Result(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => a.name == b.name,
            (Value::Builtin(a), Value::Builtin(b)) => a == b,
            (Value::Closure(a), Value::Closure(b)) => a.func.name == b.func.name,
            // --- Reference types: identity equality ---
            (Value::NativeFunction(a), Value::NativeFunction(b)) => Arc::ptr_eq(a, b),
            (Value::SharedValue(a), Value::SharedValue(b)) => a == b,
            (Value::Future(a), Value::Future(b)) => Arc::ptr_eq(a, b),
            (Value::TaskHandle(a), Value::TaskHandle(b)) => Arc::ptr_eq(a, b),
            (Value::ChannelSender(a), Value::ChannelSender(b)) => Arc::ptr_eq(a, b),
            (Value::ChannelReceiver(a), Value::ChannelReceiver(b)) => Arc::ptr_eq(a, b),
            (Value::AsyncMutex(a), Value::AsyncMutex(b)) => Arc::ptr_eq(a, b),
            // Different variants are never equal
            _ => false,
        }
    }
}

impl Eq for Value {}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => {
                // Format number nicely (no trailing .0 for whole numbers)
                if n.fract() == 0.0 && n.is_finite() {
                    write!(f, "{:.0}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "{}", s.as_ref()),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", elements.join(", "))
            }
            Value::Function(func) => write!(f, "<fn {}>", func.name),
            Value::Builtin(name) => write!(f, "<builtin {}>", name),
            Value::NativeFunction(_) => write!(f, "<native fn>"),
            Value::JsonValue(json) => write!(f, "{}", json),
            Value::Option(opt) => match opt {
                Some(val) => write!(f, "Some({})", val),
                None => write!(f, "None"),
            },
            Value::Result(res) => match res {
                Ok(val) => write!(f, "Ok({})", val),
                Err(err) => write!(f, "Err({})", err),
            },
            Value::HashMap(map) => write!(f, "<HashMap size={}>", map.inner().len()),
            Value::HashSet(set) => write!(f, "<HashSet size={}>", set.inner().len()),
            Value::Queue(queue) => write!(f, "<Queue size={}>", queue.inner().len()),
            Value::Stack(stack) => write!(f, "<Stack size={}>", stack.inner().len()),
            Value::Regex(r) => write!(f, "<Regex /{}/>", r.as_str()),
            Value::DateTime(dt) => write!(f, "{}", dt.to_rfc3339()),
            Value::HttpRequest(req) => write!(f, "<HttpRequest {} {}>", req.method(), req.url()),
            Value::HttpResponse(res) => write!(f, "<HttpResponse {}>", res.status()),
            Value::Future(future) => write!(f, "{}", future.as_ref()),
            Value::TaskHandle(handle) => write!(f, "<TaskHandle #{}>", handle.lock().unwrap().id()),
            Value::ChannelSender(_) => write!(f, "<ChannelSender>"),
            Value::ChannelReceiver(_) => write!(f, "<ChannelReceiver>"),
            Value::AsyncMutex(_) => write!(f, "<AsyncMutex>"),
            Value::Closure(c) => write!(f, "<fn {}>", c.func.name),
            Value::SharedValue(s) => s.with(|v| write!(f, "shared({})", v)),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "Number({})", n),
            Value::String(s) => write!(f, "String({:?})", s),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::Null => write!(f, "Null"),
            Value::Array(arr) => write!(f, "Array({:?})", arr.as_slice()),
            Value::Function(func) => write!(f, "Function({:?})", func),
            Value::Builtin(name) => write!(f, "Builtin({:?})", name),
            Value::NativeFunction(_) => write!(f, "NativeFunction(<closure>)"),
            Value::JsonValue(json) => write!(f, "JsonValue({:?})", json),
            Value::Option(opt) => write!(f, "Option({:?})", opt),
            Value::Result(res) => write!(f, "Result({:?})", res),
            Value::HashMap(map) => write!(f, "HashMap(size={})", map.inner().len()),
            Value::HashSet(set) => write!(f, "HashSet(size={})", set.inner().len()),
            Value::Queue(queue) => write!(f, "Queue(size={})", queue.inner().len()),
            Value::Stack(stack) => write!(f, "Stack(size={})", stack.inner().len()),
            Value::Regex(r) => write!(f, "Regex(/{}/)", r.as_str()),
            Value::DateTime(dt) => write!(f, "DateTime({})", dt.to_rfc3339()),
            Value::HttpRequest(req) => write!(f, "HttpRequest({} {})", req.method(), req.url()),
            Value::HttpResponse(res) => write!(f, "HttpResponse({})", res.status()),
            Value::Future(future) => write!(f, "{:?}", future.as_ref()),
            Value::TaskHandle(handle) => write!(f, "TaskHandle(#{})", handle.lock().unwrap().id()),
            Value::ChannelSender(_) => write!(f, "ChannelSender"),
            Value::ChannelReceiver(_) => write!(f, "ChannelReceiver"),
            Value::AsyncMutex(_) => write!(f, "AsyncMutex"),
            Value::Closure(c) => write!(f, "Closure({:?})", c.func),
            Value::SharedValue(s) => s.with(|v| write!(f, "SharedValue({:?})", v)),
        }
    }
}

/// Runtime error type with source span information
#[derive(Debug, Error, Clone, PartialEq)]
pub enum RuntimeError {
    /// Type error
    #[error("Type error: {msg}")]
    TypeError {
        msg: String,
        span: crate::span::Span,
    },
    /// Undefined variable
    #[error("Undefined variable: {name}")]
    UndefinedVariable {
        name: String,
        span: crate::span::Span,
    },
    /// Division by zero
    #[error("Division by zero")]
    DivideByZero { span: crate::span::Span },
    /// Array index out of bounds
    #[error("Array index out of bounds")]
    OutOfBounds { span: crate::span::Span },
    /// Invalid numeric result (NaN, Infinity)
    #[error("Invalid numeric result")]
    InvalidNumericResult { span: crate::span::Span },
    /// Unknown opcode (VM error)
    #[error("Unknown opcode")]
    UnknownOpcode { span: crate::span::Span },
    /// Stack underflow (VM error)
    #[error("Stack underflow")]
    StackUnderflow { span: crate::span::Span },
    /// Unknown function
    #[error("Unknown function: {name}")]
    UnknownFunction {
        name: String,
        span: crate::span::Span,
    },
    /// Invalid stdlib argument
    #[error("{msg}")]
    InvalidStdlibArgument {
        msg: String,
        span: crate::span::Span,
    },
    /// Invalid index (non-integer)
    #[error("Invalid index: expected number")]
    InvalidIndex { span: crate::span::Span },
    /// Permission denied - filesystem
    #[error("Permission denied: {operation} access to {path}")]
    FilesystemPermissionDenied {
        operation: String,
        path: String,
        span: crate::span::Span,
    },
    /// Permission denied - network
    #[error("Permission denied: network access to {host}")]
    NetworkPermissionDenied {
        host: String,
        span: crate::span::Span,
    },
    /// Permission denied - process
    #[error("Permission denied: process execution of {command}")]
    ProcessPermissionDenied {
        command: String,
        span: crate::span::Span,
    },
    /// Permission denied - environment
    #[error("Permission denied: environment variable {var}")]
    EnvironmentPermissionDenied {
        var: String,
        span: crate::span::Span,
    },
    /// I/O error (file operations)
    #[error("I/O error: {message}")]
    IoError {
        message: String,
        span: crate::span::Span,
    },
    /// Unhashable type (collections)
    #[error("Cannot hash type {type_name} - only number, string, bool, null are hashable")]
    UnhashableType {
        type_name: String,
        span: crate::span::Span,
    },
}

impl RuntimeError {
    /// Get the source span for this error
    pub fn span(&self) -> crate::span::Span {
        match self {
            RuntimeError::TypeError { span, .. } => *span,
            RuntimeError::UndefinedVariable { span, .. } => *span,
            RuntimeError::DivideByZero { span } => *span,
            RuntimeError::OutOfBounds { span } => *span,
            RuntimeError::InvalidNumericResult { span } => *span,
            RuntimeError::UnknownOpcode { span } => *span,
            RuntimeError::StackUnderflow { span } => *span,
            RuntimeError::UnknownFunction { span, .. } => *span,
            RuntimeError::InvalidStdlibArgument { span, .. } => *span,
            RuntimeError::InvalidIndex { span } => *span,
            RuntimeError::FilesystemPermissionDenied { span, .. } => *span,
            RuntimeError::NetworkPermissionDenied { span, .. } => *span,
            RuntimeError::ProcessPermissionDenied { span, .. } => *span,
            RuntimeError::EnvironmentPermissionDenied { span, .. } => *span,
            RuntimeError::IoError { span, .. } => *span,
            RuntimeError::UnhashableType { span, .. } => *span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_creation() {
        let val = Value::Number(42.0);
        assert_eq!(val.to_display_string(), "42");
    }

    #[test]
    fn test_string_value() {
        let val = Value::string("hello");
        assert_eq!(val.to_display_string(), "hello");
    }

    #[test]
    fn test_array_value() {
        let val = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);
        assert_eq!(val.to_display_string(), "[1, 2]");
    }

    #[test]
    fn test_type_names() {
        assert_eq!(Value::Number(42.0).type_name(), "number");
        assert_eq!(Value::string("hi").type_name(), "string");
        assert_eq!(Value::Bool(true).type_name(), "bool");
        assert_eq!(Value::Null.type_name(), "null");
        assert_eq!(Value::array(vec![]).type_name(), "array");
        assert_eq!(
            Value::Function(FunctionRef {
                name: "test".to_string(),
                arity: 0,
                bytecode_offset: 0,
                local_count: 0,
            })
            .type_name(),
            "function"
        );
    }

    #[test]
    fn test_is_truthy() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(!Value::Number(1.0).is_truthy()); // Numbers are not truthy
        assert!(!Value::Null.is_truthy());
        assert!(!Value::string("hello").is_truthy());
    }

    #[test]
    fn test_to_string_number() {
        assert_eq!(Value::Number(42.0).to_string(), "42");
        assert_eq!(Value::Number(2.5).to_string(), "2.5");
        assert_eq!(Value::Number(-5.0).to_string(), "-5");
    }

    #[test]
    fn test_to_string_string() {
        assert_eq!(Value::string("hello").to_string(), "hello");
    }

    #[test]
    fn test_to_string_bool() {
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Bool(false).to_string(), "false");
    }

    #[test]
    fn test_to_string_null() {
        assert_eq!(Value::Null.to_string(), "null");
    }

    #[test]
    fn test_to_string_array() {
        let arr = Value::array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        assert_eq!(arr.to_string(), "[1, 2, 3]");
    }

    #[test]
    fn test_to_string_nested_array() {
        let inner = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);
        let outer = Value::array(vec![inner, Value::Number(3.0)]);
        assert_eq!(outer.to_string(), "[[1, 2], 3]");
    }

    #[test]
    fn test_to_string_function() {
        let func = Value::Function(FunctionRef {
            name: "test".to_string(),
            arity: 2,
            bytecode_offset: 0,
            local_count: 0,
        });
        assert_eq!(func.to_string(), "<fn test>");
    }

    #[test]
    fn test_equality_numbers() {
        assert_eq!(Value::Number(42.0), Value::Number(42.0));
        assert_ne!(Value::Number(42.0), Value::Number(43.0));
    }

    #[test]
    fn test_equality_strings() {
        assert_eq!(Value::string("hello"), Value::string("hello"));
        assert_ne!(Value::string("hello"), Value::string("world"));
    }

    #[test]
    fn test_equality_bools() {
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_ne!(Value::Bool(true), Value::Bool(false));
    }

    #[test]
    fn test_equality_null() {
        assert_eq!(Value::Null, Value::Null);
    }

    #[test]
    fn test_equality_different_types() {
        assert_ne!(Value::Number(1.0), Value::Bool(true));
        assert_ne!(Value::Null, Value::Number(0.0));
    }

    #[test]
    fn test_array_value_equality() {
        let arr1 = Value::array(vec![Value::Number(1.0)]);
        let arr2 = arr1.clone(); // Shared Arc — same content
        let arr3 = Value::array(vec![Value::Number(1.0)]); // Different allocation

        assert_eq!(arr1, arr2); // same content
        assert_eq!(arr1, arr3); // content equal — value semantics
    }

    #[test]
    fn test_array_cow_mutation_independent() {
        let arr1 = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);
        let mut arr2 = arr1.clone(); // Shared Arc before mutation

        // Mutate arr2 — triggers CoW, arr1 is unaffected
        if let Value::Array(ref mut a) = arr2 {
            a.set(0, Value::Number(42.0));
        }

        // arr1 still has original value
        if let Value::Array(ref a) = arr1 {
            assert_eq!(a[0], Value::Number(1.0));
        }
    }

    #[test]
    fn value_array_clone_is_independent() {
        let a = Value::Array(ValueArray::from_vec(vec![Value::Number(1.0)]));
        let mut b = a.clone();
        if let Value::Array(ref mut arr) = b {
            arr.push(Value::Number(2.0));
        }
        if let Value::Array(ref arr) = a {
            assert_eq!(arr.len(), 1);
        }
    }

    #[test]
    fn value_array_equality_is_by_content() {
        let a = Value::Array(ValueArray::from_vec(vec![Value::Number(1.0)]));
        let b = Value::Array(ValueArray::from_vec(vec![Value::Number(1.0)]));
        assert_eq!(a, b); // content equal, different allocation
    }

    #[test]
    fn test_function_equality() {
        let func1 = Value::Function(FunctionRef {
            name: "test".to_string(),
            arity: 0,
            bytecode_offset: 0,
            local_count: 0,
        });
        let func2 = Value::Function(FunctionRef {
            name: "test".to_string(),
            arity: 1,
            bytecode_offset: 100,
            local_count: 0,
        });
        let func3 = Value::Function(FunctionRef {
            name: "other".to_string(),
            arity: 0,
            bytecode_offset: 0,
            local_count: 0,
        });

        assert_eq!(func1, func2); // Same name, different arity/offset
        assert_ne!(func1, func3); // Different name
    }

    #[test]
    fn test_runtime_errors() {
        use crate::span::Span;

        let err1 = RuntimeError::DivideByZero {
            span: Span::dummy(),
        };
        let err2 = RuntimeError::OutOfBounds {
            span: Span::dummy(),
        };
        let err3 = RuntimeError::UnknownFunction {
            name: "foo".to_string(),
            span: Span::dummy(),
        };

        assert_eq!(err1.to_string(), "Division by zero");
        assert_eq!(err2.to_string(), "Array index out of bounds");
        assert_eq!(err3.to_string(), "Unknown function: foo");
    }

    // =========================================================================
    // Value Send Tests (from value_send_test.rs, Phase Infra-01)
    // =========================================================================

    #[test]
    fn test_value_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Value>();
    }

    #[test]
    fn test_value_can_be_sent_to_thread() {
        use std::thread;

        let value = Value::String(Arc::new("test".to_string()));
        let handle = thread::spawn(move || value);
        let result = handle.join().unwrap();
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_array_can_be_sent_to_thread() {
        use std::thread;

        let arr = Value::Array(Arc::new(Mutex::new(vec![
            Value::Number(1.0),
            Value::Number(2.0),
        ])));
        let handle = thread::spawn(move || arr);
        let result = handle.join().unwrap();
        assert!(matches!(result, Value::Array(_)));
    }

    // =========================================================================
    // Value Model Tests (from value_model_tests.rs, Phase Infra-01)
    // =========================================================================

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_number_equality_same_values() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int = 42;
            let b: int = 42;
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(result, "42 should equal 42"),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_number_equality_different_values() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int = 42;
            let b: int = 43;
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(!result, "42 should not equal 43"),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_number_inequality() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int = 10;
            let b: int = 20;
            a != b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(result, "10 should not equal 20"),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_number_zero_equality() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int = 0;
            let b: int = 0;
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(result, "0 should equal 0"),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_number_negative_equality() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int = -5;
            let b: int = -5;
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(result, "-5 should equal -5"),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_string_equality_same_content() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: string = "hello";
            let b: string = "hello";
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(result, "Strings with same content should be equal"),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_string_equality_different_content() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: string = "hello";
            let b: string = "world";
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(!result, "Different strings should not be equal"),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_string_empty_equality() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: string = "";
            let b: string = "";
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(result, "Empty strings should be equal"),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_string_assignment_equality() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: string = "test";
            let b: string = a;
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(result, "Assigned strings should be equal"),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_bool_equality_both_true() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: bool = true;
            let b: bool = true;
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(result, "true should equal true"),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_bool_equality_both_false() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: bool = false;
            let b: bool = false;
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(result, "false should equal false"),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_bool_equality_different() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: bool = true;
            let b: bool = false;
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(!result, "true should not equal false"),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_null_equality() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int? = null;
            let b: int? = null;
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(result, "null should equal null"),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_null_inequality_with_value() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int? = null;
            let b: int? = 42;
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(!result, "null should not equal a value"),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready (should fail at type checking)"]
    fn test_number_string_type_mismatch() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int = 42;
            let b: string = "42";
            a == b
        "#;
        assert!(
            runtime.eval(code).is_err(),
            "Should reject comparing int and string"
        );
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_array_reference_equality_same_reference() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int[] = [1, 2, 3];
            let b: int[] = a;
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(result, "Same array reference should be equal"),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_array_reference_equality_different_references() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int[] = [1, 2, 3];
            let b: int[] = [1, 2, 3];
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(
                !result,
                "Different array references should not be equal even with same contents"
            ),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_array_reference_equality_empty_arrays() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int[] = [];
            let b: int[] = [];
            a == b
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(
                !result,
                "Different empty array references should not be equal"
            ),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_array_reference_chain() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int[] = [1, 2];
            let b: int[] = a;
            let c: int[] = b;
            a == c
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => {
                assert!(result, "Chained references should point to same array")
            }
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_array_mutation_visible_through_alias() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int[] = [1, 2, 3];
            let b: int[] = a;
            a[0] = 42;
            b[0]
        "#;
        match runtime.eval(code) {
            Ok(Value::Number(n)) => assert_eq!(n, 42.0, "Mutation should be visible through alias"),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_array_mutation_bidirectional() {
        let runtime = crate::Atlas::new();
        let code1 = r#"
            let arr1: int[] = [1, 2, 3];
            let arr2: int[] = arr1;
            arr1[1] = 99;
            arr2[1]
        "#;
        match runtime.eval(code1) {
            Ok(Value::Number(n)) => {
                assert_eq!(n, 99.0, "Mutation via arr1 should be visible in arr2")
            }
            _ => panic!("Expected Number(99.0)"),
        }

        let code2 = r#"
            let arr1: int[] = [1, 2, 3];
            let arr2: int[] = arr1;
            arr2[2] = 88;
            arr1[2]
        "#;
        match runtime.eval(code2) {
            Ok(Value::Number(n)) => {
                assert_eq!(n, 88.0, "Mutation via arr2 should be visible in arr1")
            }
            _ => panic!("Expected Number(88.0)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_array_mutation_multiple_aliases() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int[] = [10, 20, 30];
            let b: int[] = a;
            let c: int[] = a;
            let d: int[] = b;
            c[0] = 100;
            d[0]
        "#;
        match runtime.eval(code) {
            Ok(Value::Number(n)) => {
                assert_eq!(n, 100.0, "Mutation should be visible through all aliases")
            }
            _ => panic!("Expected Number(100.0)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_array_independent_arrays_no_interference() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int[] = [1, 2, 3];
            let b: int[] = [1, 2, 3];
            a[0] = 99;
            b[0]
        "#;
        match runtime.eval(code) {
            Ok(Value::Number(n)) => {
                assert_eq!(n, 1.0, "Independent arrays should not affect each other")
            }
            _ => panic!("Expected Number(1.0)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_array_mutation_preserves_other_elements() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int[] = [1, 2, 3, 4, 5];
            let b: int[] = a;
            a[2] = 999;
            b[4]
        "#;
        match runtime.eval(code) {
            Ok(Value::Number(n)) => assert_eq!(n, 5.0, "Mutation should not affect other elements"),
            _ => panic!("Expected Number(5.0)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready (should fail at type checking)"]
    fn test_string_immutability() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let s: string = "hello";
            s[0] = "H";
        "#;
        assert!(runtime.eval(code).is_err(), "Should reject string mutation");
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_array_content_equality_primitive_types() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let a: int[] = [1, 2, 3];
            let b: int[] = a;
            a[0] == b[0]
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(result)) => assert!(result, "Array elements should be equal"),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    #[ignore = "Enable when interpreter is ready"]
    fn test_value_semantics_documentation_example() {
        let runtime = crate::Atlas::new();
        let code = r#"
            let n1: int = 42;
            let n2: int = 42;
            let numbers_equal: bool = n1 == n2;
            let arr1: int[] = [1, 2];
            let arr2: int[] = arr1;
            let arr3: int[] = [1, 2];
            let same_ref: bool = arr1 == arr2;
            let diff_ref: bool = arr1 == arr3;
            arr1[0] = 99;
            let seen_by_arr2: int = arr2[0];
            let not_seen_by_arr3: int = arr3[0];
            numbers_equal
        "#;
        match runtime.eval(code) {
            Ok(Value::Bool(b)) => assert!(b),
            _ => panic!("Expected successful execution"),
        }
    }
}

#[cfg(test)]
mod cow_type_tests {
    use super::*;

    #[test]
    fn value_array_cow_push_does_not_affect_clone() {
        let mut a = ValueArray::from_vec(vec![Value::Number(1.0)]);
        let b = a.clone();
        a.push(Value::Number(2.0));
        assert_eq!(a.len(), 2);
        assert_eq!(b.len(), 1); // b is unaffected
    }

    #[test]
    fn value_array_in_place_mutation_when_exclusive() {
        let mut a = ValueArray::from_vec(vec![Value::Number(1.0)]);
        assert!(a.is_exclusively_owned());
        a.push(Value::Number(2.0)); // no copy — exclusive owner
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn value_array_equality_by_content() {
        let a = ValueArray::from_vec(vec![Value::Number(1.0), Value::Number(2.0)]);
        let b = ValueArray::from_vec(vec![Value::Number(1.0), Value::Number(2.0)]);
        assert_eq!(a, b); // same content, different Arc
    }

    #[test]
    fn value_map_cow_insert_does_not_affect_clone() {
        let mut a = ValueMap::new();
        a.insert("x".to_string(), Value::Number(1.0));
        let b = a.clone();
        a.insert("y".to_string(), Value::Number(2.0));
        assert_eq!(a.len(), 2);
        assert_eq!(b.len(), 1); // b is unaffected
    }

    #[test]
    fn value_hashmap_cow_insert_does_not_affect_clone() {
        let mut a = ValueHashMap::new();
        a.inner_mut().insert("x".to_string(), Value::Number(1.0));
        let b = a.clone();
        a.inner_mut().insert("y".to_string(), Value::Number(2.0));
        assert_eq!(b.inner().len(), 1);
    }

    #[test]
    fn value_collection_equality_by_content() {
        let mut a = ValueHashMap::new();
        a.inner_mut().insert("k".to_string(), Value::Number(1.0));
        let mut b = ValueHashMap::new();
        b.inner_mut().insert("k".to_string(), Value::Number(1.0));
        assert_eq!(a, b);
    }

    #[test]
    fn value_map_equality_by_content() {
        let mut a = ValueMap::new();
        a.insert("k".to_string(), Value::Number(42.0));
        let mut b = ValueMap::new();
        b.insert("k".to_string(), Value::Number(42.0));
        assert_eq!(a, b);
    }
}

#[cfg(test)]
mod equality_tests {
    use super::*;

    #[test]
    fn array_equality_by_content_not_identity() {
        let a = Value::Array(ValueArray::from_vec(vec![Value::Number(1.0)]));
        let b = Value::Array(ValueArray::from_vec(vec![Value::Number(1.0)]));
        assert_eq!(a, b); // different allocations, same content
    }

    #[test]
    fn array_inequality_after_mutation() {
        let a = Value::Array(ValueArray::from_vec(vec![Value::Number(1.0)]));
        let mut b = a.clone();
        if let Value::Array(ref mut arr) = b {
            arr.push(Value::Number(2.0));
        }
        assert_ne!(a, b);
    }

    #[test]
    fn regex_equality_by_pattern() {
        use regex::Regex;
        let a = Value::Regex(Arc::new(Regex::new(r"\d+").unwrap()));
        let b = Value::Regex(Arc::new(Regex::new(r"\d+").unwrap()));
        assert_eq!(a, b);
    }

    #[test]
    fn native_function_inequality_different_closures() {
        let f1: NativeFn = Arc::new(|_| Ok(Value::Null));
        let f2: NativeFn = Arc::new(|_| Ok(Value::Null));
        let a = Value::NativeFunction(f1);
        let b = Value::NativeFunction(f2);
        assert_ne!(a, b); // different closures — identity inequality
    }
}

#[cfg(test)]
mod shared_tests {
    use super::*;

    #[test]
    fn shared_mutation_visible_through_all_aliases() {
        let s = Shared::new(42i64);
        let s2 = s.clone();
        s.with_mut(|v| *v = 100);
        assert_eq!(s2.with(|v| *v), 100);
    }

    #[test]
    fn shared_equality_is_reference_not_content() {
        let a: Shared<i64> = Shared::new(42);
        let b: Shared<i64> = Shared::new(42); // same content, different allocation
        let c = a.clone(); // same allocation as a
        assert_ne!(a, b); // different references — not equal
        assert_eq!(a, c); // same reference — equal
    }

    #[test]
    fn value_shared_clone_shares_mutation() {
        let original = Value::SharedValue(Shared::new(Box::new(Value::Number(1.0))));
        let alias = original.clone();
        if let Value::SharedValue(ref s) = original {
            s.with_mut(|v| **v = Value::Number(99.0));
        }
        if let Value::SharedValue(ref s) = alias {
            s.with(|v| assert_eq!(**v, Value::Number(99.0)));
        }
    }
}
