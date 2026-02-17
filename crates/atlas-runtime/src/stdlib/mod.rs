//! Standard library functions

pub mod array;
pub mod async_io;
pub mod async_primitives;
pub mod collections;
pub mod datetime;
pub mod future;
pub mod http;
pub mod io;
pub mod json;
pub mod math;
pub mod process;
pub mod reflect;
pub mod regex;
pub mod string;
pub mod types;

use crate::security::SecurityContext;
use crate::value::{RuntimeError, Value};

/// Check if a function name is a builtin (stdlib function, not intrinsic)
pub fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "print" | "len" | "str"
            // String functions
            | "split" | "join" | "trim" | "trimStart" | "trimEnd"
            | "indexOf" | "lastIndexOf" | "includes"
            | "toUpperCase" | "toLowerCase" | "substring" | "charAt" | "repeat" | "replace"
            | "padStart" | "padEnd" | "startsWith" | "endsWith"
            // Array functions (pure)
            | "pop" | "shift" | "unshift" | "reverse" | "concat" | "flatten"
            | "arrayIndexOf" | "arrayLastIndexOf" | "arrayIncludes" | "slice"
            // Math functions
            | "abs" | "floor" | "ceil" | "round" | "min" | "max"
            | "sqrt" | "pow" | "log"
            | "sin" | "cos" | "tan" | "asin" | "acos" | "atan"
            | "clamp" | "sign" | "random"
            // JSON functions
            | "parseJSON" | "toJSON" | "isValidJSON" | "prettifyJSON" | "minifyJSON"
            // JSON extraction functions (called via method syntax)
            | "jsonAsString" | "jsonAsNumber" | "jsonAsBool" | "jsonIsNull"
            // Type checking functions
            | "typeof" | "isString" | "isNumber" | "isBool" | "isNull" | "isArray" | "isFunction"
            // Type conversion functions
            | "toString" | "toNumber" | "toBool" | "parseInt" | "parseFloat"
            // Option functions
            | "Some" | "None" | "is_some" | "is_none"
            // Result functions
            | "Ok" | "Err" | "is_ok" | "is_err"
            // Generic unwrap functions (work with both Option and Result)
            | "unwrap" | "unwrap_or" | "expect"
            // Result conversion functions
            | "result_ok" | "result_err"
            // File I/O functions
            | "readFile" | "writeFile" | "appendFile" | "fileExists"
            | "readDir" | "createDir" | "removeFile" | "removeDir"
            | "fileInfo" | "pathJoin"
            // Reflection functions
            | "reflect_typeof" | "reflect_is_callable" | "reflect_is_primitive"
            | "reflect_same_type" | "reflect_get_length" | "reflect_is_empty"
            | "reflect_type_describe" | "reflect_clone" | "reflect_value_to_string"
            | "reflect_deep_equals" | "reflect_get_function_name" | "reflect_get_function_arity"
            // HashMap functions
            | "hashMapNew" | "hashMapFromEntries"
            | "hashMapPut" | "hashMapGet" | "hashMapRemove"
            | "hashMapHas" | "hashMapSize" | "hashMapIsEmpty"
            | "hashMapClear" | "hashMapKeys" | "hashMapValues" | "hashMapEntries"
            // HashSet functions
            | "hashSetNew" | "hashSetFromArray"
            | "hashSetAdd" | "hashSetRemove" | "hashSetHas"
            | "hashSetSize" | "hashSetIsEmpty" | "hashSetClear"
            | "hashSetUnion" | "hashSetIntersection" | "hashSetDifference" | "hashSetSymmetricDifference"
            | "hashSetIsSubset" | "hashSetIsSuperset" | "hashSetToArray"
            // Queue functions
            | "queueNew" | "queueEnqueue" | "queueDequeue" | "queuePeek"
            | "queueSize" | "queueIsEmpty" | "queueClear" | "queueToArray"
            // Stack functions
            | "stackNew" | "stackPush" | "stackPop" | "stackPeek"
            | "stackSize" | "stackIsEmpty" | "stackClear" | "stackToArray"
            // Regex functions
            | "regexNew" | "regexNewWithFlags" | "regexEscape"
            | "regexIsMatch" | "regexFind" | "regexFindAll"
            | "regexCaptures" | "regexCapturesNamed"
            | "regexReplace" | "regexReplaceAll"
            | "regexSplit" | "regexSplitN"
            | "regexMatchIndices" | "regexTest"
            // DateTime functions
            | "dateTimeNow" | "dateTimeFromTimestamp" | "dateTimeFromComponents"
            | "dateTimeParseIso" | "dateTimeUtc"
            | "dateTimeYear" | "dateTimeMonth" | "dateTimeDay"
            | "dateTimeHour" | "dateTimeMinute" | "dateTimeSecond"
            | "dateTimeWeekday" | "dateTimeDayOfYear"
            | "dateTimeAddSeconds" | "dateTimeAddMinutes" | "dateTimeAddHours" | "dateTimeAddDays"
            | "dateTimeDiff" | "dateTimeCompare"
            | "dateTimeToTimestamp" | "dateTimeToIso"
            | "dateTimeFormat" | "dateTimeToRfc3339" | "dateTimeToRfc2822" | "dateTimeToCustom"
            | "dateTimeParse" | "dateTimeParseRfc3339" | "dateTimeParseRfc2822" | "dateTimeTryParse"
            | "dateTimeToUtc" | "dateTimeToLocal" | "dateTimeToTimezone"
            | "dateTimeGetTimezone" | "dateTimeGetOffset" | "dateTimeInTimezone"
            | "durationFromSeconds" | "durationFromMinutes" | "durationFromHours" | "durationFromDays"
            | "durationFormat"
            // HTTP functions
            | "httpRequest" | "httpRequestGet" | "httpRequestPost"
            | "httpRequestPut" | "httpRequestDelete" | "httpRequestPatch"
            | "httpSetHeader" | "httpSetBody" | "httpSetTimeout"
            | "httpSetQuery" | "httpSetFollowRedirects" | "httpSetMaxRedirects"
            | "httpSetUserAgent" | "httpSetAuth"
            | "httpStatus" | "httpBody" | "httpHeader" | "httpHeaders" | "httpUrl" | "httpIsSuccess"
            | "httpStatusText" | "httpContentType" | "httpContentLength"
            | "httpIsRedirect" | "httpIsClientError" | "httpIsServerError"
            | "httpSend" | "httpGet" | "httpPost" | "httpPut" | "httpDelete" | "httpPatch"
            | "httpPostJson" | "httpParseJson" | "httpGetJson"
            | "httpCheckPermission"
            // Future/async functions
            | "futureResolve" | "futureReject" | "futureNew"
            | "futureThen" | "futureCatch"
            | "futureAll" | "futureRace"
            | "futureIsPending" | "futureIsResolved" | "futureIsRejected"
            // Async I/O functions
            | "readFileAsync" | "writeFileAsync" | "appendFileAsync"
            | "httpSendAsync" | "httpGetAsync" | "httpPostAsync" | "httpPutAsync" | "httpDeleteAsync"
            | "await"
            // Async primitives - tasks
            | "spawn" | "taskJoin" | "taskStatus" | "taskCancel"
            | "taskId" | "taskName" | "joinAll"
            // Async primitives - channels
            | "channelBounded" | "channelUnbounded" | "channelSend" | "channelReceive"
            | "channelSelect" | "channelIsClosed"
            // Async primitives - sleep/timers
            | "sleep" | "timer" | "interval"
            // Async primitives - timeout
            | "timeout"
            // Async primitives - mutex
            | "asyncMutex" | "asyncMutexGet" | "asyncMutexSet"
            // Process management
            | "exec" | "shell" | "getEnv" | "setEnv" | "unsetEnv" | "listEnv"
            | "getCwd" | "getPid"
    )
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
fn extract_string(value: &Value, span: crate::span::Span) -> Result<&str, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.as_ref()),
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
    }
}

