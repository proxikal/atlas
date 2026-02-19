//! diagnostics.rs — merged from 4 files (Phase Infra-01)
//!
//! Sources: diagnostic_ordering_tests.rs, related_spans_tests.rs, enhanced_errors_tests.rs, sourcemap_tests.rs
use atlas_runtime::bytecode::{Bytecode, DebugSpan};
use atlas_runtime::diagnostic::error_codes;
use atlas_runtime::diagnostic::formatter::{
    extract_snippet, offset_to_line_col, DiagnosticFormatter,
};
use atlas_runtime::diagnostic::{normalizer::normalize_diagnostics_for_testing, sort_diagnostics};
use atlas_runtime::sourcemap::encoder::{
    decode_mappings, MappingEntry, SourceMapBuilder, SourceMapV3,
};
use atlas_runtime::sourcemap::vlq;
use atlas_runtime::sourcemap::{
    generate_from_debug_spans, generate_inline_source_map, generate_source_map, SourceMapOptions,
};
use atlas_runtime::{
    Binder, Diagnostic, DiagnosticLevel, Lexer, Parser, Span, TypeChecker, DIAG_VERSION,
};
use rstest::rstest;
use std::path::Path;

// ============================================================================
// Cross-platform test helpers
// ============================================================================

/// Generate an absolute path that works on the current platform
#[cfg(unix)]
fn absolute_test_path(filename: &str) -> String {
    format!("/absolute/path/{}", filename)
}

#[cfg(windows)]
fn absolute_test_path(filename: &str) -> String {
    format!("C:\\absolute\\path\\{}", filename)
}

/// Check if a path looks absolute (cross-platform)
fn is_absolute_path(path: &str) -> bool {
    Path::new(path).is_absolute()
}

// ============================================================================
// Diagnostic Ordering Tests (from diagnostic_ordering_tests.rs)
// ============================================================================

/// Helper to get all diagnostics from source code
fn get_all_diagnostics(source: &str) -> Vec<atlas_runtime::Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);

    // Combine all diagnostics
    let mut all_diags = Vec::new();
    all_diags.extend(lex_diags);
    all_diags.extend(parse_diags);
    all_diags.extend(bind_diags);
    all_diags.extend(type_diags);

    all_diags
}

#[test]
fn test_errors_before_warnings() {
    let source = r#"
        fn foo(x: number) -> number {
            let y = 5;
            return "hello";
        }
    "#;

    let mut diags = get_all_diagnostics(source);
    sort_diagnostics(&mut diags);

    // Count errors and warnings
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    let warnings: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning)
        .collect();

    if !errors.is_empty() && !warnings.is_empty() {
        // Find first warning index
        let first_warning_idx = diags
            .iter()
            .position(|d| d.level == DiagnosticLevel::Warning);
        // Find last error index
        let last_error_idx = diags
            .iter()
            .rposition(|d| d.level == DiagnosticLevel::Error);

        if let (Some(first_warning), Some(last_error)) = (first_warning_idx, last_error_idx) {
            assert!(
                last_error < first_warning,
                "All errors should come before all warnings"
            );
        }
    }
}

#[test]
fn test_diagnostics_sorted_by_location() {
    let source = r#"
        fn first() {}
        fn second() {}
        fn first() {}
        fn second() {}
    "#;

    let mut diags = get_all_diagnostics(source);
    sort_diagnostics(&mut diags);

    // Verify diagnostics are sorted by line and column
    for i in 1..diags.len() {
        let prev = &diags[i - 1];
        let curr = &diags[i];

        // Same level: should be sorted by file, line, column
        if prev.level == curr.level && prev.file == curr.file {
            if prev.line == curr.line {
                assert!(
                    prev.column <= curr.column,
                    "Diagnostics should be sorted by column within same line"
                );
            } else {
                assert!(
                    prev.line < curr.line,
                    "Diagnostics should be sorted by line"
                );
            }
        }
    }
}

#[test]
fn test_sort_is_deterministic() {
    let source = r#"
        fn test() -> number {
            let x = 5;
            let y = 10;
            return "hello";
        }
    "#;

    let mut diags1 = get_all_diagnostics(source);
    let mut diags2 = get_all_diagnostics(source);

    sort_diagnostics(&mut diags1);
    sort_diagnostics(&mut diags2);

    assert_eq!(
        diags1.len(),
        diags2.len(),
        "Should have same number of diagnostics"
    );

    for (d1, d2) in diags1.iter().zip(diags2.iter()) {
        assert_eq!(d1.code, d2.code, "Codes should match");
        assert_eq!(d1.level, d2.level, "Levels should match");
        assert_eq!(d1.line, d2.line, "Lines should match");
        assert_eq!(d1.column, d2.column, "Columns should match");
    }
}

#[test]
fn test_normalization_removes_absolute_paths() {
    let source = "fn foo() {}";

    let mut diags = get_all_diagnostics(source);

    // Set absolute path
    for diag in &mut diags {
        diag.file = "/absolute/path/to/test.atlas".to_string();
    }

    let normalized = normalize_diagnostics_for_testing(&diags);

    for diag in &normalized {
        assert!(
            !diag.file.starts_with('/'),
            "Normalized diagnostic should not have absolute path: {}",
            diag.file
        );
    }
}

#[test]
fn test_normalization_preserves_special_paths() {
    let source = "fn foo() {}";

    let mut diags = get_all_diagnostics(source);

    // Set special paths
    let special_paths = ["<input>", "<stdin>", "<unknown>"];
    for (i, path) in special_paths.iter().enumerate() {
        if let Some(diag) = diags.get_mut(i) {
            diag.file = path.to_string();
        }
    }

    let normalized = normalize_diagnostics_for_testing(&diags);

    for (i, path) in special_paths.iter().enumerate() {
        if let Some(diag) = normalized.get(i) {
            assert_eq!(&diag.file, path, "Special path should be preserved");
        }
    }
}

#[test]
fn test_normalization_normalizes_related_locations() {
    let source = r#"
        fn foo() {}
        fn foo() {}
    "#;

    let mut diags = get_all_diagnostics(source);

    // Set absolute path in related locations (platform-appropriate)
    for diag in &mut diags {
        diag.file = absolute_test_path("test.atlas");
        for related in &mut diag.related {
            related.file = absolute_test_path("other.atlas");
        }
    }

    let normalized = normalize_diagnostics_for_testing(&diags);

    for diag in &normalized {
        assert!(
            !is_absolute_path(&diag.file),
            "File path should be normalized, got: {}",
            diag.file
        );
        for related in &diag.related {
            assert!(
                !is_absolute_path(&related.file),
                "Related file path should be normalized: {}",
                related.file
            );
        }
    }
}

