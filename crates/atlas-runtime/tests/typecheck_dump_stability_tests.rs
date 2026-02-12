//! Tests for typecheck dump format stability
//!
//! Verifies that:
//! - Typecheck dumps include version field
//! - Version field is always set correctly
//! - Typecheck dump format is stable and deterministic
//! - Version mismatch handling for future-proofing

use atlas_runtime::{
    Binder, Lexer, Parser, TypecheckDump, TYPECHECK_VERSION,
};

/// Helper to create a typecheck dump from source code
fn typecheck_dump_from_source(source: &str) -> TypecheckDump {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (table, _) = binder.bind(&program);

    TypecheckDump::from_symbol_table(&table)
}

#[test]
fn test_version_field_always_present() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);

    assert_eq!(dump.typecheck_version, TYPECHECK_VERSION);
    assert_eq!(dump.typecheck_version, 1);
}

#[test]
fn test_version_field_in_json() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);
    let json = dump.to_json_string().unwrap();

    assert!(
        json.contains("\"typecheck_version\": 1"),
        "JSON must contain version field: {}",
        json
    );
}

#[test]
fn test_version_field_in_compact_json() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);
    let json = dump.to_json_compact().unwrap();

    assert!(
        json.contains("\"typecheck_version\":1"),
        "Compact JSON must contain version field: {}",
        json
    );
}

#[test]
fn test_typecheck_dump_is_deterministic() {
    let source = r#"
        fn foo(x: number) -> number {
            let y = x + 5;
            return y;
        }
    "#;

    let dump1 = typecheck_dump_from_source(source);
    let dump2 = typecheck_dump_from_source(source);

    let json1 = dump1.to_json_string().unwrap();
    let json2 = dump2.to_json_string().unwrap();

    assert_eq!(json1, json2, "Same source should produce identical JSON output");
}

#[test]
fn test_typecheck_dump_compact_is_deterministic() {
    let source = r#"
        fn bar(a: string) -> string {
            let b = a;
            return b;
        }
    "#;

    let dump1 = typecheck_dump_from_source(source);
    let dump2 = typecheck_dump_from_source(source);

    let json1 = dump1.to_json_compact().unwrap();
    let json2 = dump2.to_json_compact().unwrap();

    assert_eq!(json1, json2, "Same source should produce identical compact JSON");
}

#[test]
fn test_symbols_sorted_by_position() {
    let source = r#"
        let z = 10;
        let a = 5;
        let m = 7;
    "#;

    let dump = typecheck_dump_from_source(source);

    // Verify symbols are sorted by start position
    for i in 1..dump.symbols.len() {
        assert!(
            dump.symbols[i - 1].start <= dump.symbols[i].start,
            "Symbols must be sorted by start position for deterministic output"
        );
    }
}

#[test]
fn test_types_sorted_alphabetically() {
    let source = r#"
        let s = "hello";
        let n = 42;
        let b = true;
    "#;

    let dump = typecheck_dump_from_source(source);

    // Verify types are sorted alphabetically
    let type_names: Vec<String> = dump.types.iter().map(|t| t.name.clone()).collect();
    let mut sorted_names = type_names.clone();
    sorted_names.sort();

    assert_eq!(
        type_names, sorted_names,
        "Types must be sorted alphabetically for deterministic output"
    );
}

#[test]
fn test_json_roundtrip_preserves_version() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);
    let json = dump.to_json_string().unwrap();

    let deserialized: TypecheckDump = serde_json::from_str(&json).unwrap();

    assert_eq!(
        deserialized.typecheck_version, TYPECHECK_VERSION,
        "Version must be preserved through JSON roundtrip"
    );
    assert_eq!(deserialized, dump);
}

#[test]
fn test_version_mismatch_detection() {
    // Create a JSON with a different version
    let json_v2 = r#"{
        "typecheck_version": 2,
        "symbols": [],
        "types": []
    }"#;

    let result: Result<TypecheckDump, _> = serde_json::from_str(json_v2);
    assert!(result.is_ok(), "Should be able to deserialize different versions");

    let dump = result.unwrap();
    assert_eq!(dump.typecheck_version, 2, "Should preserve version from JSON");
    assert_ne!(
        dump.typecheck_version, TYPECHECK_VERSION,
        "Version mismatch should be detectable"
    );
}

#[test]
fn test_typecheck_dump_schema_stability() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);
    let json = dump.to_json_string().unwrap();

    // Parse as generic JSON to verify structure
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Verify required fields exist
    assert!(parsed["typecheck_version"].is_number(), "Must have typecheck_version");
    assert!(parsed["symbols"].is_array(), "Must have symbols array");
    assert!(parsed["types"].is_array(), "Must have types array");

    // Verify version value
    assert_eq!(parsed["typecheck_version"].as_u64(), Some(1));
}

