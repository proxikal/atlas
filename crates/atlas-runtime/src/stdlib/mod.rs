//! Standard library functions

pub mod array;
pub mod async_io;
pub mod async_primitives;
pub mod collections;
pub mod compression;
pub mod datetime;
pub mod fs;
pub mod future;
pub mod http;
pub mod io;
pub mod json;
pub mod math;
pub mod path;
pub mod process;
pub mod reflect;
pub mod regex;
pub mod string;
pub mod test;
pub mod types;

use crate::security::SecurityContext;
use crate::value::{RuntimeError, Value};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};

/// Shared, thread-safe output writer.
/// Default implementation writes to stdout.
pub type OutputWriter = Arc<Mutex<Box<dyn Write + Send>>>;

/// Construct a writer that goes to real stdout (the default).
pub fn stdout_writer() -> OutputWriter {
    Arc::new(Mutex::new(Box::new(std::io::stdout())))
}

/// A builtin dispatch function: takes args, span, security, output → Result<Value, RuntimeError>
type BuiltinFn =
    fn(&[Value], crate::span::Span, &SecurityContext, &OutputWriter) -> Result<Value, RuntimeError>;

/// Construct an InvalidStdlibArgument error with context.
pub fn stdlib_arg_error(
    func_name: &str,
    expected: &str,
    actual: &Value,
    span: crate::span::Span,
) -> RuntimeError {
    RuntimeError::InvalidStdlibArgument {
        msg: format!(
            "{}(): expected {}, got {}",
            func_name,
            expected,
            actual.type_name()
        ),
        span,
    }
}

/// Construct an arity error for stdlib functions.
pub fn stdlib_arity_error(
    func_name: &str,
    expected: usize,
    actual: usize,
    span: crate::span::Span,
) -> RuntimeError {
    RuntimeError::InvalidStdlibArgument {
        msg: format!(
            "{}(): expected {} argument(s), got {}",
            func_name, expected, actual
        ),
        span,
    }
}

static BUILTIN_REGISTRY: OnceLock<HashMap<&'static str, BuiltinFn>> = OnceLock::new();