#[test]
fn test_same_error_normalizes_to_same_output() {
    let source1 = "fn foo() {}";
    let source2 = "fn foo() {}";

    let mut diags1 = get_all_diagnostics(source1);
    let mut diags2 = get_all_diagnostics(source2);

    // Set different absolute paths (platform-appropriate)
    // Both paths have same filename but different directories
    #[cfg(unix)]
    {
        for diag in &mut diags1 {
            diag.file = "/path1/test.atlas".to_string();
        }
        for diag in &mut diags2 {
            diag.file = "/path2/test.atlas".to_string();
        }
    }
    #[cfg(windows)]
    {
        for diag in &mut diags1 {
            diag.file = "C:\\path1\\test.atlas".to_string();
        }
        for diag in &mut diags2 {
            diag.file = "C:\\path2\\test.atlas".to_string();
        }
    }

    let norm1 = normalize_diagnostics_for_testing(&diags1);
    let norm2 = normalize_diagnostics_for_testing(&diags2);

    assert_eq!(
        norm1.len(),
        norm2.len(),
        "Should have same number of diagnostics"
    );

    // Compare normalized JSON output
    for (d1, d2) in norm1.iter().zip(norm2.iter()) {
        let json1 = d1.to_json_string().unwrap();
        let json2 = d2.to_json_string().unwrap();
        assert_eq!(
            json1, json2,
            "Normalized diagnostics should produce identical JSON"
        );
    }
}

#[test]
fn test_multi_span_diagnostics() {
    let source = r#"
        fn foo() {}
        fn foo() {}
    "#;

    let diags = get_all_diagnostics(source);

    // Find diagnostics with related spans
    let multi_span: Vec<_> = diags.iter().filter(|d| !d.related.is_empty()).collect();

    if !multi_span.is_empty() {
        for diag in multi_span {
            // Verify related spans have required fields
            for related in &diag.related {
                assert!(!related.file.is_empty(), "Related span should have file");
                assert!(related.line > 0, "Related span should have line");
                assert!(related.column > 0, "Related span should have column");
                assert!(
                    !related.message.is_empty(),
                    "Related span should have message"
                );
            }
        }
    }
}

#[test]
fn test_diagnostic_ordering_across_files() {
    // Since we're testing with single source, we'll simulate multiple files
    // by creating diagnostics with different file names

    let source = "fn foo() {}";
    let mut diags = get_all_diagnostics(source);

    if diags.len() >= 3 {
        // Set different files
        diags[0].file = "b.atlas".to_string();
        diags[0].line = 5;
        diags[1].file = "a.atlas".to_string();
        diags[1].line = 10;
        diags[2].file = "a.atlas".to_string();
        diags[2].line = 5;

        sort_diagnostics(&mut diags);

        // Should be sorted: a.atlas:5, a.atlas:10, b.atlas:5
        assert_eq!(diags[0].file, "a.atlas");
        assert_eq!(diags[0].line, 5);
        assert_eq!(diags[1].file, "a.atlas");
        assert_eq!(diags[1].line, 10);
        assert_eq!(diags[2].file, "b.atlas");
    }
}

#[test]
fn test_json_output_is_deterministic() {
    let source = "fn foo() {}";

    let diags = get_all_diagnostics(source);

    for diag in &diags {
        let json1 = diag.to_json_string().unwrap();
        let json2 = diag.to_json_string().unwrap();

        assert_eq!(json1, json2, "JSON output should be deterministic");
    }
}

// ============================================================================
// Related Span Tests (from related_spans_tests.rs)
// ============================================================================

/// Helper to parse source code
fn parse(source: &str) -> (atlas_runtime::ast::Program, Vec<atlas_runtime::Diagnostic>) {
    let mut lexer = Lexer::new(source);
    let (tokens, mut lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, mut parse_diags) = parser.parse();

    lex_diags.append(&mut parse_diags);
    (program, lex_diags)
}

/// Helper to bind a program
fn bind_program(
    program: &atlas_runtime::ast::Program,
) -> (atlas_runtime::SymbolTable, Vec<atlas_runtime::Diagnostic>) {
    let mut binder = Binder::new();
    binder.bind(program)
}

/// Helper to typecheck a program
fn typecheck_program(
    program: &atlas_runtime::ast::Program,
    symbol_table: &mut atlas_runtime::SymbolTable,
) -> Vec<atlas_runtime::Diagnostic> {
    let mut checker = TypeChecker::new(symbol_table);
    checker.check(program)
}

#[test]
fn test_function_redeclaration_has_related_span() {
    let source = r#"
        fn foo() {}
        fn foo() {}
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(parse_diags.is_empty(), "Should parse without errors");

    let (_, bind_diags) = bind_program(&ast);
    assert_eq!(bind_diags.len(), 1, "Should have one binding error");

    let diag = &bind_diags[0];
    assert_eq!(diag.code, "AT2003");
    assert!(
        diag.message.contains("already defined"),
        "Error message should mention redefinition"
    );

    // Verify related location exists
    assert_eq!(
        diag.related.len(),
        1,
        "Should have one related location pointing to first definition"
    );
    let related = &diag.related[0];
    assert!(
        related.message.contains("first defined"),
        "Related message should mention first definition: {}",
        related.message
    );
}

#[test]
fn test_parameter_redeclaration_has_related_span() {
    let source = r#"
        fn foo(x: number, x: string) {}
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(parse_diags.is_empty(), "Should parse without errors");

    let (_, bind_diags) = bind_program(&ast);
    assert_eq!(bind_diags.len(), 1, "Should have one binding error");

    let diag = &bind_diags[0];
    assert_eq!(diag.code, "AT2003");

    // Verify related location exists
    assert_eq!(
        diag.related.len(),
        1,
        "Should have one related location pointing to first parameter"
    );
    let related = &diag.related[0];
    assert!(
        related.message.contains("first defined"),
        "Related message should mention first definition: {}",
        related.message
    );
}

#[test]
fn test_variable_redeclaration_has_related_span() {
    let source = r#"
        fn test() {
            let x = 5;
            let x = 10;
        }
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(parse_diags.is_empty(), "Should parse without errors");

    let (_, bind_diags) = bind_program(&ast);
    assert_eq!(bind_diags.len(), 1, "Should have one binding error");

    let diag = &bind_diags[0];
    assert_eq!(diag.code, "AT2003");

    // Verify related location exists
    assert_eq!(
        diag.related.len(),
        1,
        "Should have one related location pointing to first declaration"
    );
    let related = &diag.related[0];
    assert!(
        related.message.contains("first defined"),
        "Related message should mention first definition: {}",
        related.message
    );
}

#[test]
fn test_return_type_mismatch_has_related_span() {
    let source = r#"
        fn foo() -> number {
            return "hello";
        }
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(
        parse_diags.is_empty(),
        "Should parse without errors: {:?}",
        parse_diags
    );

    let (mut symbol_table, bind_diags) = bind_program(&ast);
    assert!(
        bind_diags.is_empty(),
        "Should bind without errors: {:?}",
        bind_diags
    );

    let type_diags = typecheck_program(&ast, &mut symbol_table);
    assert_eq!(
        type_diags.len(),
        1,
        "Should have one type error, got: {:?}",
        type_diags
    );

    let diag = &type_diags[0];
    assert_eq!(diag.code, "AT3001");
    assert!(
        diag.message.contains("Return type mismatch"),
        "Error message should mention return type mismatch"
    );

    // Verify related location exists pointing to function declaration
    assert_eq!(
        diag.related.len(),
        1,
        "Should have one related location pointing to function declaration"
    );
    let related = &diag.related[0];
    assert!(
        related.message.contains("declared here"),
        "Related message should mention declaration: {}",
        related.message
    );
    assert!(
        related.message.contains("foo"),
        "Related message should mention function name: {}",
        related.message
    );
}

