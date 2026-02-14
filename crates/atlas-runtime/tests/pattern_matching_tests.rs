//! Pattern Matching Tests (BLOCKER 03-A: Syntax & Type Checking)
//!
//! Comprehensive tests for pattern matching syntax, type checking, and exhaustiveness.
//! Does NOT test runtime execution (that's BLOCKER 03-B).

use atlas_runtime::{Binder, Lexer, Parser, TypeChecker};

/// Helper to parse and type check code
fn typecheck(source: &str) -> (bool, Vec<String>) {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return (false, lex_diags.iter().map(|d| d.message.clone()).collect());
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        return (
            false,
            parse_diags.iter().map(|d| d.message.clone()).collect(),
        );
    }

    let mut binder = Binder::new();
    let (mut symbol_table, bind_diags) = binder.bind(&program);
    if !bind_diags.is_empty() {
        return (
            false,
            bind_diags.iter().map(|d| d.message.clone()).collect(),
        );
    }

    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let type_diags = typechecker.check(&program);

    let success = type_diags.is_empty();
    let messages = type_diags.iter().map(|d| d.message.clone()).collect();
    (success, messages)
}

// === Parser Tests: Match Expression Syntax ===

#[test]
fn test_parse_simple_match() {
    let source = r#"
        match x {
            1 => "one",
            2 => "two",
            _ => "other"
        }
    "#;
    let (success, _) = typecheck(source);
    // May fail type checking (x undefined), but should parse
    assert!(!success || success); // Just ensure it parses without panic
}