fn builtin_registry() -> &'static HashMap<&'static str, BuiltinFn> {
    BUILTIN_REGISTRY.get_or_init(|| {
        let mut m: HashMap<&'static str, BuiltinFn> = HashMap::with_capacity(300);

        // ====================================================================
        // Core
        // ====================================================================
        m.insert("print", |args, span, _, output| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("print", 1, args.len(), span));
            }
            print(&args[0], span, output)?;
            Ok(Value::Null)
        });
        m.insert("len", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("len", 1, args.len(), span));
            }
            Ok(Value::Number(len(&args[0], span)?))
        });
        m.insert("str", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("str", 1, args.len(), span));
            }
            let s = str(&args[0], span)?;
            Ok(Value::string(s))
        });

        // ====================================================================
        // String functions
        // ====================================================================
        m.insert("split", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("split", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "split", span)?;
            let sep = extract_string(&args[1], "split", span)?;
            string::split(s, sep, span)
        });
        m.insert("join", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("join", 2, args.len(), span));
            }
            let arr = extract_array(&args[0], "join", span)?;
            let sep = extract_string(&args[1], "join", span)?;
            let result = string::join(&arr, sep, span)?;
            Ok(Value::string(result))
        });
        m.insert("trim", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("trim", 1, args.len(), span));
            }
            let s = extract_string(&args[0], "trim", span)?;
            Ok(Value::string(string::trim(s)))
        });
        m.insert("trimStart", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("trimStart", 1, args.len(), span));
            }
            let s = extract_string(&args[0], "trimStart", span)?;
            Ok(Value::string(string::trim_start(s)))
        });
        m.insert("trimEnd", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("trimEnd", 1, args.len(), span));
            }
            let s = extract_string(&args[0], "trimEnd", span)?;
            Ok(Value::string(string::trim_end(s)))
        });
        m.insert("indexOf", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("indexOf", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "indexOf", span)?;
            let search = extract_string(&args[1], "indexOf", span)?;
            Ok(Value::Number(string::index_of(s, search)))
        });
        m.insert("lastIndexOf", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("lastIndexOf", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "lastIndexOf", span)?;
            let search = extract_string(&args[1], "lastIndexOf", span)?;
            Ok(Value::Number(string::last_index_of(s, search)))
        });
        m.insert("includes", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("includes", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "includes", span)?;
            let search = extract_string(&args[1], "includes", span)?;
            Ok(Value::Bool(string::includes(s, search)))
        });
        m.insert("toUpperCase", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("toUpperCase", 1, args.len(), span));
            }
            let s = extract_string(&args[0], "toUpperCase", span)?;
            Ok(Value::string(string::to_upper_case(s)))
        });
        m.insert("toLowerCase", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("toLowerCase", 1, args.len(), span));
            }
            let s = extract_string(&args[0], "toLowerCase", span)?;
            Ok(Value::string(string::to_lower_case(s)))
        });
        m.insert("substring", |args, span, _, _| {
            if args.len() != 3 {
                return Err(stdlib_arity_error("substring", 3, args.len(), span));
            }
            let s = extract_string(&args[0], "substring", span)?;
            let start = extract_number(&args[1], "substring", span)?;
            let end = extract_number(&args[2], "substring", span)?;
            let result = string::substring(s, start, end, span)?;
            Ok(Value::string(result))
        });
        m.insert("charAt", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("charAt", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "charAt", span)?;
            let index = extract_number(&args[1], "charAt", span)?;
            let result = string::char_at(s, index, span)?;
            Ok(Value::string(result))
        });
        m.insert("repeat", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("repeat", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "repeat", span)?;
            let count = extract_number(&args[1], "repeat", span)?;
            let result = string::repeat(s, count, span)?;
            Ok(Value::string(result))
        });
        m.insert("replace", |args, span, _, _| {
            if args.len() != 3 {
                return Err(stdlib_arity_error("replace", 3, args.len(), span));
            }
            let s = extract_string(&args[0], "replace", span)?;
            let search = extract_string(&args[1], "replace", span)?;
            let replacement = extract_string(&args[2], "replace", span)?;
            Ok(Value::string(string::replace(s, search, replacement)))
        });
        m.insert("padStart", |args, span, _, _| {
            if args.len() != 3 {
                return Err(stdlib_arity_error("padStart", 3, args.len(), span));
            }
            let s = extract_string(&args[0], "padStart", span)?;
            let length = extract_number(&args[1], "padStart", span)?;
            let fill = extract_string(&args[2], "padStart", span)?;
            let result = string::pad_start(s, length, fill, span)?;
            Ok(Value::string(result))
        });
        m.insert("padEnd", |args, span, _, _| {
            if args.len() != 3 {
                return Err(stdlib_arity_error("padEnd", 3, args.len(), span));
            }
            let s = extract_string(&args[0], "padEnd", span)?;
            let length = extract_number(&args[1], "padEnd", span)?;
            let fill = extract_string(&args[2], "padEnd", span)?;
            let result = string::pad_end(s, length, fill, span)?;
            Ok(Value::string(result))
        });
        m.insert("startsWith", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("startsWith", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "startsWith", span)?;
            let prefix = extract_string(&args[1], "startsWith", span)?;
            Ok(Value::Bool(string::starts_with(s, prefix)))
        });
        m.insert("endsWith", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("endsWith", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "endsWith", span)?;
            let suffix = extract_string(&args[1], "endsWith", span)?;
            Ok(Value::Bool(string::ends_with(s, suffix)))
        });

        // ====================================================================
        // Array functions
        // ====================================================================
        m.insert("pop", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pop", 1, args.len(), span));
            }
            let arr = extract_array(&args[0], "pop", span)?;
            array::pop(&arr, span)
        });
        m.insert("shift", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("shift", 1, args.len(), span));
            }
            let arr = extract_array(&args[0], "shift", span)?;
            array::shift(&arr, span)
        });
        m.insert("unshift", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("unshift", 2, args.len(), span));
            }
            let arr = extract_array(&args[0], "unshift", span)?;
            Ok(array::unshift(&arr, args[1].clone()))
        });
        m.insert("reverse", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("reverse", 1, args.len(), span));
            }
            let arr = extract_array(&args[0], "reverse", span)?;
            Ok(array::reverse(&arr))
        });
        m.insert("concat", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("concat", 2, args.len(), span));
            }
            let arr1 = extract_array(&args[0], "concat", span)?;
            let arr2 = extract_array(&args[1], "concat", span)?;
            Ok(array::concat(&arr1, &arr2))
        });
        m.insert("flatten", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("flatten", 1, args.len(), span));
            }
            let arr = extract_array(&args[0], "flatten", span)?;
            array::flatten(&arr, span)
        });
        m.insert("arrayIndexOf", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("arrayIndexOf", 2, args.len(), span));
            }
            let arr = extract_array(&args[0], "arrayIndexOf", span)?;
            Ok(Value::Number(array::index_of(&arr, &args[1])))
        });
        m.insert("arrayLastIndexOf", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("arrayLastIndexOf", 2, args.len(), span));
            }
            let arr = extract_array(&args[0], "arrayLastIndexOf", span)?;
            Ok(Value::Number(array::last_index_of(&arr, &args[1])))
        });
        m.insert("arrayIncludes", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("arrayIncludes", 2, args.len(), span));
            }
            let arr = extract_array(&args[0], "arrayIncludes", span)?;
            Ok(Value::Bool(array::includes(&arr, &args[1])))
        });
        m.insert("slice", |args, span, _, _| {
            if args.len() != 3 {
                return Err(stdlib_arity_error("slice", 3, args.len(), span));
            }
            let arr = extract_array(&args[0], "slice", span)?;
            let start = extract_number(&args[1], "slice", span)?;
            let end = extract_number(&args[2], "slice", span)?;
            array::slice(&arr, start, end, span)
        });

        // ====================================================================
        // Math functions
        // ====================================================================
        m.insert("abs", |a, s, _, _| math::abs(a, s));
        m.insert("floor", |a, s, _, _| math::floor(a, s));
        m.insert("ceil", |a, s, _, _| math::ceil(a, s));
        m.insert("round", |a, s, _, _| math::round(a, s));
        m.insert("min", |a, s, _, _| math::min(a, s));
        m.insert("max", |a, s, _, _| math::max(a, s));
        m.insert("sqrt", |a, s, _, _| math::sqrt(a, s));
        m.insert("pow", |a, s, _, _| math::pow(a, s));
        m.insert("log", |a, s, _, _| math::log(a, s));
        m.insert("sin", |a, s, _, _| math::sin(a, s));
        m.insert("cos", |a, s, _, _| math::cos(a, s));
        m.insert("tan", |a, s, _, _| math::tan(a, s));
        m.insert("asin", |a, s, _, _| math::asin(a, s));
        m.insert("acos", |a, s, _, _| math::acos(a, s));
        m.insert("atan", |a, s, _, _| math::atan(a, s));
        m.insert("clamp", |a, s, _, _| math::clamp(a, s));
        m.insert("sign", |a, s, _, _| math::sign(a, s));
        m.insert("random", |a, s, _, _| math::random(a, s));

        // ====================================================================
        // JSON functions
        // ====================================================================
        m.insert("parseJSON", |a, s, _, _| json::parse_json(a, s));
        m.insert("toJSON", |a, s, _, _| json::to_json(a, s));
        m.insert("isValidJSON", |a, s, _, _| json::is_valid_json(a, s));
        m.insert("prettifyJSON", |a, s, _, _| json::prettify_json(a, s));
        m.insert("minifyJSON", |a, s, _, _| json::minify_json(a, s));
        m.insert("jsonAsString", |a, s, _, _| json::json_as_string(a, s));
        m.insert("jsonAsNumber", |a, s, _, _| json::json_as_number(a, s));
        m.insert("jsonAsBool", |a, s, _, _| json::json_as_bool(a, s));
        m.insert("jsonIsNull", |a, s, _, _| json::json_is_null(a, s));

        // ====================================================================
        // Type checking functions
        // ====================================================================
        m.insert("typeof", |a, s, _, _| types::type_of(a, s));
        m.insert("isString", |a, s, _, _| types::is_string(a, s));
        m.insert("isNumber", |a, s, _, _| types::is_number(a, s));
        m.insert("isBool", |a, s, _, _| types::is_bool(a, s));
        m.insert("isNull", |a, s, _, _| types::is_null(a, s));
        m.insert("isArray", |a, s, _, _| types::is_array(a, s));
        m.insert("isFunction", |a, s, _, _| types::is_function(a, s));
        m.insert("isObject", |a, s, _, _| types::is_object(a, s));
        m.insert("isType", |a, s, _, _| types::is_type(a, s));
        m.insert("hasField", |a, s, _, _| types::has_field(a, s));
        m.insert("hasMethod", |a, s, _, _| types::has_method(a, s));
        m.insert("hasTag", |a, s, _, _| types::has_tag(a, s));

        // ====================================================================
        // Type conversion functions
        // ====================================================================
        m.insert("toString", |a, s, _, _| types::to_string(a, s));
        m.insert("toNumber", |a, s, _, _| types::to_number(a, s));
        m.insert("toBool", |a, s, _, _| types::to_bool(a, s));
        m.insert("parseInt", |a, s, _, _| types::parse_int(a, s));
        m.insert("parseFloat", |a, s, _, _| types::parse_float(a, s));

        // ====================================================================
        // Option<T> constructors and helpers
        // ====================================================================
        m.insert("Some", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("Some", 1, args.len(), span));
            }
            Ok(types::some(args[0].clone()))
        });
        m.insert("None", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("None", 0, args.len(), span));
            }
            Ok(types::none())
        });
        m.insert("is_some", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("is_some", 1, args.len(), span));
            }
            Ok(Value::Bool(types::is_some(&args[0], span)?))
        });
        m.insert("is_none", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("is_none", 1, args.len(), span));
            }
            Ok(Value::Bool(types::is_none(&args[0], span)?))
        });

        // ====================================================================
        // Result<T,E> constructors and helpers
        // ====================================================================
        m.insert("Ok", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("Ok", 1, args.len(), span));
            }
            Ok(types::ok(args[0].clone()))
        });
        m.insert("Err", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("Err", 1, args.len(), span));
            }
            Ok(types::err(args[0].clone()))
        });
        m.insert("is_ok", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("is_ok", 1, args.len(), span));
            }
            Ok(Value::Bool(types::is_ok(&args[0], span)?))
        });
        m.insert("is_err", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("is_err", 1, args.len(), span));
            }
            Ok(Value::Bool(types::is_err(&args[0], span)?))
        });

        // ====================================================================
        // Generic unwrap functions (Option + Result)
        // ====================================================================
        m.insert("unwrap", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("unwrap", 1, args.len(), span));
            }
            match &args[0] {
                Value::Option(_) => types::unwrap_option(&args[0], span),
                Value::Result(_) => types::unwrap_result(&args[0], span),
                _ => Err(RuntimeError::TypeError {
                    msg: "unwrap() requires Option or Result value".to_string(),
                    span,
                }),
            }
        });
        m.insert("unwrap_or", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("unwrap_or", 2, args.len(), span));
            }
            match &args[0] {
                Value::Option(_) => types::unwrap_or_option(&args[0], args[1].clone(), span),
                Value::Result(_) => types::unwrap_or_result(&args[0], args[1].clone(), span),
                _ => Err(RuntimeError::TypeError {
                    msg: "unwrap_or() requires Option or Result value".to_string(),
                    span,
                }),
            }
        });
        m.insert("expect", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("expect", 2, args.len(), span));
            }
            let message = extract_string(&args[1], "expect", span)?;
            match &args[0] {
                Value::Option(_) => types::expect_option(&args[0], message, span),
                Value::Result(_) => types::expect_result(&args[0], message, span),
                _ => Err(RuntimeError::TypeError {
                    msg: "expect() requires Option or Result value".to_string(),
                    span,
                }),
            }
        });
        m.insert("result_ok", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("result_ok", 1, args.len(), span));
            }
            types::result_ok(&args[0], span)
        });
        m.insert("result_err", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("result_err", 1, args.len(), span));
            }
            types::result_err(&args[0], span)
        });

        // ====================================================================
        // File I/O functions
        // ====================================================================
        m.insert("readFile", |a, s, sc, _| io::read_file(a, s, sc));
        m.insert("writeFile", |a, s, sc, _| io::write_file(a, s, sc));
        m.insert("appendFile", |a, s, sc, _| io::append_file(a, s, sc));
        m.insert("fileExists", |a, s, sc, _| io::file_exists(a, s, sc));
        m.insert("readDir", |a, s, sc, _| io::read_dir(a, s, sc));
        m.insert("createDir", |a, s, sc, _| io::create_dir(a, s, sc));
        m.insert("removeFile", |a, s, sc, _| io::remove_file(a, s, sc));
        m.insert("removeDir", |a, s, sc, _| io::remove_dir(a, s, sc));
        m.insert("fileInfo", |a, s, sc, _| io::file_info(a, s, sc));
        m.insert("pathJoin", |a, s, sc, _| io::path_join(a, s, sc));

        // ====================================================================
        // Reflection functions
        // ====================================================================
        m.insert("reflect_typeof", |a, s, _, _| reflect::typeof_fn(a, s));
        m.insert("reflect_is_callable", |a, s, _, _| {
            reflect::is_callable_fn(a, s)
        });
        m.insert("reflect_is_primitive", |a, s, _, _| {
            reflect::is_primitive_fn(a, s)
        });
        m.insert("reflect_same_type", |a, s, _, _| {
            reflect::same_type_fn(a, s)
        });
        m.insert("reflect_get_length", |a, s, _, _| {
            reflect::get_length_fn(a, s)
        });
        m.insert("reflect_is_empty", |a, s, _, _| reflect::is_empty_fn(a, s));
        m.insert("reflect_type_describe", |a, s, _, _| {
            reflect::type_describe_fn(a, s)
        });
        m.insert("reflect_clone", |a, s, _, _| reflect::clone_fn(a, s));
        m.insert("reflect_value_to_string", |a, s, _, _| {
            reflect::value_to_string_fn(a, s)
        });
        m.insert("reflect_deep_equals", |a, s, _, _| {
            reflect::deep_equals_fn(a, s)
        });
        m.insert("reflect_get_function_name", |a, s, _, _| {
            reflect::get_function_name_fn(a, s)
        });
        m.insert("reflect_get_function_arity", |a, s, _, _| {
            reflect::get_function_arity_fn(a, s)
        });

        // ====================================================================
        // HashMap functions
        // ====================================================================
        m.insert("hashMapNew", |a, s, _, _| {
            collections::hashmap::new_map(a, s)
        });
        m.insert("hashMapFromEntries", |a, s, _, _| {
            collections::hashmap::from_entries(a, s)
        });
        m.insert("hashMapPut", |a, s, _, _| collections::hashmap::put(a, s));
        m.insert("hashMapGet", |a, s, _, _| collections::hashmap::get(a, s));
        m.insert("hashMapRemove", |a, s, _, _| {
            collections::hashmap::remove(a, s)
        });
        m.insert("hashMapHas", |a, s, _, _| collections::hashmap::has(a, s));
        m.insert("hashMapSize", |a, s, _, _| collections::hashmap::size(a, s));
        m.insert("hashMapIsEmpty", |a, s, _, _| {
            collections::hashmap::is_empty(a, s)
        });
        m.insert("hashMapClear", |a, s, _, _| {
            collections::hashmap::clear(a, s)
        });
        m.insert("hashMapKeys", |a, s, _, _| collections::hashmap::keys(a, s));
        m.insert("hashMapValues", |a, s, _, _| {
            collections::hashmap::values(a, s)
        });
        m.insert("hashMapEntries", |a, s, _, _| {
            collections::hashmap::entries(a, s)
        });

        // ====================================================================
        // HashSet functions
        // ====================================================================
        m.insert("hashSetNew", |a, s, _, _| {
            collections::hashset::new_set(a, s)
        });
        m.insert("hashSetFromArray", |a, s, _, _| {
            collections::hashset::from_array(a, s)
        });
        m.insert("hashSetAdd", |a, s, _, _| collections::hashset::add(a, s));
        m.insert("hashSetRemove", |a, s, _, _| {
            collections::hashset::remove(a, s)
        });
        m.insert("hashSetHas", |a, s, _, _| collections::hashset::has(a, s));
        m.insert("hashSetSize", |a, s, _, _| collections::hashset::size(a, s));
        m.insert("hashSetIsEmpty", |a, s, _, _| {
            collections::hashset::is_empty(a, s)
        });
        m.insert("hashSetClear", |a, s, _, _| {
            collections::hashset::clear(a, s)
        });
        m.insert("hashSetUnion", |a, s, _, _| {
            collections::hashset::union(a, s)
        });
        m.insert("hashSetIntersection", |a, s, _, _| {
            collections::hashset::intersection(a, s)
        });
        m.insert("hashSetDifference", |a, s, _, _| {
            collections::hashset::difference(a, s)
        });
        m.insert("hashSetSymmetricDifference", |a, s, _, _| {
            collections::hashset::symmetric_difference(a, s)
        });
        m.insert("hashSetIsSubset", |a, s, _, _| {
            collections::hashset::is_subset(a, s)
        });
        m.insert("hashSetIsSuperset", |a, s, _, _| {
            collections::hashset::is_superset(a, s)
        });
        m.insert("hashSetToArray", |a, s, _, _| {
            collections::hashset::to_array(a, s)
        });

        // ====================================================================
        // Queue functions
        // ====================================================================
        m.insert("queueNew", |a, s, _, _| collections::queue::new_queue(a, s));
        m.insert("queueEnqueue", |a, s, _, _| {
            collections::queue::enqueue(a, s)
        });
        m.insert("queueDequeue", |a, s, _, _| {
            collections::queue::dequeue(a, s)
        });
        m.insert("queuePeek", |a, s, _, _| collections::queue::peek(a, s));
        m.insert("queueSize", |a, s, _, _| collections::queue::size(a, s));
        m.insert("queueIsEmpty", |a, s, _, _| {
            collections::queue::is_empty(a, s)
        });
        m.insert("queueClear", |a, s, _, _| collections::queue::clear(a, s));
        m.insert("queueToArray", |a, s, _, _| {
            collections::queue::to_array(a, s)
        });

        // ====================================================================
        // Stack functions
        // ====================================================================
        m.insert("stackNew", |a, s, _, _| collections::stack::new_stack(a, s));
        m.insert("stackPush", |a, s, _, _| collections::stack::push(a, s));
        m.insert("stackPop", |a, s, _, _| collections::stack::pop(a, s));
        m.insert("stackPeek", |a, s, _, _| collections::stack::peek(a, s));
        m.insert("stackSize", |a, s, _, _| collections::stack::size(a, s));
        m.insert("stackIsEmpty", |a, s, _, _| {
            collections::stack::is_empty(a, s)
        });
        m.insert("stackClear", |a, s, _, _| collections::stack::clear(a, s));
        m.insert("stackToArray", |a, s, _, _| {
            collections::stack::to_array(a, s)
        });

        // ====================================================================
        // Regex functions
        // ====================================================================
        m.insert("regexNew", |a, s, _, _| regex::regex_new(a, s));
        m.insert("regexNewWithFlags", |a, s, _, _| {
            regex::regex_new_with_flags(a, s)
        });
        m.insert("regexEscape", |a, s, _, _| regex::regex_escape(a, s));
        m.insert("regexIsMatch", |a, s, _, _| regex::regex_is_match(a, s));
        m.insert("regexFind", |a, s, _, _| regex::regex_find(a, s));
        m.insert("regexFindAll", |a, s, _, _| regex::regex_find_all(a, s));
        m.insert("regexCaptures", |a, s, _, _| regex::regex_captures(a, s));
        m.insert("regexCapturesNamed", |a, s, _, _| {
            regex::regex_captures_named(a, s)
        });
        m.insert("regexReplace", |a, s, _, _| regex::regex_replace(a, s));
        m.insert("regexReplaceAll", |a, s, _, _| {
            regex::regex_replace_all(a, s)
        });
        m.insert("regexSplit", |a, s, _, _| regex::regex_split(a, s));
        m.insert("regexSplitN", |a, s, _, _| regex::regex_split_n(a, s));
        m.insert("regexMatchIndices", |a, s, _, _| {
            regex::regex_match_indices(a, s)
        });
        m.insert("regexTest", |a, s, _, _| regex::regex_test(a, s));

        // ====================================================================
        // DateTime functions
        // ====================================================================
        m.insert("dateTimeNow", |a, s, _, _| datetime::date_time_now(a, s));
        m.insert("dateTimeFromTimestamp", |a, s, _, _| {
            datetime::date_time_from_timestamp(a, s)
        });
        m.insert("dateTimeFromComponents", |a, s, _, _| {
            datetime::date_time_from_components(a, s)
        });
        m.insert("dateTimeParseIso", |a, s, _, _| {
            datetime::date_time_parse_iso(a, s)
        });
        m.insert("dateTimeUtc", |a, s, _, _| datetime::date_time_utc(a, s));
        m.insert("dateTimeYear", |a, s, _, _| datetime::date_time_year(a, s));
        m.insert("dateTimeMonth", |a, s, _, _| {
            datetime::date_time_month(a, s)
        });
        m.insert("dateTimeDay", |a, s, _, _| datetime::date_time_day(a, s));
        m.insert("dateTimeHour", |a, s, _, _| datetime::date_time_hour(a, s));
        m.insert("dateTimeMinute", |a, s, _, _| {
            datetime::date_time_minute(a, s)
        });
        m.insert("dateTimeSecond", |a, s, _, _| {
            datetime::date_time_second(a, s)
        });
        m.insert("dateTimeWeekday", |a, s, _, _| {
            datetime::date_time_weekday(a, s)
        });
        m.insert("dateTimeDayOfYear", |a, s, _, _| {
            datetime::date_time_day_of_year(a, s)
        });
        m.insert("dateTimeAddSeconds", |a, s, _, _| {
            datetime::date_time_add_seconds(a, s)
        });
        m.insert("dateTimeAddMinutes", |a, s, _, _| {
            datetime::date_time_add_minutes(a, s)
        });
        m.insert("dateTimeAddHours", |a, s, _, _| {
            datetime::date_time_add_hours(a, s)
        });
        m.insert("dateTimeAddDays", |a, s, _, _| {
            datetime::date_time_add_days(a, s)
        });
        m.insert("dateTimeDiff", |a, s, _, _| datetime::date_time_diff(a, s));
        m.insert("dateTimeCompare", |a, s, _, _| {
            datetime::date_time_compare(a, s)
        });
        m.insert("dateTimeToTimestamp", |a, s, _, _| {
            datetime::date_time_to_timestamp(a, s)
        });
        m.insert("dateTimeToIso", |a, s, _, _| {
            datetime::date_time_to_iso(a, s)
        });
        m.insert("dateTimeFormat", |a, s, _, _| {
            datetime::date_time_format(a, s)
        });
        m.insert("dateTimeToRfc3339", |a, s, _, _| {
            datetime::date_time_to_rfc3339(a, s)
        });
        m.insert("dateTimeToRfc2822", |a, s, _, _| {
            datetime::date_time_to_rfc2822(a, s)
        });
        m.insert("dateTimeToCustom", |a, s, _, _| {
            datetime::date_time_to_custom(a, s)
        });
        m.insert("dateTimeParse", |a, s, _, _| {
            datetime::date_time_parse(a, s)
        });
        m.insert("dateTimeParseRfc3339", |a, s, _, _| {
            datetime::date_time_parse_rfc3339(a, s)
        });
        m.insert("dateTimeParseRfc2822", |a, s, _, _| {
            datetime::date_time_parse_rfc2822(a, s)
        });
        m.insert("dateTimeTryParse", |a, s, _, _| {
            datetime::date_time_try_parse(a, s)
        });
        m.insert("dateTimeToUtc", |a, s, _, _| {
            datetime::date_time_to_utc(a, s)
        });
        m.insert("dateTimeToLocal", |a, s, _, _| {
            datetime::date_time_to_local(a, s)
        });
        m.insert("dateTimeToTimezone", |a, s, _, _| {
            datetime::date_time_to_timezone(a, s)
        });
        m.insert("dateTimeGetTimezone", |a, s, _, _| {
            datetime::date_time_get_timezone(a, s)
        });
        m.insert("dateTimeGetOffset", |a, s, _, _| {
            datetime::date_time_get_offset(a, s)
        });
        m.insert("dateTimeInTimezone", |a, s, _, _| {
            datetime::date_time_in_timezone(a, s)
        });
        m.insert("durationFromSeconds", |a, s, _, _| {
            datetime::duration_from_seconds(a, s)
        });
        m.insert("durationFromMinutes", |a, s, _, _| {
            datetime::duration_from_minutes(a, s)
        });
        m.insert("durationFromHours", |a, s, _, _| {
            datetime::duration_from_hours(a, s)
        });
        m.insert("durationFromDays", |a, s, _, _| {
            datetime::duration_from_days(a, s)
        });
        m.insert("durationFormat", |a, s, _, _| {
            datetime::duration_format(a, s)
        });

        // ====================================================================
        // HTTP functions
        // ====================================================================
        m.insert("httpRequest", |a, s, _, _| http::http_request(a, s));
        m.insert("httpRequestGet", |a, s, _, _| http::http_request_get(a, s));
        m.insert("httpRequestPost", |a, s, _, _| {
            http::http_request_post(a, s)
        });
        m.insert("httpRequestPut", |a, s, _, _| http::http_request_put(a, s));
        m.insert("httpRequestDelete", |a, s, _, _| {
            http::http_request_delete(a, s)
        });
        m.insert("httpRequestPatch", |a, s, _, _| {
            http::http_request_patch(a, s)
        });
        m.insert("httpSetHeader", |a, s, _, _| http::http_set_header(a, s));
        m.insert("httpSetBody", |a, s, _, _| http::http_set_body(a, s));
        m.insert("httpSetTimeout", |a, s, _, _| http::http_set_timeout(a, s));
        m.insert("httpSetQuery", |a, s, _, _| http::http_set_query(a, s));
        m.insert("httpSetFollowRedirects", |a, s, _, _| {
            http::http_set_follow_redirects(a, s)
        });
        m.insert("httpSetMaxRedirects", |a, s, _, _| {
            http::http_set_max_redirects(a, s)
        });
        m.insert("httpSetUserAgent", |a, s, _, _| {
            http::http_set_user_agent(a, s)
        });
        m.insert("httpSetAuth", |a, s, _, _| http::http_set_auth(a, s));
        m.insert("httpStatus", |a, s, _, _| http::http_status(a, s));
        m.insert("httpBody", |a, s, _, _| http::http_body(a, s));
        m.insert("httpHeader", |a, s, _, _| http::http_header(a, s));
        m.insert("httpHeaders", |a, s, _, _| http::http_headers(a, s));
        m.insert("httpUrl", |a, s, _, _| http::http_url(a, s));
        m.insert("httpIsSuccess", |a, s, _, _| http::http_is_success(a, s));
        m.insert("httpStatusText", |a, s, _, _| http::http_status_text(a, s));
        m.insert("httpContentType", |a, s, _, _| {
            http::http_content_type(a, s)
        });
        m.insert("httpContentLength", |a, s, _, _| {
            http::http_content_length(a, s)
        });
        m.insert("httpIsRedirect", |a, s, _, _| http::http_is_redirect(a, s));
        m.insert("httpIsClientError", |a, s, _, _| {
            http::http_is_client_error(a, s)
        });
        m.insert("httpIsServerError", |a, s, _, _| {
            http::http_is_server_error(a, s)
        });
        m.insert("httpSend", |a, s, sec, _| http::http_send(a, s, sec));
        m.insert("httpGet", |a, s, sec, _| http::http_get(a, s, sec));
        m.insert("httpPost", |a, s, sec, _| http::http_post(a, s, sec));
        m.insert("httpPut", |a, s, sec, _| http::http_put(a, s, sec));
        m.insert("httpDelete", |a, s, sec, _| http::http_delete(a, s, sec));
        m.insert("httpPatch", |a, s, sec, _| http::http_patch(a, s, sec));
        m.insert("httpPostJson", |a, s, sec, _| {
            http::http_post_json(a, s, sec)
        });
        m.insert("httpParseJson", |a, s, _, _| http::http_parse_json(a, s));
        m.insert("httpGetJson", |a, s, sec, _| http::http_get_json(a, s, sec));
        m.insert("httpCheckPermission", |a, s, sec, _| {
            http::http_check_permission(a, s, sec)
        });

        // ====================================================================
        // Future/async functions
        // ====================================================================
        m.insert("futureResolve", |a, s, _, _| future::future_resolve(a, s));
        m.insert("futureReject", |a, s, _, _| future::future_reject(a, s));
        m.insert("futureNew", |a, s, _, _| future::future_new(a, s));
        m.insert("futureIsPending", |a, s, _, _| {
            future::future_is_pending(a, s)
        });
        m.insert("futureIsResolved", |a, s, _, _| {
            future::future_is_resolved(a, s)
        });
        m.insert("futureIsRejected", |a, s, _, _| {
            future::future_is_rejected(a, s)
        });
        m.insert("futureThen", |a, s, _, _| future::future_then(a, s));
        m.insert("futureCatch", |a, s, _, _| future::future_catch(a, s));
        m.insert("futureAll", |a, s, _, _| future::future_all_fn(a, s));
        m.insert("futureRace", |a, s, _, _| future::future_race_fn(a, s));

        // ====================================================================
        // Async I/O functions
        // ====================================================================
        m.insert("readFileAsync", |a, s, sc, _| {
            async_io::read_file_async(a, s, sc)
        });
        m.insert("writeFileAsync", |a, s, sc, _| {
            async_io::write_file_async(a, s, sc)
        });
        m.insert("appendFileAsync", |a, s, sc, _| {
            async_io::append_file_async(a, s, sc)
        });
        m.insert("httpSendAsync", |a, s, _, _| {
            async_io::http_send_async(a, s)
        });
        m.insert("httpGetAsync", |a, s, _, _| async_io::http_get_async(a, s));
        m.insert("httpPostAsync", |a, s, _, _| {
            async_io::http_post_async(a, s)
        });
        m.insert("httpPutAsync", |a, s, _, _| async_io::http_put_async(a, s));
        m.insert("httpDeleteAsync", |a, s, _, _| {
            async_io::http_delete_async(a, s)
        });
        m.insert("await", |a, s, _, _| async_io::await_future(a, s));

        // ====================================================================
        // Async primitives - tasks
        // ====================================================================
        m.insert("spawn", |a, s, _, _| async_primitives::spawn(a, s));
        m.insert("taskJoin", |a, s, _, _| async_primitives::task_join(a, s));
        m.insert("taskStatus", |a, s, _, _| {
            async_primitives::task_status(a, s)
        });
        m.insert("taskCancel", |a, s, _, _| {
            async_primitives::task_cancel(a, s)
        });
        m.insert("taskId", |a, s, _, _| async_primitives::task_id(a, s));
        m.insert("taskName", |a, s, _, _| async_primitives::task_name(a, s));
        m.insert("joinAll", |a, s, _, _| async_primitives::join_all(a, s));

        // Async primitives - channels
        m.insert("channelBounded", |a, s, _, _| {
            async_primitives::channel_bounded(a, s)
        });
        m.insert("channelUnbounded", |a, s, _, _| {
            async_primitives::channel_unbounded(a, s)
        });
        m.insert("channelSend", |a, s, _, _| {
            async_primitives::channel_send(a, s)
        });
        m.insert("channelReceive", |a, s, _, _| {
            async_primitives::channel_receive(a, s)
        });
        m.insert("channelSelect", |a, s, _, _| {
            async_primitives::channel_select(a, s)
        });
        m.insert("channelIsClosed", |a, s, _, _| {
            async_primitives::channel_is_closed(a, s)
        });

        // Async primitives - sleep/timers
        m.insert("sleep", |a, s, _, _| async_primitives::sleep_fn(a, s));
        m.insert("timer", |a, s, _, _| async_primitives::timer_fn(a, s));
        m.insert("interval", |a, s, _, _| async_primitives::interval_fn(a, s));

        // Async primitives - timeout
        m.insert("timeout", |a, s, _, _| async_primitives::timeout_fn(a, s));

        // Async primitives - mutex
        m.insert("asyncMutex", |a, s, _, _| {
            async_primitives::async_mutex_new(a, s)
        });
        m.insert("asyncMutexGet", |a, s, _, _| {
            async_primitives::async_mutex_get(a, s)
        });
        m.insert("asyncMutexSet", |a, s, _, _| {
            async_primitives::async_mutex_set(a, s)
        });

        // ====================================================================
        // Process management
        // ====================================================================
        m.insert("exec", |a, s, sc, _| process::exec(a, s, sc));
        m.insert("shell", |a, s, sc, _| process::shell(a, s, sc));
        m.insert("getEnv", |a, s, sc, _| process::get_env(a, s, sc));
        m.insert("setEnv", |a, s, sc, _| process::set_env(a, s, sc));
        m.insert("unsetEnv", |a, s, sc, _| process::unset_env(a, s, sc));
        m.insert("listEnv", |a, s, sc, _| process::list_env(a, s, sc));
        m.insert("getCwd", |a, s, sc, _| process::get_cwd(a, s, sc));
        m.insert("getPid", |a, s, sc, _| process::get_pid(a, s, sc));

        // ====================================================================
        // Path manipulation
        // ====================================================================
        m.insert("pathJoinArray", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathJoinArray", 1, args.len(), span));
            }
            let segments = extract_array(&args[0], "pathJoinArray", span)?;
            let result = path::path_join(&segments, span)?;
            Ok(Value::string(result))
        });
        m.insert("pathParse", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathParse", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathParse", span)?;
            path::path_parse(path_str, span)
        });
        m.insert("pathNormalize", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathNormalize", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathNormalize", span)?;
            Ok(Value::string(path::path_normalize(path_str, span)?))
        });
        m.insert("pathAbsolute", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathAbsolute", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathAbsolute", span)?;
            Ok(Value::string(path::path_absolute(path_str, span)?))
        });
        m.insert("pathRelative", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("pathRelative", 2, args.len(), span));
            }
            let from = extract_string(&args[0], "pathRelative", span)?;
            let to = extract_string(&args[1], "pathRelative", span)?;
            Ok(Value::string(path::path_relative(from, to, span)?))
        });
        m.insert("pathParent", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathParent", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathParent", span)?;
            Ok(Value::string(path::path_parent(path_str, span)?))
        });
        m.insert("pathBasename", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathBasename", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathBasename", span)?;
            Ok(Value::string(path::path_basename(path_str, span)?))
        });
        m.insert("pathDirname", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathDirname", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathDirname", span)?;
            Ok(Value::string(path::path_dirname(path_str, span)?))
        });
        m.insert("pathExtension", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathExtension", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathExtension", span)?;
            Ok(Value::string(path::path_extension(path_str, span)?))
        });
        m.insert("pathIsAbsolute", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathIsAbsolute", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathIsAbsolute", span)?;
            Ok(Value::Bool(path::path_is_absolute(path_str, span)?))
        });
        m.insert("pathIsRelative", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathIsRelative", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathIsRelative", span)?;
            Ok(Value::Bool(path::path_is_relative(path_str, span)?))
        });
        m.insert("pathExists", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathExists", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathExists", span)?;
            Ok(Value::Bool(path::path_exists(path_str, span)?))
        });
        m.insert("pathCanonical", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathCanonical", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathCanonical", span)?;
            Ok(Value::string(path::path_canonical(path_str, span)?))
        });
        m.insert("pathEquals", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("pathEquals", 2, args.len(), span));
            }
            let path1 = extract_string(&args[0], "pathEquals", span)?;
            let path2 = extract_string(&args[1], "pathEquals", span)?;
            Ok(Value::Bool(path::path_equals(path1, path2, span)?))
        });
        m.insert("pathHomedir", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("pathHomedir", 0, args.len(), span));
            }
            Ok(Value::string(path::path_homedir(span)?))
        });
        m.insert("pathCwd", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("pathCwd", 0, args.len(), span));
            }
            Ok(Value::string(path::path_cwd(span)?))
        });
        m.insert("pathTempdir", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("pathTempdir", 0, args.len(), span));
            }
            Ok(Value::string(path::path_tempdir(span)?))
        });
        m.insert("pathSeparator", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("pathSeparator", 0, args.len(), span));
            }
            Ok(Value::string(path::path_separator(span)?))
        });
        m.insert("pathDelimiter", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("pathDelimiter", 0, args.len(), span));
            }
            Ok(Value::string(path::path_delimiter(span)?))
        });
        m.insert("pathExtSeparator", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("pathExtSeparator", 0, args.len(), span));
            }
            Ok(Value::string(path::path_ext_separator(span)?))
        });
        m.insert("pathDrive", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathDrive", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathDrive", span)?;
            Ok(Value::string(path::path_drive(path_str, span)?))
        });
        m.insert("pathToPlatform", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathToPlatform", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathToPlatform", span)?;
            Ok(Value::string(path::path_to_platform(path_str, span)?))
        });
        m.insert("pathToPosix", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathToPosix", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathToPosix", span)?;
            Ok(Value::string(path::path_to_posix(path_str, span)?))
        });
        m.insert("pathToWindows", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathToWindows", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathToWindows", span)?;
            Ok(Value::string(path::path_to_windows(path_str, span)?))
        });

        // ====================================================================
        // File system operations - directory operations
        // ====================================================================
        m.insert("fsMkdir", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsMkdir", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsMkdir", span)?;
            fs::mkdir(path, span)
        });
        m.insert("fsMkdirp", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsMkdirp", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsMkdirp", span)?;
            fs::mkdirp(path, span)
        });
        m.insert("fsRmdir", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsRmdir", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsRmdir", span)?;
            fs::rmdir(path, span)
        });
        m.insert("fsRmdirRecursive", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsRmdirRecursive", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsRmdirRecursive", span)?;
            fs::rmdir_recursive(path, span)
        });
        m.insert("fsReaddir", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsReaddir", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsReaddir", span)?;
            fs::readdir(path, span)
        });
        m.insert("fsWalk", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsWalk", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsWalk", span)?;
            fs::walk(path, span)
        });
        m.insert("fsFilterEntries", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("fsFilterEntries", 2, args.len(), span));
            }
            let entries = extract_array(&args[0], "fsFilterEntries", span)?;
            let pattern = extract_string(&args[1], "fsFilterEntries", span)?;
            fs::filter_entries(&entries, pattern, span)
        });
        m.insert("fsSortEntries", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsSortEntries", 1, args.len(), span));
            }
            let entries = extract_array(&args[0], "fsSortEntries", span)?;
            fs::sort_entries(&entries, span)
        });

        // File system operations - metadata
        m.insert("fsSize", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsSize", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsSize", span)?;
            fs::size(path, span)
        });
        m.insert("fsMtime", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsMtime", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsMtime", span)?;
            fs::mtime(path, span)
        });
        m.insert("fsCtime", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsCtime", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsCtime", span)?;
            fs::ctime(path, span)
        });
        m.insert("fsAtime", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsAtime", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsAtime", span)?;
            fs::atime(path, span)
        });
        m.insert("fsPermissions", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsPermissions", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsPermissions", span)?;
            fs::permissions(path, span)
        });
        m.insert("fsIsDir", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsIsDir", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsIsDir", span)?;
            fs::is_dir(path, span)
        });
        m.insert("fsIsFile", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsIsFile", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsIsFile", span)?;
            fs::is_file(path, span)
        });
        m.insert("fsIsSymlink", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsIsSymlink", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsIsSymlink", span)?;
            fs::is_symlink(path, span)
        });
        m.insert("fsInode", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsInode", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsInode", span)?;
            fs::inode(path, span)
        });

        // File system operations - temporary files
        m.insert("fsTmpfile", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("fsTmpfile", 0, args.len(), span));
            }
            fs::tmpfile(span)
        });
        m.insert("fsTmpdir", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("fsTmpdir", 0, args.len(), span));
            }
            fs::tmpdir(span)
        });
        m.insert("fsTmpfileNamed", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsTmpfileNamed", 1, args.len(), span));
            }
            let prefix = extract_string(&args[0], "fsTmpfileNamed", span)?;
            fs::tmpfile_named(prefix, span)
        });
        m.insert("fsGetTempDir", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("fsGetTempDir", 0, args.len(), span));
            }
            fs::get_temp_dir(span)
        });

        // File system operations - symlinks
        m.insert("fsSymlink", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("fsSymlink", 2, args.len(), span));
            }
            let target = extract_string(&args[0], "fsSymlink", span)?;
            let link = extract_string(&args[1], "fsSymlink", span)?;
            fs::symlink(target, link, span)
        });
        m.insert("fsReadlink", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsReadlink", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsReadlink", span)?;
            fs::readlink(path, span)
        });
        m.insert("fsResolveSymlink", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fsResolveSymlink", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fsResolveSymlink", span)?;
            fs::resolve_symlink(path, span)
        });

        // ====================================================================
        // Compression - gzip
        // ====================================================================
        m.insert("gzipCompress", |args, span, _, _| {
            if args.is_empty() || args.len() > 2 {
                return Err(stdlib_arity_error("gzipCompress", 1, args.len(), span));
            }
            let level_opt = args.get(1);
            compression::gzip::gzip_compress(&args[0], level_opt, span)
        });
        m.insert("gzipDecompress", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("gzipDecompress", 1, args.len(), span));
            }
            compression::gzip::gzip_decompress(&args[0], span)
        });
        m.insert("gzipDecompressString", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error(
                    "gzipDecompressString",
                    1,
                    args.len(),
                    span,
                ));
            }
            compression::gzip::gzip_decompress_string(&args[0], span)
        });
        m.insert("gzipIsGzip", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("gzipIsGzip", 1, args.len(), span));
            }
            compression::gzip::gzip_is_gzip(&args[0], span)
        });
        m.insert("gzipCompressionRatio", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error(
                    "gzipCompressionRatio",
                    2,
                    args.len(),
                    span,
                ));
            }
            compression::gzip::gzip_compression_ratio(&args[0], &args[1], span)
        });

        // Compression - tar
        m.insert("tarCreate", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("tarCreate", 2, args.len(), span));
            }
            compression::tar::tar_create(&args[0], &args[1], span)
        });
        m.insert("tarCreateGz", |args, span, _, _| {
            if args.len() < 2 || args.len() > 3 {
                return Err(stdlib_arity_error("tarCreateGz", 2, args.len(), span));
            }
            let level_opt = args.get(2);
            compression::tar::tar_create_gz(&args[0], &args[1], level_opt, span)
        });
        m.insert("tarExtract", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("tarExtract", 2, args.len(), span));
            }
            compression::tar::tar_extract(&args[0], &args[1], span)
        });
        m.insert("tarExtractGz", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("tarExtractGz", 2, args.len(), span));
            }
            compression::tar::tar_extract_gz(&args[0], &args[1], span)
        });
        m.insert("tarList", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("tarList", 1, args.len(), span));
            }
            compression::tar::tar_list(&args[0], span)
        });
        m.insert("tarContains", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("tarContains", 2, args.len(), span));
            }
            compression::tar::tar_contains_file(&args[0], &args[1], span)
        });

        // Compression - zip
        m.insert("zipCreate", |args, span, _, _| {
            if args.is_empty() || args.len() > 3 {
                return Err(stdlib_arity_error("zipCreate", 2, args.len(), span));
            }
            let level_opt = args.get(2);
            compression::zip::zip_create(&args[0], &args[1], level_opt, span)
        });
        m.insert("zipCreateWithComment", |args, span, _, _| {
            if args.len() < 3 || args.len() > 4 {
                return Err(stdlib_arity_error(
                    "zipCreateWithComment",
                    3,
                    args.len(),
                    span,
                ));
            }
            let level_opt = args.get(3);
            compression::zip::zip_create_with_comment(&args[0], &args[1], &args[2], level_opt, span)
        });
        m.insert("zipExtract", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("zipExtract", 2, args.len(), span));
            }
            compression::zip::zip_extract(&args[0], &args[1], span)
        });
        m.insert("zipExtractFiles", |args, span, _, _| {
            if args.len() != 3 {
                return Err(stdlib_arity_error("zipExtractFiles", 3, args.len(), span));
            }
            compression::zip::zip_extract_files(&args[0], &args[1], &args[2], span)
        });
        m.insert("zipList", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("zipList", 1, args.len(), span));
            }
            compression::zip::zip_list(&args[0], span)
        });
        m.insert("zipContains", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("zipContains", 2, args.len(), span));
            }
            compression::zip::zip_contains_file(&args[0], &args[1], span)
        });
        m.insert("zipCompressionRatio", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error(
                    "zipCompressionRatio",
                    1,
                    args.len(),
                    span,
                ));
            }
            compression::zip::zip_compression_ratio(&args[0], span)
        });
        m.insert("zipAddFile", |args, span, _, _| {
            if args.len() < 2 || args.len() > 4 {
                return Err(stdlib_arity_error("zipAddFile", 2, args.len(), span));
            }
            let entry_name_opt = args.get(2);
            let level_opt = args.get(3);
            compression::zip::zip_add_file_fn(&args[0], &args[1], entry_name_opt, level_opt, span)
        });
        m.insert("zipValidate", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("zipValidate", 1, args.len(), span));
            }
            compression::zip::zip_validate_fn(&args[0], span)
        });
        m.insert("zipComment", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("zipComment", 1, args.len(), span));
            }
            compression::zip::zip_comment_fn(&args[0], span)
        });

        // ====================================================================
        // Testing primitives (assertions)
        // ====================================================================
        m.insert("assert", |a, s, _, _| test::assert(a, s));
        m.insert("assertFalse", |a, s, _, _| test::assert_false(a, s));
        m.insert("assertEqual", |a, s, _, _| test::assert_equal(a, s));
        m.insert("assertNotEqual", |a, s, _, _| test::assert_not_equal(a, s));
        m.insert("assertOk", |a, s, _, _| test::assert_ok(a, s));
        m.insert("assertErr", |a, s, _, _| test::assert_err(a, s));
        m.insert("assertSome", |a, s, _, _| test::assert_some(a, s));
        m.insert("assertNone", |a, s, _, _| test::assert_none(a, s));
        m.insert("assertContains", |a, s, _, _| test::assert_contains(a, s));
        m.insert("assertEmpty", |a, s, _, _| test::assert_empty(a, s));
        m.insert("assertLength", |a, s, _, _| test::assert_length(a, s));
        m.insert("assertThrows", |a, s, _, _| test::assert_throws(a, s));
        m.insert("assertNoThrow", |a, s, _, _| test::assert_no_throw(a, s));

        m
    })
}