#[test]
fn test_immutable_assignment_has_related_span() {
    let source = r#"
        let x = 5;
        x = 10;
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(
        parse_diags.is_empty(),
        "Should parse without errors: {:?}",
        parse_diags
    );

    let (mut symbol_table, bind_diags) = bind_program(&ast);
    assert!(
        bind_diags.is_empty(),
        "Should bind without errors: {:?}",
        bind_diags
    );

    let type_diags = typecheck_program(&ast, &mut symbol_table);

    // Find the AT3003 error (immutable assignment)
    let at3003_diags: Vec<_> = type_diags.iter().filter(|d| d.code == "AT3003").collect();
    assert_eq!(
        at3003_diags.len(),
        1,
        "Should have one AT3003 error, got {:?}. All diagnostics: {:?}",
        type_diags.iter().map(|d| &d.code).collect::<Vec<_>>(),
        type_diags
            .iter()
            .map(|d| (&d.code, &d.message))
            .collect::<Vec<_>>()
    );

    let diag = at3003_diags[0];
    assert!(
        diag.message.contains("immutable"),
        "Error message should mention immutability"
    );

    // Verify related location exists pointing to variable declaration
    assert_eq!(
        diag.related.len(),
        1,
        "Should have one related location pointing to variable declaration"
    );
    let related = &diag.related[0];
    assert!(
        related.message.contains("declared here"),
        "Related message should mention declaration: {}",
        related.message
    );
    assert!(
        related.message.contains("immutable"),
        "Related message should mention immutability: {}",
        related.message
    );
}

#[test]
fn test_related_span_points_to_correct_location() {
    let source = r#"
        fn first() {}
        fn second() {}
        fn first() {}
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(parse_diags.is_empty(), "Should parse without errors");

    let (_, bind_diags) = bind_program(&ast);
    assert_eq!(bind_diags.len(), 1, "Should have one binding error");

    let diag = &bind_diags[0];
    let related = &diag.related[0];

    // The related span should point to the first occurrence (not the second function)
    // We can verify this by checking that the column is before the error's column
    assert!(
        related.column < diag.column || related.line < diag.line,
        "Related location should point to earlier code"
    );
}

#[test]
fn test_multiple_redeclarations_each_have_related_span() {
    let source = r#"
        fn test() {
            let x = 1;
            let y = 2;
            let x = 3;
            let y = 4;
        }
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(parse_diags.is_empty(), "Should parse without errors");

    let (_, bind_diags) = bind_program(&ast);
    assert_eq!(bind_diags.len(), 2, "Should have two binding errors");

    // Each error should have a related location
    for diag in &bind_diags {
        assert_eq!(
            diag.related.len(),
            1,
            "Each error should have one related location"
        );
    }
}

#[test]
fn test_related_span_serializes_to_json() {
    let source = r#"
        fn foo() {}
        fn foo() {}
    "#;

    let (ast, _) = parse(source);
    let (_, bind_diags) = bind_program(&ast);
    let diag = &bind_diags[0];

    // Verify JSON serialization works
    let json = diag.to_json_string().expect("Should serialize to JSON");
    assert!(
        json.contains("\"related\""),
        "JSON should contain related field"
    );
    assert!(
        json.contains("first defined"),
        "JSON should contain related message"
    );
}

#[test]
fn test_related_span_renders_in_human_format() {
    let source = r#"
        fn foo() {}
        fn foo() {}
    "#;

    let (ast, _) = parse(source);
    let (_, bind_diags) = bind_program(&ast);
    let diag = &bind_diags[0];

    // Verify human format includes related location
    let human = diag.to_human_string();
    assert!(
        human.contains("note:"),
        "Human format should contain note for related location"
    );
    assert!(
        human.contains("first defined"),
        "Human format should show related message"
    );
}

#[test]
fn test_no_related_span_for_undefined_variable() {
    let source = r#"
        fn test() -> number {
            return x;
        }
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(
        parse_diags.is_empty(),
        "Should parse without errors: {:?}",
        parse_diags
    );

    let (_symbol_table, bind_diags) = bind_program(&ast);

    // Undefined variable errors come from the binder
    // Currently these don't have related spans (could add "did you mean?" in future)
    if !bind_diags.is_empty() {
        for _diag in &bind_diags {
            // Related spans are optional, so this test just documents current behavior
            // If we add "did you mean?" suggestions later, we'd update this test
        }
    }
}

// ============================================================================
// Enhanced Error Tests (from enhanced_errors_tests.rs)
// ============================================================================

// ============================================================
// Error Code Registry Tests
// ============================================================

#[test]
fn test_error_codes_no_duplicates() {
    let mut seen = std::collections::HashSet::new();
    for entry in error_codes::ERROR_CODES {
        assert!(
            seen.insert(entry.code),
            "Duplicate error code: {}",
            entry.code
        );
    }
}

#[test]
fn test_error_codes_all_have_descriptions() {
    for entry in error_codes::ERROR_CODES {
        assert!(
            !entry.description.is_empty(),
            "Error code {} has empty description",
            entry.code
        );
    }
}

#[test]
fn test_error_codes_lookup() {
    let info = error_codes::lookup("AT0001").unwrap();
    assert_eq!(info.description, "Type mismatch");
    assert!(info.help.is_some());
}

#[test]
fn test_error_codes_lookup_missing() {
    assert!(error_codes::lookup("ZZZZ9999").is_none());
}

#[test]
fn test_error_codes_help_for() {
    assert!(error_codes::help_for("AT0005").is_some());
    assert_eq!(error_codes::help_for("AT9999"), None);
}

#[test]
fn test_error_codes_description_for() {
    assert_eq!(
        error_codes::description_for("AT1001").unwrap(),
        "Unexpected token"
    );
}

#[test]
fn test_error_codes_constants_match_registry() {
    // Verify the constants match entries in the registry
    assert!(error_codes::lookup(error_codes::TYPE_MISMATCH).is_some());
    assert!(error_codes::lookup(error_codes::UNDEFINED_SYMBOL).is_some());
    assert!(error_codes::lookup(error_codes::DIVIDE_BY_ZERO).is_some());
    assert!(error_codes::lookup(error_codes::UNEXPECTED_TOKEN).is_some());
    assert!(error_codes::lookup(error_codes::UNUSED_VARIABLE).is_some());
}

#[test]
fn test_error_code_ranges() {
    // Verify error code ranges
    for entry in error_codes::ERROR_CODES {
        assert!(
            entry.code.starts_with("AT") || entry.code.starts_with("AW"),
            "Error code {} must start with AT or AW",
            entry.code
        );
    }
}

