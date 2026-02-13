# Testing Strategy

Patterns for unit, golden, and integration tests.

## Unit Tests

Place unit tests in the same file as the implementation:

```rust
// lexer/ module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_number() {
        let mut lexer = Lexer::new("42".to_string());
        let (tokens, diags) = lexer.tokenize();
        assert_eq!(diags.len(), 0);
        assert_eq!(tokens.len(), 2); // Number + EOF
        assert_eq!(tokens[0].kind, TokenKind::Number);
        assert_eq!(tokens[0].lexeme, "42");
    }

    #[test]
    fn test_tokenize_string_with_escapes() {
        let mut lexer = Lexer::new(r#""hello\nworld""#.to_string());
        let (tokens, diags) = lexer.tokenize();
        assert_eq!(diags.len(), 0);
        assert_eq!(tokens[0].kind, TokenKind::String);
        assert_eq!(tokens[0].lexeme, "hello\nworld");
    }

    #[test]
    fn test_unterminated_string() {
        let mut lexer = Lexer::new(r#""hello"#.to_string());
        let (_, diags) = lexer.tokenize();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "AT1002");
    }
}
```

## Golden Tests (with insta)

```rust
// tests/integration_test.rs
use insta::assert_snapshot;
use atlas_runtime::*;

fn run_atlas(source: &str) -> String {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, diags) = lexer.tokenize();

    if !diags.is_empty() {
        return format_diagnostics(&diags, source);
    }

    let mut parser = Parser::new(tokens);
    let (ast, diags) = parser.parse();

    if !diags.is_empty() {
        return format_diagnostics(&diags, source);
    }

    let mut binder = Binder::new();
    let (symbol_table, diags) = binder.bind(&ast);

    if !diags.is_empty() {
        return format_diagnostics(&diags, source);
    }

    let mut typechecker = TypeChecker::new(&symbol_table);
    let diags = typechecker.check(&ast);

    if !diags.is_empty() {
        return format_diagnostics(&diags, source);
    }

    let mut interpreter = Interpreter::new();
    match interpreter.eval(&ast) {
        Ok(value) => value.to_string(),
        Err(e) => format!("Runtime error: {:?}", e),
    }
}

fn format_diagnostics(diags: &[Diagnostic], source: &str) -> String {
    diags.iter()
        .map(|d| d.to_human(source))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_arithmetic() {
    let output = run_atlas("let x = 2 + 3 * 4; x");
    assert_snapshot!(output, @"14");
}

#[test]
fn test_type_error() {
    let output = run_atlas("let x: number = \"hello\";");
    assert_snapshot!(output);  // Captures the diagnostic output
}

#[test]
fn test_array_indexing() {
    let output = run_atlas(r#"
        let arr = [1, 2, 3];
        arr[1]
    "#);
    assert_snapshot!(output, @"2");
}
```

## File-Based Golden Tests

Organize test files:
```
tests/
├── lexer/
│   ├── numbers.atl
│   ├── numbers.out
│   ├── strings.atl
│   └── strings.out
├── parser/
│   ├── expressions.atl
│   └── expressions.out
└── interpreter/
    ├── arithmetic.atl
    └── arithmetic.out
```

Test runner:
```rust
#[test]
fn test_golden_files() {
    for entry in std::fs::read_dir("tests/interpreter").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension() == Some(std::ffi::OsStr::new("atl")) {
            let source = std::fs::read_to_string(&path).unwrap();
            let output = run_atlas(&source);

            let expected_path = path.with_extension("out");
            if expected_path.exists() {
                let expected = std::fs::read_to_string(&expected_path).unwrap();
                assert_eq!(output.trim(), expected.trim(), "Failed: {:?}", path);
            } else {
                // Write output for first run
                std::fs::write(&expected_path, output).unwrap();
            }
        }
    }
}
```

## Integration Tests (CLI)

```rust
#[test]
fn test_cli_run() {
    let output = Command::new("target/debug/atlas")
        .arg("run")
        .arg("tests/e2e/hello.atl")
        .output()
        .expect("Failed to execute atlas");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Hello, Atlas"
    );
}

#[test]
fn test_cli_check() {
    let output = Command::new("target/debug/atlas")
        .arg("check")
        .arg("tests/e2e/type-error.atl")
        .output()
        .expect("Failed to execute atlas");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("AT0001"));
}
```

## Test Organization

```rust
// Test categories
mod lexer_tests;
mod parser_tests;
mod typechecker_tests;
mod interpreter_tests;
mod vm_tests;
mod e2e_tests;
```

## Key Principles

- **Deterministic:** No time-based or random assertions
- **Small inputs:** Focus on one behavior per test
- **Comprehensive:** Cover normal cases, edge cases, and error cases
- **Snapshot tests:** Use `insta` for complex outputs (diagnostics, AST)
- **Avoid flakiness:** No network, no file system (except test fixtures)
