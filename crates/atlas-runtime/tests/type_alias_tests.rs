//! Tests for type aliases (Phase typing-03)

mod common;

use atlas_runtime::diagnostic::{Diagnostic, DiagnosticLevel};
use atlas_runtime::module_loader::{ModuleLoader, ModuleRegistry};
use atlas_runtime::{Binder, Lexer, Parser, TypeChecker};
use rstest::rstest;
use std::fs;
use tempfile::TempDir;

fn typecheck(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize_with_comments();
    if !lex_diags.is_empty() {
        return lex_diags;
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        return parse_diags;
    }

    let mut binder = Binder::new();
    let (mut table, mut bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&mut table);
    let mut type_diags = checker.check(&program);

    bind_diags.append(&mut type_diags);
    bind_diags
}

fn errors(source: &str) -> Vec<Diagnostic> {
    typecheck(source)
        .into_iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect()
}

fn warnings(source: &str) -> Vec<Diagnostic> {
    typecheck(source)
        .into_iter()
        .filter(|d| d.level == DiagnosticLevel::Warning)
        .collect()
}

fn parse_with_comments(source: &str) -> (atlas_runtime::ast::Program, Vec<Diagnostic>) {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize_with_comments();
    if !lex_diags.is_empty() {
        return (atlas_runtime::ast::Program { items: vec![] }, lex_diags);
    }

    let mut parser = Parser::new(tokens);
    parser.parse()
}

fn typecheck_modules(entry: &str, modules: &[(&str, &str)]) -> Vec<Diagnostic> {
    let temp_dir = TempDir::new().unwrap();
    for (name, content) in modules {
        let path = temp_dir.path().join(format!("{}.atl", name));
        fs::write(&path, content).unwrap();
    }

    let entry_path = temp_dir.path().join(format!("{}.atl", entry));
    let mut loader = ModuleLoader::new(temp_dir.path().to_path_buf());
    let loaded_modules = loader.load_module(&entry_path).unwrap();

    let mut registry = ModuleRegistry::new();
    let mut diagnostics = Vec::new();
    let mut entry_ast = None;
    let mut entry_table = None;

    for module in &loaded_modules {
        let mut binder = Binder::new();
        let (table, mut bind_diags) =
            binder.bind_with_modules(&module.ast, &module.path, &registry);
        diagnostics.append(&mut bind_diags);

        if module.path == entry_path {
            entry_ast = Some(module.ast.clone());
            entry_table = Some(table.clone());
        }

        registry.register(module.path.clone(), table);
    }

    if let (Some(ast), Some(mut table)) = (entry_ast, entry_table) {
        let mut checker = TypeChecker::new(&mut table);
        let mut type_diags = checker.check_with_modules(&ast, &entry_path, &registry);
        diagnostics.append(&mut type_diags);
    }

    diagnostics
}

// ============================================================================
// Alias declaration tests
// ============================================================================

