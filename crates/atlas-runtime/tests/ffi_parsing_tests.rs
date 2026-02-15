//! Tests for FFI extern declaration parsing (phase-10b)

use atlas_runtime::ast::{ExternTypeAnnotation, Item};
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;

fn parse_program(source: &str) -> (Vec<Item>, Vec<atlas_runtime::diagnostic::Diagnostic>) {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty(), "Lexer errors: {:?}", lex_diags);

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    (program.items, parse_diags)
}

#[test]
fn test_extern_basic_declaration() {
    let source = r#"extern "libm" fn pow(base: CDouble, exp: CDouble) -> CDouble;"#;
    let (items, diagnostics) = parse_program(source);

    assert_eq!(diagnostics.len(), 0, "Parse errors: {:?}", diagnostics);
    assert_eq!(items.len(), 1);

    if let Item::Extern(extern_decl) = &items[0] {
        assert_eq!(extern_decl.name, "pow");
        assert_eq!(extern_decl.library, "libm");
        assert!(extern_decl.symbol.is_none());
        assert_eq!(extern_decl.params.len(), 2);
        assert_eq!(extern_decl.params[0].0, "base");
        assert!(matches!(extern_decl.params[0].1, ExternTypeAnnotation::CDouble));
        assert_eq!(extern_decl.params[1].0, "exp");
        assert!(matches!(extern_decl.params[1].1, ExternTypeAnnotation::CDouble));
        assert!(matches!(extern_decl.return_type, ExternTypeAnnotation::CDouble));
    } else {
        panic!("Expected extern declaration, got: {:?}", items[0]);
    }
}

#[test]
fn test_extern_with_symbol_renaming() {
    let source = r#"extern "libc" fn string_length as "strlen"(s: CCharPtr) -> CLong;"#;
    let (items, diagnostics) = parse_program(source);

    assert_eq!(diagnostics.len(), 0, "Parse errors: {:?}", diagnostics);
    assert_eq!(items.len(), 1);

    if let Item::Extern(extern_decl) = &items[0] {
        assert_eq!(extern_decl.name, "string_length");
        assert_eq!(extern_decl.library, "libc");
        assert_eq!(extern_decl.symbol, Some("strlen".to_string()));
        assert_eq!(extern_decl.params.len(), 1);
        assert_eq!(extern_decl.params[0].0, "s");
        assert!(matches!(extern_decl.params[0].1, ExternTypeAnnotation::CCharPtr));
        assert!(matches!(extern_decl.return_type, ExternTypeAnnotation::CLong));
    } else {
        panic!("Expected extern declaration");
    }
}

#[test]
fn test_extern_no_params() {
    let source = r#"extern "libc" fn getpid() -> CInt;"#;
    let (items, diagnostics) = parse_program(source);

    assert_eq!(diagnostics.len(), 0, "Parse errors: {:?}", diagnostics);
    assert_eq!(items.len(), 1);

    if let Item::Extern(extern_decl) = &items[0] {
        assert_eq!(extern_decl.name, "getpid");
        assert_eq!(extern_decl.library, "libc");
        assert!(extern_decl.symbol.is_none());
        assert_eq!(extern_decl.params.len(), 0);
        assert!(matches!(extern_decl.return_type, ExternTypeAnnotation::CInt));
    } else {
        panic!("Expected extern declaration");
    }
}

#[test]
fn test_extern_void_return() {
    let source = r#"extern "libc" fn exit(code: CInt) -> CVoid;"#;
    let (items, diagnostics) = parse_program(source);

    assert_eq!(diagnostics.len(), 0, "Parse errors: {:?}", diagnostics);
    assert_eq!(items.len(), 1);

    if let Item::Extern(extern_decl) = &items[0] {
        assert_eq!(extern_decl.name, "exit");
        assert_eq!(extern_decl.library, "libc");
        assert_eq!(extern_decl.params.len(), 1);
        assert_eq!(extern_decl.params[0].0, "code");
        assert!(matches!(extern_decl.params[0].1, ExternTypeAnnotation::CInt));
        assert!(matches!(extern_decl.return_type, ExternTypeAnnotation::CVoid));
    } else {
        panic!("Expected extern declaration");
    }
}

#[test]
fn test_extern_multiple_declarations() {
    let source = r#"
        extern "libm" fn sin(x: CDouble) -> CDouble;
        extern "libm" fn cos(x: CDouble) -> CDouble;
        extern "libm" fn tan(x: CDouble) -> CDouble;
    "#;
    let (items, diagnostics) = parse_program(source);

    assert_eq!(diagnostics.len(), 0, "Parse errors: {:?}", diagnostics);
    assert_eq!(items.len(), 3);

    let names: Vec<_> = items
        .iter()
        .filter_map(|item| {
            if let Item::Extern(extern_decl) = item {
                Some(extern_decl.name.as_str())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(names, vec!["sin", "cos", "tan"]);
}

#[test]
fn test_extern_all_types() {
    let source = r#"
        extern "test" fn test_int(x: CInt) -> CInt;
        extern "test" fn test_long(x: CLong) -> CLong;
        extern "test" fn test_double(x: CDouble) -> CDouble;
        extern "test" fn test_charptr(x: CCharPtr) -> CCharPtr;
        extern "test" fn test_void(x: CInt) -> CVoid;
        extern "test" fn test_bool(x: CBool) -> CBool;
    "#;
    let (items, diagnostics) = parse_program(source);

    assert_eq!(diagnostics.len(), 0, "Parse errors: {:?}", diagnostics);
    assert_eq!(items.len(), 6);

    // Verify all items are extern declarations
    for item in &items {
        assert!(matches!(item, Item::Extern(_)));
    }
}

#[test]
fn test_extern_invalid_type_error() {
    let source = r#"extern "lib" fn bad(x: InvalidType) -> CInt;"#;
    let (_items, diagnostics) = parse_program(source);

    // Should have a parse error for unknown type
    assert!(!diagnostics.is_empty(), "Expected parse error for invalid type");
}

#[test]
fn test_extern_mixed_with_functions() {
    let source = r#"
        extern "libm" fn sqrt(x: CDouble) -> CDouble;
        fn double(x: number) -> number { return x * 2; }
        extern "libc" fn strlen(s: CCharPtr) -> CLong;
    "#;
    let (items, diagnostics) = parse_program(source);

    assert_eq!(diagnostics.len(), 0, "Parse errors: {:?}", diagnostics);
    assert_eq!(items.len(), 3);

    assert!(matches!(items[0], Item::Extern(_)));
    assert!(matches!(items[1], Item::Function(_)));
    assert!(matches!(items[2], Item::Extern(_)));
}
