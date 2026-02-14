//! Module Syntax Tests (BLOCKER 04-A)
//!
//! Tests for import/export syntax parsing.
//! Does NOT test module loading or execution (that's BLOCKER 04-B, 04-C, 04-D).

use atlas_runtime::{Lexer, Parser};

/// Helper to parse code and check for errors
fn parse(source: &str) -> (bool, Vec<String>) {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return (false, lex_diags.iter().map(|d| d.message.clone()).collect());
    }

    let mut parser = Parser::new(tokens);
    let (_, parse_diags) = parser.parse();

    let success = parse_diags.is_empty();
    let messages = parse_diags.iter().map(|d| d.message.clone()).collect();
    (success, messages)
}

// ============================================================================
// Import Syntax Tests
// ============================================================================

#[test]
fn test_parse_named_import_single() {
    let source = r#"import { add } from "./math""#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse single named import: {:?}", msgs);
}

#[test]
fn test_parse_named_import_multiple() {
    let source = r#"import { add, sub, mul } from "./math""#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse multiple named imports: {:?}", msgs);
}

#[test]
fn test_parse_namespace_import() {
    let source = r#"import * as math from "./math""#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse namespace import: {:?}", msgs);
}

#[test]
fn test_parse_import_relative_path() {
    let source = r#"import { x } from "./sibling""#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse relative path: {:?}", msgs);
}

#[test]
fn test_parse_import_parent_path() {
    let source = r#"import { x } from "../parent""#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse parent path: {:?}", msgs);
}

#[test]
fn test_parse_import_absolute_path() {
    let source = r#"import { x } from "/src/utils""#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse absolute path: {:?}", msgs);
}

#[test]
fn test_parse_import_with_extension() {
    let source = r#"import { x } from "./mod.atl""#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse path with .atl extension: {:?}", msgs);
}

#[test]
fn test_parse_multiple_imports() {
    let source = r#"
        import { add } from "./math"
        import { log } from "./logger"
    "#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse multiple imports: {:?}", msgs);
}

// ============================================================================
// Export Syntax Tests
// ============================================================================

#[test]
fn test_parse_export_function() {
    let source = r#"
        export fn add(a: number, b: number) -> number {
            return a + b;
        }
    "#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse export function: {:?}", msgs);
}

#[test]
fn test_parse_export_let() {
    let source = r#"export let PI = 3.14159;"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse export let: {:?}", msgs);
}

#[test]
fn test_parse_export_var() {
    let source = r#"export var counter = 0;"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse export var: {:?}", msgs);
}

#[test]
fn test_parse_export_generic_function() {
    let source = r#"
        export fn identity<T>(x: T) -> T {
            return x;
        }
    "#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse export generic function: {:?}", msgs);
}

#[test]
fn test_parse_multiple_exports() {
    let source = r#"
        export fn add(a: number, b: number) -> number {
            return a + b;
        }
        export let PI = 3.14;
    "#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse multiple exports: {:?}", msgs);
}

// ============================================================================
// Combined Import/Export Tests
// ============================================================================

#[test]
fn test_parse_module_with_import_and_export() {
    let source = r#"
        import { log } from "./logger"

        export fn greet(name: string) -> string {
            log("greeting " + name);
            return "Hello, " + name;
        }
    "#;
    let (success, msgs) = parse(source);
    assert!(
        success,
        "Should parse module with imports and exports: {:?}",
        msgs
    );
}

#[test]
fn test_parse_module_with_multiple_imports_exports() {
    let source = r#"
        import { add, sub } from "./math"
        import * as logger from "./logger"

        export fn calculate(a: number, b: number) -> number {
            return add(a, b);
        }

        export let VERSION = "1.0";
    "#;
    let (success, msgs) = parse(source);
    assert!(
        success,
        "Should parse module with multiple imports/exports: {:?}",
        msgs
    );
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_import_missing_from() {
    let source = r#"import { x }"#;
    let (success, _) = parse(source);
    assert!(!success, "Should fail: missing 'from' keyword");
}

#[test]
fn test_import_missing_braces() {
    let source = r#"import x from "./mod""#;
    let (success, _) = parse(source);
    assert!(!success, "Should fail: missing braces for named import");
}

#[test]
fn test_namespace_import_missing_as() {
    let source = r#"import * from "./mod""#;
    let (success, _) = parse(source);
    assert!(!success, "Should fail: namespace import missing 'as'");
}

#[test]
fn test_export_without_item() {
    let source = r#"export"#;
    let (success, _) = parse(source);
    assert!(!success, "Should fail: export without fn/let/var");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_import_with_trailing_comma() {
    let source = r#"import { x, y, } from "./mod""#;
    let (success, msgs) = parse(source);
    assert!(
        success,
        "Should parse import with trailing comma: {:?}",
        msgs
    );
}

#[test]
fn test_import_empty_list() {
    let source = r#"import { } from "./mod""#;
    let (success, msgs) = parse(source);
    // This should parse but might be semantically invalid
    // For now, just check it doesn't crash the parser
    let _ = (success, msgs);
}

#[test]
fn test_complex_nested_paths() {
    let source = r#"import { x } from "../../utils/helpers/math""#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse complex nested paths: {:?}", msgs);
}