/// Check if a function name is a builtin (stdlib function, not intrinsic)
pub fn is_builtin(name: &str) -> bool {
    builtin_registry().contains_key(name)
}

/// Check if a function name is an array intrinsic (handled in interpreter/VM)
pub fn is_array_intrinsic(name: &str) -> bool {
    matches!(
        name,
        "map"
            | "filter"
            | "reduce"
            | "forEach"
            | "find"
            | "findIndex"
            | "flatMap"
            | "some"
            | "every"
            | "sort"
            | "sortBy"
            // Result intrinsics (callback-based)
            | "result_map"
            | "result_map_err"
            | "result_and_then"
            | "result_or_else"
            // HashMap intrinsics (callback-based)
            | "hashMapForEach"
            | "hashMapMap"
            | "hashMapFilter"
            // HashSet intrinsics (callback-based)
            | "hashSetForEach"
            | "hashSetMap"
            | "hashSetFilter"
            // Regex intrinsics (callback-based)
            | "regexReplaceWith"
            | "regexReplaceAllWith"
    )
}

/// Extract string from value
fn extract_string<'a>(
    value: &'a Value,
    func_name: &str,
    span: crate::span::Span,
) -> Result<&'a str, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.as_ref()),
        _ => Err(stdlib_arg_error(func_name, "string", value, span)),
    }
}