/// Extract number from value
fn extract_number(value: &Value, span: crate::span::Span) -> Result<f64, RuntimeError> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
    }
}

/// Extract array from value
fn extract_array(value: &Value, span: crate::span::Span) -> Result<Vec<Value>, RuntimeError> {
    match value {
        Value::Array(arr) => Ok(arr.lock().unwrap().clone()),
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
    }
}

/// Call a builtin function
///
/// The `call_span` parameter should be the span of the function call expression
/// in the source code, used for error reporting.
/// The `security` parameter is used for permission checks in I/O operations.
pub fn call_builtin(
    name: &str,
    args: &[Value],
    call_span: crate::span::Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    match name {
        "print" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            print(&args[0], call_span)?;
            Ok(Value::Null)
        }
        "len" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let length = len(&args[0], call_span)?;
            Ok(Value::Number(length))
        }
        "str" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = str(&args[0], call_span)?;
            Ok(Value::string(s))
        }

        // String functions - Core Operations
        "split" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let sep = extract_string(&args[1], call_span)?;
            string::split(s, sep, call_span)
        }
        "join" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            let sep = extract_string(&args[1], call_span)?;
            let result = string::join(&arr, sep, call_span)?;
            Ok(Value::string(result))
        }
        "trim" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            Ok(Value::string(string::trim(s)))
        }
        "trimStart" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            Ok(Value::string(string::trim_start(s)))
        }
        "trimEnd" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            Ok(Value::string(string::trim_end(s)))
        }

        // String functions - Search Operations
        "indexOf" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let search = extract_string(&args[1], call_span)?;
            Ok(Value::Number(string::index_of(s, search)))
        }
        "lastIndexOf" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let search = extract_string(&args[1], call_span)?;
            Ok(Value::Number(string::last_index_of(s, search)))
        }
        "includes" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let search = extract_string(&args[1], call_span)?;
            Ok(Value::Bool(string::includes(s, search)))
        }

        // String functions - Transformation
        "toUpperCase" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            Ok(Value::string(string::to_upper_case(s)))
        }
        "toLowerCase" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            Ok(Value::string(string::to_lower_case(s)))
        }
        "substring" => {
            if args.len() != 3 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let start = extract_number(&args[1], call_span)?;
            let end = extract_number(&args[2], call_span)?;
            let result = string::substring(s, start, end, call_span)?;
            Ok(Value::string(result))
        }
        "charAt" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let index = extract_number(&args[1], call_span)?;
            let result = string::char_at(s, index, call_span)?;
            Ok(Value::string(result))
        }
        "repeat" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let count = extract_number(&args[1], call_span)?;
            let result = string::repeat(s, count, call_span)?;
            Ok(Value::string(result))
        }
        "replace" => {
            if args.len() != 3 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let search = extract_string(&args[1], call_span)?;
            let replacement = extract_string(&args[2], call_span)?;
            Ok(Value::string(string::replace(s, search, replacement)))
        }

        // String functions - Formatting
        "padStart" => {
            if args.len() != 3 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let length = extract_number(&args[1], call_span)?;
            let fill = extract_string(&args[2], call_span)?;
            let result = string::pad_start(s, length, fill, call_span)?;
            Ok(Value::string(result))
        }
        "padEnd" => {
            if args.len() != 3 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let length = extract_number(&args[1], call_span)?;
            let fill = extract_string(&args[2], call_span)?;
            let result = string::pad_end(s, length, fill, call_span)?;
            Ok(Value::string(result))
        }
        "startsWith" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let prefix = extract_string(&args[1], call_span)?;
            Ok(Value::Bool(string::starts_with(s, prefix)))
        }
        "endsWith" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let suffix = extract_string(&args[1], call_span)?;
            Ok(Value::Bool(string::ends_with(s, suffix)))
        }

        // Array functions - Core Operations
        "pop" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            array::pop(&arr, call_span)
        }
        "shift" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            array::shift(&arr, call_span)
        }
        "unshift" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            Ok(array::unshift(&arr, args[1].clone()))
        }
        "reverse" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            Ok(array::reverse(&arr))
        }
        "concat" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr1 = extract_array(&args[0], call_span)?;
            let arr2 = extract_array(&args[1], call_span)?;
            Ok(array::concat(&arr1, &arr2))
        }
        "flatten" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            array::flatten(&arr, call_span)
        }

        // Array functions - Search Operations
        "arrayIndexOf" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            Ok(Value::Number(array::index_of(&arr, &args[1])))
        }
        "arrayLastIndexOf" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            Ok(Value::Number(array::last_index_of(&arr, &args[1])))
        }
        "arrayIncludes" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            Ok(Value::Bool(array::includes(&arr, &args[1])))
        }

        // Array functions - Slicing
        "slice" => {
            if args.len() != 3 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            let start = extract_number(&args[1], call_span)?;
            let end = extract_number(&args[2], call_span)?;
            array::slice(&arr, start, end, call_span)
        }

        // Math functions - Basic Operations
        "abs" => math::abs(args, call_span),
        "floor" => math::floor(args, call_span),
        "ceil" => math::ceil(args, call_span),
        "round" => math::round(args, call_span),
        "min" => math::min(args, call_span),
        "max" => math::max(args, call_span),

        // Math functions - Exponential/Power
        "sqrt" => math::sqrt(args, call_span),
        "pow" => math::pow(args, call_span),
        "log" => math::log(args, call_span),

        // Math functions - Trigonometry
        "sin" => math::sin(args, call_span),
        "cos" => math::cos(args, call_span),
        "tan" => math::tan(args, call_span),
        "asin" => math::asin(args, call_span),
        "acos" => math::acos(args, call_span),
        "atan" => math::atan(args, call_span),

        // Math functions - Utilities
        "clamp" => math::clamp(args, call_span),
        "sign" => math::sign(args, call_span),
        "random" => math::random(args, call_span),

        // JSON functions
        "parseJSON" => json::parse_json(args, call_span),
        "toJSON" => json::to_json(args, call_span),
        "isValidJSON" => json::is_valid_json(args, call_span),
        "prettifyJSON" => json::prettify_json(args, call_span),
        "minifyJSON" => json::minify_json(args, call_span),

        // JSON extraction functions (called via method syntax)
        "jsonAsString" => json::json_as_string(args, call_span),
        "jsonAsNumber" => json::json_as_number(args, call_span),
        "jsonAsBool" => json::json_as_bool(args, call_span),
        "jsonIsNull" => json::json_is_null(args, call_span),

        // Type checking functions
        "typeof" => types::type_of(args, call_span),
        "isString" => types::is_string(args, call_span),
        "isNumber" => types::is_number(args, call_span),
        "isBool" => types::is_bool(args, call_span),
        "isNull" => types::is_null(args, call_span),
        "isArray" => types::is_array(args, call_span),
        "isFunction" => types::is_function(args, call_span),

        // Type conversion functions
        "toString" => types::to_string(args, call_span),
        "toNumber" => types::to_number(args, call_span),
        "toBool" => types::to_bool(args, call_span),
        "parseInt" => types::parse_int(args, call_span),
        "parseFloat" => types::parse_float(args, call_span),

        // Option<T> constructors
        "Some" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            Ok(types::some(args[0].clone()))
        }
        "None" => {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            Ok(types::none())
        }

        // Option<T> helpers
        "is_some" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let is_some = types::is_some(&args[0], call_span)?;
            Ok(Value::Bool(is_some))
        }
        "is_none" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let is_none = types::is_none(&args[0], call_span)?;
            Ok(Value::Bool(is_none))
        }

        // Result<T,E> constructors
        "Ok" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            Ok(types::ok(args[0].clone()))
        }
        "Err" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            Ok(types::err(args[0].clone()))
        }

        // Result<T,E> helpers
        "is_ok" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let is_ok = types::is_ok(&args[0], call_span)?;
            Ok(Value::Bool(is_ok))
        }
        "is_err" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let is_err = types::is_err(&args[0], call_span)?;
            Ok(Value::Bool(is_err))
        }

        // File I/O functions
        "readFile" => io::read_file(args, call_span, security),
        "writeFile" => io::write_file(args, call_span, security),
        "appendFile" => io::append_file(args, call_span, security),
        "fileExists" => io::file_exists(args, call_span, security),
        "readDir" => io::read_dir(args, call_span, security),
        "createDir" => io::create_dir(args, call_span, security),
        "removeFile" => io::remove_file(args, call_span, security),
        "removeDir" => io::remove_dir(args, call_span, security),
        "fileInfo" => io::file_info(args, call_span, security),
        "pathJoin" => io::path_join(args, call_span, security),

        // Generic unwrap (works with both Option and Result)
        "unwrap" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            match &args[0] {
                Value::Option(_) => types::unwrap_option(&args[0], call_span),
                Value::Result(_) => types::unwrap_result(&args[0], call_span),
                _ => Err(RuntimeError::TypeError {
                    msg: "unwrap() requires Option or Result value".to_string(),
                    span: call_span,
                }),
            }
        }
        "unwrap_or" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            match &args[0] {
                Value::Option(_) => types::unwrap_or_option(&args[0], args[1].clone(), call_span),
                Value::Result(_) => types::unwrap_or_result(&args[0], args[1].clone(), call_span),
                _ => Err(RuntimeError::TypeError {
                    msg: "unwrap_or() requires Option or Result value".to_string(),
                    span: call_span,
                }),
            }
        }
        "expect" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let message = extract_string(&args[1], call_span)?;
            match &args[0] {
                Value::Option(_) => types::expect_option(&args[0], message, call_span),
                Value::Result(_) => types::expect_result(&args[0], message, call_span),
                _ => Err(RuntimeError::TypeError {
                    msg: "expect() requires Option or Result value".to_string(),
                    span: call_span,
                }),
            }
        }
        "result_ok" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            types::result_ok(&args[0], call_span)
        }
        "result_err" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            types::result_err(&args[0], call_span)
        }

        // Reflection functions
        "reflect_typeof" => reflect::typeof_fn(args, call_span),
        "reflect_is_callable" => reflect::is_callable_fn(args, call_span),
        "reflect_is_primitive" => reflect::is_primitive_fn(args, call_span),
        "reflect_same_type" => reflect::same_type_fn(args, call_span),
        "reflect_get_length" => reflect::get_length_fn(args, call_span),
        "reflect_is_empty" => reflect::is_empty_fn(args, call_span),
        "reflect_type_describe" => reflect::type_describe_fn(args, call_span),
        "reflect_clone" => reflect::clone_fn(args, call_span),
        "reflect_value_to_string" => reflect::value_to_string_fn(args, call_span),
        "reflect_deep_equals" => reflect::deep_equals_fn(args, call_span),
        "reflect_get_function_name" => reflect::get_function_name_fn(args, call_span),
        "reflect_get_function_arity" => reflect::get_function_arity_fn(args, call_span),

        // HashMap functions
        "hashMapNew" => collections::hashmap::new_map(args, call_span),
        "hashMapFromEntries" => collections::hashmap::from_entries(args, call_span),
        "hashMapPut" => collections::hashmap::put(args, call_span),
        "hashMapGet" => collections::hashmap::get(args, call_span),
        "hashMapRemove" => collections::hashmap::remove(args, call_span),
        "hashMapHas" => collections::hashmap::has(args, call_span),
        "hashMapSize" => collections::hashmap::size(args, call_span),
        "hashMapIsEmpty" => collections::hashmap::is_empty(args, call_span),
        "hashMapClear" => collections::hashmap::clear(args, call_span),
        "hashMapKeys" => collections::hashmap::keys(args, call_span),
        "hashMapValues" => collections::hashmap::values(args, call_span),
        "hashMapEntries" => collections::hashmap::entries(args, call_span),

        // HashSet functions
        "hashSetNew" => collections::hashset::new_set(args, call_span),
        "hashSetFromArray" => collections::hashset::from_array(args, call_span),
        "hashSetAdd" => collections::hashset::add(args, call_span),
        "hashSetRemove" => collections::hashset::remove(args, call_span),
        "hashSetHas" => collections::hashset::has(args, call_span),
        "hashSetSize" => collections::hashset::size(args, call_span),
        "hashSetIsEmpty" => collections::hashset::is_empty(args, call_span),
        "hashSetClear" => collections::hashset::clear(args, call_span),
        "hashSetUnion" => collections::hashset::union(args, call_span),
        "hashSetIntersection" => collections::hashset::intersection(args, call_span),
        "hashSetDifference" => collections::hashset::difference(args, call_span),
        "hashSetSymmetricDifference" => collections::hashset::symmetric_difference(args, call_span),
        "hashSetIsSubset" => collections::hashset::is_subset(args, call_span),
        "hashSetIsSuperset" => collections::hashset::is_superset(args, call_span),
        "hashSetToArray" => collections::hashset::to_array(args, call_span),

        // Queue functions
        "queueNew" => collections::queue::new_queue(args, call_span),
        "queueEnqueue" => collections::queue::enqueue(args, call_span),
        "queueDequeue" => collections::queue::dequeue(args, call_span),
        "queuePeek" => collections::queue::peek(args, call_span),
        "queueSize" => collections::queue::size(args, call_span),
        "queueIsEmpty" => collections::queue::is_empty(args, call_span),
        "queueClear" => collections::queue::clear(args, call_span),
        "queueToArray" => collections::queue::to_array(args, call_span),

        // Stack functions
        "stackNew" => collections::stack::new_stack(args, call_span),
        "stackPush" => collections::stack::push(args, call_span),
        "stackPop" => collections::stack::pop(args, call_span),
        "stackPeek" => collections::stack::peek(args, call_span),
        "stackSize" => collections::stack::size(args, call_span),
        "stackIsEmpty" => collections::stack::is_empty(args, call_span),
        "stackClear" => collections::stack::clear(args, call_span),
        "stackToArray" => collections::stack::to_array(args, call_span),

        // Regex functions
        "regexNew" => regex::regex_new(args, call_span),
        "regexNewWithFlags" => regex::regex_new_with_flags(args, call_span),
        "regexEscape" => regex::regex_escape(args, call_span),
        "regexIsMatch" => regex::regex_is_match(args, call_span),
        "regexFind" => regex::regex_find(args, call_span),
        "regexFindAll" => regex::regex_find_all(args, call_span),
        "regexCaptures" => regex::regex_captures(args, call_span),
        "regexCapturesNamed" => regex::regex_captures_named(args, call_span),
        "regexReplace" => regex::regex_replace(args, call_span),
        "regexReplaceAll" => regex::regex_replace_all(args, call_span),
        "regexSplit" => regex::regex_split(args, call_span),
        "regexSplitN" => regex::regex_split_n(args, call_span),
        "regexMatchIndices" => regex::regex_match_indices(args, call_span),
        "regexTest" => regex::regex_test(args, call_span),

        // DateTime functions
        "dateTimeNow" => datetime::date_time_now(args, call_span),
        "dateTimeFromTimestamp" => datetime::date_time_from_timestamp(args, call_span),
        "dateTimeFromComponents" => datetime::date_time_from_components(args, call_span),
        "dateTimeParseIso" => datetime::date_time_parse_iso(args, call_span),
        "dateTimeUtc" => datetime::date_time_utc(args, call_span),
        "dateTimeYear" => datetime::date_time_year(args, call_span),
        "dateTimeMonth" => datetime::date_time_month(args, call_span),
        "dateTimeDay" => datetime::date_time_day(args, call_span),
        "dateTimeHour" => datetime::date_time_hour(args, call_span),
        "dateTimeMinute" => datetime::date_time_minute(args, call_span),
        "dateTimeSecond" => datetime::date_time_second(args, call_span),
        "dateTimeWeekday" => datetime::date_time_weekday(args, call_span),
        "dateTimeDayOfYear" => datetime::date_time_day_of_year(args, call_span),
        "dateTimeAddSeconds" => datetime::date_time_add_seconds(args, call_span),
        "dateTimeAddMinutes" => datetime::date_time_add_minutes(args, call_span),
        "dateTimeAddHours" => datetime::date_time_add_hours(args, call_span),
        "dateTimeAddDays" => datetime::date_time_add_days(args, call_span),
        "dateTimeDiff" => datetime::date_time_diff(args, call_span),
        "dateTimeCompare" => datetime::date_time_compare(args, call_span),
        "dateTimeToTimestamp" => datetime::date_time_to_timestamp(args, call_span),
        "dateTimeToIso" => datetime::date_time_to_iso(args, call_span),
        // Advanced formatting
        "dateTimeFormat" => datetime::date_time_format(args, call_span),
        "dateTimeToRfc3339" => datetime::date_time_to_rfc3339(args, call_span),
        "dateTimeToRfc2822" => datetime::date_time_to_rfc2822(args, call_span),
        "dateTimeToCustom" => datetime::date_time_to_custom(args, call_span),
        // Advanced parsing
        "dateTimeParse" => datetime::date_time_parse(args, call_span),
        "dateTimeParseRfc3339" => datetime::date_time_parse_rfc3339(args, call_span),
        "dateTimeParseRfc2822" => datetime::date_time_parse_rfc2822(args, call_span),
        "dateTimeTryParse" => datetime::date_time_try_parse(args, call_span),
        // Timezone operations
        "dateTimeToUtc" => datetime::date_time_to_utc(args, call_span),
        "dateTimeToLocal" => datetime::date_time_to_local(args, call_span),
        "dateTimeToTimezone" => datetime::date_time_to_timezone(args, call_span),
        "dateTimeGetTimezone" => datetime::date_time_get_timezone(args, call_span),
        "dateTimeGetOffset" => datetime::date_time_get_offset(args, call_span),
        "dateTimeInTimezone" => datetime::date_time_in_timezone(args, call_span),
        // Duration operations
        "durationFromSeconds" => datetime::duration_from_seconds(args, call_span),
        "durationFromMinutes" => datetime::duration_from_minutes(args, call_span),
        "durationFromHours" => datetime::duration_from_hours(args, call_span),
        "durationFromDays" => datetime::duration_from_days(args, call_span),
        "durationFormat" => datetime::duration_format(args, call_span),
        // HTTP functions
        "httpRequest" => http::http_request(args, call_span),
        "httpRequestGet" => http::http_request_get(args, call_span),
        "httpRequestPost" => http::http_request_post(args, call_span),
        "httpSetHeader" => http::http_set_header(args, call_span),
        "httpSetBody" => http::http_set_body(args, call_span),
        "httpSetTimeout" => http::http_set_timeout(args, call_span),
        "httpStatus" => http::http_status(args, call_span),
        "httpBody" => http::http_body(args, call_span),
        "httpHeader" => http::http_header(args, call_span),
        "httpHeaders" => http::http_headers(args, call_span),
        "httpUrl" => http::http_url(args, call_span),
        "httpIsSuccess" => http::http_is_success(args, call_span),
        "httpSend" => http::http_send(args, call_span),
        "httpGet" => http::http_get(args, call_span),
        "httpPost" => http::http_post(args, call_span),
        "httpPostJson" => http::http_post_json(args, call_span),
        "httpParseJson" => http::http_parse_json(args, call_span),
        // Phase 10b: Advanced HTTP
        "httpRequestPut" => http::http_request_put(args, call_span),
        "httpRequestDelete" => http::http_request_delete(args, call_span),
        "httpRequestPatch" => http::http_request_patch(args, call_span),
        "httpPut" => http::http_put(args, call_span),
        "httpDelete" => http::http_delete(args, call_span),
        "httpPatch" => http::http_patch(args, call_span),
        "httpSetQuery" => http::http_set_query(args, call_span),
        "httpSetFollowRedirects" => http::http_set_follow_redirects(args, call_span),
        "httpSetMaxRedirects" => http::http_set_max_redirects(args, call_span),
        "httpSetUserAgent" => http::http_set_user_agent(args, call_span),
        "httpSetAuth" => http::http_set_auth(args, call_span),
        "httpStatusText" => http::http_status_text(args, call_span),
        "httpContentType" => http::http_content_type(args, call_span),
        "httpContentLength" => http::http_content_length(args, call_span),
        "httpIsRedirect" => http::http_is_redirect(args, call_span),
        "httpIsClientError" => http::http_is_client_error(args, call_span),
        "httpIsServerError" => http::http_is_server_error(args, call_span),
        "httpGetJson" => http::http_get_json(args, call_span),
        "httpCheckPermission" => http::http_check_permission(args, call_span),
        // Future/async functions
        "futureResolve" => future::future_resolve(args, call_span),
        "futureReject" => future::future_reject(args, call_span),
        "futureNew" => future::future_new(args, call_span),
        "futureIsPending" => future::future_is_pending(args, call_span),
        "futureIsResolved" => future::future_is_resolved(args, call_span),
        "futureIsRejected" => future::future_is_rejected(args, call_span),
        "futureThen" => future::future_then(args, call_span),
        "futureCatch" => future::future_catch(args, call_span),
        "futureAll" => future::future_all_fn(args, call_span),
        "futureRace" => future::future_race_fn(args, call_span),

        // Async I/O functions
        "readFileAsync" => async_io::read_file_async(args, call_span, security),
        "writeFileAsync" => async_io::write_file_async(args, call_span, security),
        "appendFileAsync" => async_io::append_file_async(args, call_span, security),
        "httpSendAsync" => async_io::http_send_async(args, call_span),
        "httpGetAsync" => async_io::http_get_async(args, call_span),
        "httpPostAsync" => async_io::http_post_async(args, call_span),
        "httpPutAsync" => async_io::http_put_async(args, call_span),
        "httpDeleteAsync" => async_io::http_delete_async(args, call_span),
        "await" => async_io::await_future(args, call_span),

        // Async primitives - tasks
        "spawn" => async_primitives::spawn(args, call_span),
        "taskJoin" => async_primitives::task_join(args, call_span),
        "taskStatus" => async_primitives::task_status(args, call_span),
        "taskCancel" => async_primitives::task_cancel(args, call_span),
        "taskId" => async_primitives::task_id(args, call_span),
        "taskName" => async_primitives::task_name(args, call_span),
        "joinAll" => async_primitives::join_all(args, call_span),

        // Async primitives - channels
        "channelBounded" => async_primitives::channel_bounded(args, call_span),
        "channelUnbounded" => async_primitives::channel_unbounded(args, call_span),
        "channelSend" => async_primitives::channel_send(args, call_span),
        "channelReceive" => async_primitives::channel_receive(args, call_span),
        "channelSelect" => async_primitives::channel_select(args, call_span),
        "channelIsClosed" => async_primitives::channel_is_closed(args, call_span),

        // Async primitives - sleep/timers
        "sleep" => async_primitives::sleep_fn(args, call_span),
        "timer" => async_primitives::timer_fn(args, call_span),
        "interval" => async_primitives::interval_fn(args, call_span),

        // Async primitives - timeout
        "timeout" => async_primitives::timeout_fn(args, call_span),

        // Async primitives - mutex
        "asyncMutex" => async_primitives::async_mutex_new(args, call_span),
        "asyncMutexGet" => async_primitives::async_mutex_get(args, call_span),
        "asyncMutexSet" => async_primitives::async_mutex_set(args, call_span),

        // Process management
        "exec" => process::exec(args, call_span, security),
        "shell" => process::shell(args, call_span, security),
        "getEnv" => process::get_env(args, call_span, security),
        "setEnv" => process::set_env(args, call_span, security),
        "unsetEnv" => process::unset_env(args, call_span, security),
        "listEnv" => process::list_env(args, call_span, security),
        "getCwd" => process::get_cwd(args, call_span, security),
        "getPid" => process::get_pid(args, call_span, security),

        _ => Err(RuntimeError::UnknownFunction {
            name: name.to_string(),
            span: call_span,
        }),
    }
}