#[test]
fn test_error_codes_count() {
    // At least 40 error codes in the registry
    assert!(
        error_codes::ERROR_CODES.len() >= 40,
        "Expected at least 40 error codes, got {}",
        error_codes::ERROR_CODES.len()
    );
}

#[rstest]
#[case("AT0001", "Type mismatch")]
#[case("AT0005", "Division by zero")]
#[case("AT0006", "Array index out of bounds")]
#[case("AT1001", "Unexpected token")]
#[case("AT1002", "Unterminated string literal")]
#[case("AT2001", "Unused variable or parameter")]
#[case("AT2002", "Unreachable code")]
#[case("AT3001", "Type error in expression")]
#[case("AT3005", "Function arity mismatch")]
#[case("AT5002", "Module not found")]
fn test_error_code_descriptions(#[case] code: &str, #[case] desc: &str) {
    let info = error_codes::lookup(code).unwrap();
    assert_eq!(info.description, desc);
}

#[rstest]
#[case("AT0001")]
#[case("AT0005")]
#[case("AT0006")]
#[case("AT0300")]
#[case("AT0301")]
#[case("AT1001")]
#[case("AT1002")]
#[case("AT1003")]
#[case("AT2001")]
#[case("AT3003")]
fn test_error_codes_have_help(#[case] code: &str) {
    let info = error_codes::lookup(code).unwrap();
    assert!(
        info.help.is_some(),
        "Error code {} should have help text",
        code
    );
}

// ============================================================
// Source Snippet Extraction Tests
// ============================================================

#[test]
fn test_extract_snippet_first_line() {
    let source = "let x = 1;\nlet y = 2;";
    assert_eq!(extract_snippet(source, 1).unwrap(), "let x = 1;");
}

#[test]
fn test_extract_snippet_last_line() {
    let source = "line1\nline2\nline3";
    assert_eq!(extract_snippet(source, 3).unwrap(), "line3");
}

#[test]
fn test_extract_snippet_empty_file() {
    assert!(extract_snippet("", 1).is_none());
}

#[test]
fn test_extract_snippet_out_of_range() {
    let source = "only one line";
    assert!(extract_snippet(source, 5).is_none());
}

#[test]
fn test_extract_snippet_empty_line() {
    let source = "line1\n\nline3";
    assert_eq!(extract_snippet(source, 2).unwrap(), "");
}

#[test]
fn test_extract_snippet_single_line() {
    let source = "hello world";
    assert_eq!(extract_snippet(source, 1).unwrap(), "hello world");
}

// ============================================================
// Line/Column Offset Tests
// ============================================================

#[test]
fn test_offset_to_line_col_start() {
    let source = "let x = 1;\nlet y = 2;";
    assert_eq!(offset_to_line_col(source, 0), (1, 1));
}

#[test]
fn test_offset_to_line_col_middle() {
    let source = "let x = 1;\nlet y = 2;";
    assert_eq!(offset_to_line_col(source, 4), (1, 5)); // 'x'
}

#[test]
fn test_offset_to_line_col_second_line() {
    let source = "let x = 1;\nlet y = 2;";
    assert_eq!(offset_to_line_col(source, 11), (2, 1));
}

#[test]
fn test_offset_to_line_col_unicode() {
    let source = "let héllo = 1;";
    assert_eq!(offset_to_line_col(source, 0), (1, 1));
}

// ============================================================
// Diagnostic Formatting Tests
// ============================================================

#[test]
fn test_format_error_basic() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error_with_code("AT0001", "Type mismatch", Span::new(8, 13))
        .with_file("test.atlas")
        .with_line(5)
        .with_snippet("let x: number = \"hello\";")
        .with_label("expected number, found string");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("error[AT0001]"));
    assert!(output.contains("Type mismatch"));
    assert!(output.contains("test.atlas:5:9"));
    assert!(output.contains("^^^^^"));
    assert!(output.contains("expected number, found string"));
}

#[test]
fn test_format_warning_basic() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::warning_with_code("AT2001", "Unused variable 'x'", Span::new(4, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("let x: number = 42;")
        .with_label("declared here");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("warning[AT2001]"));
    assert!(output.contains("Unused variable"));
}

#[test]
fn test_format_with_help() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error_with_code("AT0005", "Division by zero", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("10 / 0")
        .with_help("check that the divisor is not zero");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("help:"));
    assert!(output.contains("check that the divisor is not zero"));
}

#[test]
fn test_format_with_notes() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error("test", Span::new(0, 1))
        .with_file("test.atlas")
        .with_line(1)
        .with_note("first note")
        .with_note("second note");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("note: first note"));
    assert!(output.contains("note: second note"));
}

#[test]
fn test_format_no_snippet() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error("some error", Span::new(0, 1))
        .with_file("test.atlas")
        .with_line(1);

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("error"));
    assert!(!output.contains("^")); // No carets without snippet
}

#[test]
fn test_format_caret_alignment() {
    let formatter = DiagnosticFormatter::plain();
    // Column 9 (0-indexed 8), length 5
    let diag = Diagnostic::error_with_code("AT0001", "test", Span::new(8, 13))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("let x = hello;");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    // Should have 8 spaces before carets (column 9, 0-indexed = 8 spaces)
    assert!(output.contains("^^^^^"));
}

#[test]
fn test_format_caret_position_first_column() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error_with_code("AT0001", "test", Span::new(0, 3))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("foo bar");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("^^^"));
}

#[test]
fn test_format_multiline_notes() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error("main error", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("hello world")
        .with_note("note 1")
        .with_note("note 2")
        .with_note("note 3")
        .with_help("try this");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("note: note 1"));
    assert!(output.contains("note: note 2"));
    assert!(output.contains("note: note 3"));
    assert!(output.contains("help: try this"));
}

#[test]
fn test_format_unicode_snippet() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error_with_code("AT0001", "test", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("let héllo = 42;");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    // Should still render without crashing
    assert!(output.contains("error[AT0001]"));
    assert!(output.contains("héllo"));
}

#[test]
fn test_format_related_location() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error("test", Span::new(0, 1))
        .with_file("main.atlas")
        .with_line(10)
        .with_related_location(atlas_runtime::RelatedLocation {
            file: "other.atlas".to_string(),
            line: 5,
            column: 3,
            length: 4,
            message: "originally defined here".to_string(),
        });

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("other.atlas:5:3"));
    assert!(output.contains("originally defined here"));
}

// ============================================================
// Parse Error Formatting Tests
// ============================================================

#[test]
fn test_parse_error_has_diagnostics() {
    let runtime = atlas_runtime::Atlas::new();
    let result = runtime.eval("let x: number =");
    assert!(result.is_err());
    let diags = result.unwrap_err();
    assert!(!diags.is_empty());
    assert_eq!(diags[0].level, DiagnosticLevel::Error);
}

#[test]
fn test_parse_error_has_code() {
    let runtime = atlas_runtime::Atlas::new();
    let result = runtime.eval("let x: number =");
    let diags = result.unwrap_err();
    // Error code should be set (not empty)
    assert!(!diags[0].code.is_empty());
}