/// Extract number from value
fn extract_number(
    value: &Value,
    func_name: &str,
    span: crate::span::Span,
) -> Result<f64, RuntimeError> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err(stdlib_arg_error(func_name, "number", value, span)),
    }
}

/// Extract array from value (clones elements from the mutex-guarded vec)
fn extract_array(
    value: &Value,
    func_name: &str,
    span: crate::span::Span,
) -> Result<Vec<Value>, RuntimeError> {
    match value {
        Value::Array(arr) => Ok(arr.lock().unwrap().clone()),
        _ => Err(stdlib_arg_error(func_name, "array", value, span)),
    }
}

/// Call a builtin function by name
pub fn call_builtin(
    name: &str,
    args: &[Value],
    call_span: crate::span::Span,
    security: &SecurityContext,
    output: &OutputWriter,
) -> Result<Value, RuntimeError> {
    match builtin_registry().get(name) {
        Some(dispatch_fn) => dispatch_fn(args, call_span, security, output),
        None => Err(RuntimeError::UnknownFunction {
            name: name.to_string(),
            span: call_span,
        }),
    }
}

/// Print a value to the configured output writer.
///
/// Only accepts string, number, bool, or null per stdlib specification.
pub fn print(
    value: &Value,
    span: crate::span::Span,
    output: &OutputWriter,
) -> Result<(), RuntimeError> {
    match value {
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => {
            let mut w = output.lock().unwrap();
            writeln!(w, "{}", value.to_display_string()).map_err(|_| RuntimeError::TypeError {
                msg: "write failed".into(),
                span,
            })?;
            Ok(())
        }
        _ => Err(stdlib_arg_error(
            "print",
            "string, number, bool, or null",
            value,
            span,
        )),
    }
}

