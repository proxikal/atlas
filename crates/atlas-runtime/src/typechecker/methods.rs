//! Method table infrastructure for method resolution

use crate::types::Type;
use std::collections::HashMap;

/// Signature of a method
#[derive(Debug, Clone)]
pub struct MethodSignature {
    /// Argument types (excluding the receiver/target)
    pub arg_types: Vec<Type>,
    /// Return type
    pub return_type: Type,
}

/// Method table for resolving method calls
pub struct MethodTable {
    /// Map of (type_name, method_name) -> MethodSignature
    methods: HashMap<(String, String), MethodSignature>,
}

impl MethodTable {
    /// Create a new method table with built-in methods registered
    pub fn new() -> Self {
        let mut table = Self {
            methods: HashMap::new(),
        };
        table.populate_builtin_methods();
        table
    }

    /// Register a method for a type
    pub fn register(
        &mut self,
        type_name: &str,
        method_name: &str,
        arg_types: Vec<Type>,
        return_type: Type,
    ) {
        let key = (type_name.to_string(), method_name.to_string());
        let sig = MethodSignature {
            arg_types,
            return_type,
        };
        self.methods.insert(key, sig);
    }

    /// Look up a method for a type
    pub fn lookup(&self, receiver_type: &Type, method_name: &str) -> Option<&MethodSignature> {
        let receiver_type = receiver_type.normalized();
        // Convert Type to string for lookup
        let type_name = match receiver_type {
            Type::JsonValue => "json",
            Type::String => "string",
            Type::Number => "number",
            Type::Bool => "bool",
            Type::Array(_) => "array",
            _ => return None,
        };

        let key = (type_name.to_string(), method_name.to_string());
        self.methods.get(&key)
    }

    /// Populate built-in methods for stdlib types
    fn populate_builtin_methods(&mut self) {
        // JSON extraction methods
        self.register("json", "as_string", vec![], Type::String);
        self.register("json", "as_number", vec![], Type::Number);
        self.register("json", "as_bool", vec![], Type::Bool);
        self.register("json", "is_null", vec![], Type::Bool);

        // Array methods
        // Mutating collection methods — return the updated array
        let any_array = Type::Array(Box::new(Type::Unknown));
        self.register("array", "push", vec![Type::Unknown], any_array.clone());
        self.register("array", "unshift", vec![Type::Unknown], any_array.clone());
        self.register("array", "reverse", vec![], any_array.clone());
        // Mutating pair methods — return the extracted element (receiver updated as side effect)
        self.register("array", "pop", vec![], Type::Unknown);
        self.register("array", "shift", vec![], Type::Unknown);
        // Non-mutating methods — return new value, receiver unchanged
        self.register("array", "sort", vec![], any_array.clone());
        self.register("array", "len", vec![], Type::Number);
        self.register("array", "includes", vec![Type::Unknown], Type::Bool);
        self.register("array", "indexOf", vec![Type::Unknown], Type::Number);
        self.register("array", "lastIndexOf", vec![Type::Unknown], Type::Number);
        self.register(
            "array",
            "slice",
            vec![Type::Number, Type::Number],
            any_array.clone(),
        );
        self.register(
            "array",
            "concat",
            vec![any_array.clone()],
            any_array.clone(),
        );
        self.register("array", "flatten", vec![], any_array.clone());
        self.register("array", "join", vec![Type::String], Type::String);
    }
}

impl Default for MethodTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_method_lookup() {
        let table = MethodTable::new();

        // Test as_string
        let sig = table.lookup(&Type::JsonValue, "as_string").unwrap();
        assert_eq!(sig.arg_types.len(), 0);
        assert_eq!(sig.return_type, Type::String);

        // Test as_number
        let sig = table.lookup(&Type::JsonValue, "as_number").unwrap();
        assert_eq!(sig.return_type, Type::Number);

        // Test as_bool
        let sig = table.lookup(&Type::JsonValue, "as_bool").unwrap();
        assert_eq!(sig.return_type, Type::Bool);

        // Test is_null
        let sig = table.lookup(&Type::JsonValue, "is_null").unwrap();
        assert_eq!(sig.return_type, Type::Bool);
    }

    #[test]
    fn test_invalid_method_lookup() {
        let table = MethodTable::new();

        // Non-existent method
        assert!(table.lookup(&Type::JsonValue, "invalid_method").is_none());

        // Method on wrong type
        assert!(table.lookup(&Type::String, "as_string").is_none());
        assert!(table.lookup(&Type::Number, "as_number").is_none());
    }

    #[test]
    fn test_register_custom_method() {
        let mut table = MethodTable::new();

        // Register a custom method
        table.register("string", "to_upper", vec![], Type::String);

        let sig = table.lookup(&Type::String, "to_upper").unwrap();
        assert_eq!(sig.arg_types.len(), 0);
        assert_eq!(sig.return_type, Type::String);
    }
}