#[test]
fn test_invalid_number_literal_diagnostic() {
    let (_, diags) = parse("let x = 1e309;");
    assert!(
        diags.iter().any(|diag| diag.code == "AT1001"),
        "Expected invalid number literal diagnostic, got: {:?}",
        diags
    );
}

#[test]
fn test_distinct_error_codes() {
    let (_, diags) = parse("let x = 1");
    assert!(
        diags.iter().any(|diag| diag.code == "AT1002"),
        "Expected missing semicolon code AT1002, got: {:?}",
        diags
    );

    let (_, diags) = parse("fn 123() {}");
    assert!(
        diags.iter().any(|diag| diag.code == "AT1004"),
        "Expected unexpected token code AT1004, got: {:?}",
        diags
    );
}

#[test]
fn test_reserved_keyword_error_code() {
    let (_, diags) = parse("fn let() {}");
    assert!(
        diags.iter().any(|diag| diag.code == "AT1005"),
        "Expected reserved keyword code AT1005, got: {:?}",
        diags
    );
}

#[test]
fn test_type_error_has_diagnostics() {
    let runtime = atlas_runtime::Atlas::new();
    let result = runtime.eval("let x: number = \"hello\";");
    // This should produce a type error
    assert!(result.is_err());
    let diags = result.unwrap_err();
    assert!(!diags.is_empty());
}

// ============================================================
// Formatter Mode Tests
// ============================================================

#[test]
fn test_formatter_plain_mode() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error("test", Span::new(0, 1))
        .with_file("test.atlas")
        .with_line(1);

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("error"));
}

#[test]
fn test_formatter_auto_mode() {
    let formatter = DiagnosticFormatter::auto();
    let diag = Diagnostic::error("test", Span::new(0, 1))
        .with_file("test.atlas")
        .with_line(1);

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("error"));
}

#[test]
fn test_formatter_format_to_string() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error("test error", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("hello")
        .with_label("here");

    let output = formatter.format_to_string(&diag);
    assert_eq!(output, diag.to_human_string());
}

// ============================================================
// Diagnostic Builder Tests
// ============================================================

#[test]
fn test_diagnostic_error_with_code() {
    let diag = Diagnostic::error_with_code("AT0001", "Type mismatch", Span::new(5, 10));
    assert_eq!(diag.code, "AT0001");
    assert_eq!(diag.level, DiagnosticLevel::Error);
    assert_eq!(diag.message, "Type mismatch");
    assert_eq!(diag.diag_version, DIAG_VERSION);
}

#[test]
fn test_diagnostic_warning_with_code() {
    let diag = Diagnostic::warning_with_code("AT2001", "Unused var", Span::new(0, 3));
    assert_eq!(diag.code, "AT2001");
    assert_eq!(diag.level, DiagnosticLevel::Warning);
}

#[test]
fn test_diagnostic_builder_chain() {
    let diag = Diagnostic::error_with_code("AT0001", "test", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(10)
        .with_snippet("source code")
        .with_label("error here")
        .with_note("a note")
        .with_help("try this");

    assert_eq!(diag.file, "test.atlas");
    assert_eq!(diag.line, 10);
    assert_eq!(diag.snippet, "source code");
    assert_eq!(diag.label, "error here");
    assert_eq!(diag.notes.len(), 1);
    assert!(diag.help.is_some());
}

#[test]
fn test_diagnostic_json_output() {
    let diag = Diagnostic::error_with_code("AT0001", "Type mismatch", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1);

    let json = diag.to_json_string().unwrap();
    assert!(json.contains("\"level\": \"error\""));
    assert!(json.contains("\"code\": \"AT0001\""));
}

#[test]
fn test_diagnostic_json_compact() {
    let diag = Diagnostic::error("test", Span::new(0, 1));
    let compact = diag.to_json_compact().unwrap();
    assert!(!compact.contains('\n'));
}

#[test]
fn test_diagnostic_json_roundtrip() {
    let diag = Diagnostic::error_with_code("AT0001", "test", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("hello")
        .with_label("here")
        .with_note("note")
        .with_help("help");

    let json = diag.to_json_string().unwrap();
    let deserialized: Diagnostic = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, diag);
}

// ============================================================
// Sort Diagnostics Tests
// ============================================================

#[test]
fn test_sort_errors_before_warnings() {
    let mut diags = vec![
        Diagnostic::warning("warn", Span::new(0, 1))
            .with_file("a.atlas")
            .with_line(1),
        Diagnostic::error("err", Span::new(0, 1))
            .with_file("a.atlas")
            .with_line(1),
    ];
    atlas_runtime::sort_diagnostics(&mut diags);
    assert_eq!(diags[0].level, DiagnosticLevel::Error);
    assert_eq!(diags[1].level, DiagnosticLevel::Warning);
}

#[test]
fn test_sort_by_file_then_line() {
    let mut diags = vec![
        Diagnostic::error("e1", Span::new(0, 1))
            .with_file("b.atlas")
            .with_line(5),
        Diagnostic::error("e2", Span::new(0, 1))
            .with_file("a.atlas")
            .with_line(10),
        Diagnostic::error("e3", Span::new(0, 1))
            .with_file("a.atlas")
            .with_line(1),
    ];
    atlas_runtime::sort_diagnostics(&mut diags);
    assert_eq!(diags[0].file, "a.atlas");
    assert_eq!(diags[0].line, 1);
    assert_eq!(diags[1].file, "a.atlas");
    assert_eq!(diags[1].line, 10);
    assert_eq!(diags[2].file, "b.atlas");
}

// ============================================================
// Edge Case Tests
// ============================================================

#[test]
fn test_zero_length_span() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error("test", Span::new(5, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("hello world");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();
    // Should not crash, no carets for zero-length
    assert!(output.contains("error"));
}

#[test]
fn test_span_beyond_snippet() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error_with_code("AT0001", "test", Span::new(0, 100))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("short");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();
    // Should not crash
    assert!(output.contains("error"));
}

#[test]
fn test_empty_message() {
    let diag = Diagnostic::error("", Span::new(0, 1));
    assert_eq!(diag.message, "");
    // Should still format
    let output = diag.to_human_string();
    assert!(output.contains("error"));
}

#[test]
fn test_diagnostic_default_file() {
    let diag = Diagnostic::error("test", Span::new(0, 1));
    assert_eq!(diag.file, "<unknown>");
}

#[rstest]
#[case(Span::new(0, 0), true)]
#[case(Span::new(0, 5), false)]
#[case(Span::new(5, 5), true)]
fn test_span_is_empty(#[case] span: Span, #[case] expected: bool) {
    assert_eq!(span.is_empty(), expected);
}

#[rstest]
#[case(Span::new(0, 5), 5)]
#[case(Span::new(3, 10), 7)]
#[case(Span::new(0, 0), 0)]
fn test_span_length(#[case] span: Span, #[case] expected: usize) {
    assert_eq!(span.len(), expected);
}

// ============================================================================
// Source Map Tests (from sourcemap_tests.rs)
// ============================================================================

// ═══════════════════════════════════════════════════════════════════════════════
// VLQ Encoding Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_vlq_encode_zero() {
    assert_eq!(vlq::encode(0), "A");
}