/// Get the length of a string or array
///
/// For strings, returns Unicode scalar count (not byte length).
/// For arrays, returns element count.
pub fn len(value: &Value, span: crate::span::Span) -> Result<f64, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.chars().count() as f64), // Unicode scalar count
        Value::Array(arr) => Ok(arr.lock().unwrap().len() as f64),
        _ => Err(stdlib_arg_error("len", "string or array", value, span)),
    }
}

/// Convert a value to a string
///
/// Only accepts number, bool, or null per stdlib specification.
pub fn str(value: &Value, span: crate::span::Span) -> Result<String, RuntimeError> {
    match value {
        Value::Number(_) | Value::Bool(_) | Value::Null => Ok(value.to_display_string()),
        _ => Err(stdlib_arg_error(
            "str",
            "number, bool, or null",
            value,
            span,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    #[test]
    fn test_len_string() {
        let val = Value::string("hello");
        assert_eq!(len(&val, Span::dummy()).unwrap() as i64, 5);
    }

    #[test]
    fn test_len_array() {
        let val = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);
        assert_eq!(len(&val, Span::dummy()).unwrap() as i64, 2);
    }

    #[test]
    fn test_str() {
        let val = Value::Number(42.0);
        assert_eq!(str(&val, Span::dummy()).unwrap(), "42");
    }

    #[test]
    fn test_len_unicode_string() {
        // Test Unicode scalar count vs byte length
        let val = Value::string("hello");
        assert_eq!(len(&val, Span::dummy()).unwrap(), 5.0); // 5 chars, 5 bytes

        let val = Value::string("héllo");
        assert_eq!(len(&val, Span::dummy()).unwrap(), 5.0); // 5 chars, 6 bytes

        let val = Value::string("你好");
        assert_eq!(len(&val, Span::dummy()).unwrap(), 2.0); // 2 chars, 6 bytes

        let val = Value::string("🎉");
        assert_eq!(len(&val, Span::dummy()).unwrap(), 1.0); // 1 char (emoji), 4 bytes
    }

    #[test]
    fn test_len_empty_string() {
        let val = Value::string("");
        assert_eq!(len(&val, Span::dummy()).unwrap(), 0.0);
    }

    #[test]
    fn test_len_empty_array() {
        let val = Value::array(vec![]);
        assert_eq!(len(&val, Span::dummy()).unwrap(), 0.0);
    }

    #[test]
    fn test_len_invalid_type() {
        let val = Value::Number(42.0);
        assert!(len(&val, Span::dummy()).is_err());
        assert!(matches!(
            len(&val, Span::dummy()).unwrap_err(),
            RuntimeError::InvalidStdlibArgument { .. }
        ));
    }

    #[test]
    fn test_str_number() {
        assert_eq!(str(&Value::Number(42.0), Span::dummy()).unwrap(), "42");
        assert_eq!(str(&Value::Number(2.5), Span::dummy()).unwrap(), "2.5");
        assert_eq!(str(&Value::Number(-10.0), Span::dummy()).unwrap(), "-10");
    }

    #[test]
    fn test_str_bool() {
        assert_eq!(str(&Value::Bool(true), Span::dummy()).unwrap(), "true");
        assert_eq!(str(&Value::Bool(false), Span::dummy()).unwrap(), "false");
    }

    #[test]
    fn test_str_null() {
        assert_eq!(str(&Value::Null, Span::dummy()).unwrap(), "null");
    }

    #[test]
    fn test_call_builtin_print() {
        let security = SecurityContext::allow_all();
        let result = call_builtin(
            "print",
            &[Value::string("test")],
            Span::dummy(),
            &security,
            &stdout_writer(),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_call_builtin_len() {
        let security = SecurityContext::allow_all();
        let result = call_builtin(
            "len",
            &[Value::string("hello")],
            Span::dummy(),
            &security,
            &stdout_writer(),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(5.0));
    }

    #[test]
    fn test_call_builtin_str() {
        let security = SecurityContext::allow_all();
        let result = call_builtin(
            "str",
            &[Value::Number(42.0)],
            Span::dummy(),
            &security,
            &stdout_writer(),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::string("42"));
    }

    #[test]
    fn test_call_builtin_wrong_arg_count() {
        let security = SecurityContext::allow_all();
        let result = call_builtin("print", &[], Span::dummy(), &security, &stdout_writer());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidStdlibArgument { .. }
        ));
    }

    #[test]
    fn test_call_builtin_unknown_function() {
        let security = SecurityContext::allow_all();
        let result = call_builtin(
            "unknown",
            &[Value::Null],
            Span::dummy(),
            &security,
            &stdout_writer(),
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::UnknownFunction { .. }
        ));
    }

    #[test]
    fn test_is_builtin() {
        assert!(is_builtin("print"));
        assert!(is_builtin("len"));
        assert!(is_builtin("str"));
        assert!(!is_builtin("unknown"));
        assert!(!is_builtin("foo"));
    }

    #[test]
    fn test_registry_completeness() {
        // Exhaustive list of all builtin names — the single source of truth.
        // If a name is accidentally removed from the registry, this test fails.
        let known = [
            // Core
            "print",
            "len",
            "str",
            // String functions
            "split",
            "join",
            "trim",
            "trimStart",
            "trimEnd",
            "indexOf",
            "lastIndexOf",
            "includes",
            "toUpperCase",
            "toLowerCase",
            "substring",
            "charAt",
            "repeat",
            "replace",
            "padStart",
            "padEnd",
            "startsWith",
            "endsWith",
            // Array functions
            "pop",
            "shift",
            "unshift",
            "reverse",
            "concat",
            "flatten",
            "arrayIndexOf",
            "arrayLastIndexOf",
            "arrayIncludes",
            "slice",
            // Math functions
            "abs",
            "floor",
            "ceil",
            "round",
            "min",
            "max",
            "sqrt",
            "pow",
            "log",
            "sin",
            "cos",
            "tan",
            "asin",
            "acos",
            "atan",
            "clamp",
            "sign",
            "random",
            // JSON functions
            "parseJSON",
            "toJSON",
            "isValidJSON",
            "prettifyJSON",
            "minifyJSON",
            "jsonAsString",
            "jsonAsNumber",
            "jsonAsBool",
            "jsonIsNull",
            // Type checking functions
            "typeof",
            "isString",
            "isNumber",
            "isBool",
            "isNull",
            "isArray",
            "isFunction",
            "isObject",
            "isType",
            "hasField",
            "hasMethod",
            "hasTag",
            // Type conversion functions
            "toString",
            "toNumber",
            "toBool",
            "parseInt",
            "parseFloat",
            // Option functions
            "Some",
            "None",
            "is_some",
            "is_none",
            // Result functions
            "Ok",
            "Err",
            "is_ok",
            "is_err",
            // Generic unwrap functions
            "unwrap",
            "unwrap_or",
            "expect",
            // Result conversion functions
            "result_ok",
            "result_err",
            // File I/O functions
            "readFile",
            "writeFile",
            "appendFile",
            "fileExists",
            "readDir",
            "createDir",
            "removeFile",
            "removeDir",
            "fileInfo",
            "pathJoin",
            // Reflection functions
            "reflect_typeof",
            "reflect_is_callable",
            "reflect_is_primitive",
            "reflect_same_type",
            "reflect_get_length",
            "reflect_is_empty",
            "reflect_type_describe",
            "reflect_clone",
            "reflect_value_to_string",
            "reflect_deep_equals",
            "reflect_get_function_name",
            "reflect_get_function_arity",
            // HashMap functions
            "hashMapNew",
            "hashMapFromEntries",
            "hashMapPut",
            "hashMapGet",
            "hashMapRemove",
            "hashMapHas",
            "hashMapSize",
            "hashMapIsEmpty",
            "hashMapClear",
            "hashMapKeys",
            "hashMapValues",
            "hashMapEntries",
            // HashSet functions
            "hashSetNew",
            "hashSetFromArray",
            "hashSetAdd",
            "hashSetRemove",
            "hashSetHas",
            "hashSetSize",
            "hashSetIsEmpty",
            "hashSetClear",
            "hashSetUnion",
            "hashSetIntersection",
            "hashSetDifference",
            "hashSetSymmetricDifference",
            "hashSetIsSubset",
            "hashSetIsSuperset",
            "hashSetToArray",
            // Queue functions
            "queueNew",
            "queueEnqueue",
            "queueDequeue",
            "queuePeek",
            "queueSize",
            "queueIsEmpty",
            "queueClear",
            "queueToArray",
            // Stack functions
            "stackNew",
            "stackPush",
            "stackPop",
            "stackPeek",
            "stackSize",
            "stackIsEmpty",
            "stackClear",
            "stackToArray",
            // Regex functions
            "regexNew",
            "regexNewWithFlags",
            "regexEscape",
            "regexIsMatch",
            "regexFind",
            "regexFindAll",
            "regexCaptures",
            "regexCapturesNamed",
            "regexReplace",
            "regexReplaceAll",
            "regexSplit",
            "regexSplitN",
            "regexMatchIndices",
            "regexTest",
            // DateTime functions
            "dateTimeNow",
            "dateTimeFromTimestamp",
            "dateTimeFromComponents",
            "dateTimeParseIso",
            "dateTimeUtc",
            "dateTimeYear",
            "dateTimeMonth",
            "dateTimeDay",
            "dateTimeHour",
            "dateTimeMinute",
            "dateTimeSecond",
            "dateTimeWeekday",
            "dateTimeDayOfYear",
            "dateTimeAddSeconds",
            "dateTimeAddMinutes",
            "dateTimeAddHours",
            "dateTimeAddDays",
            "dateTimeDiff",
            "dateTimeCompare",
            "dateTimeToTimestamp",
            "dateTimeToIso",
            "dateTimeFormat",
            "dateTimeToRfc3339",
            "dateTimeToRfc2822",
            "dateTimeToCustom",
            "dateTimeParse",
            "dateTimeParseRfc3339",
            "dateTimeParseRfc2822",
            "dateTimeTryParse",
            "dateTimeToUtc",
            "dateTimeToLocal",
            "dateTimeToTimezone",
            "dateTimeGetTimezone",
            "dateTimeGetOffset",
            "dateTimeInTimezone",
            "durationFromSeconds",
            "durationFromMinutes",
            "durationFromHours",
            "durationFromDays",
            "durationFormat",
            // HTTP functions
            "httpRequest",
            "httpRequestGet",
            "httpRequestPost",
            "httpRequestPut",
            "httpRequestDelete",
            "httpRequestPatch",
            "httpSetHeader",
            "httpSetBody",
            "httpSetTimeout",
            "httpSetQuery",
            "httpSetFollowRedirects",
            "httpSetMaxRedirects",
            "httpSetUserAgent",
            "httpSetAuth",
            "httpStatus",
            "httpBody",
            "httpHeader",
            "httpHeaders",
            "httpUrl",
            "httpIsSuccess",
            "httpStatusText",
            "httpContentType",
            "httpContentLength",
            "httpIsRedirect",
            "httpIsClientError",
            "httpIsServerError",
            "httpSend",
            "httpGet",
            "httpPost",
            "httpPut",
            "httpDelete",
            "httpPatch",
            "httpPostJson",
            "httpParseJson",
            "httpGetJson",
            "httpCheckPermission",
            // Future/async functions
            "futureResolve",
            "futureReject",
            "futureNew",
            "futureThen",
            "futureCatch",
            "futureAll",
            "futureRace",
            "futureIsPending",
            "futureIsResolved",
            "futureIsRejected",
            // Async I/O functions
            "readFileAsync",
            "writeFileAsync",
            "appendFileAsync",
            "httpSendAsync",
            "httpGetAsync",
            "httpPostAsync",
            "httpPutAsync",
            "httpDeleteAsync",
            "await",
            // Async primitives - tasks
            "spawn",
            "taskJoin",
            "taskStatus",
            "taskCancel",
            "taskId",
            "taskName",
            "joinAll",
            // Async primitives - channels
            "channelBounded",
            "channelUnbounded",
            "channelSend",
            "channelReceive",
            "channelSelect",
            "channelIsClosed",
            // Async primitives - sleep/timers
            "sleep",
            "timer",
            "interval",
            // Async primitives - timeout
            "timeout",
            // Async primitives - mutex
            "asyncMutex",
            "asyncMutexGet",
            "asyncMutexSet",
            // Process management
            "exec",
            "shell",
            "getEnv",
            "setEnv",
            "unsetEnv",
            "listEnv",
            "getCwd",
            "getPid",
            // Path manipulation
            "pathJoinArray",
            "pathParse",
            "pathNormalize",
            "pathAbsolute",
            "pathRelative",
            "pathParent",
            "pathBasename",
            "pathDirname",
            "pathExtension",
            "pathIsAbsolute",
            "pathIsRelative",
            "pathExists",
            "pathCanonical",
            "pathEquals",
            "pathHomedir",
            "pathCwd",
            "pathTempdir",
            "pathSeparator",
            "pathDelimiter",
            "pathExtSeparator",
            "pathDrive",
            "pathToPlatform",
            "pathToPosix",
            "pathToWindows",
            // File system operations - directory operations
            "fsMkdir",
            "fsMkdirp",
            "fsRmdir",
            "fsRmdirRecursive",
            "fsReaddir",
            "fsWalk",
            "fsFilterEntries",
            "fsSortEntries",
            // File system operations - metadata
            "fsSize",
            "fsMtime",
            "fsCtime",
            "fsAtime",
            "fsPermissions",
            "fsIsDir",
            "fsIsFile",
            "fsIsSymlink",
            "fsInode",
            // File system operations - temporary files
            "fsTmpfile",
            "fsTmpdir",
            "fsTmpfileNamed",
            "fsGetTempDir",
            // File system operations - symlinks
            "fsSymlink",
            "fsReadlink",
            "fsResolveSymlink",
            // Compression - gzip
            "gzipCompress",
            "gzipDecompress",
            "gzipDecompressString",
            "gzipIsGzip",
            "gzipCompressionRatio",
            // Compression - tar
            "tarCreate",
            "tarCreateGz",
            "tarExtract",
            "tarExtractGz",
            "tarList",
            "tarContains",
            // Compression - zip
            "zipCreate",
            "zipCreateWithComment",
            "zipExtract",
            "zipExtractFiles",
            "zipList",
            "zipContains",
            "zipCompressionRatio",
            "zipAddFile",
            "zipValidate",
            "zipComment",
            // Testing primitives (assertions)
            "assert",
            "assertFalse",
            "assertEqual",
            "assertNotEqual",
            "assertOk",
            "assertErr",
            "assertSome",
            "assertNone",
            "assertContains",
            "assertEmpty",
            "assertLength",
            "assertThrows",
            "assertNoThrow",
        ];
        for name in &known {
            assert!(is_builtin(name), "Missing from registry: {}", name);
        }
    }

    // ========================================================================
    // Type Restriction Tests (Spec Compliance)
    // ========================================================================

    #[test]
    fn test_print_accepts_all_valid_types() {
        let security = SecurityContext::allow_all();
        // print() should accept string, number, bool, null per spec
        let w = stdout_writer();
        assert!(call_builtin(
            "print",
            &[Value::string("test")],
            Span::dummy(),
            &security,
            &w
        )
        .is_ok());
        assert!(call_builtin(
            "print",
            &[Value::Number(42.0)],
            Span::dummy(),
            &security,
            &w
        )
        .is_ok());
        assert!(call_builtin("print", &[Value::Bool(true)], Span::dummy(), &security, &w).is_ok());
        assert!(call_builtin("print", &[Value::Null], Span::dummy(), &security, &w).is_ok());
    }

    #[test]
    fn test_print_rejects_array() {
        let security = SecurityContext::allow_all();
        // print() should reject arrays per spec
        let result = call_builtin(
            "print",
            &[Value::array(vec![Value::Number(1.0)])],
            Span::dummy(),
            &security,
            &stdout_writer(),
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidStdlibArgument { .. }
        ));
    }

    #[test]
    fn test_print_null_displays_correctly() {
        let security = SecurityContext::allow_all();
        // Verify that null prints as "null" per spec
        let result = call_builtin(
            "print",
            &[Value::Null],
            Span::dummy(),
            &security,
            &stdout_writer(),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_str_rejects_string() {
        let security = SecurityContext::allow_all();
        // str() should only accept number|bool|null, not strings
        let result = call_builtin(
            "str",
            &[Value::string("already a string")],
            Span::dummy(),
            &security,
            &stdout_writer(),
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidStdlibArgument { .. }
        ));
    }

    #[test]
    fn test_str_rejects_array() {
        let security = SecurityContext::allow_all();
        // str() should only accept number|bool|null, not arrays
        let result = call_builtin(
            "str",
            &[Value::array(vec![Value::Number(1.0)])],
            Span::dummy(),
            &security,
            &stdout_writer(),
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidStdlibArgument { .. }
        ));
    }

    #[test]
    fn test_str_accepts_all_valid_types() {
        let security = SecurityContext::allow_all();
        // str() should accept number, bool, null per spec
        let w = stdout_writer();
        assert!(call_builtin("str", &[Value::Number(42.0)], Span::dummy(), &security, &w).is_ok());
        assert!(call_builtin("str", &[Value::Bool(true)], Span::dummy(), &security, &w).is_ok());
        assert!(call_builtin("str", &[Value::Null], Span::dummy(), &security, &w).is_ok());
    }

    // ========================================================================
    // OutputWriter Tests
    // ========================================================================

    /// A thin Write wrapper around Arc<Mutex<Vec<u8>>> for capturing output in tests.
    struct VecWriter(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for VecWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_print_writes_to_custom_writer() {
        let buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        let writer: OutputWriter = Arc::new(Mutex::new(Box::new(VecWriter(buf.clone()))));
        let security = SecurityContext::allow_all();
        call_builtin(
            "print",
            &[Value::string("hello")],
            Span::dummy(),
            &security,
            &writer,
        )
        .unwrap();
        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert_eq!(output, "hello\n");
    }
}