#[test]
fn test_symbol_info_has_required_fields() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);

    if let Some(symbol) = dump.symbols.first() {
        let json = serde_json::to_string(symbol).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed["name"].is_string(), "Symbol must have name");
        assert!(parsed["kind"].is_string(), "Symbol must have kind");
        assert!(parsed["start"].is_number(), "Symbol must have start");
        assert!(parsed["end"].is_number(), "Symbol must have end");
        assert!(parsed["type"].is_string(), "Symbol must have type");
        assert!(parsed["mutable"].is_boolean(), "Symbol must have mutable");
    }
}

#[test]
fn test_type_info_has_required_fields() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);

    if let Some(type_info) = dump.types.first() {
        let json = serde_json::to_string(type_info).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed["name"].is_string(), "Type must have name");
        assert!(parsed["kind"].is_string(), "Type must have kind");
        // details is optional, so we don't assert its presence
    }
}

#[test]
fn test_empty_program_typecheck_dump() {
    let source = "";
    let dump = typecheck_dump_from_source(source);

    assert_eq!(dump.typecheck_version, TYPECHECK_VERSION);
    assert_eq!(dump.symbols.len(), 0, "Empty program should have no symbols");
    assert_eq!(dump.types.len(), 0, "Empty program should have no types");
}

#[test]
fn test_complex_program_typecheck_dump() {
    let source = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        fn main() -> void {
            let x = 5;
            let y = 10;
            let z = add(x, y);
            print(z);
        }
    "#;

    let dump = typecheck_dump_from_source(source);

    assert_eq!(dump.typecheck_version, TYPECHECK_VERSION);
    assert!(dump.symbols.len() > 0, "Complex program should have symbols");
    assert!(dump.types.len() > 0, "Complex program should have types");

    // Verify JSON is valid
    let json = dump.to_json_string().unwrap();
    let _: TypecheckDump = serde_json::from_str(&json).unwrap();
}

#[test]
fn test_array_types_in_typecheck_dump() {
    let source = r#"
        fn test() -> void {
            let arr: number[] = [1, 2, 3];
        }
    "#;

    let dump = typecheck_dump_from_source(source);

    // The dump should be valid even if empty
    assert_eq!(dump.typecheck_version, TYPECHECK_VERSION);

    // Find array type (if any)
    let array_types: Vec<_> = dump.types.iter().filter(|t| t.name.contains("[]")).collect();

    // Verify array type has correct kind if it exists
    for array_type in array_types {
        assert_eq!(array_type.kind, "array", "Array type should have 'array' kind");
        assert!(array_type.details.is_some(), "Array type should have details");
    }
}

#[test]
fn test_function_types_in_typecheck_dump() {
    let source = r#"
        fn foo(x: number) -> string {
            return "hello";
        }
    "#;

    let dump = typecheck_dump_from_source(source);

    // Find function type
    let func_types: Vec<_> = dump.types.iter().filter(|t| t.name.contains("->")).collect();
    assert!(func_types.len() > 0, "Should have function type");

    // Verify function type has correct kind
    for func_type in func_types {
        assert_eq!(func_type.kind, "function", "Function type should have 'function' kind");
        assert!(func_type.details.is_some(), "Function type should have details");
    }
}

#[test]
fn test_typecheck_dump_stability_across_runs() {
    let source = r#"
        fn test(x: number, y: string) -> bool {
            let z = x + 5;
            return true;
        }
    "#;

    // Run multiple times to ensure stability
    let dumps: Vec<_> = (0..5)
        .map(|_| typecheck_dump_from_source(source))
        .collect();

    let jsons: Vec<_> = dumps.iter()
        .map(|d| d.to_json_string().unwrap())
        .collect();

    // All outputs should be identical
    for (i, json) in jsons.iter().enumerate().skip(1) {
        assert_eq!(
            &jsons[0], json,
            "Run {} produced different output than run 0",
            i
        );
    }
}

#[test]
fn test_version_field_is_first_in_json() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);
    let json = dump.to_json_string().unwrap();

    // The version field should appear early in the JSON
    // (This is ensured by serde field ordering)
    let version_pos = json.find("\"typecheck_version\"").expect("Version field must exist");
    let symbols_pos = json.find("\"symbols\"").expect("Symbols field must exist");

    assert!(
        version_pos < symbols_pos,
        "Version field should appear before symbols for easier parsing"
    );
}