#[rstest]
#[case("type UserId = string; let _x: UserId = \"abc\";")]
#[case("type Count = number; let _x: Count = 42;")]
#[case("type Flag = bool; let _x: Flag = true;")]
#[case("type Numbers = number[]; let _x: Numbers = [1, 2, 3];")]
#[case("type Handler = (number, string) -> bool; fn h(x: number, y: string) -> bool { return true; } let _x: Handler = h;")]
#[case("type Pair<T, U> = (T, U) -> T; fn fst<T, U>(x: T, _y: U) -> T { return x; } let _x: Pair<number, string> = fst;")]
#[case(
    "type ResultMap = HashMap<string, Result<number, string>>; let _x: ResultMap = hashMapNew();"
)]
#[case("export type PublicId = string; let _x: PublicId = \"ok\";")]
fn test_alias_declarations(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Alias resolution tests
// ============================================================================

#[rstest]
#[case("type A = number; let _x: A = 1;")]
#[case("type A = string; type B = A; let _x: B = \"ok\";")]
#[case("type A = number[]; let _x: A = [1, 2];")]
#[case("type A = (number) -> number; fn f(x: number) -> number { return x; } let _x: A = f;")]
#[case("type A = Result<number, string>; let _x: A = Ok(1);")]
#[case("type A = HashMap<string, number>; let _x: A = hashMapNew();")]
#[case("type A = Option<number>; let _x: A = Some(1);")]
#[case("type A = Result<number, string>; let _x: A = Err(\"no\");")]
fn test_alias_resolution(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Generic alias tests
// ============================================================================

#[rstest]
#[case("type Box<T> = T[]; let _x: Box<number> = [1, 2];")]
#[case("type Box<T> = T[]; let _x: Box<string> = [\"a\", \"b\"]; ")]
#[case("type Pair<A, B> = (A, B) -> A; fn fst<A, B>(a: A, _b: B) -> A { return a; } let _x: Pair<number, string> = fst;")]
#[case("type Pair<A, B> = (A, B) -> B; fn snd<A, B>(_a: A, b: B) -> B { return b; } let _x: Pair<number, string> = snd;")]
#[case("type MapEntry<K, V> = (K, V) -> V; fn pick<K, V>(_k: K, v: V) -> V { return v; } let _x: MapEntry<string, number> = pick;")]
#[case("type Wrap<T> = Option<T>; let _x: Wrap<number> = Some(1);")]
#[case("type Wrap<T> = Result<T, string>; let _x: Wrap<number> = Ok(1);")]
#[case("type Wrap<T> = Result<T, string>; let _x: Wrap<number> = Err(\"no\");")]
#[case("type Nested<T> = Option<Result<T, string>>; let _x: Nested<number> = Some(Ok(1));")]
#[case("type Nested<T> = Option<Result<T, string>>; let _x: Nested<number> = Some(Err(\"no\"));")]
#[case("type Alias<T> = T; let _x: Alias<number> = 1;")]
#[case("type Alias<T> = T; let _x: Alias<string> = \"ok\";")]
#[case("type Alias<T> = T[]; let _x: Alias<number> = [1];")]
#[case("type Alias<T> = T[]; let _x: Alias<string> = [\"a\"]; ")]
fn test_generic_aliases(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Type equivalence with aliases
// ============================================================================

#[rstest]
#[case("type A = number; type B = number; let _x: A = 1; let _y: B = _x;")]
#[case("type A = string; type B = string; let _x: A = \"ok\"; let _y: B = _x;")]
#[case("type A = number[]; type B = number[]; let _x: A = [1]; let _y: B = _x;")]
#[case("type A = (number) -> number; type B = (number) -> number; fn f(x: number) -> number { return x; } let _x: A = f; let _y: B = _x;")]
#[case("type A = Result<number, string>; type B = Result<number, string>; let _x: A = Ok(1); let _y: B = _x;")]
#[case("type A = Option<number>; type B = Option<number>; let _x: A = Some(1); let _y: B = _x;")]
#[case("type A = HashMap<string, number>; type B = HashMap<string, number>; let _x: A = hashMapNew(); let _y: B = _x;")]
#[case("type A = string; type B = A; let _x: B = \"ok\";")]
#[case("type A = number; type B = A; type C = B; let _x: C = 1;")]
#[case("type A<T> = T[]; type B<T> = A<T>; let _x: B<number> = [1];")]
fn test_type_equivalence_with_aliases(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Alias in annotations
// ============================================================================

#[rstest]
#[case("type UserId = string; fn f(id: UserId) -> UserId { return id; }")]
#[case("type Count = number; fn f(x: Count) -> number { return x; }")]
#[case("type Name = string; let _x: Name = \"ok\";")]
#[case("type Ok = Result<number, string>; fn f() -> Ok { return Ok(1); }")]
#[case("type MaybeNum = Option<number>; fn f() -> MaybeNum { return Some(1); }")]
#[case("type Arr = number[]; let _x: Arr = [1, 2];")]
fn test_alias_in_annotations(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Circular alias detection
// ============================================================================

#[rstest]
#[case("type A = A; let _x: A = 1;")]
#[case("type A = B; type B = A; let _x: A = 1;")]
#[case("type A = B; type B = C; type C = A; let _x: A = 1;")]
#[case("type A<T> = A<T>; let _x: A<number> = 1;")]
#[case("type A = B; type B = C; type C = D; type D = A; let _x: A = 1;")]
#[case("type A = B; type B = (number) -> A; let _x: A = 1;")]
fn test_circular_alias_detection(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected circular alias errors");
}

// ============================================================================
// Doc comments and deprecation
// ============================================================================

#[test]
fn test_doc_comment_on_alias() {
    let source = "/// A user id\ntype UserId = string;";
    let (program, diags) = parse_with_comments(source);
    assert!(diags.is_empty(), "Unexpected diagnostics: {:?}", diags);
    match &program.items[0] {
        atlas_runtime::ast::Item::TypeAlias(alias) => {
            assert_eq!(alias.doc_comment.as_deref(), Some("A user id"));
        }
        _ => panic!("Expected type alias"),
    }
}

#[rstest]
#[case("/// @deprecated\ntype OldId = string; let _x: OldId = \"ok\";")]
#[case("/// deprecated\ntype Legacy = number; let _x: Legacy = 1;")]
#[case("/// @deprecated\n/// @since 0.3\ntype Old = string; let _x: Old = \"ok\";")]
fn test_deprecated_alias_warning(#[case] source: &str) {
    let diags = warnings(source);
    assert!(
        diags.iter().any(|d| d.code == "AT2009"),
        "Expected deprecated warning, got: {:?}",
        diags
    );
}

// ============================================================================
// Error messages include alias names
// ============================================================================

#[test]
fn test_alias_name_in_error_message() {
    let diags = errors("type UserId = string; let _x: UserId = 1;");
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("UserId"));
}

// ============================================================================
// Generic alias inference
// ============================================================================

#[test]
fn test_infer_alias_type_args_from_initializer() {
    let diags = errors("type Box<T> = T[]; let _x: Box = [1, 2, 3];");
    assert!(
        diags.is_empty(),
        "Expected inference to succeed, got: {:?}",
        diags
    );
}

// ============================================================================
// Module integration tests
// ============================================================================

#[test]
fn test_alias_across_modules() {
    let diags = typecheck_modules(
        "main",
        &[
            ("types", "export type UserId = string;"),
            (
                "main",
                "import { UserId } from \"./types\"; let _x: UserId = \"ok\";",
            ),
        ],
    );
    assert!(diags.is_empty(), "Unexpected diagnostics: {:?}", diags);
}

#[test]
fn test_alias_export_import_generic() {
    let diags = typecheck_modules(
        "main",
        &[
            ("types", "export type Box<T> = T[];"),
            (
                "main",
                "import { Box } from \"./types\"; let _x: Box<number> = [1, 2];",
            ),
        ],
    );
    assert!(diags.is_empty(), "Unexpected diagnostics: {:?}", diags);
}

#[test]
fn test_alias_export_import_nested() {
    let diags = typecheck_modules(
        "main",
        &[
            ("types", "export type ResultStr = Result<number, string>;"),
            (
                "main",
                "import { ResultStr } from \"./types\"; let _x: ResultStr = Ok(1);",
            ),
        ],
    );
    assert!(diags.is_empty(), "Unexpected diagnostics: {:?}", diags);
}

#[test]
fn test_alias_import_missing() {
    let diags = typecheck_modules(
        "main",
        &[
            ("types", "export type UserId = string;"),
            (
                "main",
                "import { UnknownAlias } from \"./types\"; let _x: UnknownAlias = \"ok\";",
            ),
        ],
    );
    assert!(!diags.is_empty());
}

// ============================================================================
// Alias cache reuse
// ============================================================================

#[test]
fn test_alias_cache_reuse() {
    let diags = errors("type UserId = string; let _a: UserId = \"a\"; let _b: UserId = \"b\";");
    assert!(diags.is_empty());
}
