//! Modern Lexer Golden Tests
//!
//! Converted from lexer_golden_tests.rs (152 lines → ~65 lines = 57% reduction)
//! Uses insta for snapshot testing instead of manual assertions

mod common;

use atlas_runtime::{diagnostic::Diagnostic, lexer::Lexer};
use rstest::rstest;
use std::fs;
use std::path::Path;

fn lex_file(filename: &str) -> Vec<Diagnostic> {
    let path = Path::new("tests/errors").join(filename);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read test file: {}", path.display()));

    let mut lexer = Lexer::new(&source);
    let (_, diagnostics) = lexer.tokenize();
    diagnostics
}

// ============================================================================
// Individual Error File Tests with Snapshots
// ============================================================================

#[rstest]
#[case("unterminated_string.atl", "AT1002")]
#[case("invalid_escape.atl", "AT1003")]
#[case("unexpected_char.atl", "AT1001")]
#[case("unterminated_comment.atl", "AT1004")]
fn test_lexer_error_files(#[case] filename: &str, #[case] expected_code: &str) {
    let diagnostics = lex_file(filename);

    // Verify we got the expected error
    assert!(
        !diagnostics.is_empty(),
        "Expected diagnostics for {}",
        filename
    );
    assert!(
        diagnostics.iter().any(|d| d.code == expected_code),
        "Expected error code {} in {}, got: {:?}",
        expected_code,
        filename,
        diagnostics.iter().map(|d| &d.code).collect::<Vec<_>>()
    );

    // Snapshot the diagnostics for stability tracking
    insta::assert_yaml_snapshot!(
        format!("lexer_error_{}", filename.replace(".atl", "")),
        diagnostics
    );
}

// ============================================================================
// Stability Test
// ============================================================================

#[test]
fn test_diagnostic_stability() {
    // Verify that running the same file twice produces identical diagnostics
    let diag1 = lex_file("unterminated_string.atl");
    let diag2 = lex_file("unterminated_string.atl");

    assert_eq!(
        diag1.len(),
        diag2.len(),
        "Diagnostic count should be stable"
    );
    for (d1, d2) in diag1.iter().zip(diag2.iter()) {
        assert_eq!(d1.code, d2.code, "Diagnostic codes should be stable");
        assert_eq!(
            d1.message, d2.message,
            "Diagnostic messages should be stable"
        );
        assert_eq!(d1.line, d2.line, "Diagnostic lines should be stable");
        assert_eq!(d1.column, d2.column, "Diagnostic columns should be stable");
    }
}