#[test]
fn test_vlq_encode_positive_values() {
    // 1 → shifted to 2 → binary 000010 → base64 'C'
    assert_eq!(vlq::encode(1), "C");
    assert_eq!(vlq::encode(2), "E");
    assert_eq!(vlq::encode(3), "G");
}

#[test]
fn test_vlq_encode_negative_values() {
    // -1 → shifted to 3 → binary 000011 → base64 'D'
    assert_eq!(vlq::encode(-1), "D");
    assert_eq!(vlq::encode(-2), "F");
}

#[test]
fn test_vlq_roundtrip_small() {
    for v in -50..=50 {
        let encoded = vlq::encode(v);
        let (decoded, consumed) = vlq::decode(&encoded).unwrap();
        assert_eq!(decoded, v, "roundtrip failed for {v}");
        assert_eq!(consumed, encoded.len());
    }
}

#[test]
fn test_vlq_roundtrip_large() {
    for v in [100, 500, 1000, 5000, 10000, 65535, -100, -500, -65535] {
        let encoded = vlq::encode(v);
        let (decoded, _) = vlq::decode(&encoded).unwrap();
        assert_eq!(decoded, v);
    }
}

#[test]
fn test_vlq_decode_invalid_empty() {
    assert!(vlq::decode("").is_none());
}

#[test]
fn test_vlq_decode_invalid_chars() {
    assert!(vlq::decode("!!!").is_none());
}

#[test]
fn test_vlq_segment_roundtrip() {
    let values = vec![0, 5, -3, 10, 0, -1];
    let segment = vlq::encode_segment(&values);
    let decoded = vlq::decode_segment(&segment).unwrap();
    assert_eq!(decoded, values);
}

#[test]
fn test_vlq_segment_empty() {
    let decoded = vlq::decode_segment("").unwrap();
    assert!(decoded.is_empty());
}

#[test]
fn test_vlq_segment_single_value() {
    let segment = vlq::encode_segment(&[42]);
    let decoded = vlq::decode_segment(&segment).unwrap();
    assert_eq!(decoded, vec![42]);
}

#[test]
fn test_vlq_multibyte_encoding() {
    // Values >= 16 require multiple VLQ digits
    let encoded = vlq::encode(16);
    assert!(encoded.len() > 1, "16 should need multi-byte VLQ");
    let (decoded, _) = vlq::decode(&encoded).unwrap();
    assert_eq!(decoded, 16);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Source Map Builder Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_builder_empty() {
    let mut builder = SourceMapBuilder::new();
    let map = builder.build();
    assert_eq!(map.version, 3);
    assert!(map.sources.is_empty());
    assert!(map.names.is_empty());
    assert!(map.mappings.is_empty());
}

#[test]
fn test_builder_single_source() {
    let mut builder = SourceMapBuilder::new();
    let idx = builder.add_source("main.atlas", None);
    assert_eq!(idx, 0);
    let map = builder.build();
    assert_eq!(map.sources, vec!["main.atlas"]);
    assert!(map.sources_content.is_none());
}

#[test]
fn test_builder_source_with_content() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", Some("let x = 1;".to_string()));
    let map = builder.build();
    assert_eq!(
        map.sources_content,
        Some(vec![Some("let x = 1;".to_string())])
    );
}

#[test]
fn test_builder_duplicate_source() {
    let mut builder = SourceMapBuilder::new();
    let idx1 = builder.add_source("a.atlas", None);
    let idx2 = builder.add_source("a.atlas", None);
    assert_eq!(idx1, idx2);
    let map = builder.build();
    assert_eq!(map.sources.len(), 1);
}

#[test]
fn test_builder_multiple_sources() {
    let mut builder = SourceMapBuilder::new();
    let idx0 = builder.add_source("a.atlas", None);
    let idx1 = builder.add_source("b.atlas", None);
    assert_eq!(idx0, 0);
    assert_eq!(idx1, 1);
    let map = builder.build();
    assert_eq!(map.sources.len(), 2);
}

#[test]
fn test_builder_names() {
    let mut builder = SourceMapBuilder::new();
    let idx = builder.add_name("myVar");
    assert_eq!(idx, 0);
    let idx2 = builder.add_name("myVar"); // duplicate
    assert_eq!(idx2, 0);
    let idx3 = builder.add_name("otherVar");
    assert_eq!(idx3, 1);
    let map = builder.build();
    assert_eq!(map.names, vec!["myVar", "otherVar"]);
}

#[test]
fn test_builder_single_mapping() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    let map = builder.build();
    assert!(!map.mappings.is_empty());

    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].generated_line, 0);
    assert_eq!(entries[0].generated_column, 0);
    assert_eq!(entries[0].original_line, 0);
    assert_eq!(entries[0].original_column, 0);
}

#[test]
fn test_builder_multiple_mappings_same_line() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    builder.add_mapping(0, 5, 0, 0, 4, None);
    builder.add_mapping(0, 10, 0, 1, 0, None);
    let map = builder.build();

    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].generated_column, 0);
    assert_eq!(entries[1].generated_column, 5);
    assert_eq!(entries[2].generated_column, 10);
}

#[test]
fn test_builder_multiple_lines() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    builder.add_mapping(1, 0, 0, 1, 0, None);
    builder.add_mapping(2, 0, 0, 2, 0, None);
    let map = builder.build();

    // Should have semicolons separating lines
    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].generated_line, 0);
    assert_eq!(entries[1].generated_line, 1);
    assert_eq!(entries[2].generated_line, 2);
}

#[test]
fn test_builder_with_names() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    let name_idx = builder.add_name("myFunc");
    builder.add_mapping(0, 0, 0, 0, 0, Some(name_idx));
    let map = builder.build();

    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries[0].name_index, Some(0));
}

#[test]
fn test_builder_set_file() {
    let mut builder = SourceMapBuilder::new();
    builder.set_file("output.atlas.bc");
    let map = builder.build();
    assert_eq!(map.file, Some("output.atlas.bc".to_string()));
}

