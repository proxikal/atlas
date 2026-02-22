//! Comprehensive error code registry with descriptions and help text
//!
//! Error codes follow the ATxxxx scheme for errors and AWxxxx for warnings.
//! Ranges:
//! - AT0xxx: Runtime errors (type, undefined, bounds, etc.)
//! - AT01xx: Stdlib errors
//! - AT03xx: Permission errors
//! - AT04xx: I/O errors
//! - AT1xxx: Syntax/lexer errors
//! - AT2xxx: Warnings (unused, unreachable, etc.)
//! - AT3xxx: Semantic/type checking errors
//! - AT5xxx: Module system errors
//! - AT9xxx: Internal errors

// === Error Code Constants ===

// AT0xxx - Type and Runtime Errors
pub const TYPE_MISMATCH: &str = "AT0001";
pub const UNDEFINED_SYMBOL: &str = "AT0002";
pub const DIVIDE_BY_ZERO: &str = "AT0005";
pub const ARRAY_OUT_OF_BOUNDS: &str = "AT0006";
pub const INVALID_NUMERIC_RESULT: &str = "AT0007";
pub const STDLIB_ARG_ERROR: &str = "AT0102";
pub const STDLIB_VALUE_ERROR: &str = "AT0103";

// AT03xx - Permission Errors
pub const FILESYSTEM_PERMISSION_DENIED: &str = "AT0300";
pub const NETWORK_PERMISSION_DENIED: &str = "AT0301";
pub const PROCESS_PERMISSION_DENIED: &str = "AT0302";
pub const ENVIRONMENT_PERMISSION_DENIED: &str = "AT0303";

// AT1xxx - Syntax Errors
pub const SYNTAX_ERROR: &str = "AT1000";
pub const UNEXPECTED_TOKEN: &str = "AT1001";
pub const UNTERMINATED_STRING: &str = "AT1002";
pub const INVALID_ESCAPE: &str = "AT1003";
pub const UNTERMINATED_COMMENT: &str = "AT1004";
pub const INVALID_NUMBER: &str = "AT1005";
pub const UNEXPECTED_EOF: &str = "AT1006";
pub const SHADOWING_PRELUDE: &str = "AT1012";

// AT2xxx - Warnings
pub const UNUSED_VARIABLE: &str = "AT2001";
pub const UNREACHABLE_CODE: &str = "AT2002";
pub const DUPLICATE_DECLARATION: &str = "AT2003";
pub const UNUSED_FUNCTION: &str = "AT2004";
pub const VARIABLE_SHADOWING: &str = "AT2005";
pub const CONSTANT_CONDITION: &str = "AT2006";
pub const UNNECESSARY_ANNOTATION: &str = "AT2007";
pub const UNUSED_IMPORT: &str = "AT2008";
pub const DEPRECATED_TYPE_ALIAS: &str = "AT2009";
pub const OWN_ON_PRIMITIVE: &str = "AT2010";
pub const BORROW_ON_SHARED: &str = "AT2011";
pub const BORROW_TO_OWN: &str = "AT2012";
/// Warning: a non-Copy (Move) type is passed to a parameter without an ownership annotation.
/// Add `own` or `borrow` to the parameter to clarify ownership transfer semantics.
pub const MOVE_TYPE_REQUIRES_OWNERSHIP_ANNOTATION: &str = "AT2013";

