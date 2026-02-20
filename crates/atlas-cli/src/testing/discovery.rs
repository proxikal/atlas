//! Test discovery - find test functions in Atlas source files

use atlas_runtime::ast::Item;
use atlas_runtime::{DiagnosticLevel, Lexer, Parser};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A discovered test function
#[derive(Debug, Clone)]
pub struct TestFunction {
    /// Name of the test function (e.g., "test_addition")
    pub name: String,
    /// File containing the test
    pub file: PathBuf,
    /// Line number where the test is defined
    pub line: usize,
}

/// A suite of discovered tests
#[derive(Debug, Default)]
pub struct TestSuite {
    /// All discovered test functions
    pub tests: Vec<TestFunction>,
    /// Files that had parse errors
    pub parse_errors: Vec<(PathBuf, String)>,
}

impl TestSuite {
    /// Discover all test functions in a directory tree
    pub fn discover(root: &Path) -> Self {
        let mut suite = TestSuite::default();

        // Walk directory tree finding .at and .atlas files
        for entry in WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();

            // Check for Atlas file extensions
            if let Some(ext) = path.extension() {
                if ext == OsStr::new("at") || ext == OsStr::new("atlas") {
                    match discover_tests_in_file(path) {
                        Ok(tests) => suite.tests.extend(tests),
                        Err(e) => suite.parse_errors.push((path.to_path_buf(), e)),
                    }
                }
            }
        }

        // Sort tests by file, then by line number for deterministic order
        suite
            .tests
            .sort_by(|a, b| a.file.cmp(&b.file).then_with(|| a.line.cmp(&b.line)));

        suite
    }

    /// Filter tests by name pattern
    pub fn filter(&self, pattern: &str) -> Self {
        let filtered = self
            .tests
            .iter()
            .filter(|t| t.name.contains(pattern))
            .cloned()
            .collect();

        TestSuite {
            tests: filtered,
            parse_errors: Vec::new(),
        }
    }

    /// Check if suite has any tests
    pub fn is_empty(&self) -> bool {
        self.tests.is_empty()
    }

    /// Get count of tests
    pub fn len(&self) -> usize {
        self.tests.len()
    }
}

/// Discover test functions in a single file
fn discover_tests_in_file(path: &Path) -> Result<Vec<TestFunction>, String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();

    // Check for lexer errors
    if lex_diags.iter().any(|d| d.level == DiagnosticLevel::Error) {
        let errors: Vec<_> = lex_diags
            .iter()
            .filter(|d| d.level == DiagnosticLevel::Error)
            .map(|d| d.message.clone())
            .collect();
        return Err(format!("Lexer errors: {}", errors.join("; ")));
    }

    let mut parser = Parser::new(tokens);
    let (ast, parse_diags) = parser.parse();

    // Check for parser errors
    if parse_diags
        .iter()
        .any(|d| d.level == DiagnosticLevel::Error)
    {
        let errors: Vec<_> = parse_diags
            .iter()
            .filter(|d| d.level == DiagnosticLevel::Error)
            .map(|d| d.message.clone())
            .collect();
        return Err(format!("Parse errors: {}", errors.join("; ")));
    }

    let mut tests = Vec::new();

    // Walk AST finding functions starting with "test_"
    for item in &ast.items {
        if let Item::Function(func) = item {
            let name = &func.name.name;

            if name.starts_with("test_") {
                // Verify test function signature: no parameters
                if !func.params.is_empty() {
                    eprintln!(
                        "Warning: {} takes parameters, skipping (tests must have no parameters)",
                        name
                    );
                    continue;
                }

                // Use span start as approximate location
                // (calculating exact line requires source scanning)
                let line = func.span.start;

                tests.push(TestFunction {
                    name: name.clone(),
                    file: path.to_path_buf(),
                    line,
                });
            }
        }
    }

    Ok(tests)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_discover_tests_in_file() {
        let mut file = NamedTempFile::with_suffix(".at").unwrap();
        write!(
            file,
            r#"
fn test_addition() {{
    assertEqual(2 + 2, 4);
}}

fn test_subtraction() {{
    assertEqual(5 - 3, 2);
}}

fn helper() {{
    // Not a test
}}
"#
        )
        .unwrap();

        let tests = discover_tests_in_file(file.path()).unwrap();
        assert_eq!(tests.len(), 2);
        assert_eq!(tests[0].name, "test_addition");
        assert_eq!(tests[1].name, "test_subtraction");
    }

    #[test]
    fn test_discover_skips_tests_with_params() {
        let mut file = NamedTempFile::with_suffix(".at").unwrap();
        write!(
            file,
            r#"
fn test_with_param(x: number) {{
    // Should be skipped
}}

fn test_no_param() {{
    assert(true, "ok");
}}
"#
        )
        .unwrap();

        let tests = discover_tests_in_file(file.path()).unwrap();
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].name, "test_no_param");
    }

    #[test]
    fn test_suite_filter() {
        let suite = TestSuite {
            tests: vec![
                TestFunction {
                    name: "test_addition".to_string(),
                    file: PathBuf::from("test.at"),
                    line: 1,
                },
                TestFunction {
                    name: "test_subtraction".to_string(),
                    file: PathBuf::from("test.at"),
                    line: 5,
                },
                TestFunction {
                    name: "test_multiply".to_string(),
                    file: PathBuf::from("test.at"),
                    line: 10,
                },
            ],
            parse_errors: Vec::new(),
        };

        let filtered = suite.filter("add");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered.tests[0].name, "test_addition");
    }

    #[test]
    fn test_suite_discover_directory() {
        let dir = tempdir().unwrap();

        // Create test files
        let test1_path = dir.path().join("math_tests.at");
        fs::write(
            &test1_path,
            r#"
fn test_add() {
    assertEqual(1 + 1, 2);
}
"#,
        )
        .unwrap();

        let test2_path = dir.path().join("string_tests.at");
        fs::write(
            &test2_path,
            r#"
fn test_concat() {
    assertEqual("a" + "b", "ab");
}
"#,
        )
        .unwrap();

        let suite = TestSuite::discover(dir.path());
        assert_eq!(suite.len(), 2);
    }

    #[test]
    fn test_suite_handles_parse_errors() {
        let dir = tempdir().unwrap();

        // Create file with parse error
        let bad_path = dir.path().join("bad.at");
        fs::write(&bad_path, "fn test_broken( { }").unwrap();

        let suite = TestSuite::discover(dir.path());
        assert!(suite.is_empty());
        assert!(!suite.parse_errors.is_empty());
    }
}