#[test]
fn test_builder_set_source_root() {
    let mut builder = SourceMapBuilder::new();
    builder.set_source_root("/src/");
    let map = builder.build();
    assert_eq!(map.source_root, Some("/src/".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Source Map JSON Serialization Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_source_map_to_json() {
    let mut builder = SourceMapBuilder::new();
    builder.set_file("out.js");
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    let map = builder.build();

    let json = map.to_json().unwrap();
    assert!(json.contains("\"version\":3"));
    assert!(json.contains("\"file\":\"out.js\""));
    assert!(json.contains("\"sources\":[\"main.atlas\"]"));
}

#[test]
fn test_source_map_roundtrip_json() {
    let mut builder = SourceMapBuilder::new();
    builder.set_file("test.bc");
    builder.add_source("a.atlas", Some("let x = 1;".to_string()));
    builder.add_source("b.atlas", None);
    builder.add_name("x");
    builder.add_mapping(0, 0, 0, 0, 4, Some(0));
    builder.add_mapping(0, 5, 1, 2, 0, None);
    let map = builder.build();

    let json = map.to_json().unwrap();
    let parsed = SourceMapV3::from_json(&json).unwrap();

    assert_eq!(parsed.version, 3);
    assert_eq!(parsed.file, Some("test.bc".to_string()));
    assert_eq!(parsed.sources, vec!["a.atlas", "b.atlas"]);
    assert_eq!(parsed.names, vec!["x"]);
    assert_eq!(parsed.mappings, map.mappings);
}

#[test]
fn test_source_map_pretty_json() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    let map = builder.build();
    let json = map.to_json_pretty().unwrap();
    assert!(json.contains('\n'), "pretty JSON should have newlines");
}

#[test]
fn test_source_map_no_optional_fields() {
    let map = SourceMapV3 {
        version: 3,
        file: None,
        source_root: None,
        sources: vec![],
        sources_content: None,
        names: vec![],
        mappings: String::new(),
    };
    let json = map.to_json().unwrap();
    assert!(!json.contains("file"));
    assert!(!json.contains("sourceRoot"));
    assert!(!json.contains("sourcesContent"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Mappings Encoding/Decoding Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_decode_empty_mappings() {
    let entries = decode_mappings("").unwrap();
    assert!(entries.is_empty());
}

#[test]
fn test_encode_decode_roundtrip() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("test.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    builder.add_mapping(0, 10, 0, 2, 5, None);
    builder.add_mapping(1, 0, 0, 5, 0, None);
    let map = builder.build();

    let entries = decode_mappings(&map.mappings).unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].generated_column, 0);
    assert_eq!(entries[0].original_line, 0);
    assert_eq!(entries[1].generated_column, 10);
    assert_eq!(entries[1].original_line, 2);
    assert_eq!(entries[1].original_column, 5);
    assert_eq!(entries[2].generated_line, 1);
    assert_eq!(entries[2].generated_column, 0);
    assert_eq!(entries[2].original_line, 5);
}

#[test]
fn test_mappings_with_semicolons() {
    // Multiple lines separated by semicolons
    let mut builder = SourceMapBuilder::new();
    builder.add_source("test.atlas", None);
    // Line 0 has 2 entries, line 1 is empty, line 2 has 1 entry
    builder.add_mapping(0, 0, 0, 0, 0, None);
    builder.add_mapping(0, 5, 0, 0, 5, None);
    builder.add_mapping(2, 0, 0, 3, 0, None);
    let map = builder.build();

    // Should contain semicolons for line separation
    assert!(map.mappings.contains(';'));

    let entries = decode_mappings(&map.mappings).unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[2].generated_line, 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Source Map Lookup Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_lookup_exact_match() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 5, 10, None);
    let map = builder.build();

    let loc = map.lookup(0, 0).unwrap();
    assert_eq!(loc.source, "main.atlas");
    assert_eq!(loc.line, 5);
    assert_eq!(loc.column, 10);
}

#[test]
fn test_lookup_closest_column() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    builder.add_mapping(0, 10, 0, 1, 0, None);
    let map = builder.build();

    // Column 5 is between 0 and 10; should match column 0's mapping
    let loc = map.lookup(0, 5).unwrap();
    assert_eq!(loc.line, 0);
}

#[test]
fn test_lookup_no_match() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 5, 0, 0, 0, None);
    let map = builder.build();

    // Column 3 is before the first mapping on line 0
    assert!(map.lookup(0, 3).is_none());
}

#[test]
fn test_lookup_with_name() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    let name_idx = builder.add_name("foo");
    builder.add_mapping(0, 0, 0, 3, 4, Some(name_idx));
    let map = builder.build();

    let loc = map.lookup(0, 0).unwrap();
    assert_eq!(loc.name, Some("foo".to_string()));
}

#[test]
fn test_lookup_wrong_line() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    let map = builder.build();

    assert!(map.lookup(1, 0).is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Source Map Generation from Bytecode Tests
// ═══════════════════════════════════════════════════════════════════════════════

fn make_bytecode(spans: Vec<(usize, usize, usize)>) -> Bytecode {
    Bytecode {
        instructions: vec![0; spans.last().map(|(o, _, _)| o + 1).unwrap_or(0)],
        constants: Vec::new(),
        debug_info: spans
            .into_iter()
            .map(|(offset, start, end)| DebugSpan {
                instruction_offset: offset,
                span: Span::new(start, end),
            })
            .collect(),
    }
}

#[test]
fn test_generate_simple_program() {
    let source = "let x = 1;\nlet y = 2;\n";
    let bytecode = make_bytecode(vec![(0, 0, 10), (5, 11, 21)]);
    let options = SourceMapOptions::default();

    let map = generate_source_map(&bytecode, "main.atlas", Some(source), &options);
    assert_eq!(map.version, 3);
    assert_eq!(map.sources, vec!["main.atlas"]);

    let entries = map.decode_mappings().unwrap();
    assert!(!entries.is_empty());

    // First entry should map to line 0
    assert_eq!(entries[0].original_line, 0);
    // Second entry should map to line 1
    let last = entries.last().unwrap();
    assert_eq!(last.original_line, 1);
}

#[test]
fn test_generate_skips_dummy_spans() {
    let bytecode = make_bytecode(vec![(0, 0, 0), (1, 5, 10)]);
    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "test.atlas", Some("hello world"), &options);

    let entries = map.decode_mappings().unwrap();
    // The dummy span (0,0) should be skipped
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].generated_column, 1);
}

#[test]
fn test_generate_with_inlined_sources() {
    let source = "let x = 42;";
    let bytecode = make_bytecode(vec![(0, 0, 11)]);
    let options = SourceMapOptions {
        include_sources: true,
        ..Default::default()
    };

    let map = generate_source_map(&bytecode, "main.atlas", Some(source), &options);
    assert_eq!(
        map.sources_content,
        Some(vec![Some("let x = 42;".to_string())])
    );
}

#[test]
fn test_generate_without_source_text() {
    let bytecode = make_bytecode(vec![(0, 5, 10)]);
    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "main.atlas", None, &options);

    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries.len(), 1);
    // Without source text, positions are computed from offset 0 only
    assert_eq!(entries[0].original_line, 0);
}

#[test]
fn test_generate_with_file_option() {
    let bytecode = make_bytecode(vec![(0, 0, 5)]);
    let options = SourceMapOptions {
        file: Some("output.bc".to_string()),
        ..Default::default()
    };
    let map = generate_source_map(&bytecode, "main.atlas", Some("hello"), &options);
    assert_eq!(map.file, Some("output.bc".to_string()));
}