// AT3xxx - Semantic and Type Checking Errors
pub const TYPE_ERROR: &str = "AT3001";
pub const BINARY_OP_TYPE_ERROR: &str = "AT3002";
pub const IMMUTABLE_ASSIGNMENT: &str = "AT3003";
pub const MISSING_RETURN: &str = "AT3004";
pub const ARITY_MISMATCH: &str = "AT3005";
pub const NOT_CALLABLE: &str = "AT3006";
pub const INVALID_INDEX_TYPE: &str = "AT3010";
pub const NOT_INDEXABLE: &str = "AT3011";
pub const MATCH_EMPTY: &str = "AT3020";
pub const MATCH_ARM_TYPE_MISMATCH: &str = "AT3021";
pub const PATTERN_TYPE_MISMATCH: &str = "AT3022";
pub const CONSTRUCTOR_ARITY: &str = "AT3023";
pub const UNKNOWN_CONSTRUCTOR: &str = "AT3024";
pub const UNSUPPORTED_PATTERN_TYPE: &str = "AT3025";
pub const ARRAY_PATTERN_TYPE_MISMATCH: &str = "AT3026";
pub const NON_EXHAUSTIVE_MATCH: &str = "AT3027";
pub const NON_SHARED_TO_SHARED: &str = "AT3028";

/// Fired when an `impl Trait for Type` already exists for the same `(Type, Trait)` pair.
/// Each type may only have one impl per trait. Remove or merge duplicate impls.
pub const IMPL_ALREADY_EXISTS: &str = "AT3029";

/// Fired when a `trait` declaration attempts to redefine a built-in trait (Copy, Move, Drop,
/// Display, Debug). Built-in traits are provided by the runtime and cannot be redeclared.
pub const TRAIT_REDEFINES_BUILTIN: &str = "AT3030";

/// Fired when a `trait` with the same name is declared more than once in the same scope.
/// Trait names must be unique. Rename or remove the duplicate declaration.
pub const TRAIT_ALREADY_DEFINED: &str = "AT3031";

/// Fired when an `impl` block references a trait that has not been declared.
/// Ensure the trait is declared with `trait TraitName { ... }` before the impl.
pub const TRAIT_NOT_FOUND: &str = "AT3032";

/// Fired when an `impl` block is missing a method required by the trait.
/// Every method listed in the trait declaration must be implemented.
pub const IMPL_METHOD_MISSING: &str = "AT3033";

/// Fired when an `impl` block's method signature does not match the trait's declaration.
/// Parameter types and return type must match exactly (excluding the `self` parameter type).
pub const IMPL_METHOD_SIGNATURE_MISMATCH: &str = "AT3034";

/// Fired when a method is called on a type that does not implement the required trait.
/// Implement the trait for the type with `impl TraitName for TypeName { ... }`.
pub const TYPE_DOES_NOT_IMPLEMENT_TRAIT: &str = "AT3035";

/// Fired when a context requires a Copy type but a non-Copy type is provided.
/// Primitive types (number, string, bool) are Copy. User-defined types default to Move.
pub const COPY_TYPE_REQUIRED: &str = "AT3036";

/// Fired when a generic type argument does not satisfy a trait bound.
/// For example, `fn f<T: Display>(x: T)` requires `T` to implement `Display`.
pub const TRAIT_BOUND_NOT_SATISFIED: &str = "AT3037";

// AT5xxx - Module System Errors
pub const INVALID_MODULE_PATH: &str = "AT5001";
pub const MODULE_NOT_FOUND: &str = "AT5002";
pub const CIRCULAR_DEPENDENCY: &str = "AT5003";
pub const EXPORT_NOT_FOUND: &str = "AT5004";
pub const IMPORT_RESOLUTION_FAILED: &str = "AT5005";
pub const MODULE_NOT_EXPORTED: &str = "AT5006";
pub const NAMESPACE_IMPORT_UNSUPPORTED: &str = "AT5007";
pub const DUPLICATE_EXPORT: &str = "AT5008";

// AT9xxx - Internal Errors
pub const INTERNAL_ERROR: &str = "AT9995";
pub const STACK_UNDERFLOW: &str = "AT9997";
pub const UNKNOWN_OPCODE: &str = "AT9998";
pub const GENERIC_ERROR: &str = "AT9999";
pub const GENERIC_WARNING: &str = "AW9999";

// === Error Code Info Registry ===