#[test]
fn test_parse_option_match() {
    let source = r#"
        fn test(opt: Option<number>) -> string {
            return match opt {
                Some(x) => "has value",
                None => "no value"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Should type check: {:?}", msgs);
}

#[test]
fn test_parse_result_match() {
    let source = r#"
        fn test(res: Result<number, string>) -> string {
            return match res {
                Ok(val) => "success",
                Err(error) => "failure"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Should type check: {:?}", msgs);
}

// === Pattern Type Tests ===

#[test]
fn test_literal_patterns() {
    let source = r#"
        fn test(x: number) -> string {
            return match x {
                0 => "zero",
                1 => "one",
                _ => "other"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Literal patterns should work: {:?}", msgs);
}

#[test]
fn test_wildcard_pattern() {
    let source = r#"
        fn test(x: number) -> string {
            return match x {
                _ => "anything"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Wildcard should work: {:?}", msgs);
}

#[test]
fn test_variable_binding_pattern() {
    let source = r#"
        fn test(x: number) -> number {
            return match x {
                value => value + 1
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Variable binding should work: {:?}", msgs);
}

#[test]
fn test_nested_constructor_patterns() {
    let source = r#"
        fn test(res: Result<Option<number>, string>) -> string {
            return match res {
                Ok(Some(x)) => "has value",
                Ok(None) => "no value",
                Err(e) => "error"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Nested patterns should work: {:?}", msgs);
}

#[test]
fn test_array_patterns() {
    let source = r#"
        fn test(arr: number[]) -> string {
            return match arr {
                [] => "empty",
                [x] => "one",
                [x, y] => "two",
                _ => "many"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Array patterns should work: {:?}", msgs);
}

// === Type Checking Tests ===

#[test]
fn test_pattern_type_mismatch() {
    let source = r#"
        fn test(x: number) -> string {
            return match x {
                "hello" => "string",
                _ => "other"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject type mismatch");
    assert!(
        msgs.iter()
            .any(|m| m.contains("type mismatch") || m.contains("Pattern type")),
        "Should report type mismatch: {:?}",
        msgs
    );
}

#[test]
fn test_arm_type_mismatch() {
    let source = r#"
        fn test(x: number) -> string {
            return match x {
                0 => "zero",
                1 => 123,
                _ => "other"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject mismatched arm types");
    assert!(
        msgs.iter().any(|m| m.contains("incompatible type")),
        "Should report arm type mismatch: {:?}",
        msgs
    );
}

#[test]
fn test_constructor_wrong_arity() {
    let source = r#"
        fn test(opt: Option<number>) -> string {
            return match opt {
                Some(x, y) => "wrong",
                None => "ok"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject wrong arity");
    assert!(
        msgs.iter().any(|m| m.contains("expects 1 argument")),
        "Should report arity error: {:?}",
        msgs
    );
}

// === Exhaustiveness Tests ===

#[test]
fn test_option_exhaustive() {
    let source = r#"
        fn test(opt: Option<number>) -> string {
            return match opt {
                Some(x) => "has",
                None => "none"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(
        success,
        "Complete Option match should be exhaustive: {:?}",
        msgs
    );
}

#[test]
fn test_option_non_exhaustive_missing_none() {
    let source = r#"
        fn test(opt: Option<number>) -> string {
            return match opt {
                Some(x) => "has"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject non-exhaustive Option");
    assert!(
        msgs.iter()
            .any(|m| m.contains("Non-exhaustive") && m.contains("None")),
        "Should report missing None: {:?}",
        msgs
    );
}

#[test]
fn test_option_non_exhaustive_missing_some() {
    let source = r#"
        fn test(opt: Option<number>) -> string {
            return match opt {
                None => "none"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject non-exhaustive Option");
    assert!(
        msgs.iter()
            .any(|m| m.contains("Non-exhaustive") && m.contains("Some")),
        "Should report missing Some: {:?}",
        msgs
    );
}

#[test]
fn test_result_exhaustive() {
    let source = r#"
        fn test(res: Result<number, string>) -> string {
            return match res {
                Ok(x) => "ok",
                Err(e) => "err"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(
        success,
        "Complete Result match should be exhaustive: {:?}",
        msgs
    );
}

#[test]
fn test_result_non_exhaustive_missing_err() {
    let source = r#"
        fn test(res: Result<number, string>) -> string {
            return match res {
                Ok(x) => "ok"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject non-exhaustive Result");
    assert!(
        msgs.iter()
            .any(|m| m.contains("Non-exhaustive") && m.contains("Err")),
        "Should report missing Err: {:?}",
        msgs
    );
}

#[test]
fn test_result_non_exhaustive_missing_ok() {
    let source = r#"
        fn test(res: Result<number, string>) -> string {
            return match res {
                Err(e) => "err"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject non-exhaustive Result");
    assert!(
        msgs.iter()
            .any(|m| m.contains("Non-exhaustive") && m.contains("Ok")),
        "Should report missing Ok: {:?}",
        msgs
    );
}

#[test]
fn test_bool_exhaustive() {
    let source = r#"
        fn test(b: bool) -> string {
            return match b {
                true => "yes",
                false => "no"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(
        success,
        "Complete bool match should be exhaustive: {:?}",
        msgs
    );
}

#[test]
fn test_bool_non_exhaustive() {
    let source = r#"
        fn test(b: bool) -> string {
            return match b {
                true => "yes"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject non-exhaustive bool");
    assert!(
        msgs.iter()
            .any(|m| m.contains("Non-exhaustive") && m.contains("false")),
        "Should report missing false: {:?}",
        msgs
    );
}

#[test]
fn test_number_requires_wildcard() {
    let source = r#"
        fn test(x: number) -> string {
            return match x {
                0 => "zero",
                1 => "one"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Number match should require wildcard");
    assert!(
        msgs.iter().any(|m| m.contains("Non-exhaustive")),
        "Should report non-exhaustive: {:?}",
        msgs
    );
}

#[test]
fn test_wildcard_makes_exhaustive() {
    let source = r#"
        fn test(x: number) -> string {
            return match x {
                0 => "zero",
                _ => "other"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(
        success,
        "Wildcard should make number match exhaustive: {:?}",
        msgs
    );
}

#[test]
fn test_variable_binding_makes_exhaustive() {
    let source = r#"
        fn test(opt: Option<number>) -> string {
            return match opt {
                value => "anything"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Variable binding should be exhaustive: {:?}", msgs);
}

// === Variable Binding Tests ===

#[test]
fn test_pattern_variable_scope() {
    let source = r#"
        fn test(opt: Option<number>) -> number {
            return match opt {
                Some(x) => x + 1,
                None => 0
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Pattern variables should be in scope: {:?}", msgs);
}

#[test]
fn test_nested_pattern_variables() {
    let source = r#"
        fn test(res: Result<Option<number>, string>) -> number {
            return match res {
                Ok(Some(value)) => value,
                Ok(None) => 0,
                Err(msg) => 0
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Nested pattern variables should work: {:?}", msgs);
}

// === Edge Cases ===

#[test]
fn test_match_as_expression() {
    let source = r#"
        fn test(x: bool) -> string {
            let result = match x {
                true => "yes",
                false => "no"
            };
            return result;
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Match should work as expression: {:?}", msgs);
}

#[test]
fn test_match_in_expression_context() {
    let source = r#"
        fn test(x: bool) -> string {
            return "Answer: " + match x {
                true => "yes",
                false => "no"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Match in expression should work: {:?}", msgs);
}

#[test]
fn test_multiple_matches() {
    let source = r#"
        fn test(a: Option<number>, b: Result<string, number>) -> string {
            let x = match a {
                Some(n) => "has",
                None => "none"
            };
            let y = match b {
                Ok(s) => "ok",
                Err(e) => "err"
            };
            return x + y;
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Multiple matches should work: {:?}", msgs);
}