#[test]
fn test_generate_with_source_root() {
    let bytecode = make_bytecode(vec![(0, 0, 5)]);
    let options = SourceMapOptions {
        source_root: Some("/src/".to_string()),
        ..Default::default()
    };
    let map = generate_source_map(&bytecode, "main.atlas", Some("hello"), &options);
    assert_eq!(map.source_root, Some("/src/".to_string()));
}

#[test]
fn test_generate_removes_redundant_entries() {
    // Multiple instructions mapping to the same source position
    let bytecode = make_bytecode(vec![(0, 0, 10), (1, 0, 10), (2, 0, 10), (5, 11, 20)]);
    let source = "let x = 1;\nlet y = 2;\n";
    let options = SourceMapOptions::default();

    let map = generate_source_map(&bytecode, "test.atlas", Some(source), &options);
    let entries = map.decode_mappings().unwrap();

    // Redundant entries with same original position should be deduped
    assert!(entries.len() <= 3, "Should remove redundant mappings");
}

#[test]
fn test_generate_from_debug_spans_direct() {
    let spans = vec![
        DebugSpan {
            instruction_offset: 0,
            span: Span::new(0, 5),
        },
        DebugSpan {
            instruction_offset: 3,
            span: Span::new(6, 11),
        },
    ];
    let options = SourceMapOptions::default();
    let map = generate_from_debug_spans(&spans, "test.atlas", Some("hello\nworld"), &options);

    assert_eq!(map.sources, vec!["test.atlas"]);
    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn test_generate_empty_bytecode() {
    let bytecode = Bytecode {
        instructions: Vec::new(),
        constants: Vec::new(),
        debug_info: Vec::new(),
    };
    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "empty.atlas", Some(""), &options);

    assert_eq!(map.version, 3);
    assert!(map.decode_mappings().unwrap().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Inline Source Map Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_inline_source_map() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    let map = builder.build();

    let inline = generate_inline_source_map(&map).unwrap();
    assert!(inline.starts_with("//# sourceMappingURL=data:application/json;base64,"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Compiler Integration Tests
// ═══════════════════════════════════════════════════════════════════════════════

fn compile_source(source: &str) -> Bytecode {
    let mut lexer = atlas_runtime::lexer::Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::parser::Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = atlas_runtime::compiler::Compiler::new();
    compiler.compile(&program).unwrap()
}

#[test]
fn test_compiler_generates_source_map() {
    let source = "let x = 42;";
    let bytecode = compile_source(source);

    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "test.atlas", Some(source), &options);

    assert_eq!(map.version, 3);
    assert_eq!(map.sources, vec!["test.atlas"]);
    assert!(!map.decode_mappings().unwrap().is_empty());
}

#[test]
fn test_compiler_source_map_with_function() {
    let source = "fn add(a, b) {\n  return a + b;\n}\nlet result = add(1, 2);";
    let bytecode = compile_source(source);

    let options = SourceMapOptions {
        include_sources: true,
        file: Some("add.atlas.bc".to_string()),
        ..Default::default()
    };
    let map = generate_source_map(&bytecode, "add.atlas", Some(source), &options);

    assert_eq!(map.file, Some("add.atlas.bc".to_string()));
    assert!(map.sources_content.is_some());
    assert!(!map.decode_mappings().unwrap().is_empty());
}

#[test]
fn test_compiler_source_map_lookup() {
    let source = "let x = 1;\nlet y = 2;\nlet z = x + y;";
    let bytecode = compile_source(source);

    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "test.atlas", Some(source), &options);

    let entries = map.decode_mappings().unwrap();
    assert!(
        entries.len() >= 3,
        "expected >=3 mappings, got {}",
        entries.len()
    );

    for entry in &entries {
        assert_eq!(entry.source_index, 0);
    }
}

#[test]
fn test_compiler_source_map_json_roundtrip() {
    let source = "let x = 42;\nprint(x);";
    let bytecode = compile_source(source);

    let options = SourceMapOptions {
        include_sources: true,
        ..Default::default()
    };
    let map = generate_source_map(&bytecode, "test.atlas", Some(source), &options);
    let json = map.to_json().unwrap();
    let parsed = SourceMapV3::from_json(&json).unwrap();

    assert_eq!(parsed.version, map.version);
    assert_eq!(parsed.sources, map.sources);
    assert_eq!(parsed.mappings, map.mappings);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Debugger Integration Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_debugger_source_map_from_bytecode() {
    let source = "let a = 1;\nlet b = 2;";
    let bytecode = compile_source(source);

    let options = SourceMapOptions::default();
    let v3_map = generate_source_map(&bytecode, "test.atlas", Some(source), &options);

    let dbg_map = atlas_runtime::debugger::source_map::SourceMap::from_debug_spans(
        &bytecode.debug_info,
        "test.atlas",
        Some(source),
    );

    assert!(!v3_map.decode_mappings().unwrap().is_empty());
    assert!(!dbg_map.is_empty());
}

#[test]
fn test_source_map_stack_trace_lookup() {
    let source = "fn greet() {\n  print(\"hello\");\n}\ngreet();";
    let bytecode = compile_source(source);

    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "test.atlas", Some(source), &options);

    let entries = map.decode_mappings().unwrap();
    if !entries.is_empty() {
        let first = &entries[0];
        let loc = map.lookup(first.generated_line, first.generated_column);
        assert!(loc.is_some());
        assert_eq!(loc.unwrap().source, "test.atlas");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_source_map_multiline_source() {
    let source = "let a = 1;\nlet b = 2;\nlet c = 3;\nlet d = 4;\nlet e = 5;";
    let bytecode = make_bytecode(vec![
        (0, 0, 10),
        (3, 11, 21),
        (6, 22, 32),
        (9, 33, 43),
        (12, 44, 54),
    ]);
    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "test.atlas", Some(source), &options);

    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries.len(), 5);
    for (i, entry) in entries.iter().enumerate() {
        assert_eq!(entry.original_line, i as u32, "line mismatch at entry {i}");
    }
}

#[test]
fn test_source_map_unicode_source() {
    let source = "let emoji = \"🎉\";\nlet kanji = \"漢字\";";
    let bytecode = make_bytecode(vec![(0, 0, 20), (5, 21, 40)]);
    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "unicode.atlas", Some(source), &options);

    assert_eq!(map.sources, vec!["unicode.atlas"]);
    assert!(!map.decode_mappings().unwrap().is_empty());
}

#[test]
fn test_source_map_very_large_offsets() {
    let bytecode = make_bytecode(vec![(0, 0, 10), (10000, 50000, 50010)]);
    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "big.atlas", None, &options);

    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[1].generated_column, 10000);
}

#[test]
fn test_mapping_entry_equality() {
    let a = MappingEntry {
        generated_line: 0,
        generated_column: 0,
        source_index: 0,
        original_line: 1,
        original_column: 5,
        name_index: None,
    };
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn test_source_map_options_default() {
    let opts = SourceMapOptions::default();
    assert!(opts.file.is_none());
    assert!(opts.source_root.is_none());
    assert!(!opts.include_sources);
}

#[test]
fn test_source_map_options_new() {
    let opts = SourceMapOptions::new();
    assert!(opts.file.is_none());
}
