//! Typecheck dump for AI-friendly JSON output
//!
//! Provides a stable JSON representation of inferred types and symbol bindings
//! for AI agents to analyze and understand type checking results.

use crate::symbol::{SymbolKind, SymbolTable};
use crate::types::Type;
use serde::{Deserialize, Serialize};

/// Typecheck dump schema version
pub const TYPECHECK_VERSION: u32 = 1;

/// Symbol information for typecheck dump
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolInfo {
    /// Symbol name
    pub name: String,
    /// Symbol kind (variable, parameter, function)
    pub kind: String,
    /// Start position in source
    pub start: usize,
    /// End position in source
    pub end: usize,
    /// Inferred or declared type
    #[serde(rename = "type")]
    pub ty: String,
    /// Whether the symbol is mutable
    pub mutable: bool,
}

/// Type information for typecheck dump
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeInfo {
    /// Type name/description
    pub name: String,
    /// Kind of type (primitive, array, function)
    pub kind: String,
    /// Additional type details (for arrays, functions, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Typecheck dump output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypecheckDump {
    /// Typecheck dump schema version
    pub typecheck_version: u32,
    /// Symbols in the program
    pub symbols: Vec<SymbolInfo>,
    /// Types encountered during type checking
    pub types: Vec<TypeInfo>,
}

impl TypecheckDump {
    /// Create a new typecheck dump
    pub fn new() -> Self {
        Self {
            typecheck_version: TYPECHECK_VERSION,
            symbols: Vec::new(),
            types: Vec::new(),
        }
    }

    /// Create a typecheck dump from a symbol table
    pub fn from_symbol_table(symbol_table: &SymbolTable) -> Self {
        let mut dump = Self::new();

        // Collect all symbols
        dump.symbols = symbol_table
            .all_symbols()
            .iter()
            .map(|symbol| SymbolInfo {
                name: symbol.name.clone(),
                kind: symbol_kind_to_string(&symbol.kind),
                start: symbol.span.start,
                end: symbol.span.end,
                ty: type_to_string(&symbol.ty),
                mutable: symbol.mutable,
            })
            .collect();

        // Sort symbols by position, then by name for deterministic output
        dump.symbols
            .sort_by(|a, b| a.start.cmp(&b.start).then(a.name.cmp(&b.name)));

        // Collect unique types
        let mut type_names = std::collections::HashSet::new();
        for symbol in symbol_table.all_symbols() {
            collect_types(&symbol.ty, &mut type_names);
        }

        dump.types = type_names
            .into_iter()
            .map(|type_name| {
                let (kind, details) = parse_type_info(&type_name);
                TypeInfo {
                    name: type_name,
                    kind,
                    details,
                }
            })
            .collect();

        // Sort types by name for deterministic output
        dump.types.sort_by(|a, b| a.name.cmp(&b.name));

        dump
    }