/// Error code descriptor with code, description, and optional help text
#[derive(Debug, Clone)]
pub struct ErrorCodeInfo {
    /// The error code string (e.g., "AT0001")
    pub code: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Optional contextual help text
    pub help: Option<&'static str>,
}

/// Get info for an error code, if known
pub fn lookup(code: &str) -> Option<ErrorCodeInfo> {
    ERROR_CODES.iter().find(|e| e.code == code).cloned()
}

/// Get help text for an error code
pub fn help_for(code: &str) -> Option<&'static str> {
    lookup(code).and_then(|e| e.help)
}

/// Get description for an error code
pub fn description_for(code: &str) -> Option<&'static str> {
    lookup(code).map(|e| e.description)
}

/// All known error codes with descriptions and help
pub static ERROR_CODES: &[ErrorCodeInfo] = &[
    // === AT0xxx: Runtime Errors ===
    ErrorCodeInfo {
        code: "AT0001",
        description: "Type mismatch",
        help: Some("Ensure the types match. Use explicit type conversions if needed."),
    },
    ErrorCodeInfo {
        code: "AT0002",
        description: "Undefined symbol",
        help: Some("Check spelling. The variable or function may not be in scope."),
    },
    ErrorCodeInfo {
        code: "AT0003",
        description: "Arity mismatch",
        help: Some("Check the function signature for the correct number of arguments."),
    },
    ErrorCodeInfo {
        code: "AT0004",
        description: "Invalid operation",
        help: Some("This operation is not supported for the given types."),
    },
    ErrorCodeInfo {
        code: "AT0005",
        description: "Division by zero",
        help: Some("Check that the divisor is not zero before dividing."),
    },
    ErrorCodeInfo {
        code: "AT0006",
        description: "Array index out of bounds",
        help: Some("Check array length with len() before accessing elements."),
    },
    ErrorCodeInfo {
        code: "AT0007",
        description: "Invalid numeric result (NaN or Infinity)",
        help: Some("Ensure the number is finite. Check inputs to math operations."),
    },
    // AT01xx: Stdlib errors
    ErrorCodeInfo {
        code: "AT0102",
        description: "Invalid stdlib argument",
        help: Some("Check the function documentation for valid argument types and ranges."),
    },
    ErrorCodeInfo {
        code: "AT0103",
        description: "Invalid value for stdlib operation",
        help: Some("The provided value is outside the expected range or type."),
    },
    ErrorCodeInfo {
        code: "AT0140",
        description: "Unhashable type",
        help: Some("Only number, string, bool, and null are hashable. Convert your value first."),
    },
    // AT03xx: Permission errors
    ErrorCodeInfo {
        code: "AT0300",
        description: "Filesystem permission denied",
        help: Some("Enable file permissions with --allow-file or adjust security settings."),
    },
    ErrorCodeInfo {
        code: "AT0301",
        description: "Network permission denied",
        help: Some("Enable network permissions with --allow-network or adjust security settings."),
    },
    ErrorCodeInfo {
        code: "AT0302",
        description: "Process permission denied",
        help: Some("Enable process permissions with --allow-process or adjust security settings."),
    },
    ErrorCodeInfo {
        code: "AT0303",
        description: "Environment variable permission denied",
        help: Some("Enable environment permissions with --allow-env or adjust security settings."),
    },
    // AT04xx: I/O errors
    ErrorCodeInfo {
        code: "AT0400",
        description: "I/O error",
        help: Some("Check file paths, permissions, and that the file system is accessible."),
    },
    // === AT1xxx: Syntax/Lexer Errors ===
    ErrorCodeInfo {
        code: "AT1000",
        description: "Syntax error",
        help: Some("Check the syntax near the indicated location."),
    },
    ErrorCodeInfo {
        code: "AT1001",
        description: "Unexpected token",
        help: Some("The parser encountered a token it didn't expect. Check for missing semicolons, brackets, or operators."),
    },
    ErrorCodeInfo {
        code: "AT1002",
        description: "Unterminated string literal",
        help: Some("Add the closing quote to complete the string."),
    },
    ErrorCodeInfo {
        code: "AT1003",
        description: "Invalid escape sequence",
        help: Some("Valid escapes: \\n, \\t, \\r, \\\\, \\\", \\0. Use \\\\ for a literal backslash."),
    },
    ErrorCodeInfo {
        code: "AT1004",
        description: "Unterminated block comment",
        help: Some("Add */ to close the block comment."),
    },
    ErrorCodeInfo {
        code: "AT1005",
        description: "Invalid number literal",
        help: Some("Check the number format. Numbers must be valid decimal or floating-point."),
    },
    ErrorCodeInfo {
        code: "AT1006",
        description: "Unexpected end of file",
        help: Some("The file ended unexpectedly. Check for missing closing brackets or semicolons."),
    },
    ErrorCodeInfo {
        code: "AT1012",
        description: "Cannot shadow prelude builtin at top level",
        help: Some("Prelude builtins cannot be redefined at the top level. Use a different name or shadow in a nested scope."),
    },
    // === AT2xxx: Warnings ===
    ErrorCodeInfo {
        code: "AT2001",
        description: "Unused variable or parameter",
        help: Some("Remove the unused binding or prefix with underscore: _name"),
    },
    ErrorCodeInfo {
        code: "AT2002",
        description: "Unreachable code",
        help: Some("Remove this code or restructure your control flow."),
    },
    ErrorCodeInfo {
        code: "AT2003",
        description: "Duplicate declaration",
        help: Some("Remove the duplicate or rename one of the declarations."),
    },
    ErrorCodeInfo {
        code: "AT2004",
        description: "Unused function",
        help: Some("Remove the unused function or prefix with underscore: _name"),
    },
    ErrorCodeInfo {
        code: "AT2005",
        description: "Variable shadowing",
        help: Some("This variable shadows a variable from an outer scope. Use a different name if unintentional."),
    },
    ErrorCodeInfo {
        code: "AT2006",
        description: "Constant condition",
        help: Some("This condition is always true or always false. Simplify the expression."),
    },
    ErrorCodeInfo {
        code: "AT2007",
        description: "Unnecessary type annotation",
        help: Some("The type can be inferred. Consider removing the explicit annotation."),
    },
    ErrorCodeInfo {
        code: "AT2008",
        description: "Unused import",
        help: Some("Remove the unused import statement."),
    },
    ErrorCodeInfo {
        code: "AT2009",
        description: "Deprecated type alias",
        help: Some("Use the recommended replacement instead of the deprecated alias."),
    },
    ErrorCodeInfo {
        code: "AT2010",
        description: "`own` annotation on primitive type has no effect",
        help: Some("Primitive types (number, bool, string) are always copied. The `own` annotation is ignored."),
    },
    ErrorCodeInfo {
        code: "AT2011",
        description: "`borrow` annotation on `shared<T>` type is redundant",
        help: Some("`shared<T>` already has reference semantics. The `borrow` annotation has no additional effect."),
    },
    ErrorCodeInfo {
        code: "AT2012",
        description: "Passing borrowed value to `own` parameter — ownership cannot transfer",
        help: Some("A `borrow` parameter cannot give up ownership. Pass an owned value instead."),
    },
    ErrorCodeInfo {
        code: "AT2013",
        description: "Non-Copy type passed without ownership annotation",
        help: Some("This type is not Copy. Annotate the parameter with `own` or `borrow` to clarify ownership intent."),
    },
    // === AT3xxx: Semantic/Type Checking Errors ===
    ErrorCodeInfo {
        code: "AT3001",
        description: "Type error in expression",
        help: Some("Check that the expression types are compatible."),
    },
    ErrorCodeInfo {
        code: "AT3002",
        description: "Binary operation type error",
        help: Some("Ensure both operands have compatible types for this operator."),
    },
    ErrorCodeInfo {
        code: "AT3003",
        description: "Assignment to immutable variable",
        help: Some("Use 'let mut' to declare a mutable variable."),
    },
    ErrorCodeInfo {
        code: "AT3004",
        description: "Missing return value",
        help: Some("Ensure all code paths return a value of the declared return type."),
    },
    ErrorCodeInfo {
        code: "AT3005",
        description: "Function arity mismatch",
        help: Some("Check the function signature for the correct number of arguments."),
    },
    ErrorCodeInfo {
        code: "AT3006",
        description: "Expression is not callable",
        help: Some("Only functions can be called. Check the type of this expression."),
    },
    ErrorCodeInfo {
        code: "AT3010",
        description: "Invalid index type",
        help: Some("Array indices must be numbers. HashMap keys must match the key type."),
    },
    ErrorCodeInfo {
        code: "AT3011",
        description: "Type is not indexable",
        help: Some("Only arrays and hashmaps can be indexed."),
    },
    ErrorCodeInfo {
        code: "AT3020",
        description: "Empty match expression",
        help: Some("Add at least one arm to the match expression."),
    },
    ErrorCodeInfo {
        code: "AT3021",
        description: "Match arm type mismatch",
        help: Some("All match arms must return the same type."),
    },
    ErrorCodeInfo {
        code: "AT3022",
        description: "Pattern type mismatch",
        help: Some("The pattern type must be compatible with the matched value."),
    },
    ErrorCodeInfo {
        code: "AT3023",
        description: "Constructor arity mismatch",
        help: Some("Check the constructor for the correct number of fields."),
    },
    ErrorCodeInfo {
        code: "AT3024",
        description: "Unknown constructor",
        help: Some("This constructor is not defined. Check the type definition."),
    },
    ErrorCodeInfo {
        code: "AT3025",
        description: "Unsupported pattern type",
        help: Some("This pattern form is not supported in this context."),
    },
    ErrorCodeInfo {
        code: "AT3026",
        description: "Array pattern type mismatch",
        help: Some("The array pattern must match the array element type."),
    },
    ErrorCodeInfo {
        code: "AT3027",
        description: "Non-exhaustive match",
        help: Some("Add a wildcard arm (_) or cover all possible cases."),
    },
    ErrorCodeInfo {
        code: "AT3028",
        description: "Passing non-`shared<T>` value to `shared` parameter",
        help: Some("Wrap the value in a shared reference before passing it to a `shared` parameter."),
    },
    ErrorCodeInfo {
        code: "AT3029",
        description: "Duplicate impl block",
        help: Some("A type can only implement a given trait once. Remove the duplicate impl block."),
    },
    // === AT3030+: Trait System Errors ===
    ErrorCodeInfo {
        code: "AT3030",
        description: "Cannot redefine built-in trait",
        help: Some("Built-in traits (Copy, Move, Drop, Display, Debug) cannot be redeclared by user code."),
    },
    ErrorCodeInfo {
        code: "AT3031",
        description: "Trait already defined",
        help: Some("A trait with this name is already declared in scope. Use a different name."),
    },
    ErrorCodeInfo {
        code: "AT3032",
        description: "Trait not found",
        help: Some("The trait name was not declared. Declare it with `trait Name { ... }` before using it."),
    },
    ErrorCodeInfo {
        code: "AT3033",
        description: "impl block is missing required method",
        help: Some("The impl block must implement all methods declared in the trait."),
    },
    ErrorCodeInfo {
        code: "AT3034",
        description: "impl method signature does not match trait declaration",
        help: Some("The method's parameter types and return type must exactly match the trait definition."),
    },
    ErrorCodeInfo {
        code: "AT3035",
        description: "Type does not implement required trait",
        help: Some("Add an `impl TraitName for TypeName { ... }` block to satisfy the trait requirement."),
    },
    ErrorCodeInfo {
        code: "AT3036",
        description: "Copy type required",
        help: Some("This operation requires a Copy type. Implement the Copy trait or use a value type."),
    },
    ErrorCodeInfo {
        code: "AT3037",
        description: "Trait bound not satisfied",
        help: Some("The type argument does not satisfy the required trait bound on this type parameter."),
    },
    // === AT5xxx: Module System Errors ===
    ErrorCodeInfo {
        code: "AT5001",
        description: "Invalid module path",
        help: Some("Module paths must be valid file paths relative to the project root."),
    },
    ErrorCodeInfo {
        code: "AT5002",
        description: "Module not found",
        help: Some("Check the module path and ensure the file exists."),
    },
    ErrorCodeInfo {
        code: "AT5003",
        description: "Circular dependency detected",
        help: Some("Reorganize modules to break the circular import chain."),
    },
    ErrorCodeInfo {
        code: "AT5004",
        description: "Export not found in module",
        help: Some("Check the module's exports. The symbol may not be exported."),
    },
    ErrorCodeInfo {
        code: "AT5005",
        description: "Import resolution failed",
        help: Some("Check the import path and module structure."),
    },
    ErrorCodeInfo {
        code: "AT5006",
        description: "Module does not export this symbol",
        help: Some("Add 'export' to the symbol declaration in the source module."),
    },
    ErrorCodeInfo {
        code: "AT5007",
        description: "Namespace import not supported",
        help: Some("Use named imports: import { name } from \"module\""),
    },
    ErrorCodeInfo {
        code: "AT5008",
        description: "Duplicate export",
        help: Some("Each symbol can only be exported once per module."),
    },
    // === AT9xxx: Internal Errors ===
    ErrorCodeInfo {
        code: "AT9995",
        description: "Internal compiler error",
        help: Some("This is a bug in the compiler. Please report it."),
    },
    ErrorCodeInfo {
        code: "AT9997",
        description: "Stack underflow",
        help: Some("This is a VM internal error. Please report it."),
    },
    ErrorCodeInfo {
        code: "AT9998",
        description: "Unknown bytecode opcode",
        help: Some("This is a VM internal error. Please report it."),
    },
    ErrorCodeInfo {
        code: "AT9999",
        description: "Generic error",
        help: None,
    },
    ErrorCodeInfo {
        code: "AW9999",
        description: "Generic warning",
        help: None,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_duplicate_codes() {
        let mut seen = std::collections::HashSet::new();
        for entry in ERROR_CODES {
            assert!(
                seen.insert(entry.code),
                "Duplicate error code: {}",
                entry.code
            );
        }
    }

    #[test]
    fn test_lookup_existing() {
        let info = lookup("AT0001").unwrap();
        assert_eq!(info.description, "Type mismatch");
        assert!(info.help.is_some());
    }

    #[test]
    fn test_lookup_missing() {
        assert!(lookup("ZZZZ").is_none());
    }

    #[test]
    fn test_help_for() {
        assert!(help_for("AT0005").is_some());
        assert!(help_for("AT9999").is_none()); // Generic has no help
    }

    #[test]
    fn test_description_for() {
        assert_eq!(description_for("AT1001").unwrap(), "Unexpected token");
    }

    #[test]
    fn test_all_codes_have_descriptions() {
        for entry in ERROR_CODES {
            assert!(
                !entry.description.is_empty(),
                "{} has empty description",
                entry.code
            );
        }
    }

    #[test]
    fn test_warning_codes_start_with_at2_or_aw() {
        for entry in ERROR_CODES {
            if entry.description.to_lowercase().contains("unused")
                || entry.description.to_lowercase().contains("unreachable")
                || entry.description.to_lowercase().contains("shadowing")
                || entry
                    .description
                    .to_lowercase()
                    .contains("constant condition")
                || entry.description.to_lowercase().contains("unnecessary")
            {
                assert!(
                    entry.code.starts_with("AT2") || entry.code.starts_with("AW"),
                    "Warning-like code {} should be AT2xxx or AWxxxx",
                    entry.code
                );
            }
        }
    }
}