/// Print a value to stdout
///
/// Only accepts string, number, bool, or null per stdlib specification.
pub fn print(value: &Value, span: crate::span::Span) -> Result<(), RuntimeError> {
    match value {
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => {
            println!("{}", value.to_display_string());
            Ok(())
        }
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
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
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
    }
}

/// Convert a value to a string
///
/// Only accepts number, bool, or null per stdlib specification.
pub fn str(value: &Value, span: crate::span::Span) -> Result<String, RuntimeError> {
    match value {
        Value::Number(_) | Value::Bool(_) | Value::Null => Ok(value.to_display_string()),
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
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
        let result = call_builtin("print", &[Value::string("test")], Span::dummy(), &security);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_call_builtin_len() {
        let security = SecurityContext::allow_all();
        let result = call_builtin("len", &[Value::string("hello")], Span::dummy(), &security);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(5.0));
    }

    #[test]
    fn test_call_builtin_str() {
        let security = SecurityContext::allow_all();
        let result = call_builtin("str", &[Value::Number(42.0)], Span::dummy(), &security);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::string("42"));
    }

    #[test]
    fn test_call_builtin_wrong_arg_count() {
        let security = SecurityContext::allow_all();
        let result = call_builtin("print", &[], Span::dummy(), &security);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidStdlibArgument { .. }
        ));
    }

    #[test]
    fn test_call_builtin_unknown_function() {
        let security = SecurityContext::allow_all();
        let result = call_builtin("unknown", &[Value::Null], Span::dummy(), &security);
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

    // ========================================================================
    // Type Restriction Tests (Spec Compliance)
    // ========================================================================

    #[test]
    fn test_print_accepts_all_valid_types() {
        let security = SecurityContext::allow_all();
        // print() should accept string, number, bool, null per spec
        assert!(call_builtin("print", &[Value::string("test")], Span::dummy(), &security).is_ok());
        assert!(call_builtin("print", &[Value::Number(42.0)], Span::dummy(), &security).is_ok());
        assert!(call_builtin("print", &[Value::Bool(true)], Span::dummy(), &security).is_ok());
        assert!(call_builtin("print", &[Value::Null], Span::dummy(), &security).is_ok());
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
        // This is a behavioral test - actual stdout not captured in unit test
        let result = call_builtin("print", &[Value::Null], Span::dummy(), &security);
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
        assert!(call_builtin("str", &[Value::Number(42.0)], Span::dummy(), &security).is_ok());
        assert!(call_builtin("str", &[Value::Bool(true)], Span::dummy(), &security).is_ok());
        assert!(call_builtin("str", &[Value::Null], Span::dummy(), &security).is_ok());
    }
}