    /// Convert to JSON string (pretty-printed)
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Convert to compact JSON string
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl Default for TypecheckDump {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert symbol kind to string
fn symbol_kind_to_string(kind: &SymbolKind) -> String {
    match kind {
        SymbolKind::Variable => "variable".to_string(),
        SymbolKind::Parameter => "parameter".to_string(),
        SymbolKind::Function => "function".to_string(),
        SymbolKind::Builtin => "builtin".to_string(),
    }
}

/// Convert type to string representation
fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Number => "number".to_string(),
        Type::String => "string".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Void => "void".to_string(),
        Type::Null => "null".to_string(),
        Type::Array(elem) => format!("{}[]", type_to_string(elem)),
        Type::Function {
            params,
            return_type,
            ..
        } => {
            let param_types: Vec<String> = params.iter().map(type_to_string).collect();
            format!(
                "({}) -> {}",
                param_types.join(", "),
                type_to_string(return_type)
            )
        }
        Type::JsonValue => "json".to_string(),
        Type::Generic { name, type_args } => {
            let args = type_args
                .iter()
                .map(type_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", name, args)
        }
        Type::Alias {
            name, type_args, ..
        } => {
            if type_args.is_empty() {
                name.clone()
            } else {
                let args = type_args
                    .iter()
                    .map(type_to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", name, args)
            }
        }
        Type::TypeParameter { name } => name.clone(),
        Type::Unknown => "unknown".to_string(),
        Type::Extern(extern_type) => extern_type.display_name().to_string(),
    }
}

/// Collect all types mentioned in a type (including nested types)
fn collect_types(ty: &Type, types: &mut std::collections::HashSet<String>) {
    let type_str = type_to_string(ty);
    types.insert(type_str);

    match ty {
        Type::Array(elem) => collect_types(elem, types),
        Type::Function {
            params,
            return_type,
            ..
        } => {
            for param in params {
                collect_types(param, types);
            }
            collect_types(return_type, types);
        }
        Type::Generic { type_args, .. } => {
            for arg in type_args {
                collect_types(arg, types);
            }
        }
        Type::Alias {
            type_args, target, ..
        } => {
            for arg in type_args {
                collect_types(arg, types);
            }
            collect_types(target, types);
        }
        Type::Extern(_) => {
            // Extern types are primitives, no nested types to collect
        }
        _ => {}
    }
}

/// Parse type information into kind and details
fn parse_type_info(type_name: &str) -> (String, Option<String>) {
    if let Some(stripped) = type_name.strip_suffix("[]") {
        (
            "array".to_string(),
            Some(format!("element type: {}", stripped)),
        )
    } else if type_name.contains("->") {
        (
            "function".to_string(),
            Some(format!("signature: {}", type_name)),
        )
    } else {
        ("primitive".to_string(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;
    use crate::symbol::Symbol;

    #[test]
    fn test_typecheck_dump_version() {
        let dump = TypecheckDump::new();
        assert_eq!(dump.typecheck_version, TYPECHECK_VERSION);
        assert_eq!(dump.typecheck_version, 1);
    }

    #[test]
    fn test_typecheck_dump_json_contains_version() {
        let dump = TypecheckDump::new();
        let json = dump.to_json_string().unwrap();
        assert!(json.contains("\"typecheck_version\": 1"));
    }

    #[test]
    fn test_typecheck_dump_json_compact() {
        let dump = TypecheckDump::new();
        let json = dump.to_json_compact().unwrap();
        assert!(json.contains("\"typecheck_version\":1"));
    }

    #[test]
    fn test_symbol_info_serialization() {
        let symbol = SymbolInfo {
            name: "x".to_string(),
            kind: "variable".to_string(),
            start: 0,
            end: 5,
            ty: "number".to_string(),
            mutable: true,
        };

        let json = serde_json::to_string(&symbol).unwrap();
        assert!(json.contains("\"name\":\"x\""));
        assert!(json.contains("\"kind\":\"variable\""));
        assert!(json.contains("\"type\":\"number\""));
        assert!(json.contains("\"mutable\":true"));
    }

    #[test]
    fn test_type_info_serialization() {
        let type_info = TypeInfo {
            name: "number".to_string(),
            kind: "primitive".to_string(),
            details: None,
        };

        let json = serde_json::to_string(&type_info).unwrap();
        assert!(json.contains("\"name\":\"number\""));
        assert!(json.contains("\"kind\":\"primitive\""));
        assert!(!json.contains("\"details\""));
    }

    #[test]
    fn test_type_to_string_primitives() {
        assert_eq!(type_to_string(&Type::Number), "number");
        assert_eq!(type_to_string(&Type::String), "string");
        assert_eq!(type_to_string(&Type::Bool), "bool");
        assert_eq!(type_to_string(&Type::Void), "void");
        assert_eq!(type_to_string(&Type::Null), "null");
    }

    #[test]
    fn test_type_to_string_array() {
        let array_type = Type::Array(Box::new(Type::Number));
        assert_eq!(type_to_string(&array_type), "number[]");
    }

    #[test]
    fn test_type_to_string_function() {
        let func_type = Type::Function {
            type_params: vec![],
            params: vec![Type::Number, Type::String],
            return_type: Box::new(Type::Bool),
        };
        assert_eq!(type_to_string(&func_type), "(number, string) -> bool");
    }

    #[test]
    fn test_symbol_kind_to_string() {
        assert_eq!(symbol_kind_to_string(&SymbolKind::Variable), "variable");
        assert_eq!(symbol_kind_to_string(&SymbolKind::Parameter), "parameter");
        assert_eq!(symbol_kind_to_string(&SymbolKind::Function), "function");
        assert_eq!(symbol_kind_to_string(&SymbolKind::Builtin), "builtin");
    }

    #[test]
    fn test_parse_type_info_primitive() {
        let (kind, details) = parse_type_info("number");
        assert_eq!(kind, "primitive");
        assert!(details.is_none());
    }

    #[test]
    fn test_parse_type_info_array() {
        let (kind, details) = parse_type_info("number[]");
        assert_eq!(kind, "array");
        assert!(details.is_some());
        assert!(details.unwrap().contains("number"));
    }

    #[test]
    fn test_parse_type_info_function() {
        let (kind, details) = parse_type_info("(number) -> string");
        assert_eq!(kind, "function");
        assert!(details.is_some());
    }

    #[test]
    fn test_typecheck_dump_deterministic_json() {
        let dump = TypecheckDump::new();
        let json1 = dump.to_json_string().unwrap();
        let json2 = dump.clone().to_json_string().unwrap();
        assert_eq!(json1, json2);
    }

    #[test]
    fn test_typecheck_dump_roundtrip() {
        let dump = TypecheckDump::new();
        let json = dump.to_json_string().unwrap();
        let deserialized: TypecheckDump = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, dump);
        assert_eq!(deserialized.typecheck_version, TYPECHECK_VERSION);
    }

    #[test]
    fn test_from_symbol_table_sorts_symbols() {
        // Create a symbol table with symbols in non-sorted order
        let mut table = SymbolTable::new();
        table
            .define(Symbol {
                name: "z".to_string(),
                kind: SymbolKind::Variable,
                ty: Type::Number,
                mutable: false,
                span: Span::new(10, 15),
                exported: false,
            })
            .ok();
        table
            .define(Symbol {
                name: "a".to_string(),
                kind: SymbolKind::Variable,
                ty: Type::String,
                mutable: false,
                span: Span::new(0, 5),
                exported: false,
            })
            .ok();

        let dump = TypecheckDump::from_symbol_table(&table);

        // Filter out builtin symbols to focus on user-defined ones
        let user_symbols: Vec<_> = dump
            .symbols
            .iter()
            .filter(|s| s.kind != "builtin")
            .collect();

        // Symbols should be sorted by position (start)
        assert_eq!(user_symbols.len(), 2);
        assert_eq!(user_symbols[0].name, "a");
        assert_eq!(user_symbols[0].start, 0);
        assert_eq!(user_symbols[1].name, "z");
        assert_eq!(user_symbols[1].start, 10);
    }

    #[test]
    fn test_from_symbol_table_sorts_types() {
        let mut table = SymbolTable::new();
        table
            .define(Symbol {
                name: "z".to_string(),
                kind: SymbolKind::Variable,
                ty: Type::String,
                mutable: false,
                span: Span::new(0, 1),
                exported: false,
            })
            .ok();
        table
            .define(Symbol {
                name: "a".to_string(),
                kind: SymbolKind::Variable,
                ty: Type::Number,
                mutable: false,
                span: Span::new(5, 6),
                exported: false,
            })
            .ok();

        let dump = TypecheckDump::from_symbol_table(&table);

        // Types should be sorted alphabetically
        assert!(dump.types.len() >= 2);
        let type_names: Vec<String> = dump.types.iter().map(|t| t.name.clone()).collect();
        let mut sorted_names = type_names.clone();
        sorted_names.sort();
        assert_eq!(type_names, sorted_names);
    }
}
