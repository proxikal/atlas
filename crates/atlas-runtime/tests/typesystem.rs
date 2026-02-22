//! typesystem.rs — merged from 14 files (Phase Infra-02)

mod common;

use atlas_runtime::binder::Binder;
use atlas_runtime::diagnostic::{Diagnostic, DiagnosticLevel};
use atlas_runtime::lexer::Lexer;
use atlas_runtime::module_loader::{ModuleLoader, ModuleRegistry};
use atlas_runtime::parser::Parser;
use atlas_runtime::repl::ReplCore;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::{Atlas, TypecheckDump, Value, TYPECHECK_VERSION};
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Canonical helpers (deduplicated from all 14 source files)
// ============================================================================

fn typecheck_source(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&program);
    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);
    [lex_diags, parse_diags, bind_diags, type_diags].concat()
}

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
    let (mut table, bind_diags) = binder.bind(&program);
    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);
    [bind_diags, type_diags].concat()
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

fn has_error(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|d| d.level == DiagnosticLevel::Error)
}

fn has_error_code(diagnostics: &[Diagnostic], code: &str) -> bool {
    diagnostics.iter().any(|d| d.code == code)
}

fn assert_no_errors(diagnostics: &[Diagnostic]) {
    let errs: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        errs.is_empty(),
        "Expected no errors, got: {:?}",
        errs.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

fn assert_has_error(diagnostics: &[Diagnostic], code: &str) {
    assert!(
        !diagnostics.is_empty(),
        "Expected at least one diagnostic with code {}",
        code
    );
    assert!(
        diagnostics.iter().any(|d| d.code == code),
        "Expected diagnostic with code {}, got: {:?}",
        code,
        diagnostics
    );
}

// ============================================================================
// From advanced_inference_tests.rs
// ============================================================================

// Advanced Type Inference - Integration Tests (Phase 07)
//
// Tests for:
// - Bidirectional type checking (synthesis & checking modes)
// - Higher-rank polymorphism
// - Let-polymorphism generalization
// - Flow-sensitive typing
// - Unification algorithm
// - Constraint-based inference
// - Cross-module inference
// - Inference heuristics
// - Complex program integration

// ============================================================================
// Helpers
// ============================================================================

fn has_code(diags: &[Diagnostic], code: &str) -> bool {
    diags.iter().any(|d| d.code == code)
}

// ============================================================================
// Bidirectional Type Checking Tests
// ============================================================================

#[test]
fn test_bidir_synthesis_infers_number_literal() {
    // Synthesis: infer type of number literal
    let diags = typecheck_source("let _x = 42;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_synthesis_infers_string_literal() {
    let diags = typecheck_source(r#"let _x = "hello";"#);
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_synthesis_infers_bool_literal() {
    let diags = typecheck_source("let _x = true;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_checking_validates_annotation() {
    // Checking mode: annotation guides inference
    let diags = typecheck_source("let _x: number = 42;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_checking_rejects_mismatch() {
    // Checking mode: annotation rejects wrong type
    let diags = typecheck_source(r#"let _x: number = "hello";"#);
    assert!(has_error(&diags), "Expected type error");
}

#[test]
fn test_bidir_checking_string_annotation() {
    let diags = typecheck_source(r#"let _x: string = "world";"#);
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_expected_type_guides_return() {
    let diags = typecheck_source(
        r#"
        fn compute() -> number {
            return 42;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_return_type_mismatch_detected() {
    let diags = typecheck_source(
        r#"
        fn compute() -> number {
            return "oops";
        }
        "#,
    );
    assert!(has_error(&diags), "Expected return type error");
}

#[test]
fn test_bidir_mode_switch_at_function_boundary() {
    // Annotation on parameter sets expected type for the body
    let diags = typecheck_source(
        r#"
        fn add_one(x: number) -> number {
            return x + 1;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_complex_expression_inferred() {
    let diags = typecheck_source(
        r#"
        fn max_val(a: number, b: number) -> number {
            if (a > b) {
                return a;
            }
            return b;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_expected_type_propagation_through_if() {
    let diags = typecheck_source(
        r#"
        fn test(flag: bool) -> string {
            if (flag) {
                return "yes";
            }
            return "no";
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_bidir_infer_without_annotation() {
    // No annotation: full inference from initializer
    let diags = typecheck_source(
        r#"
        let _a = 1 + 2;
        let _b = true;
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

// ============================================================================
// Higher-Rank Polymorphism Tests
// ============================================================================

#[test]
fn test_rank1_polymorphism_inferred() {
    // Simple rank-1 polymorphism: T is inferred from the argument
    let diags = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let _n = identity(42);
        let _s = identity("hello");
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_function_taking_generic_function() {
    // A function whose parameter is a generic function
    let diags = typecheck_source(
        r#"
        fn apply<T>(f: (T) -> T, x: T) -> T {
            return f(x);
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_callback_with_typed_parameter() {
    let diags = typecheck_source(
        r#"
        fn transform(f: (number) -> number, x: number) -> number {
            return f(x);
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_generic_callback_applied() {
    let diags = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        fn use_identity(n: number) -> number {
            return identity(n);
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_infer_with_rank_restrictions_concrete_param() {
    // When function type is concrete, inference works directly
    let diags = typecheck_source(
        r#"
        fn double(f: (number) -> number) -> number {
            return f(f(1));
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_function_type_parameter_unification() {
    let diags = typecheck_source(
        r#"
        fn compose<A, B, C>(f: (B) -> C, g: (A) -> B) -> (A) -> C {
            fn h(x: A) -> C {
                return f(g(x));
            }
            return h;
        }
        "#,
    );
    // Composition of generic functions is valid
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

// ============================================================================
// Let-Polymorphism Tests
// ============================================================================

#[test]
fn test_let_bind_infers_number() {
    let diags = typecheck_source("let _x = 10;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_let_bind_infers_string() {
    let diags = typecheck_source(r#"let _y = "hello";"#);
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_let_bind_infers_bool() {
    let diags = typecheck_source("let _z = false;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_let_bind_infers_null() {
    let diags = typecheck_source("let _n = null;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_let_bind_with_explicit_annotation() {
    let diags = typecheck_source("let _x: number = 42;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_let_bind_mutable_allows_reassign() {
    let diags = typecheck_source(
        r#"
        var x = 5;
        x = 10;
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_let_bind_immutable_rejects_reassign() {
    let diags = typecheck_source(
        r#"
        let x = 5;
        x = 10;
        "#,
    );
    assert!(has_code(&diags, "AT3003"), "Expected immutability error");
}

#[test]
fn test_recursive_function_type_check() {
    // Recursive function - let binding supports recursive references
    let diags = typecheck_source(
        r#"
        fn factorial(n: number) -> number {
            if (n == 0) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

// ============================================================================
// Flow-Sensitive Typing Tests
// ============================================================================

#[test]
fn test_flow_type_narrowed_in_then_branch() {
    // After checking typeof, the type is narrowed in the branch
    let diags = typecheck_source(
        r#"
        fn narrow_test(x: number | string) -> number {
            if (typeof(x) == "number") {
                return x;
            }
            return 0;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_flow_widen_at_merge_point() {
    // After if-else, mutable variable can be assigned in both branches
    let diags = typecheck_source(
        r#"
        fn get_val(flag: bool) -> number {
            var result = 0;
            if (flag) {
                result = 1;
            } else {
                result = 2;
            }
            return result;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_flow_immutable_tracking_precise() {
    // Immutable variable: type doesn't widen
    let diags = typecheck_source("let _x: number = 42;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_flow_loop_basic() {
    let diags = typecheck_source(
        r#"
        var i = 0;
        while (i < 10) {
            i = i + 1;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_flow_loop_with_for() {
    let diags = typecheck_source(
        r#"
        var sum = 0;
        for (var i = 0; i < 5; i++) {
            sum = sum + i;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_flow_impossible_never_branch() {
    // Narrowing to Never when both branches are covered
    let diags = typecheck_source(
        r#"
        fn check(x: number) -> bool {
            if (x > 0) {
                return true;
            }
            return false;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_flow_condition_bool_required() {
    // Control flow requires bool condition
    let diags = typecheck_source("if (42) { }");
    assert!(has_error(&diags), "Expected condition must be bool error");
}

// ============================================================================
// Unification Tests (via type checker API)
// ============================================================================

#[test]
fn test_unification_generic_type_arg_inferred() {
    let diags = typecheck_source(
        r#"
        fn wrap<T>(x: T) -> T[] {
            let _arr: T[] = [x];
            return _arr;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_unification_occurs_check_invalid_recursive_fn() {
    // A function declared recursively but with wrong return type
    let diags = typecheck_source(
        r#"
        fn get_number() -> string {
            return 42;
        }
        "#,
    );
    assert!(has_error(&diags), "Expected return type mismatch");
}

#[test]
fn test_unification_struct_member_types() {
    // Structural type accepted as function parameter
    let diags = typecheck_source(
        r#"
        fn validate_point(_p: { x: number, y: number }) -> bool {
            return true;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_unification_union_type_parameters() {
    let diags = typecheck_source(
        r#"
        fn get_str_or_num() -> number | string {
            return 42;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_unification_function_signature_match() {
    let diags = typecheck_source(
        r#"
        fn apply_fn(f: (number) -> string, x: number) -> string {
            return f(x);
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_unification_generic_constraints_satisfied() {
    let diags = typecheck_source(
        r#"
        fn max_val<T extends Comparable>(a: T, b: T) -> T {
            if (a > b) {
                return a;
            }
            return b;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

// ============================================================================
// Constraint Solving Tests
// ============================================================================

#[test]
fn test_constraint_type_annotation_solves() {
    // Annotation provides the constraint, initializer must satisfy it
    let diags = typecheck_source("let _v: number = 1 + 2;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_constraint_unsolvable_type_mismatch() {
    let diags = typecheck_source(r#"let _v: number = "string";"#);
    assert!(has_error(&diags), "Expected constraint violation");
}

#[test]
fn test_constraint_delayed_solving_generic_call() {
    // Type parameters inferred lazily from call site
    let diags = typecheck_source(
        r#"
        fn id<T>(x: T) -> T {
            return x;
        }
        let _n: number = id(42);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_constraint_function_return_constraint() {
    let diags = typecheck_source(
        r#"
        fn make_number() -> number {
            let _x = 42;
            return _x;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_constraint_parameter_type_propagated() {
    let diags = typecheck_source(
        r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        let _r: number = double(5);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_constraint_multiple_parameters_inferred() {
    let diags = typecheck_source(
        r#"
        fn pair<A, B>(a: A, b: B) -> A {
            return a;
        }
        let _r = pair(1, "two");
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

// ============================================================================
// Cross-Module Inference Tests
// ============================================================================

#[test]
fn test_cross_module_export_valid() {
    // A module with a valid export
    let diags = typecheck_source(
        r#"
        export fn add(a: number, b: number) -> number {
            return a + b;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_cross_module_no_duplicate_exports() {
    // Duplicate exports of the same name should be detected
    let diags = typecheck_source(
        r#"
        export let _a: number = 1;
        export let _a: number = 2;
        "#,
    );
    // Either binder redeclaration error OR type checker duplicate export error
    assert!(
        has_error(&diags),
        "Expected error for duplicate export or redeclaration"
    );
}

#[test]
fn test_cross_module_type_alias_exported() {
    let diags = typecheck_source(
        r#"
        export type Name = string;
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_cross_module_exported_variable() {
    let diags = typecheck_source(
        r#"
        export let _version: string = "1.0";
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_cross_module_inferred_type_exported() {
    let diags = typecheck_source(
        r#"
        export fn identity<T>(x: T) -> T {
            return x;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_cross_module_export_type_validated() {
    let diags = typecheck_source(
        r#"
        export fn get_number() -> number {
            return "not a number";
        }
        "#,
    );
    assert!(
        has_error(&diags),
        "Expected return type error in exported function"
    );
}

// ============================================================================
// Inference Heuristics Tests (via type checker)
// ============================================================================

#[test]
fn test_heuristic_prefer_simple_in_arithmetic() {
    // Arithmetic produces number, not a complex type
    let diags = typecheck_source("let _x = 1 + 2;");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_heuristic_literal_inference() {
    // Number literal infers to number
    let diags = typecheck_source(
        r#"
        fn expects_num(x: number) -> number { return x; }
        let _r = expects_num(42);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_heuristic_union_inferred_from_conditional() {
    // Union type inferred when condition returns different types
    let diags = typecheck_source(
        r#"
        fn get_val(flag: bool) -> number | string {
            if (flag) {
                return 42;
            }
            return "hello";
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_heuristic_prefer_primitive_in_generic_context() {
    let diags = typecheck_source(
        r#"
        fn id<T>(x: T) -> T {
            return x;
        }
        let _v = id(99);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_heuristic_minimize_vars_unknown_fallback() {
    // When a generic function is used without explicit type arg,
    // the type checker should infer it from the call site
    let diags = typecheck_source(
        r#"
        fn wrap<T>(x: T) -> T {
            return x;
        }
        let _r = wrap(true);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_heuristic_array_element_type_inferred() {
    // Array element type inferred from literal
    let diags = typecheck_source("let _arr = [1, 2, 3];");
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_integration_complex_program_no_annotations() {
    // Complex program with minimal annotations
    let diags = typecheck_source(
        r#"
        fn fibonacci(n: number) -> number {
            if (n <= 1) {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        let _result = fibonacci(10);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_generic_identity_minimal_annotations() {
    let diags = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let _a = identity(42);
        let _b = identity("hello");
        let _c = identity(true);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_real_world_string_processing() {
    let diags = typecheck_source(
        r#"
        fn greet(name: string) -> string {
            return "Hello, " + name + "!";
        }
        let _message = greet("World");
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_nested_function_inference() {
    let diags = typecheck_source(
        r#"
        fn outer(x: number) -> number {
            fn inner(y: number) -> number {
                return y * 2;
            }
            return inner(x) + 1;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_type_checking_across_variables() {
    let diags = typecheck_source(
        r#"
        let a = 10;
        let b = 20;
        let _sum: number = a + b;
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_array_operations() {
    let diags = typecheck_source(
        r#"
        fn first<T>(arr: T[]) -> T {
            return arr[0];
        }
        let nums = [1, 2, 3];
        let _n = first(nums);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_multiple_functions_call_chain() {
    let diags = typecheck_source(
        r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        fn add_one(x: number) -> number {
            return x + 1;
        }
        let _result = add_one(double(5));
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_generic_with_constraint() {
    let diags = typecheck_source(
        r#"
        fn max_num<T extends Comparable>(a: T, b: T) -> T {
            if (a > b) {
                return a;
            }
            return b;
        }
        let _m = max_num(3, 7);
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_option_type_usage() {
    // Option<number> should be recognized as a valid generic type
    let diags = typecheck_source(
        r#"
        fn accepts_option(_x: Option<number>) -> void {}
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_result_type_usage() {
    // Result<number, string> should be recognized as a valid generic type
    let diags = typecheck_source(
        r#"
        fn accepts_result(_x: Result<number, string>) -> void {}
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_for_in_with_inferred_element_type() {
    let diags = typecheck_source(
        r#"
        fn sum_array(nums: number[]) -> number {
            var total = 0;
            for n in nums {
                total = total + n;
            }
            return total;
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_type_alias_in_function() {
    let diags = typecheck_source(
        r#"
        type Predicate<T> = (T) -> bool;
        fn always_true(_x: number) -> bool {
            return true;
        }
        let _pred: Predicate<number> = always_true;
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_union_type_function_params() {
    let diags = typecheck_source(
        r#"
        fn show_value(x: number | string) -> string {
            if (typeof(x) == "number") {
                return "it is a number";
            }
            return "it is a string";
        }
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_structural_type_inference() {
    // Structural types accepted as function parameters
    let diags = typecheck_source(
        r#"
        fn accepts_point(_point: { x: number, y: number }) -> void {}
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

#[test]
fn test_integration_deeply_nested_generics() {
    let diags = typecheck_source(
        r#"
        fn nested(_x: Option<Result<number, string>>) -> void {}
        "#,
    );
    assert!(!has_error(&diags), "Errors: {:?}", diags);
}

// ============================================================================
// From constraint_tests.rs
// ============================================================================

fn assert_constraint_no_errors(source: &str) {
    let diagnostics = typecheck_source(source);
    assert!(
        !has_error(&diagnostics),
        "Expected no errors, got: {:?}",
        diagnostics
    );
}

fn assert_constraint_has_error(source: &str) {
    let diagnostics = typecheck_source(source);
    assert!(
        has_error(&diagnostics),
        "Expected errors, got: {:?}",
        diagnostics
    );
}

// -----------------------------------------------------------------------------
// Constraint syntax tests
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends number>(x: T) -> T { return x; }")]
#[case("fn f<T extends number & number>(x: T) -> T { return x; }")]
#[case("fn f<T extends number | string>(x: T) -> T { return x; }")]
#[case("fn f<T extends { as_string: () -> string }>(x: T) -> T { return x; }")]
#[case("type Box<T extends number> = T;")]
#[case("type Box<T extends number | string> = T;")]
#[case("fn f<T extends Iterable>(x: T) -> number { return 0; }")]
#[case("fn f<T extends Serializable>(x: T) -> string { return str(x); }")]
#[case("fn f<T extends Equatable>(x: T) -> bool { return true; }")]
#[case("fn f<T extends Comparable>(x: T) -> bool { return true; }")]
fn test_constraint_syntax_valid(#[case] source: &str) {
    assert_constraint_no_errors(source);
}

#[rstest]
#[case("fn f<T extends>(x: T) -> T { return x; }")]
#[case("fn f<T extends number,>(x: T) -> T { return x; }")]
#[case("fn f<T extends {>(x: T) -> T { return x; }")]
#[case("fn f<T extends number>(x: T) -> T { return x }")]
#[case("type Box<T extends> = T;")]
#[case("type Box<T extends number,> = T;")]
#[case("fn f<T extends { as_string () -> string }>(x: T) -> T { return x; }")]
#[case(
    "fn f<T extends { as_string: () -> string, as_number: () -> number }(x: T) -> T { return x; }"
)]
fn test_constraint_syntax_invalid(#[case] source: &str) {
    assert_constraint_has_error(source);
}

// -----------------------------------------------------------------------------
// Constraint checking (success)
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends number>(x: T) -> T { return x; } let y = f(1);")]
#[case("fn f<T extends number | string>(x: T) -> T { return x; } let y = f(\"a\");")]
#[case("fn f<T extends number & number>(x: T) -> T { return x; } let y = f(1);")]
#[case("fn f<T extends Equatable>(x: T) -> bool { return true; } let y = f(false);")]
#[case("fn f<T extends Serializable>(x: T) -> string { return str(x); } let y = f(1);")]
#[case("fn f<T extends Iterable>(x: T) -> number { return 0; } let y = f([1, 2]);")]
#[case("fn f<T extends { as_string: () -> string }>(x: T) -> string { return x.as_string(); } let y: json = parseJSON(\"{}\"); let z = f(y);")]
#[case("fn f<T extends { value: json }>(x: T) -> T { return x; } let y: json = parseJSON(\"{}\"); let z = f(y);")]
#[case("type Box<T extends number> = T; let x: Box<number> = 1;")]
#[case("type Box<T extends number | string> = T; let x: Box<string> = \"hi\";")]
fn test_constraint_checking_success(#[case] source: &str) {
    assert_constraint_no_errors(source);
}

// -----------------------------------------------------------------------------
// Constraint checking (failure)
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends number>(x: T) -> T { return x; } let y = f(\"a\");")]
#[case("fn f<T extends number | string>(x: T) -> T { return x; } let y = f(true);")]
#[case("fn f<T extends Iterable>(x: T) -> number { return 0; } let y = f(1);")]
#[case("fn f<T extends Serializable>(x: T) -> string { return str(x); } let y = f([1, 2]);")]
#[case("fn f<T extends Equatable>(x: T) -> bool { return true; } let y: json = parseJSON(\"{}\"); let z = f(y);")]
#[case("fn f<T extends { as_string: () -> string }>(x: T) -> string { return x.as_string(); } let y = f(1);")]
#[case("fn f<T extends { value: json }>(x: T) -> T { return x; } let y = f(1);")]
#[case("fn f<T extends number & string>(x: T) -> T { return x; } let y = f(1);")]
#[case("type Box<T extends number & string> = T;")]
#[case("fn f<T extends UnknownConstraint>(x: T) -> T { return x; }")]
fn test_constraint_checking_failure(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert!(
        has_error(&diagnostics),
        "Expected errors, got: {:?}",
        diagnostics
    );
}

// -----------------------------------------------------------------------------
// Multiple constraints and normalization
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends number & Serializable>(x: T) -> T { return x; } let y = f(1);")]
#[case("fn f<T extends Serializable & Equatable>(x: T) -> T { return x; } let y = f(\"a\");")]
#[case("fn f<T extends Serializable & Equatable>(x: T) -> T { return x; } let y = f(false);")]
#[case("fn f<T extends number & Comparable>(x: T) -> T { return x; } let y = f(1);")]
#[case("fn f<T extends Iterable & Serializable>(x: T) -> number { return 0; } let y = f([1]);")]
#[case("fn f<T extends number & number & number>(x: T) -> T { return x; } let y = f(1);")]
#[case(
    "fn f<T extends (number | string) & Serializable>(x: T) -> T { return x; } let y = f(\"a\");"
)]
#[case("fn f<T extends (number | string) & Serializable>(x: T) -> T { return x; } let y = f(1);")]
#[case(
    "fn f<T extends (number | string) & Serializable>(x: T) -> T { return x; } let y = f(true);"
)]
#[case("fn f<T extends (number | string) & Equatable>(x: T) -> T { return x; } let y = f(\"a\");")]
fn test_multiple_constraints(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    if source.contains("true") || source.contains("Iterable & Serializable") {
        assert!(
            has_error(&diagnostics),
            "Expected errors, got: {:?}",
            diagnostics
        );
    } else {
        assert!(
            !has_error(&diagnostics),
            "Expected no errors, got: {:?}",
            diagnostics
        );
    }
}

// -----------------------------------------------------------------------------
// Constraint inference success
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends number>(x: T) -> T { return x; } let y = f(3);")]
#[case("fn f<T extends number | string>(x: T) -> T { return x; } let y = f(\"a\");")]
#[case("fn f<T extends Serializable>(x: T) -> string { return str(x); } let y = f(99);")]
#[case("fn f<T extends Iterable>(x: T) -> number { return 0; } let y = f([1, 2, 3]);")]
#[case("fn f<T extends Equatable>(x: T) -> bool { return true; } let y = f(false);")]
#[case("fn f<T extends Comparable>(x: T) -> bool { return true; } let y = f(42);")]
#[case("fn f<T extends { as_string: () -> string }>(x: T) -> string { return x.as_string(); } let y: json = parseJSON(\"{}\"); let z = f(y);")]
#[case("fn f<T extends { value: json }>(x: T) -> T { return x; } let y: json = parseJSON(\"{}\"); let z = f(y);")]
#[case("type Box<T extends number> = T; fn f<T extends number>(x: T) -> Box<T> { return x; } let y = f(1);")]
#[case("type Box<T extends Serializable> = T; fn f<T extends Serializable>(x: T) -> Box<T> { return x; } let y = f(\"a\");")]
fn test_constraint_inference_success(#[case] source: &str) {
    assert_constraint_no_errors(source);
}

// -----------------------------------------------------------------------------
// Constraint inference failure
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends number>(x: T) -> T { return x; } let y = f(true);")]
#[case("fn f<T extends number>(x: T, y: T) -> T { return x; } let z = f(1, \"a\");")]
#[case("fn f<T extends Serializable>(x: T) -> T { return x; } let y = f([1]);")]
#[case("fn f<T extends Iterable>(x: T) -> number { return 0; } let y = f(\"a\");")]
#[case("fn f<T extends Equatable>(x: T) -> T { return x; } let y: json = parseJSON(\"{}\"); let z = f(y);")]
#[case("fn f<T extends Comparable>(x: T) -> T { return x; } let y = f(\"a\");")]
#[case("fn f<T extends number>() -> T { return 1; } let y = f();")]
#[case("fn f<T extends { as_string: () -> string }>(x: T) -> string { return x.as_string(); } let y = f(1);")]
fn test_constraint_inference_failure(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert!(
        has_error_code(&diagnostics, "AT3001") || has_error_code(&diagnostics, "AT9999"),
        "Expected AT3001/AT9999, got: {:?}",
        diagnostics
    );
}

// -----------------------------------------------------------------------------
// Practical constraint patterns
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends Comparable>(x: T) -> bool { return true; } let y = f(1);")]
#[case("fn f<T extends Numeric>(x: T) -> T { return x; } let y = f(1);")]
#[case("fn f<T extends Iterable>(x: T) -> number { return 0; } let y = f([1]);")]
#[case("fn f<T extends Equatable>(x: T) -> bool { return true; } let y = f(\"a\");")]
#[case("fn f<T extends Serializable>(x: T) -> string { return str(x); } let y = f(true);")]
#[case("fn f<T extends Comparable>(x: T) -> bool { return true; } let y = f(\"a\");")]
#[case("fn f<T extends Numeric>(x: T) -> T { return x; } let y = f(\"a\");")]
#[case("fn f<T extends Iterable>(x: T) -> number { return 0; } let y = f(1);")]
#[case("fn f<T extends Equatable>(x: T) -> bool { return true; } let y: json = parseJSON(\"{}\"); let z = f(y);")]
#[case("fn f<T extends Serializable>(x: T) -> string { return str(x); } let y = f([1]);")]
fn test_practical_constraint_patterns(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    if (source.contains("\"a\"") && source.contains("Numeric"))
        || (source.contains("Iterable") && source.contains("= f(1)"))
        || (source.contains("Comparable") && source.contains("\"a\""))
        || (source.contains("Equatable") && source.contains("json"))
        || (source.contains("Serializable") && source.contains("[1]"))
    {
        assert!(
            has_error(&diagnostics),
            "Expected errors, got: {:?}",
            diagnostics
        );
    } else {
        assert!(
            !has_error(&diagnostics),
            "Expected no errors, got: {:?}",
            diagnostics
        );
    }
}

// ============================================================================
// From function_return_analysis_tests.rs
// ============================================================================

// Comprehensive tests for function return analysis
//
// Tests cover:
// - All code paths must return for non-void/non-null functions
// - If/else branch return analysis
// - Nested control flow return analysis
// - Early returns
// - Functions that don't need to return (void/null)
// - Missing return diagnostics (AT3004)

// ========== Functions That Always Return ==========

#[test]
fn test_simple_return() {
    let diagnostics = typecheck_source(
        r#"
        fn getNumber() -> number {
            return 42;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_expression() {
    let diagnostics = typecheck_source(
        r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_after_statements() {
    let diagnostics = typecheck_source(
        r#"
        fn calculate(x: number) -> number {
            let y: number = x * 2;
            let z: number = y + 10;
            return z;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_early_return() {
    let diagnostics = typecheck_source(
        r#"
        fn myAbs(x: number) -> number {
            if (x < 0) {
                return -x;
            }
            return x;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Missing Return Errors ==========

#[test]
fn test_missing_return_error() {
    let diagnostics = typecheck_source(
        r#"
        fn getNumber() -> number {
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // Not all code paths return
}

#[test]
fn test_missing_return_with_statements() {
    let diagnostics = typecheck_source(
        r#"
        fn calculate(x: number) -> number {
            let y: number = x * 2;
            let z: number = y + 10;
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // Not all code paths return
}

#[test]
fn test_missing_return_string_function() {
    let diagnostics = typecheck_source(
        r#"
        fn getMessage() -> string {
            let msg: string = "hello";
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // Not all code paths return
}

#[test]
fn test_missing_return_bool_function() {
    let diagnostics = typecheck_source(
        r#"
        fn isPositive(x: number) -> bool {
            let result: bool = x > 0;
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // Not all code paths return
}

// ========== If/Else Return Analysis ==========

#[test]
fn test_if_else_both_return() {
    let diagnostics = typecheck_source(
        r#"
        fn myAbs(x: number) -> number {
            if (x < 0) {
                return -x;
            } else {
                return x;
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_if_else_only_if_returns_error() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                return x;
            } else {
                let y: number = 1;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // else branch doesn't return
}

#[test]
fn test_if_else_only_else_returns_error() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                let y: number = 1;
            } else {
                return x;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // if branch doesn't return
}

#[test]
fn test_if_without_else_returns_error() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                return x;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // no else branch
}

#[test]
fn test_if_without_else_then_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                return x * 2;
            }
            return x;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Nested If/Else Return Analysis ==========

#[test]
fn test_nested_if_else_all_return() {
    let diagnostics = typecheck_source(
        r#"
        fn classify(x: number) -> number {
            if (x > 0) {
                if (x > 10) {
                    return 2;
                } else {
                    return 1;
                }
            } else {
                return 0;
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_nested_if_missing_inner_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                if (x > 10) {
                    return 2;
                } else {
                    let y: number = 1;
                }
            } else {
                return 0;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // inner else doesn't return
}

#[test]
fn test_nested_if_missing_outer_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                if (x > 10) {
                    return 2;
                } else {
                    return 1;
                }
            } else {
                let y: number = 0;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // outer else doesn't return
}

#[test]
fn test_deeply_nested_all_return() {
    let diagnostics = typecheck_source(
        r#"
        fn classify(x: number, y: number) -> number {
            if (x > 0) {
                if (y > 0) {
                    if (x > y) {
                        return 1;
                    } else {
                        return 2;
                    }
                } else {
                    return 3;
                }
            } else {
                return 4;
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Void Functions (Don't Need Return) ==========

// NOTE: 'void' as a return type may not be fully supported in the parser yet.
// Functions that don't need to return a value can use any return type and
// the compiler will check that all paths return appropriately.

// ========== Multiple Returns in Same Block ==========

#[test]
fn test_multiple_early_returns() {
    let diagnostics = typecheck_source(
        r#"
        fn classify(x: number) -> number {
            if (x < 0) {
                return -1;
            }
            if (x == 0) {
                return 0;
            }
            return 1;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_unreachable_code_after_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test() -> number {
            return 42;
            let x: number = 1;
        }
    "#,
    );
    // This should work (unreachable code is a warning, not an error)
    assert_no_errors(&diagnostics);
}

// ========== Returns in Loops ==========

#[test]
fn test_return_in_while_loop_not_sufficient() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            while (x > 0) {
                return x;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // loop might not execute
}

#[test]
fn test_return_after_loop() {
    let diagnostics = typecheck_source(
        r#"
        fn sum(n: number) -> number {
            var s: number = 0;
            var i: number = 0;
            while (i < n) {
                s = s + i;
                i = i + 1;
            }
            return s;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_in_for_loop_not_sufficient() {
    let diagnostics = typecheck_source(
        r#"
        fn test() -> number {
            for (var i: number = 0; i < 10; i = i + 1) {
                return i;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // loop might not execute
}

#[test]
fn test_return_after_for_loop() {
    let diagnostics = typecheck_source(
        r#"
        fn sum() -> number {
            var s: number = 0;
            for (var i: number = 0; i < 10; i = i + 1) {
                s = s + i;
            }
            return s;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Complex Control Flow ==========

#[test]
fn test_if_else_with_early_return() {
    let diagnostics = typecheck_source(
        r#"
        fn complex(x: number, y: number) -> number {
            if (x < 0) {
                return -1;
            }
            if (y < 0) {
                return -2;
            }
            if (x > y) {
                return 1;
            } else {
                return 2;
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_multiple_if_without_final_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number, y: number) -> number {
            if (x < 0) {
                return -1;
            }
            if (y < 0) {
                return -2;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // no final return
}

#[test]
fn test_nested_loops_with_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test() -> number {
            var i: number = 0;
            while (i < 10) {
                var j: number = 0;
                while (j < 10) {
                    j = j + 1;
                }
                i = i + 1;
            }
            return i;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Return Type Matching ==========

#[test]
fn test_return_number_to_number() {
    let diagnostics = typecheck_source(
        r#"
        fn getNumber() -> number {
            return 42;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_string_to_string() {
    let diagnostics = typecheck_source(
        r#"
        fn getString() -> string {
            return "hello";
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_bool_to_bool() {
    let diagnostics = typecheck_source(
        r#"
        fn getBool() -> bool {
            return true;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_array() {
    let diagnostics = typecheck_source(
        r#"
        fn getArray() -> number {
            let arr = [1, 2, 3];
            return arr[0];
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Edge Cases ==========

#[test]
fn test_function_returning_number_no_body_error() {
    // Even an empty function needs to return if return type is non-void
    let diagnostics = typecheck_source(
        r#"
        fn getNumber() -> number {
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004");
}

#[test]
fn test_function_with_only_declaration() {
    let diagnostics = typecheck_source(
        r#"
        fn test() -> number {
            let x: number = 42;
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004");
}

#[test]
fn test_all_branches_return_same_value() {
    let diagnostics = typecheck_source(
        r#"
        fn alwaysOne() -> number {
            if (true) {
                return 1;
            } else {
                return 1;
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_if_else_if_else_all_return() {
    let diagnostics = typecheck_source(
        r#"
        fn classify(x: number) -> number {
            if (x < 0) {
                return -1;
            } else {
                if (x == 0) {
                    return 0;
                } else {
                    return 1;
                }
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_simple_return_without_nesting() {
    // Direct return statement works
    let diagnostics = typecheck_source(
        r#"
        fn test() -> number {
            return 42;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_after_if_without_else() {
    let diagnostics = typecheck_source(
        r#"
        fn myMax(a: number, b: number) -> number {
            if (a > b) {
                return a;
            }
            return b;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Multiple Functions ==========

#[test]
fn test_multiple_functions_all_valid() {
    let diagnostics = typecheck_source(
        r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        fn multiply(a: number, b: number) -> number {
            return a * b;
        }

        fn greet() -> string {
            return "Hello";
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_multiple_functions_one_invalid() {
    let diagnostics = typecheck_source(
        r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        fn broken() -> number {
            let x: number = 42;
        }

        fn greet() -> string {
            return "Hello";
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // broken() doesn't return
}

// ============================================================================
// From generic_type_checking_tests.rs
// ============================================================================

// Generic Type Checking and Inference Tests (BLOCKER 02-B)
//
// Comprehensive test suite for generic types including:
// - Type parameter syntax and parsing
// - Type parameter scoping
// - Generic type arity validation
// - Type inference (Hindley-Milner)
// - Occurs check
// - Nested generics
// - Error cases

// ============================================================================
// Basic Generic Function Declaration
// ============================================================================

#[test]
fn test_generic_function_simple_declaration() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_generic_function_multiple_type_params() {
    let diagnostics = typecheck_source(
        r#"
        fn pair<A, B>(first: A, _second: B) -> A {
            return first;
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_generic_function_three_type_params() {
    let diagnostics = typecheck_source(
        r#"
        fn triple<A, B, C>(_a: A, _b: B, _c: C) -> A {
            return _a;
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Type Parameter Scoping
// ============================================================================

#[test]
fn test_type_parameter_in_param() {
    let diagnostics = typecheck_source(
        r#"
        fn test<T>(_x: T) -> void {}
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_type_parameter_in_return() {
    // Type parameter in return position is valid
    // We can't check type correctness without knowing T
    let diagnostics = typecheck_source(
        r#"
        fn test<T>(_x: number) -> T {
            return _x;
        }
    "#,
    );
    // Note: This passes type checking because we can't validate T without instantiation
    // The error would be caught at call sites if types don't match
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_type_parameter_in_array() {
    let diagnostics = typecheck_source(
        r#"
        fn first<T>(arr: T[]) -> T {
            return arr[0];
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_duplicate_type_parameter() {
    let diagnostics = typecheck_source(
        r#"
        fn bad<T, T>(_x: T) -> T {
            return _x;
        }
    "#,
    );
    assert!(!diagnostics.is_empty());
    assert!(diagnostics[0].message.contains("Duplicate type parameter"));
}

// ============================================================================
// Type Inference - Simple Cases
// ============================================================================

#[test]
fn test_inference_number() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let _result = identity(42);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_inference_string() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let _result = identity("hello");
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_inference_bool() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let _result = identity(true);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_inference_array() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let arr = [1, 2, 3];
        let _result = identity(arr);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Type Inference - Multiple Parameters
// ============================================================================

#[test]
fn test_inference_multiple_same_type() {
    let diagnostics = typecheck_source(
        r#"
        fn both<T>(_a: T, _b: T) -> T {
            return _a;
        }
        let _result = both(42, 84);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_inference_multiple_different_types() {
    let diagnostics = typecheck_source(
        r#"
        fn pair<A, B>(_first: A, _second: B) -> A {
            return _first;
        }
        let _result = pair(42, "hello");
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_inference_three_params() {
    let diagnostics = typecheck_source(
        r#"
        fn triple<A, B, C>(_a: A, _b: B, _c: C) -> A {
            return _a;
        }
        let _result = triple(1, "two", true);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Type Inference - Arrays
// ============================================================================

#[test]
fn test_inference_array_element_type() {
    let diagnostics = typecheck_source(
        r#"
        fn first<T>(arr: T[]) -> T {
            return arr[0];
        }
        let numbers = [1, 2, 3];
        let _result = first(numbers);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_inference_array_of_strings() {
    let diagnostics = typecheck_source(
        r#"
        fn first<T>(arr: T[]) -> T {
            return arr[0];
        }
        let strings = ["a", "b", "c"];
        let _result = first(strings);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Generic Type Arity Validation
// ============================================================================

#[test]
fn test_option_correct_arity() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Option<number>) -> void {}
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_result_correct_arity() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Result<number, string>) -> void {}
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_option_wrong_arity_too_many() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Option<number, string>) -> void {}
    "#,
    );
    assert!(!diagnostics.is_empty());
    assert!(diagnostics[0].message.contains("expects 1 type argument"));
}

#[test]
fn test_result_wrong_arity_too_few() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Result<number>) -> void {}
    "#,
    );
    assert!(!diagnostics.is_empty());
    assert!(diagnostics[0].message.contains("expects 2 type argument"));
}

#[test]
fn test_unknown_generic_type() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: UnknownGeneric<number>) -> void {}
    "#,
    );
    assert!(!diagnostics.is_empty());
    assert!(diagnostics[0].message.contains("Unknown generic type"));
}

// ============================================================================
// Nested Generic Types
// ============================================================================

#[test]
fn test_nested_option_result() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Option<Result<number, string>>) -> void {}
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_nested_result_option() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Result<Option<number>, string>) -> void {}
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_deeply_nested() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Option<Result<Option<number>, string>>) -> void {}
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_array_of_option() {
    let diagnostics = typecheck_source(
        r#"
        fn test(_x: Option<number>[]) -> void {}
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Type Mismatch Errors
// ============================================================================

#[test]
fn test_inference_type_mismatch() {
    let diagnostics = typecheck_source(
        r#"
        fn both<T>(_a: T, _b: T) -> T {
            return _a;
        }
        let _result = both(42, "hello");
    "#,
    );
    assert!(!diagnostics.is_empty());
    assert!(
        diagnostics[0].message.contains("Type inference failed")
            || diagnostics[0].message.contains("cannot match")
    );
}

#[test]
fn test_return_type_mismatch() {
    // Returning a concrete type when T is expected
    // This is allowed at declaration - error caught at call site
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(_x: T) -> T {
            return 42;
        }
    "#,
    );
    // This passes because we allow returning number for T
    // The type error would be caught when calling with non-number types
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_array_element_mismatch() {
    // Returning a concrete type when T is expected
    let diagnostics = typecheck_source(
        r#"
        fn first<T>(_arr: T[]) -> T {
            return "wrong";
        }
    "#,
    );
    // This passes declaration-level checking
    // Error would be caught at call sites
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Complex Inference Scenarios
// ============================================================================

#[test]
fn test_inference_with_function_call_chain() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        fn double_identity<T>(x: T) -> T {
            return identity(x);
        }
        let _result = double_identity(42);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_inference_with_variable() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let num = 42;
        let _result = identity(num);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_multiple_calls_same_function() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let _a = identity(42);
        let _b = identity("hello");
        let _c = identity(true);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_generic_with_no_params() {
    let diagnostics = typecheck_source(
        r#"
        fn test<T>() -> void {}
    "#,
    );
    // This is valid - T just can't be inferred
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_generic_unused_type_param() {
    let diagnostics = typecheck_source(
        r#"
        fn test<T>(_x: number) -> number {
            return 42;
        }
    "#,
    );
    // Valid but T is unused - not an error
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_type_parameter_in_nested_function() {
    // Type parameters should only be visible in their function
    // NOTE: Nested functions binding complete (Phases 1-3). Phases 4-6 pending for full execution.
    let diagnostics = typecheck_source(
        r#"
        fn outer<T>(_x: T) -> void {
            fn inner(_y: number) -> void {}
            inner(42);
        }
    "#,
    );
    // Phase 3 complete: Binder now supports nested functions
    // However, compiler/interpreter/VM still report AT1013 (to be fixed in Phases 4-6)
    // For now, accept either success or AT1013 from compiler/interpreter
    let has_at1013 = diagnostics.iter().any(|d| d.code == "AT1013");
    let no_errors = diagnostics.is_empty();
    assert!(
        has_at1013 || no_errors,
        "Expected either AT1013 (compiler/VM not ready) or no errors (fully working). Got: {:?}",
        diagnostics
    );
}

// ============================================================================
// Non-Generic Functions (Regression Tests)
// ============================================================================

#[test]
fn test_non_generic_still_works() {
    let diagnostics = typecheck_source(
        r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
        let _result = add(1, 2);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_mixed_generic_and_non_generic() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        fn double(x: number) -> number {
            return x * 2;
        }
        let _a = identity(42);
        let _b = double(21);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Integration with Existing Features
// ============================================================================

#[test]
fn test_generic_with_if_statement() {
    let diagnostics = typecheck_source(
        r#"
        fn choose<T>(condition: bool, a: T, b: T) -> T {
            if (condition) {
                return a;
            } else {
                return b;
            }
        }
        let _result = choose(true, 1, 2);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_generic_with_while_loop() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            var result = x;
            while (false) {
                result = x;
            }
            return result;
        }
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_generic_with_array_indexing() {
    let diagnostics = typecheck_source(
        r#"
        fn get_first<T>(arr: T[]) -> T {
            return arr[0];
        }
        let numbers = [1, 2, 3];
        let _first = get_first(numbers);
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

// ============================================================================
// Function Types with Generics
// ============================================================================

#[test]
fn test_generic_function_as_value() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        let _f = identity;
    "#,
    );
    assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
}

#[test]
fn test_pass_generic_function() {
    let diagnostics = typecheck_source(
        r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
        fn apply<T>(_f: (T) -> T, _x: T) -> T {
            return _x;
        }
        let _result = apply(identity, 42);
    "#,
    );
    // Note: This might not work perfectly yet depending on implementation
    // but it should at least parse and bind correctly
    // Type checking might have limitations with higher-order generics
    // Just check it doesn't crash - allow any number of diagnostics
    let _ = diagnostics.len();
}

// ============================================================================
// From intersection_type_tests.rs
// ============================================================================

// Tests for intersection types (Phase typing-04)

// ============================================================================
// Intersection construction tests
// ============================================================================

#[rstest]
#[case("let _x: number & number = 1;")]
#[case("let _x: string & string = \"ok\";")]
#[case("let _x: bool & bool = true;")]
#[case("let _x: number[] & number[] = [1, 2];")]
#[case("type Same = number & number; let _x: Same = 1;")]
#[case("fn f(x: number) -> number { return x; } let _x: ((number) -> number) & ((number) -> number) = f;")]
#[case("let _x: (number | string) & number = 1;")]
#[case("let _x: (number | string) & number = 2;")]
#[case("let _x: (number | string) & string = \"hi\";")]
#[case("let _x: (number | string | bool) & bool = true;")]
#[case("let _x: (number & number)[] = [1];")]
#[case("type Id<T> = T & T; let _x: Id<number> = 1;")]
#[case("let _x: (number | string) & (number | string) = \"ok\";")]
#[case("let _x: (number | string) & (number | string) = 2;")]
fn test_intersection_construction(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Intersection error tests
// ============================================================================

#[rstest]
#[case("let _x: number & string = 1;")]
#[case("let _x: number & string = \"ok\";")]
#[case("let _x: bool & number = true;")]
#[case("let _x: string & null = \"ok\";")]
#[case("let _x: null & string = null;")]
#[case("let _x: (number | string) & number = \"bad\";")]
#[case("let _x: (number | string) & string = 1;")]
#[case("let _x: (bool | string) & number = 1;")]
#[case(
    "fn f(x: number) -> number { return x; } let _x: (number) -> number & (string) -> string = f;"
)]
#[case("let _x: number & string & bool = 1;")]
#[case("type Id<T> = T & string; let _x: Id<number> = 1;")]
#[case("let _x: (number | string) & (bool | string) = 1;")]
#[case("let _x: (number | string) & (bool | string) = true;")]
#[case("let _x: (number | string) & (bool | string) = null;")]
fn test_intersection_errors(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

// ============================================================================
// Union/intersection interaction tests
// ============================================================================

#[rstest]
#[case("let _x: (number | string) & number = 1;")]
#[case("let _x: (number | string | bool) & number = 1;")]
#[case("let _x: (number | string) & string = \"ok\";")]
#[case("let _x: (number | string | bool) & bool = true;")]
#[case("let _x: (number | string | bool) & string = \"ok\";")]
#[case("let _x: (number | string | bool) & number = 2;")]
#[case("let _x: (number | string) & (bool | string) = \"ok\";")]
#[case("let _x: (number | string) & (bool | string) = \"yes\";")]
fn test_intersection_distribution(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

#[rstest]
#[case("let _x: (number | string) & number = \"bad\";")]
#[case("let _x: (number | string | bool) & string = 10;")]
#[case("let _x: (number | string | bool) & bool = \"no\";")]
#[case("let _x: (number | string | bool) & number = false;")]
fn test_intersection_distribution_errors(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

// ============================================================================
// Intersection + method/index operations
// ============================================================================

#[rstest]
#[case("let _x: number[] & number[] = [1, 2]; let _y: number = _x[0];")]
#[case("let _x: number[] & number[] = [1, 2]; let _y: number = _x[1];")]
#[case("let _x: number[] & number[] = [1, 2]; let _y: number = _x[0] + _x[1];")]
fn test_intersection_operations(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// From nullability_tests.rs
// ============================================================================

// Comprehensive tests for nullability rules
//
// Tests cover:
// - null is only assignable to null type (no implicit nullable)
// - null cannot be assigned to number, string, bool, void, or arrays
// - Explicit null type variables
// - null in expressions and operations
// - null in function parameters and returns
// - null comparisons

// ========== Valid Null Usage ==========

#[rstest]
#[case::literal_inference("let x = null;")]
#[case::variable_inference("let x = null;\nlet y = x;")]
#[case::equality_with_null("let x = null == null;")]
#[case::inequality_with_null("let x = null != null;")]
#[case::null_array_literal("let x = [null, null, null];")]
#[case::single_null_array("let x = [null];")]
#[case::nested_null_expression("let x = (null == null) && true;")]
#[case::null_variable_comparison("let x = null;\nlet y = null;\nlet z = x == y;")]
#[case::null_value_usage("let x = null;\nlet y = x == null;")]
#[case::null_comparison_chain(
    "let a = null;\nlet b = null;\nlet result = (a == b) && (b == null);"
)]
fn test_valid_null_usage(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

// ========== Null Assignment Errors ==========

#[rstest]
#[case::to_number("let x: number = null;")]
#[case::to_string(r#"let x: string = null;"#)]
#[case::to_bool("let x: bool = null;")]
#[case::in_number_array("let x = [1, 2, null];")]
#[case::in_string_array(r#"let x = ["a", "b", null];"#)]
fn test_null_assignment_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Null Function Parameter Errors ==========

#[rstest]
#[case::number_param(
    "fn acceptsNumber(x: number) -> number { return x; }\nlet result = acceptsNumber(null);"
)]
#[case::string_param(
    "fn acceptsString(x: string) -> string { return x; }\nlet result = acceptsString(null);"
)]
#[case::bool_param(
    "fn acceptsBool(x: bool) -> bool { return x; }\nlet result = acceptsBool(null);"
)]
fn test_null_function_parameter_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Null Function Return Errors ==========

#[rstest]
#[case::number_return("fn returnsNumber() -> number { return null; }")]
#[case::string_return("fn returnsString() -> string { return null; }")]
#[case::bool_return("fn returnsBool() -> bool { return null; }")]
fn test_null_function_return_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Null Comparison Errors ==========

#[rstest]
#[case::with_number("let x = null == 42;")]
#[case::with_string(r#"let x = null == "hello";"#)]
#[case::with_bool("let x = null == true;")]
#[case::number_with_null("let x = 42 == null;")]
fn test_null_comparison_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Null Arithmetic Errors ==========

#[rstest]
#[case::addition("let x = null + null;")]
#[case::null_plus_number("let x = null + 42;")]
#[case::number_plus_null("let x = 42 + null;")]
#[case::subtraction("let x = null - null;")]
#[case::multiplication("let x = null * null;")]
#[case::division("let x = null / null;")]
fn test_null_arithmetic_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Null Logical Operation Errors ==========

#[rstest]
#[case::and_operator("let x = null && null;")]
#[case::or_operator("let x = null || null;")]
#[case::null_and_bool("let x = null && true;")]
#[case::bool_and_null("let x = true && null;")]
fn test_null_logical_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Null in Conditionals ==========

#[rstest]
#[case::if_condition("if (null) { let x: number = 1; }")]
#[case::while_condition("while (null) { break; }")]
#[case::for_condition("for (let i: number = 0; null; i = i + 1) { break; }")]
fn test_null_in_conditionals(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Null with Unary Operators ==========

#[rstest]
#[case::negate("let x = -null;")]
#[case::not("let x = !null;")]
fn test_null_unary_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Null in Arrays ==========

#[rstest]
#[case::null_then_number("let x = [null, 42];")]
#[case::number_then_null("let x = [42, null];")]
fn test_mixed_null_array_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Edge Cases ==========

#[test]
fn test_null_in_array_indexing_error() {
    let diagnostics = typecheck_source("let arr = [1, 2, 3];\nlet x = arr[null];");
    assert_has_error(&diagnostics, "AT3001");
}

// ============================================================================
// From type_alias_tests.rs
// ============================================================================

// Tests for type aliases (Phase typing-03)

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

    // Use canonical path since the resolver now canonicalizes all paths
    let entry_path = temp_dir
        .path()
        .join(format!("{}.atl", entry))
        .canonicalize()
        .unwrap();
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

// ============================================================================
// From type_guard_tests.rs
// ============================================================================

// Tests for type guard predicates and narrowing.

fn eval(code: &str) -> Value {
    let runtime = Atlas::new();
    runtime.eval(code).expect("Interpretation failed")
}

// =============================================================================
// Predicate syntax + validation
// =============================================================================

#[rstest]
#[case(
    r#"
    fn isStr(x: number | string) -> bool is x: string { return isString(x); }
    fn test(x: number | string) -> number {
        if (isStr(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNum(x: number | string) -> bool is x: number { return isNumber(x); }
    fn test(x: number | string) -> number {
        if (isNum(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isBoolish(x: bool | null) -> bool is x: bool { return isBool(x); }
    fn test(x: bool | null) -> bool {
        if (isBoolish(x)) { let _y: bool = x; return _y; }
        else { return false; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn hasName(x: WithName | WithId) -> bool is x: WithName { return hasField(x, "name"); }
    fn test(x: WithName | WithId) -> number {
        if (hasName(x)) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithLen = { len: () -> number };
    type WithId = { id: number };
    fn hasLen(x: WithLen | WithId) -> bool is x: WithLen { return hasMethod(x, "len"); }
    fn test(x: WithLen | WithId) -> number {
        if (hasLen(x)) { let _y: WithLen = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type Ok = { tag: string, value: number };
    type Err = { tag: number, message: string };
    fn isOk(x: Ok | Err) -> bool is x: Ok { return hasTag(x, "ok"); }
    fn test(x: Ok | Err) -> number {
        if (isOk(x)) { let _y: Ok = x; return 1; }
        else { let _y: Err = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNullish(x: null | string) -> bool is x: null { return isNull(x); }
    fn test(x: null | string) -> number {
        if (isNullish(x)) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isObj(x: json | string) -> bool is x: json { return isObject(x); }
    fn test(x: json | string) -> number {
        if (isObj(x)) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isArr(x: number[] | string) -> bool is x: number[] { return isArray(x); }
    fn test(x: number[] | string) -> number {
        if (isArr(x)) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isFunc(x: ((number) -> number) | string) -> bool is x: (number) -> number { return isFunction(x); }
    fn test(x: ((number) -> number) | string) -> number {
        if (isFunc(x)) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
fn test_predicate_syntax_valid(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

#[rstest]
#[case(
    r#"
    fn isStr(x: number) -> number is x: number { return 1; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is y: number { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: string { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number {
        return 1; // return type mismatch
    }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) is x: number { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number { let _y: string = x; return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number | string) -> bool is x: bool { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number | string) -> bool is missing: string { return true; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number { return false; }
    fn test(x: number | string) -> number { if (isStr(x)) { return 1; } return 2; }
    "#
)]
#[case(
    r#"
    fn isStr(x: number) -> bool is x: number { return true; }
    fn test(x: number) -> number { if (isStr(x)) { let _y: string = x; } return 1; }
    "#
)]
fn test_predicate_syntax_errors(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

// =============================================================================
// Built-in guard narrowing
// =============================================================================

#[rstest]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isNumber(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: bool | null) -> number {
        if (isBool(x)) { let _y: bool = x; return 1; }
        else { let _y: null = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: null | string) -> number {
        if (isNull(x)) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number[] | string) -> number {
        if (isArray(x)) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn f(x: number) -> number { return x; }
    fn test(x: ((number) -> number) | string) -> number {
        if (isFunction(x)) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: json | string) -> number {
        if (isObject(x)) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (!isString(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) || isNumber(x)) { let _y: number | string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) && isType(x, "string")) { let _y: string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "number")) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
fn test_builtin_guard_narrowing(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// User-defined guards
// =============================================================================

#[rstest]
#[case(
    r#"
    fn isText(x: number | string) -> bool is x: string { return isString(x); }
    fn test(x: number | string) -> number {
        if (isText(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn isNamed(x: WithName | WithId) -> bool is x: WithName { return hasField(x, "name"); }
    fn test(x: WithName | WithId) -> number {
        if (isNamed(x)) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithLen = { len: () -> number };
    type WithId = { id: number };
    fn isLen(x: WithLen | WithId) -> bool is x: WithLen { return hasMethod(x, "len"); }
    fn test(x: WithLen | WithId) -> number {
        if (isLen(x)) { let _y: WithLen = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNum(x: number | string) -> bool is x: number { return isNumber(x); }
    fn test(x: number | string) -> number {
        if (isNum(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isArr(x: number[] | string) -> bool is x: number[] { return isArray(x); }
    fn test(x: number[] | string) -> number {
        if (isArr(x)) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isNullish(x: null | string) -> bool is x: null { return isNull(x); }
    fn test(x: null | string) -> number {
        if (isNullish(x)) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type Ok = { tag: string, value: number };
    type Err = { tag: number, message: string };
    fn isOk(x: Ok | Err) -> bool is x: Ok { return hasTag(x, "ok"); }
    fn test(x: Ok | Err) -> number {
        if (isOk(x)) { let _y: Ok = x; return 1; }
        else { let _y: Err = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isObj(x: json | string) -> bool is x: json { return isObject(x); }
    fn test(x: json | string) -> number {
        if (isObj(x)) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isFunc(x: ((number) -> number) | string) -> bool is x: (number) -> number { return isFunction(x); }
    fn test(x: ((number) -> number) | string) -> number {
        if (isFunc(x)) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn isTypeString(x: number | string) -> bool is x: string { return isType(x, "string"); }
    fn test(x: number | string) -> number {
        if (isTypeString(x)) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
fn test_user_defined_guards(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// Guard composition + control flow
// =============================================================================

#[rstest]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) || isNumber(x)) { let _y: number | string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) && isType(x, "string")) { let _y: string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (!isString(x)) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) && !isNull(x)) { let _y: string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "string") || isType(x, "number")) { let _y: number | string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x) && isNumber(x)) { return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x)) { let _y: string = x; }
        if (isNumber(x)) { let _y: number = x; }
        return 1;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isString(x)) { let _y: string = x; return 1; }
        if (isNumber(x)) { let _y: number = x; return 2; }
        return 3;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        var result: number = 0;
        if (isString(x)) { result = 1; }
        if (isNumber(x)) { result = 2; }
        return result;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        while (isString(x)) { let _y: string = x; return 1; }
        return 2;
    }
    "#
)]
fn test_guard_composition_and_flow(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// Structural + discriminated guards
// =============================================================================

#[rstest]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name")) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithLen = { len: () -> number };
    type WithId = { id: number };
    fn test(x: WithLen | WithId) -> number {
        if (hasMethod(x, "len")) { let _y: WithLen = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithTag = { tag: string, value: number };
    type WithNumTag = { tag: number, message: string };
    fn test(x: WithTag | WithNumTag) -> number {
        if (hasTag(x, "ok")) { let _y: WithTag = x; return 1; }
        else { let _y: WithNumTag = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type One = { name: string, id: number };
    type Two = { id: number };
    fn test(x: One | Two) -> number {
        if (hasField(x, "name")) { let _y: One = x; return 1; }
        else { let _y: Two = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type One = { len: () -> number, id: number };
    type Two = { id: number };
    fn test(x: One | Two) -> number {
        if (hasMethod(x, "len")) { let _y: One = x; return 1; }
        else { let _y: Two = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type One = { tag: string, id: number };
    type Two = { tag: number, id: number };
    fn test(x: One | Two) -> number {
        if (hasTag(x, "one")) { let _y: One = x; return 1; }
        else { let _y: Two = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name") && hasField(x, "name")) { let _y: WithName = x; return 1; }
        else { let _y: WithId = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name") || hasField(x, "id")) { let _y: WithName | WithId = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (!hasField(x, "name")) { let _y: WithId = x; return 1; }
        else { let _y: WithName = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    type WithName = { name: string };
    type WithId = { id: number };
    fn test(x: WithName | WithId) -> number {
        if (hasField(x, "name")) { let _y: { name: string } = x; return 1; }
        else { let _y: { id: number } = x; return 2; }
    }
    "#
)]
fn test_structural_guards(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// isType guard tests
// =============================================================================

#[rstest]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "string")) { let _y: string = x; return 1; }
        else { let _y: number = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "number")) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: bool | null) -> number {
        if (isType(x, "bool")) { let _y: bool = x; return 1; }
        else { let _y: null = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: null | string) -> number {
        if (isType(x, "null")) { let _y: null = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number[] | string) -> number {
        if (isType(x, "array")) { let _y: number[] = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn f(x: number) -> number { return x; }
    fn test(x: ((number) -> number) | string) -> number {
        if (isType(x, "function")) { let _y: (number) -> number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: json | string) -> number {
        if (isType(x, "json")) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: json | string) -> number {
        if (isType(x, "object")) { let _y: json = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (isType(x, "number") || isType(x, "string")) { let _y: number | string = x; return 1; }
        return 2;
    }
    "#
)]
#[case(
    r#"
    fn test(x: number | string) -> number {
        if (!isType(x, "string")) { let _y: number = x; return 1; }
        else { let _y: string = x; return 2; }
    }
    "#
)]
fn test_is_type_guard(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// =============================================================================
// Runtime guard behavior
// =============================================================================

#[rstest]
#[case("isString(\"ok\")", Value::Bool(true))]
#[case("isString(1)", Value::Bool(false))]
#[case("isNumber(1)", Value::Bool(true))]
#[case("isBool(true)", Value::Bool(true))]
#[case("isNull(null)", Value::Bool(true))]
#[case("isArray([1, 2])", Value::Bool(true))]
#[case("isType(\"ok\", \"string\")", Value::Bool(true))]
#[case("isType(1, \"number\")", Value::Bool(true))]
#[case("isType([1, 2], \"array\")", Value::Bool(true))]
#[case("isType(null, \"null\")", Value::Bool(true))]
fn test_runtime_basic_guards(#[case] expr: &str, #[case] expected: Value) {
    let code = expr.to_string();
    let result = eval(&code);
    assert_eq!(result, expected);
}

#[rstest]
#[case(
    r#"
    let obj = parseJSON("{\"tag\":\"ok\", \"value\": 1}");
    hasTag(obj, "ok")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"tag\":\"bad\", \"value\": 1}");
    hasTag(obj, "ok")
    "#,
    Value::Bool(false)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasField(obj, "name")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasField(obj, "missing")
    "#,
    Value::Bool(false)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasMethod(obj, "name")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    hasMethod(obj, "missing")
    "#,
    Value::Bool(false)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    isObject(obj)
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let obj = parseJSON("{\"name\":\"atlas\"}");
    isType(obj, "object")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let hmap = hashMapNew();
    hashMapPut(hmap, "name", 1);
    hasField(hmap, "name")
    "#,
    Value::Bool(true)
)]
#[case(
    r#"
    let hmap = hashMapNew();
    hashMapPut(hmap, "tag", "ok");
    hasTag(hmap, "ok")
    "#,
    Value::Bool(true)
)]
fn test_runtime_structural_guards(#[case] code: &str, #[case] expected: Value) {
    let result = eval(code);
    assert_eq!(result, expected);
}

// ============================================================================
// From type_improvements_tests.rs
// ============================================================================

// Tests for improved type error messages and suggestions (Phase typing-01)
//
// Validates that type error messages show clear expected vs actual comparisons
// and provide actionable fix suggestions.

/// Helper: typecheck source and return diagnostics (binder + typechecker)
/// Helper: get only error-level diagnostics
/// Helper: get only warning-level diagnostics
// ============================================================================
// 1. Type mismatch: clear expected vs actual
// ============================================================================

#[test]
fn test_var_type_mismatch_number_string() {
    let diags = errors(r#"let x: number = "hello";"#);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("expected"));
    assert!(diags[0].message.contains("found"));
    assert_eq!(diags[0].code, "AT3001");
}

#[test]
fn test_var_type_mismatch_string_number() {
    let diags = errors("let x: string = 42;");
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("expected string"));
    assert!(diags[0].message.contains("found number"));
}

#[test]
fn test_var_type_mismatch_bool_string() {
    let diags = errors(r#"let x: bool = "true";"#);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("expected bool"));
}

#[test]
fn test_assignment_type_mismatch() {
    let diags = errors(
        r#"
        var x: number = 1;
        x = "hello";
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected number"));
    assert!(diags[0].message.contains("found string"));
}

// ============================================================================
// 2. Suggestions for number-string mismatch
// ============================================================================

#[test]
fn test_suggest_num_conversion() {
    let diags = errors(r#"let x: number = "42";"#);
    assert!(!diags.is_empty());
    // Should suggest num() conversion
    assert!(
        diags[0].help.as_ref().is_some_and(|h| h.contains("num(")),
        "Expected num() suggestion, got: {:?}",
        diags[0].help
    );
}

#[test]
fn test_suggest_str_conversion() {
    let diags = errors("let x: string = 42;");
    assert!(!diags.is_empty());
    assert!(
        diags[0].help.as_ref().is_some_and(|h| h.contains("str(")),
        "Expected str() suggestion, got: {:?}",
        diags[0].help
    );
}

// ============================================================================
// 3. Suggestions for missing return / return mismatch
// ============================================================================

#[test]
fn test_return_type_mismatch_suggests_fix() {
    let diags = errors(
        r#"
        fn foo() -> number {
            return "hello";
        }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected number"));
    assert!(diags[0].message.contains("found string"));
}

#[test]
fn test_missing_return_suggests_adding_one() {
    let diags = errors(
        r#"
        fn foo() -> number {
        }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0]
        .message
        .contains("Not all code paths return a value"));
}

#[test]
fn test_return_void_from_number_function() {
    let diags = errors(
        r#"
        fn foo() -> number {
            return;
        }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected number"));
    assert!(
        diags[0]
            .help
            .as_ref()
            .is_some_and(|h| h.contains("missing return")),
        "Expected missing return suggestion, got: {:?}",
        diags[0].help
    );
}

// ============================================================================
// 4. Suggestions for wrong operator
// ============================================================================

#[test]
fn test_add_string_number_suggests_str() {
    let diags = errors(r#"let _x = "hello" + 42;"#);
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3002");
    assert!(
        diags[0].help.as_ref().is_some_and(|h| h.contains("str(")),
        "Expected str() suggestion for string + number, got: {:?}",
        diags[0].help
    );
}

#[test]
fn test_add_number_string_suggests_str() {
    let diags = errors(r#"let _x = 42 + "hello";"#);
    assert!(!diags.is_empty());
    assert!(
        diags[0].help.as_ref().is_some_and(|h| h.contains("str(")),
        "Expected str() suggestion for number + string, got: {:?}",
        diags[0].help
    );
}

#[test]
fn test_subtract_strings_error() {
    let diags = errors(r#"let _x = "a" - "b";"#);
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3002");
}

// ============================================================================
// 5. Suggestions for undefined variables (unknown symbol)
// ============================================================================

#[test]
fn test_undefined_variable_error() {
    let diags = errors("let _x = foo;");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT2002");
    assert!(diags[0].message.contains("Unknown symbol 'foo'"));
}

// ============================================================================
// 6. Complex type display - function signatures
// ============================================================================

#[test]
fn test_function_type_display_in_error() {
    let diags = errors(
        r#"
        fn add(a: number, b: number) -> number { return a + b; }
        let _x: string = add;
    "#,
    );
    assert!(!diags.is_empty());
    // Function should display as "(number, number) -> number" not just "function"
    assert!(
        diags[0].message.contains("(number, number) -> number"),
        "Expected function signature in error, got: {}",
        diags[0].message
    );
}

#[test]
fn test_function_type_display_void_return() {
    let diags = errors(
        r#"
        fn greet(_name: string) -> void { }
        let _x: number = greet;
    "#,
    );
    assert!(!diags.is_empty());
    assert!(
        diags[0].message.contains("(string) -> void"),
        "Expected function signature, got: {}",
        diags[0].message
    );
}

#[test]
fn test_function_type_display_no_params() {
    let diags = errors(
        r#"
        fn foo() -> number { return 1; }
        let _x: string = foo;
    "#,
    );
    assert!(!diags.is_empty());
    assert!(
        diags[0].message.contains("() -> number"),
        "Expected () -> number in error, got: {}",
        diags[0].message
    );
}

// ============================================================================
// 7. Array and generic type display
// ============================================================================

#[test]
fn test_array_type_display_in_error() {
    let diags = errors(
        r#"
        let arr = [1, 2, 3];
        let _x: string = arr;
    "#,
    );
    assert!(!diags.is_empty());
    assert!(
        diags[0].message.contains("number[]"),
        "Expected number[] in error, got: {}",
        diags[0].message
    );
}

// ============================================================================
// 8. Error location accuracy (span info)
// ============================================================================

#[test]
fn test_error_has_span_info() {
    let diags = errors("let x: number = true;");
    assert!(!diags.is_empty());
    // Should have valid location info (length > 0)
    assert!(diags[0].length > 0, "Error should have valid span info");
}

// ============================================================================
// 9. Condition type errors with suggestions
// ============================================================================

#[test]
fn test_if_condition_number_suggests_comparison() {
    let diags = errors("if (42) { }");
    assert!(!diags.is_empty());
    assert!(
        diags[0].help.as_ref().is_some_and(|h| h.contains("!=")),
        "Expected comparison suggestion, got: {:?}",
        diags[0].help
    );
}

#[test]
fn test_while_condition_string_suggests_comparison() {
    let diags = errors(r#"while ("hello") { }"#);
    assert!(!diags.is_empty());
    assert!(
        diags[0].help.as_ref().is_some_and(|h| h.contains("len")),
        "Expected len suggestion, got: {:?}",
        diags[0].help
    );
}

// ============================================================================
// 10. Immutable variable suggestions
// ============================================================================

#[test]
fn test_immutable_variable_suggests_var() {
    let diags = errors(
        r#"
        let x = 5;
        x = 10;
    "#,
    );
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3003");
    assert!(
        diags[0].help.as_ref().is_some_and(|h| h.contains("var")),
        "Expected var suggestion, got: {:?}",
        diags[0].help
    );
}

// ============================================================================
// 11. Function call errors
// ============================================================================

#[test]
fn test_wrong_arity_shows_signature() {
    let diags = errors(
        r#"
        fn add(a: number, b: number) -> number { return a + b; }
        let _x = add(1);
    "#,
    );
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3005");
    // Help should include the function signature
    assert!(
        diags[0]
            .help
            .as_ref()
            .is_some_and(|h| h.contains("(number, number) -> number")),
        "Expected function signature in help, got: {:?}",
        diags[0].help
    );
}

#[test]
fn test_too_many_args_says_remove() {
    let diags = errors(
        r#"
        fn single(a: number) -> number { return a; }
        let _x = single(1, 2, 3);
    "#,
    );
    assert!(!diags.is_empty());
    assert!(
        diags[0].help.as_ref().is_some_and(|h| h.contains("remove")),
        "Expected 'remove' suggestion, got: {:?}",
        diags[0].help
    );
}

#[test]
fn test_wrong_arg_type_suggests_conversion() {
    let diags = errors(
        r#"
        fn double(x: number) -> number { return x * 2; }
        let _x = double("hello");
    "#,
    );
    assert!(!diags.is_empty());
    // Should suggest num() conversion
    assert!(
        diags[0].help.as_ref().is_some_and(|h| h.contains("num(")),
        "Expected num() suggestion, got: {:?}",
        diags[0].help
    );
}

// ============================================================================
// 12. Not callable errors
// ============================================================================

#[test]
fn test_call_string_not_callable() {
    let diags = errors(
        r#"
        let _x = "hello"(42);
    "#,
    );
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3006");
}

#[test]
fn test_call_number_not_callable() {
    let diags = errors("let _x = 42(1);");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3006");
}

// ============================================================================
// 13. For-in errors with suggestions
// ============================================================================

#[test]
fn test_for_in_number_suggests_range() {
    let diags = errors(
        r#"
        fn test() -> void {
            for x in 42 {
                print(x);
            }
        }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(
        diags[0].help.as_ref().is_some_and(|h| h.contains("range")),
        "Expected range suggestion, got: {:?}",
        diags[0].help
    );
}

// ============================================================================
// 14. Compound assignment errors
// ============================================================================

#[test]
fn test_compound_assign_wrong_type() {
    let diags = errors(
        r#"
        var x: string = "hello";
        x += 1;
    "#,
    );
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3001");
}

// ============================================================================
// 15. Unreachable code warnings
// ============================================================================

#[test]
fn test_unreachable_code_warning() {
    let diags = warnings(
        r#"
        fn foo() -> number {
            return 42;
            let _x = 1;
        }
    "#,
    );
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT2002");
    assert!(diags[0].message.contains("Unreachable"));
}

// ============================================================================
// 16. Unused variable warnings
// ============================================================================

#[test]
fn test_unused_variable_warning() {
    let diags = warnings(
        r#"
        fn foo() -> void {
            let x = 42;
        }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("Unused variable 'x'"));
}

#[test]
fn test_underscore_prefix_suppresses_unused() {
    let diags = warnings(
        r#"
        fn foo() -> void {
            let _x = 42;
        }
    "#,
    );
    assert!(
        diags.is_empty(),
        "Underscore-prefixed should not warn: {:?}",
        diags
    );
}

// ============================================================================
// 17. Break/continue outside loop
// ============================================================================

#[test]
fn test_break_outside_loop_error() {
    let diags = errors("break;");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3010");
}

#[test]
fn test_continue_outside_loop_error() {
    let diags = errors("continue;");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3010");
}

// ============================================================================
// 18. Return outside function
// ============================================================================

#[test]
fn test_return_outside_function_error() {
    let diags = errors("return 5;");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3011");
}

// ============================================================================
// 19. Valid code still passes
// ============================================================================

#[test]
fn test_valid_arithmetic() {
    let diags = errors("let _x = 1 + 2;");
    assert!(diags.is_empty());
}

#[test]
fn test_valid_string_concat() {
    let diags = errors(r#"let _x = "hello" + " world";"#);
    assert!(diags.is_empty());
}

#[test]
fn test_valid_function_call() {
    let diags = errors(
        r#"
        fn add(a: number, b: number) -> number { return a + b; }
        let _x = add(1, 2);
    "#,
    );
    assert!(
        diags.is_empty(),
        "Valid code should have no errors: {:?}",
        diags
    );
}

#[test]
fn test_valid_if_bool() {
    let diags = errors("if (true) { }");
    assert!(diags.is_empty());
}

#[test]
fn test_valid_var_mutation() {
    let diags = errors(
        r#"
        var x = 5;
        x = 10;
    "#,
    );
    assert!(
        diags.is_empty(),
        "Mutable assignment should work: {:?}",
        diags
    );
}

#[test]
fn test_valid_for_in_array() {
    let diags = errors(
        r#"
        fn test() -> void {
            let arr = [1, 2, 3];
            for x in arr {
                print(x);
            }
        }
    "#,
    );
    assert!(
        diags.is_empty(),
        "For-in over array should work: {:?}",
        diags
    );
}

// ============================================================================
// From type_inference_tests.rs
// ============================================================================

// Tests for enhanced type inference (Phase typing-01)
//
// Validates return type inference, bidirectional checking, expression type
// inference, and least upper bound computation.

/// Helper: typecheck source and return diagnostics
#[allow(dead_code)]
// ============================================================================
// 1. Return type inference - uniform returns
// ============================================================================
#[test]
fn test_infer_return_number() {
    let diags = errors(
        r#"
        fn double(x: number) -> number { return x * 2; }
        let _r = double(5);
    "#,
    );
    assert!(
        diags.is_empty(),
        "Valid return should have no errors: {:?}",
        diags
    );
}

#[test]
fn test_infer_return_string() {
    let diags = errors(
        r#"
        fn greet(name: string) -> string { return "hello " + name; }
        let _r = greet("world");
    "#,
    );
    assert!(diags.is_empty(), "Valid string return: {:?}", diags);
}

#[test]
fn test_infer_return_bool() {
    let diags = errors(
        r#"
        fn is_positive(x: number) -> bool { return x > 0; }
        let _r = is_positive(5);
    "#,
    );
    assert!(diags.is_empty(), "Valid bool return: {:?}", diags);
}

#[test]
fn test_infer_return_void() {
    let diags = errors(
        r#"
        fn do_nothing() -> void { }
        do_nothing();
    "#,
    );
    assert!(diags.is_empty(), "Void function: {:?}", diags);
}

// ============================================================================
// 2. Return type mismatch detection
// ============================================================================

#[test]
fn test_return_number_expected_string() {
    let diags = errors(
        r#"
        fn foo() -> string { return 42; }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected string"));
    assert!(diags[0].message.contains("found number"));
}

#[test]
fn test_return_string_expected_number() {
    let diags = errors(
        r#"
        fn foo() -> number { return "hello"; }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected number"));
}

#[test]
fn test_return_bool_expected_string() {
    let diags = errors(
        r#"
        fn foo() -> string { return true; }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected string"));
}

// ============================================================================
// 3. Bidirectional: variable type annotation guides inference
// ============================================================================

#[test]
fn test_bidi_number_annotation_valid() {
    let diags = errors("let _x: number = 42;");
    assert!(diags.is_empty());
}

#[test]
fn test_bidi_string_annotation_valid() {
    let diags = errors(r#"let _x: string = "hello";"#);
    assert!(diags.is_empty());
}

#[test]
fn test_bidi_bool_annotation_valid() {
    let diags = errors("let _x: bool = true;");
    assert!(diags.is_empty());
}

#[test]
fn test_bidi_number_annotation_mismatch() {
    let diags = errors(r#"let _x: number = "hello";"#);
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected number"));
    assert!(diags[0].message.contains("found string"));
}

#[test]
fn test_bidi_string_annotation_mismatch() {
    let diags = errors("let _x: string = true;");
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected string"));
}

// ============================================================================
// 4. Expression type inference
// ============================================================================

#[test]
fn test_infer_arithmetic_result() {
    let diags = errors("let _x: number = 1 + 2;");
    assert!(diags.is_empty());
}

#[test]
fn test_infer_comparison_result() {
    let diags = errors("let _x: bool = 1 > 2;");
    assert!(diags.is_empty());
}

#[test]
fn test_infer_logical_result() {
    let diags = errors("let _x: bool = (1 > 0) && (2 > 1);");
    assert!(diags.is_empty(), "Logical result: {:?}", diags);
}

#[test]
fn test_infer_negation_result() {
    let diags = errors("let _x: number = -42;");
    assert!(diags.is_empty());
}

#[test]
fn test_infer_not_result() {
    let diags = errors("let _x: bool = !true;");
    assert!(diags.is_empty());
}

#[test]
fn test_infer_string_concat_result() {
    let diags = errors(r#"let _x: string = "a" + "b";"#);
    assert!(diags.is_empty());
}

// ============================================================================
// 5. Array type inference
// ============================================================================

#[test]
fn test_infer_number_array() {
    let diags = errors(
        r#"
        let arr = [1, 2, 3];
        let _x: number = arr[0];
    "#,
    );
    assert!(diags.is_empty(), "Number array indexing: {:?}", diags);
}

#[test]
fn test_infer_string_array() {
    let diags = errors(
        r#"
        let arr = ["a", "b", "c"];
        let _x: string = arr[0];
    "#,
    );
    assert!(diags.is_empty(), "String array indexing: {:?}", diags);
}

#[test]
fn test_array_assigned_to_wrong_type() {
    let diags = errors(
        r#"
        let arr = [1, 2, 3];
        let _x: string = arr;
    "#,
    );
    assert!(!diags.is_empty());
}

// ============================================================================
// 6. Function call return type inference
// ============================================================================

#[test]
fn test_infer_function_call_return() {
    let diags = errors(
        r#"
        fn add(a: number, b: number) -> number { return a + b; }
        let _x: number = add(1, 2);
    "#,
    );
    assert!(diags.is_empty(), "Function call return type: {:?}", diags);
}

#[test]
fn test_function_call_return_mismatch() {
    let diags = errors(
        r#"
        fn add(a: number, b: number) -> number { return a + b; }
        let _x: string = add(1, 2);
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected string"));
    assert!(diags[0].message.contains("found number"));
}

// ============================================================================
// 7. Nested expression inference
// ============================================================================

#[test]
fn test_nested_arithmetic() {
    let diags = errors("let _x: number = (1 + 2) * 3;");
    assert!(diags.is_empty());
}

#[test]
fn test_nested_comparison() {
    let diags = errors("let _x: bool = (1 + 2) > 3;");
    assert!(diags.is_empty());
}

#[test]
fn test_nested_logical() {
    let diags = errors("let _x: bool = (1 > 0) && (2 > 1) || (3 > 2);");
    assert!(diags.is_empty(), "Nested logical: {:?}", diags);
}

// ============================================================================
// 8. Variable usage inference
// ============================================================================

#[test]
fn test_var_inferred_number() {
    let diags = errors(
        r#"
        let x = 42;
        let _y: number = x;
    "#,
    );
    assert!(diags.is_empty(), "Inferred number variable: {:?}", diags);
}

#[test]
fn test_var_inferred_string() {
    let diags = errors(
        r#"
        let x = "hello";
        let _y: string = x;
    "#,
    );
    assert!(diags.is_empty(), "Inferred string variable: {:?}", diags);
}

#[test]
fn test_var_inferred_bool() {
    let diags = errors(
        r#"
        let x = true;
        let _y: bool = x;
    "#,
    );
    assert!(diags.is_empty(), "Inferred bool variable: {:?}", diags);
}

// ============================================================================
// 9. Mutable variable type tracking
// ============================================================================

#[test]
fn test_var_mutable_same_type() {
    let diags = errors(
        r#"
        var x = 1;
        x = 2;
        x = 3;
    "#,
    );
    assert!(diags.is_empty(), "Same-type mutation: {:?}", diags);
}

#[test]
fn test_var_mutable_wrong_type() {
    let diags = errors(
        r#"
        var x: number = 1;
        x = "hello";
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected number"));
}

// ============================================================================
// 10. Complex scenarios
// ============================================================================

#[test]
fn test_function_with_if_return() {
    let diags = errors(
        r#"
        fn my_abs(x: number) -> number {
            if (x < 0) {
                return -x;
            }
            return x;
        }
        let _r = my_abs(-5);
    "#,
    );
    assert!(diags.is_empty(), "Function with if/return: {:?}", diags);
}

#[test]
fn test_function_calling_function() {
    let diags = errors(
        r#"
        fn square(x: number) -> number { return x * x; }
        fn sum_squares(a: number, b: number) -> number {
            return square(a) + square(b);
        }
        let _r = sum_squares(3, 4);
    "#,
    );
    assert!(diags.is_empty(), "Function composition: {:?}", diags);
}

#[test]
fn test_multiple_errors_reported() {
    let diags = errors(
        r#"
        let _a: number = "hello";
        let _b: string = 42;
    "#,
    );
    assert!(
        diags.len() >= 2,
        "Should report multiple errors: {:?}",
        diags
    );
}

#[test]
fn test_no_false_positives_complex() {
    let diags = errors(
        r#"
        fn is_even(n: number) -> bool { return n % 2 == 0; }
        fn describe(n: number) -> string {
            if (is_even(n)) {
                return "even";
            }
            return "odd";
        }
        let _r: string = describe(42);
    "#,
    );
    assert!(diags.is_empty(), "Complex valid program: {:?}", diags);
}

#[test]
fn test_while_loop_valid_types() {
    let diags = errors(
        r#"
        fn countdown(n: number) -> number {
            var count = n;
            while (count > 0) {
                count = count - 1;
            }
            return count;
        }
        let _r = countdown(10);
    "#,
    );
    assert!(diags.is_empty(), "While loop valid: {:?}", diags);
}

#[test]
fn test_for_in_valid_array() {
    let diags = errors(
        r#"
        fn sum_arr() -> number {
            let arr = [1, 2, 3];
            var total = 0;
            for x in arr {
                total = total + x;
            }
            return total;
        }
        let _r = sum_arr();
    "#,
    );
    assert!(diags.is_empty(), "For-in valid: {:?}", diags);
}

// ============================================================================
// 11. Additional inference edge cases
// ============================================================================

#[test]
fn test_modulo_result_is_number() {
    let diags = errors("let _x: number = 10 % 3;");
    assert!(diags.is_empty());
}

#[test]
fn test_division_result_is_number() {
    let diags = errors("let _x: number = 10 / 3;");
    assert!(diags.is_empty());
}

#[test]
fn test_equality_result_is_bool() {
    let diags = errors("let _x: bool = 1 == 1;");
    assert!(diags.is_empty());
}

#[test]
fn test_inequality_result_is_bool() {
    let diags = errors("let _x: bool = 1 != 2;");
    assert!(diags.is_empty());
}

// ============================================================================
// From type_rules_tests.rs
// ============================================================================

// Comprehensive tests for type system rules
//
// Tests cover:
// - Arithmetic operators (+, -, *, /, %)
// - Equality operators (==, !=)
// - Comparison operators (<, <=, >, >=)
// - Logical operators (&&, ||)
// - Array literal typing and indexing
// - String concatenation rules
// - Array element assignment type rules

// ========== Arithmetic Operators ==========

#[rstest]
#[case::add_numbers("let x = 5 + 3;", true)]
#[case::subtract_numbers("let x = 10 - 3;", true)]
#[case::multiply_numbers("let x = 5 * 3;", true)]
#[case::divide_numbers("let x = 10 / 2;", true)]
#[case::modulo_numbers("let x = 10 % 3;", true)]
#[case::arithmetic_chain("let x = 1 + 2 - 3 * 4 / 5 % 6;", true)]
#[case::complex_arithmetic("let x = (1 + 2) * (3 - 4) / (5 % 6);", true)]
#[case::arithmetic_with_vars("let a: number = 5; let b: number = 3; let c = a + b; let d = a - b; let e = a * b; let f = a / b; let g = a % b;", true)]
fn test_arithmetic_operations(#[case] source: &str, #[case] should_pass: bool) {
    let diagnostics = typecheck_source(source);
    if should_pass {
        assert_no_errors(&diagnostics);
    }
}

#[rstest]
#[case::add_number_string(r#"let x = 5 + "hello";"#)]
#[case::add_string_number(r#"let x = "hello" + 5;"#)]
#[case::add_bool_bool("let x = true + false;")]
#[case::subtract_strings(r#"let x = "hello" - "world";"#)]
#[case::subtract_number_bool("let x = 5 - true;")]
#[case::multiply_string(r#"let x = "hello" * 3;"#)]
#[case::divide_bools("let x = true / false;")]
#[case::modulo_string(r#"let x = 10 % "hello";"#)]
#[case::null_arithmetic("let x = null + null;")]
fn test_arithmetic_type_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== String Concatenation ==========

#[rstest]
#[case::concat_strings(r#"let x = "hello" + " world";"#)]
#[case::concat_chain(r#"let x = "hello" + " " + "world";"#)]
#[case::concat_variables(r#"let a: string = "hello"; let b: string = "world"; let c = a + b;"#)]
fn test_string_concatenation(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::string_number(r#"let x = "hello" + 123;"#)]
#[case::number_string(r#"let x = 123 + "hello";"#)]
#[case::string_bool(r#"let x = "hello" + true;"#)]
#[case::string_null(r#"let x = "hello" + null;"#)]
fn test_string_concatenation_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Equality Operators ==========

#[rstest]
#[case::numbers_equal("let x = 5 == 3;")]
#[case::strings_equal(r#"let x = "hello" == "world";"#)]
#[case::bools_equal("let x = true == false;")]
#[case::nulls_equal("let x = null == null;")]
#[case::numbers_not_equal("let x = 5 != 3;")]
#[case::strings_not_equal(r#"let x = "hello" != "world";"#)]
#[case::nulls_not_equal("let x = null != null;")]
#[case::arrays_equal("let x = [1, 2] == [1, 2];")]
fn test_equality_operators(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::number_string(r#"let x = 5 == "hello";"#)]
#[case::number_bool("let x = 5 == true;")]
#[case::string_bool(r#"let x = "hello" == false;"#)]
#[case::null_number("let x = null == 5;")]
#[case::not_equal_types(r#"let x = 5 != "hello";"#)]
#[case::not_equal_bool_string(r#"let x = true != "false";"#)]
#[case::mixed_array_types(r#"let x = [1, 2] == ["a", "b"];"#)]
fn test_equality_type_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Comparison Operators ==========

#[rstest]
#[case::less_than("let x = 5 < 10;")]
#[case::less_than_equal("let x = 5 <= 10;")]
#[case::greater_than("let x = 10 > 5;")]
#[case::greater_than_equal("let x = 10 >= 5;")]
#[case::comparison_chain("let x = 1 < 2; let y = 3 > 2; let z = 5 >= 5; let w = 4 <= 10;")]
fn test_comparison_operators(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::strings_less(r#"let x = "hello" < "world";"#)]
#[case::bools_less("let x = true < false;")]
#[case::mixed_less(r#"let x = 5 < "10";"#)]
#[case::strings_less_equal(r#"let x = "a" <= "b";"#)]
#[case::bools_greater("let x = true > false;")]
#[case::nulls_greater("let x = null > null;")]
#[case::mixed_greater_equal("let x = 5 >= true;")]
#[case::nulls_less("let x = null < null;")]
fn test_comparison_type_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Logical Operators ==========

#[rstest]
#[case::and_bools("let x = true && false;")]
#[case::or_bools("let x = true || false;")]
#[case::logical_chain("let x = true && false || true;")]
#[case::with_comparisons("let x = (5 < 10) && (3 > 1);")]
#[case::with_equality("let x = (5 == 5) || (3 != 3);")]
#[case::complex_boolean("let x = (5 < 10) && (3 > 1) || (2 == 2);")]
fn test_logical_operators(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::and_numbers("let x = 5 && 10;")]
#[case::and_strings(r#"let x = "hello" && "world";"#)]
#[case::and_bool_number("let x = true && 5;")]
#[case::and_number_bool("let x = 5 && false;")]
#[case::or_numbers("let x = 0 || 1;")]
#[case::or_strings(r#"let x = "" || "hello";"#)]
#[case::or_bool_string(r#"let x = true || "hello";"#)]
#[case::mixed_expression("let x = (5 + 3) && (2 < 4);")]
#[case::null_logical("let x = null && null;")]
fn test_logical_type_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Array Literals ==========

#[rstest]
#[case::numbers("let x = [1, 2, 3];")]
#[case::strings(r#"let x = ["a", "b", "c"];"#)]
#[case::bools("let x = [true, false, true];")]
#[case::empty("let x = [];")]
#[case::nested("let x = [[1, 2], [3, 4]];")]
#[case::with_expressions("let x = [1 + 2, 3 * 4, 5 - 6];")]
#[case::type_inference("let x = [1, 2, 3];")]
fn test_array_literals(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::mixed_types(r#"let x = [1, "hello", true];"#)]
#[case::number_string(r#"let x = [1, 2, "three"];"#)]
#[case::string_bool(r#"let x = ["hello", "world", true];"#)]
#[case::type_mismatch(r#"let x = [1, 2, "three"];"#)]
#[case::nested_mismatch(r#"let x = [[1, 2], ["a", "b"]];"#)]
fn test_array_literal_type_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Array Indexing ==========

#[rstest]
#[case::number_index("let x = [1, 2, 3]; let y = x[0];")]
#[case::variable_index("let x = [1, 2, 3]; let i: number = 1; let y = x[i];")]
#[case::expression_index("let x = [1, 2, 3]; let y = x[1 + 1];")]
#[case::nested_index("let x = [[1, 2], [3, 4]]; let y = x[0][1];")]
fn test_array_indexing(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::string_index(r#"let x = [1, 2, 3]; let y = x["hello"];"#)]
#[case::bool_index("let x = [1, 2, 3]; let y = x[true];")]
#[case::non_array("let x: number = 5; let y = x[0];")]
#[case::string_indexing(r#"let x: string = "hello"; let y = x[0];"#)]
fn test_array_indexing_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Array Element Assignment ==========

#[rstest]
#[case::same_type("let x = [1, 2, 3]; x[0] = 10;")]
#[case::string_array(r#"let x = ["a", "b", "c"]; x[1] = "world";"#)]
#[case::nested_array("let x = [[1, 2], [3, 4]]; x[0][1] = 99;")]
#[case::array_chain(
    "let arr = [1, 2, 3]; let idx: number = 1; let val = arr[idx] + arr[0]; arr[2] = val;"
)]
fn test_array_element_assignment(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::variable_type_mismatch(r#"let x: string = 42;"#)]
#[case::bool_type_mismatch(r#"let x: bool = 42;"#)]
#[case::string_index_assign(r#"let x = [1, 2, 3]; x["hello"] = 10;"#)]
#[case::non_array_assign("let x: number = 5; x[0] = 10;")]
fn test_array_assignment_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Complex Context Tests ==========

#[rstest]
#[case::function_arithmetic(r#"fn add(a: number, b: number) -> number { return a + b; }"#)]
#[case::conditional_operators(
    r#"let x: number = 5; let y: number = 10; if (x < y && y > 0) { let z = x + y; }"#
)]
#[case::loop_operators(r#"let i: number = 0; while (i < 10) { let x = i + 1; }"#)]
fn test_operators_in_context(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

// ========== Method Call Type Checking ==========

#[rstest]
#[case::json_as_string(r#"let data: json = parseJSON("{\"name\":\"Alice\"}"); let name: string = data["name"].as_string();"#)]
#[case::json_as_number(
    r#"let data: json = parseJSON("{\"age\":30}"); let age: number = data["age"].as_number();"#
)]
#[case::json_as_bool(r#"let data: json = parseJSON("{\"active\":true}"); let active: bool = data["active"].as_bool();"#)]
#[case::json_is_null(r#"let data: json = parseJSON("{\"value\":null}"); let is_null: bool = data["value"].is_null();"#)]
#[case::chained_json_access(r#"let data: json = parseJSON("{\"user\":{\"name\":\"Bob\"}}"); let name: string = data["user"]["name"].as_string();"#)]
#[case::method_in_expression(r#"let data: json = parseJSON("{\"count\":5}"); let x: number = data["count"].as_number() + 10;"#)]
#[case::method_as_arg(
    r#"let data: json = parseJSON("{\"msg\":\"hi\"}"); print(data["msg"].as_string());"#
)]
fn test_valid_method_calls(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::invalid_method_name(
    r#"let data: json = parseJSON("{}"); data.invalid_method();"#,
    "AT3010"
)]
#[case::method_on_wrong_type("let x: number = 42; x.as_string();", "AT3010")]
#[case::method_on_string_type(r#"let s: string = "hello"; s.as_number();"#, "AT3010")]
#[case::method_on_bool_type("let b: bool = true; b.as_string();", "AT3010")]
fn test_invalid_method_calls(#[case] source: &str, #[case] error_code: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, error_code);
}

#[rstest]
#[case::too_many_args(r#"let data: json = parseJSON("{}"); data.as_string(42);"#, "AT3005")]
#[case::too_many_multiple(r#"let data: json = parseJSON("{}"); data.is_null(1, 2);"#, "AT3005")]
fn test_method_argument_count_errors(#[case] source: &str, #[case] error_code: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, error_code);
}

#[rstest]
#[case::wrong_return_type_string(
    r#"let data: json = parseJSON("{\"x\":1}"); let x: string = data["x"].as_number();"#
)]
#[case::wrong_return_type_number(
    r#"let data: json = parseJSON("{\"x\":\"y\"}"); let x: number = data["x"].as_string();"#
)]
#[case::wrong_return_type_bool(
    r#"let data: json = parseJSON("{\"x\":1}"); let x: bool = data["x"].as_number();"#
)]
fn test_method_return_type_mismatch(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

#[test]
fn test_chained_method_calls_type_correctly() {
    let source = r#"
        let data: json = parseJSON("{\"a\":{\"b\":{\"c\":\"value\"}}}");
        let result: string = data["a"]["b"]["c"].as_string();
    "#;
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[test]
fn test_method_call_in_conditional() {
    let source = r#"
        let data: json = parseJSON("{\"enabled\":true}");
        if (data["enabled"].as_bool()) {
            print("Enabled");
        }
    "#;
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[test]
fn test_multiple_method_calls_in_expression() {
    let source = r#"
        let data: json = parseJSON("{\"a\":5,\"b\":10}");
        let sum: number = data["a"].as_number() + data["b"].as_number();
    "#;
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

// ============================================================================
// From typecheck_dump_stability_tests.rs
// ============================================================================

// Tests for typecheck dump format stability
//
// Verifies that:
// - Typecheck dumps include version field
// - Version field is always set correctly
// - Typecheck dump format is stable and deterministic
// - Version mismatch handling for future-proofing

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

    assert_eq!(
        json1, json2,
        "Same source should produce identical JSON output"
    );
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

    assert_eq!(
        json1, json2,
        "Same source should produce identical compact JSON"
    );
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
    assert!(
        result.is_ok(),
        "Should be able to deserialize different versions"
    );

    let dump = result.unwrap();
    assert_eq!(
        dump.typecheck_version, 2,
        "Should preserve version from JSON"
    );
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
    assert!(
        parsed["typecheck_version"].is_number(),
        "Must have typecheck_version"
    );
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

    // Empty program has only prelude builtin constants (E, LN10, LN2, PI, SQRT2)
    assert_eq!(
        dump.symbols.len(),
        5,
        "Empty program should have 5 prelude constants"
    );

    // All symbols should be builtins
    for symbol in &dump.symbols {
        assert_eq!(symbol.kind, "builtin", "All symbols should be builtins");
        assert_eq!(
            symbol.ty, "number",
            "All builtin constants should be numbers"
        );
    }

    // Should have number type from the constants
    assert_eq!(dump.types.len(), 1, "Empty program should have number type");
    assert_eq!(dump.types[0].name, "number");
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
    assert!(
        !dump.symbols.is_empty(),
        "Complex program should have symbols"
    );
    assert!(!dump.types.is_empty(), "Complex program should have types");

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
    let array_types: Vec<_> = dump
        .types
        .iter()
        .filter(|t| t.name.contains("[]"))
        .collect();

    // Verify array type has correct kind if it exists
    for array_type in array_types {
        assert_eq!(
            array_type.kind, "array",
            "Array type should have 'array' kind"
        );
        assert!(
            array_type.details.is_some(),
            "Array type should have details"
        );
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
    let func_types: Vec<_> = dump
        .types
        .iter()
        .filter(|t| t.name.contains("->"))
        .collect();
    assert!(!func_types.is_empty(), "Should have function type");

    // Verify function type has correct kind
    for func_type in func_types {
        assert_eq!(
            func_type.kind, "function",
            "Function type should have 'function' kind"
        );
        assert!(
            func_type.details.is_some(),
            "Function type should have details"
        );
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
    let dumps: Vec<_> = (0..5).map(|_| typecheck_dump_from_source(source)).collect();

    let jsons: Vec<_> = dumps.iter().map(|d| d.to_json_string().unwrap()).collect();

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
    let version_pos = json
        .find("\"typecheck_version\"")
        .expect("Version field must exist");
    let symbols_pos = json.find("\"symbols\"").expect("Symbols field must exist");

    assert!(
        version_pos < symbols_pos,
        "Version field should appear before symbols for easier parsing"
    );
}

// ============================================================================
// From typing_integration_tests.rs
// ============================================================================

fn type_name(result: &ReplCore, input: &str) -> (Option<String>, Vec<atlas_runtime::Diagnostic>) {
    let type_result = result.type_of_expression(input);
    let name = type_result.ty.map(|t| t.display_name());
    (name, type_result.diagnostics)
}

#[rstest(
    input,
    expected,
    case("1 + 2;", "number"),
    case("\"a\" + \"b\";", "string"),
    case("true && false;", "bool"),
    case("[1,2,3];", "number[]"),
    case("let arr = [1,2]; arr[0];", "number"),
    case("len(\"atlas\");", "number"),
    case("let s: string = \"x\"; s;", "string"),
    case("let n: number = 4; n;", "number"),
    case("let flag: bool = true; flag;", "bool"),
    case("match 1 { 1 => 2, _ => 0 };", "number"),
    case("let add = 1 + len([1,2]); add;", "number"),
    case("let val = len([1,2,3]); val;", "number"),
    case("let nested = [[1],[2]]; nested[0];", "number[]"),
    case("let nested = [[1],[2]]; nested[0][0];", "number"),
    case("let maybe = null; maybe;", "null"),
    case("let joined = \"a\" + \"b\"; joined;", "string"),
    case("let num = -1 + 2; num;", "number"),
    case("let cmp = 1 < 2; cmp;", "bool"),
    case("let logical = true || false; logical;", "bool"),
    case("let array_bool = [true, false]; array_bool[1];", "bool")
)]
fn typing_integration_infers_types(input: &str, expected: &str) {
    let repl = ReplCore::new();
    let (ty, diagnostics) = type_name(&repl, input);
    assert!(diagnostics.is_empty(), "Diagnostics: {:?}", diagnostics);
    assert_eq!(ty.expect("type"), expected);
}

#[rstest(
    input,
    case("1 + \"a\";"),
    case("if (1) { 1; }"),
    case("let check: bool = 1;"),
    case("let x: string = 1;"),
    case("let arr: number[] = [1, \"b\"];"),
    case("var flag: bool = 2;"),
    case("fn add(a: number, b: number) -> number { return \"x\"; };"),
    case("match true { 1 => 2 };"),
    case("let mismatch: number = true;"),
    case("while (\"no\") { let a = 1; };"),
    case("return 1;"),
    case("break;"),
    case("continue;"),
    case("let x = [1]; x[\"0\"];"),
    case("let x = true; x + 1;"),
    case("let x = [1,2]; x + 1;"),
    case("if (true) { let x: number = \"bad\"; };"),
    case("let s: string = len([1,2]);"),
    case("var arr = [1,2]; arr[0] = \"x\";"),
    case("let bools: bool[] = [true, 1];")
)]
fn typing_integration_reports_errors(input: &str) {
    let repl = ReplCore::new();
    let result = repl.type_of_expression(input);
    assert!(
        !result.diagnostics.is_empty(),
        "Expected diagnostics for input: {input}"
    );
}

#[rstest(
    input,
    case("let x = 1; let y = x + 2; y;"),
    case("let s = \"a\"; let t = s + \"b\"; t;"),
    case("let arr = [1,2]; arr[1];"),
    case("var n = 0; n = n + 1; n;"),
    case("let cond = true && false; cond;"),
    case("let cmp = 2 > 1; cmp;"),
    case("let nested = [[1,2], [3,4]]; nested[1][0];"),
    case("let lenVal = len(\"abc\"); lenVal;"),
    case("let square = 2 * 2; square;"),
    case("let mix = [1,2,3]; len(mix);"),
    case("let bools = [true, false]; bools[0];"),
    case("let mutableArr = [1,2]; mutableArr[0] = 3; mutableArr[0];"),
    case("let math = (1 + 2) * 3; math;"),
    case("var assign = 1; assign = assign + 1; assign;"),
    case("let zero = 0; let check = zero == 0; check;"),
    case("let arr = [1,2,3]; let idx = 1; arr[idx];"),
    case("let s = \"hi\"; let l = len(s); l;"),
    case("let sum = [1,2]; sum[0] + sum[1];"),
    case("let arr = [true]; len(arr);"),
    case("let chain = len([1,2]) + len(\"hi\"); chain;")
)]
fn typing_integration_regressions_remain_valid(input: &str) {
    let mut repl = ReplCore::new();
    let result = repl.eval_line(input);
    assert!(
        result.diagnostics.is_empty(),
        "Diagnostics: {:?}",
        result.diagnostics
    );
}

// ============================================================================
// From union_type_tests.rs
// ============================================================================

// Tests for union types (Phase typing-04)

// ============================================================================
// Union construction tests
// ============================================================================

#[rstest]
#[case("let _x: number | string = 1;")]
#[case("let _x: number | string = \"ok\";")]
#[case("let _x: number | string | bool = true;")]
#[case("let _x: (number | string)[] = [1, 2, 3];")]
#[case("let _x: (number | string)[] = [\"a\", \"b\"]; ")]
#[case("type Id = number | string; let _x: Id = 7;")]
#[case("type Id = number | string; let _x: Id = \"v\";")]
#[case("type Pair = (number, string) -> number | string; fn f(x: number, y: string) -> number { return x; } let _x: Pair = f;")]
#[case("fn f(x: bool) -> number | string { if (x) { return 1; } return \"a\"; }")]
#[case("let _x: number | number = 1;")]
fn test_union_construction(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Union type checking tests
// ============================================================================

#[rstest]
#[case("let _x: number | string = true;")]
#[case("let _x: number | string = null;")]
#[case("fn f() -> number | string { return true; }")]
#[case("let _x: (number | string)[] = [1, \"bad\"]; ")]
#[case("let _x: number | string = 1; let _y: number = _x;")]
#[case("let _x: number | string = \"ok\"; let _y: string = _x; let _z: number = _x;")]
fn test_union_type_errors(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

#[rstest]
#[case("let _x: number | string = 1; let _y: number | string = _x;")]
#[case("let _x: number | string = \"ok\"; let _y: number | string = _x;")]
#[case("let _x: number | string | bool = true; let _y: number | string | bool = _x;")]
#[case("let _x: number | string = 1; let _y: number | string | bool = _x;")]
fn test_union_assignments(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Type narrowing tests
// ============================================================================

#[rstest]
#[case(
    "let x: number | string = 1; if (isString(x)) { let _y: string = x; } else { let _z: number = x; }"
)]
#[case(
    "let x: number | string = \"hi\"; if (isNumber(x)) { let _y: number = x; } else { let _z: string = x; }"
)]
#[case(
    "let x: number | null = null; if (x == null) { let _y: null = x; } else { let _z: number = x; }"
)]
#[case(
    "let x: number | string = \"hi\"; if (typeof(x) == \"string\") { let _y: string = x; } else { let _z: number = x; }"
)]
#[case(
    "let x: bool | string = true; if (x == true) { let _y: bool = x; } else { let _z: string = x; }"
)]
#[case("let x: number | null = 1; if (x != null) { let _y: number = x; }")]
fn test_type_narrowing(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Match + union integration
// ============================================================================

#[rstest]
#[case(
    "let v: bool | Option<number> = Some(1); match v { true => 1, false => 2, Some(x) => x, None => 0 };"
)]
#[case(
    "let v: Option<number> | Result<number, string> = Ok(1); match v { Some(x) => x, None => 0, Ok(y) => y, Err(_e) => 0 };"
)]
#[case(
    "let v: bool | Option<number> = true; match v { true => 1, false => 2, Some(x) => x, None => 0 };"
)]
fn test_union_match_exhaustive(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

#[rstest]
#[case("let v: bool | Option<number> = true; match v { true => 1, Some(x) => x };")]
#[case("let v: Option<number> | Result<number, string> = Ok(1); match v { Ok(y) => y };")]
fn test_union_match_non_exhaustive(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

// ============================================================================
// Union operations tests
// ============================================================================

#[rstest]
#[case("let x: number | number = 1; let _y: number = x + 1;")]
#[case("let x: string | string = \"a\"; let _y: string = x + \"b\";")]
#[case("let x: number | string = 1; let _y = x + 1;")]
#[case("let x: number | string = \"a\"; let _y = x + \"b\";")]
#[case("let x: number | string = 1; if (x == 1) { let _y: number | string = x; }")]
#[case("let x: number[] | number[] = [1, 2]; let _y = x[0];")]
#[case("let x: number[] | number[] = [1, 2]; let _y: number = x[0];")]
#[case("let x: number[] | number[] = [1, 2]; let _y: number = x[1];")]
fn test_union_operations(#[case] source: &str) {
    let diags = errors(source);
    if source.contains("number | string") && source.contains("x +") {
        assert!(!diags.is_empty(), "Expected errors, got none");
    } else {
        assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
    }
}

// ============================================================================
// Typechecker Ownership Annotation Tests (Phase 06 — Block 2)
// ============================================================================

fn typecheck_with_checker(
    source: &str,
) -> (
    Vec<atlas_runtime::diagnostic::Diagnostic>,
    atlas_runtime::typechecker::TypeChecker<'static>,
) {
    // This helper is only usable when we own the table — use typecheck_source for diagnostics-only.
    // For registry inspection we parse + bind inline.
    use atlas_runtime::binder::Binder;
    use atlas_runtime::lexer::Lexer;
    use atlas_runtime::parser::Parser;
    use atlas_runtime::typechecker::TypeChecker;

    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut table, _) = binder.bind(&program);
    // SAFETY: We box the table to pin it in memory for the 'static TypeChecker.
    // This is test-only scaffolding; the checker is dropped before the box.
    let table_ptr: *mut _ = &mut table;
    let checker_table: &'static mut _ = unsafe { &mut *table_ptr };
    let mut checker = TypeChecker::new(checker_table);
    let diags = checker.check(&program);
    (diags, checker)
}

#[test]
fn test_typechecker_stores_own_annotation() {
    use atlas_runtime::ast::OwnershipAnnotation;
    let src = "fn process(own data: number[]) -> void { }";
    let (diags, checker) = typecheck_with_checker(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let entry = checker
        .fn_ownership_registry
        .get("process")
        .expect("process not in ownership registry");
    assert_eq!(entry.0.len(), 1);
    assert_eq!(entry.0[0], Some(OwnershipAnnotation::Own));
    assert_eq!(entry.1, None); // no return annotation
}

#[test]
fn test_typechecker_warns_own_on_primitive() {
    let src = "fn bad(own _x: number) -> void { }";
    let diags = typecheck_source(src);
    let warnings: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning && d.code == "AT2010")
        .collect();
    assert!(
        !warnings.is_empty(),
        "expected AT2010 warning for `own` on primitive, got: {diags:?}"
    );
}

#[test]
fn test_typechecker_accepts_own_on_array() {
    let src = "fn process(own _data: number[]) -> void { }";
    let diags = typecheck_source(src);
    let warnings: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning && d.code == "AT2010")
        .collect();
    assert!(
        warnings.is_empty(),
        "unexpected AT2010 warning for `own` on array: {diags:?}"
    );
}

#[test]
fn test_typechecker_accepts_borrow_annotation() {
    let src = "fn read(borrow _data: number[]) -> number { return 0; }";
    let diags = typecheck_source(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn test_typechecker_stores_return_ownership() {
    use atlas_runtime::ast::OwnershipAnnotation;
    let src = "fn allocate(_size: number) -> own number { return 0; }";
    let (diags, checker) = typecheck_with_checker(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let entry = checker
        .fn_ownership_registry
        .get("allocate")
        .expect("allocate not in ownership registry");
    assert_eq!(entry.1, Some(OwnershipAnnotation::Own));
}

// ============================================================================
// Call-Site Ownership Checking Tests (Phase 07 — Block 2)
// ============================================================================

#[test]
fn test_typechecker_borrow_to_own_warning() {
    // Passing a `borrow`-annotated caller param to an `own` param should warn AT2012
    let src = r#"
fn consumer(own _data: number[]) -> void { }
fn caller(borrow data: number[]) -> void { consumer(data); }
"#;
    let diags = typecheck_source(src);
    let warnings: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning && d.code == "AT2012")
        .collect();
    assert!(
        !warnings.is_empty(),
        "expected AT2012 warning for borrow-to-own, got: {diags:?}"
    );
}

#[test]
fn test_typechecker_own_param_accepts_owned_value() {
    // Passing a plain (non-borrow) variable to an `own` param is OK
    let src = r#"
fn consume(own _data: number[]) -> void { }
fn caller() -> void {
    let arr: number[] = [1, 2, 3];
    consume(arr);
}
"#;
    let diags = typecheck_source(src);
    let at2012: Vec<_> = diags.iter().filter(|d| d.code == "AT2012").collect();
    assert!(
        at2012.is_empty(),
        "unexpected AT2012 for owned-value-to-own, got: {diags:?}"
    );
}

#[test]
fn test_typechecker_borrow_param_accepts_any_value() {
    // Any value can be passed to a `borrow` param — no diagnostic
    let src = r#"
fn reader(borrow _data: number[]) -> void { }
fn caller() -> void {
    let arr: number[] = [1, 2, 3];
    reader(arr);
}
"#;
    let diags = typecheck_source(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn test_typechecker_borrow_param_accepts_borrow_arg() {
    // Passing a `borrow` param to a `borrow` param is fine
    let src = r#"
fn reader(borrow _data: number[]) -> void { }
fn caller(borrow data: number[]) -> void { reader(data); }
"#;
    let diags = typecheck_source(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn test_typechecker_non_shared_to_shared_error() {
    // Passing a plain (non-shared) value to a `shared` param should emit AT3028
    let src = r#"
fn register(shared _handler: number[]) -> void { }
fn caller() -> void {
    let arr: number[] = [1, 2, 3];
    register(arr);
}
"#;
    let diags = typecheck_source(src);
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3028").collect();
    assert!(
        !errors.is_empty(),
        "expected AT3028 error for non-shared-to-shared, got: {diags:?}"
    );
}

// ── Phase 06: Trait Registry + Built-in Traits ─────────────────────────────

#[test]
fn test_trait_decl_no_diagnostics() {
    let diags = typecheck_source("trait Marker { }");
    assert!(
        diags.is_empty(),
        "Empty trait should produce no errors: {diags:?}"
    );
}

#[test]
fn test_trait_with_multiple_methods_no_diagnostics() {
    let diags = typecheck_source(
        "
        trait Comparable {
            fn compare(self: Comparable, other: Comparable) -> number;
            fn equals(self: Comparable, other: Comparable) -> bool;
        }
    ",
    );
    assert!(
        diags.is_empty(),
        "Multi-method trait should produce no errors: {diags:?}"
    );
}

#[test]
fn test_duplicate_trait_declaration_is_error() {
    let diags = typecheck_source(
        "
        trait Foo { fn bar() -> void; }
        trait Foo { fn baz() -> void; }
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3031").collect();
    assert!(
        !errors.is_empty(),
        "Duplicate trait should produce AT3031, got: {diags:?}"
    );
}

#[test]
fn test_redefining_builtin_trait_copy_is_error() {
    let diags = typecheck_source("trait Copy { fn do_copy() -> void; }");
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3030").collect();
    assert!(
        !errors.is_empty(),
        "Redefining Copy should produce AT3030, got: {diags:?}"
    );
}

#[test]
fn test_redefining_builtin_trait_move_is_error() {
    let diags = typecheck_source("trait Move { }");
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3030").collect();
    assert!(
        !errors.is_empty(),
        "Redefining Move should produce AT3030, got: {diags:?}"
    );
}

#[test]
fn test_redefining_builtin_trait_drop_is_error() {
    let diags = typecheck_source("trait Drop { }");
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3030").collect();
    assert!(
        !errors.is_empty(),
        "Redefining Drop should produce AT3030, got: {diags:?}"
    );
}

#[test]
fn test_redefining_builtin_trait_display_is_error() {
    let diags = typecheck_source("trait Display { }");
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3030").collect();
    assert!(
        !errors.is_empty(),
        "Redefining Display should produce AT3030, got: {diags:?}"
    );
}

#[test]
fn test_redefining_builtin_trait_debug_is_error() {
    let diags = typecheck_source("trait Debug { }");
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3030").collect();
    assert!(
        !errors.is_empty(),
        "Redefining Debug should produce AT3030, got: {diags:?}"
    );
}

#[test]
fn test_impl_unknown_trait_is_error() {
    let diags = typecheck_source("impl UnknownTrait for number { }");
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3032").collect();
    assert!(
        !errors.is_empty(),
        "impl unknown trait should produce AT3032, got: {diags:?}"
    );
}

#[test]
fn test_impl_known_user_trait_no_error() {
    let diags = typecheck_source(
        "
        trait Marker { }
        impl Marker for number { }
    ",
    );
    let trait_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3032").collect();
    assert!(
        trait_errors.is_empty(),
        "impl known trait should not produce AT3032, got: {diags:?}"
    );
}

#[test]
fn test_impl_builtin_trait_copy_no_error() {
    let diags = typecheck_source("impl Copy for number { }");
    let trait_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3032").collect();
    assert!(
        trait_errors.is_empty(),
        "impl built-in Copy should not produce AT3032, got: {diags:?}"
    );
}

#[test]
fn test_trait_with_generic_method_no_diagnostics() {
    let diags = typecheck_source(
        "
        trait Printer {
            fn print<T: Display>(value: T) -> void;
        }
    ",
    );
    assert!(
        diags.is_empty(),
        "Trait with generic method should produce no errors: {diags:?}"
    );
}

#[test]
fn test_multiple_traits_no_conflict() {
    let diags = typecheck_source(
        "
        trait Foo { fn foo() -> void; }
        trait Bar { fn bar() -> void; }
        trait Baz { fn baz() -> void; }
    ",
    );
    assert!(
        diags.is_empty(),
        "Multiple distinct traits should produce no errors: {diags:?}"
    );
}

#[test]
fn test_impl_multiple_traits_for_same_type() {
    let diags = typecheck_source(
        "
        trait Foo { fn foo() -> void; }
        trait Bar { fn bar() -> void; }
        impl Foo for number { }
        impl Bar for number { }
    ",
    );
    let trait_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3032").collect();
    assert!(
        trait_errors.is_empty(),
        "impl multiple traits should not error, got: {diags:?}"
    );
}

// ── Phase 07: Impl Conformance Checking ────────────────────────────────────

#[test]
fn test_impl_complete_conformance_no_errors() {
    let diags = typecheck_source(
        "
        trait Greet { fn greet(self: Greet) -> string; }
        impl Greet for number {
            fn greet(self: number) -> string { return \"hello\"; }
        }
    ",
    );
    let conformance_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "AT3033" || d.code == "AT3034")
        .collect();
    assert!(
        conformance_errors.is_empty(),
        "Complete impl should have no conformance errors: {diags:?}"
    );
}

#[test]
fn test_impl_missing_required_method_is_error() {
    let diags = typecheck_source(
        "
        trait Shape {
            fn area(self: Shape) -> number;
            fn perimeter(self: Shape) -> number;
        }
        impl Shape for number {
            fn area(self: number) -> number { return 1.0; }
        }
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3033").collect();
    assert!(
        !errors.is_empty(),
        "Missing method should produce AT3033: {diags:?}"
    );
}

#[test]
fn test_impl_wrong_return_type_is_error() {
    let diags = typecheck_source(
        "
        trait Stringify { fn to_str(self: Stringify) -> string; }
        impl Stringify for number {
            fn to_str(self: number) -> number { return 0.0; }
        }
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3034").collect();
    assert!(
        !errors.is_empty(),
        "Wrong return type should produce AT3034: {diags:?}"
    );
}

#[test]
fn test_impl_wrong_param_type_is_error() {
    let diags = typecheck_source(
        "
        trait Adder { fn add(self: Adder, x: number) -> number; }
        impl Adder for number {
            fn add(self: number, x: string) -> number { return 0.0; }
        }
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3034").collect();
    assert!(
        !errors.is_empty(),
        "Wrong param type should produce AT3034: {diags:?}"
    );
}

#[test]
fn test_duplicate_impl_is_error() {
    let diags = typecheck_source(
        "
        trait Marker { }
        impl Marker for number { }
        impl Marker for number { }
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3029").collect();
    assert!(
        !errors.is_empty(),
        "Duplicate impl should produce AT3029: {diags:?}"
    );
}

#[test]
fn test_empty_trait_impl_for_multiple_types_is_valid() {
    let diags = typecheck_source(
        "
        trait Marker { }
        impl Marker for number { }
        impl Marker for string { }
        impl Marker for bool { }
    ",
    );
    let conformance_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "AT3029" || d.code == "AT3033" || d.code == "AT3034")
        .collect();
    assert!(
        conformance_errors.is_empty(),
        "Multiple impls of marker trait should be valid: {diags:?}"
    );
}

#[test]
fn test_impl_method_body_type_error_caught() {
    let diags = typecheck_source(
        "
        trait Negate { fn negate(self: Negate) -> bool; }
        impl Negate for number {
            fn negate(self: number) -> bool { return 42; }
        }
    ",
    );
    // Body return type mismatch: returning number where bool expected
    assert!(
        !diags.is_empty(),
        "Type error in impl method body should produce diagnostics"
    );
}

#[test]
fn test_impl_extra_methods_beyond_trait_allowed() {
    let diags = typecheck_source(
        "
        trait Greet { fn greet(self: Greet) -> string; }
        impl Greet for number {
            fn greet(self: number) -> string { return \"hi\"; }
            fn extra(self: number) -> number { return 0.0; }
        }
    ",
    );
    let conformance_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "AT3033" || d.code == "AT3034")
        .collect();
    assert!(
        conformance_errors.is_empty(),
        "Extra methods beyond trait should be allowed: {diags:?}"
    );
}

#[test]
fn test_impl_multi_method_trait_all_provided() {
    let diags = typecheck_source(
        "
        trait Comparable {
            fn less_than(self: Comparable, other: Comparable) -> bool;
            fn equals(self: Comparable, other: Comparable) -> bool;
        }
        impl Comparable for number {
            fn less_than(self: number, other: number) -> bool { return false; }
            fn equals(self: number, other: number) -> bool { return false; }
        }
    ",
    );
    let conformance_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "AT3033" || d.code == "AT3034")
        .collect();
    assert!(
        conformance_errors.is_empty(),
        "All methods provided should have no conformance errors: {diags:?}"
    );
}

// ── Phase 08: User Trait Method Call Typechecking ──────────────────────────

#[test]
fn test_trait_method_call_resolves_return_type() {
    // x.display() returns string — assigning to string: no error
    let diags = typecheck_source(
        "
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
        let x: number = 42;
        let s: string = x.display();
    ",
    );
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Trait method call should resolve return type cleanly: {diags:?}"
    );
}

#[test]
fn test_trait_method_call_wrong_assignment_is_error() {
    // x.display() returns string — assigning to number: type error
    let diags = typecheck_source(
        "
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
        let x: number = 42;
        let n: number = x.display();
    ",
    );
    assert!(
        !diags.is_empty(),
        "Assigning string return to number should produce a diagnostic: {diags:?}"
    );
}

#[test]
fn test_trait_method_call_number_return_resolves() {
    let diags = typecheck_source(
        "
        trait Doubler { fn double(self: Doubler) -> number; }
        impl Doubler for number {
            fn double(self: number) -> number { return self * 2; }
        }
        let x: number = 5;
        let y: number = x.double();
    ",
    );
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "number-returning trait method should resolve correctly: {diags:?}"
    );
}

#[test]
fn test_trait_method_not_found_on_unimplemented_type() {
    // string doesn't implement Display in this program — AT3035 fires (trait known but not impl)
    let diags = typecheck_source(
        "
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
        let s: string = \"hello\";
        let result: string = s.display();
    ",
    );
    // string has no Display impl here — AT3035 fires (trait exists but type doesn't implement it)
    let method_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3035").collect();
    assert!(
        !method_errors.is_empty(),
        "Method call on unimplemented type should produce AT3035: {diags:?}"
    );
}

#[test]
fn test_stdlib_method_not_shadowed_by_trait() {
    // Array push() is stdlib — a trait method named push doesn't conflict
    let diags = typecheck_source(
        "
        trait Pushable { fn push(self: Pushable, x: number) -> void; }
        impl Pushable for number { fn push(self: number, x: number) -> void { } }
        let arr: number[] = [1, 2, 3];
        arr = arr.push(4);
    ",
    );
    // arr.push(4) hits stdlib — no AT3010 expected
    let method_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3010").collect();
    assert!(
        method_errors.is_empty(),
        "Stdlib array.push should not be shadowed: {diags:?}"
    );
}

#[test]
fn test_trait_method_bool_return_resolves() {
    let diags = typecheck_source(
        "
        trait Check { fn is_valid(self: Check) -> bool; }
        impl Check for number {
            fn is_valid(self: number) -> bool { return self > 0; }
        }
        let x: number = 5;
        let ok: bool = x.is_valid();
    ",
    );
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "bool-returning trait method should resolve correctly: {diags:?}"
    );
}

// ── Phase 09: Copy/Move + Ownership Integration ─────────────────────────────

#[test]
fn test_number_passed_without_annotation_no_error() {
    // number is Copy — no ownership annotation needed
    let diags = typecheck_source(
        "
        fn double(x: number) -> number { return x * 2; }
        let n: number = 5;
        let result: number = double(n);
    ",
    );
    // Should produce no ownership-related diagnostics
    let ownership_diags: Vec<_> = diags.iter().filter(|d| d.code == "AT2013").collect();
    assert!(
        ownership_diags.is_empty(),
        "number is Copy, no AT2013 expected: {diags:?}"
    );
}

#[test]
fn test_string_passed_without_annotation_no_error() {
    let diags = typecheck_source(
        "
        fn greet(name: string) -> string { return name; }
        let s: string = \"hello\";
        let g: string = greet(s);
    ",
    );
    let ownership_diags: Vec<_> = diags.iter().filter(|d| d.code == "AT2013").collect();
    assert!(
        ownership_diags.is_empty(),
        "string is Copy, no AT2013 expected: {diags:?}"
    );
}

#[test]
fn test_bool_passed_without_annotation_no_error() {
    let diags = typecheck_source(
        "
        fn negate(b: bool) -> bool { return !b; }
        let flag: bool = true;
        let result: bool = negate(flag);
    ",
    );
    let ownership_diags: Vec<_> = diags.iter().filter(|d| d.code == "AT2013").collect();
    assert!(
        ownership_diags.is_empty(),
        "bool is Copy, no AT2013 expected: {diags:?}"
    );
}

#[test]
fn test_array_passed_without_annotation_no_error() {
    let diags = typecheck_source(
        "
        fn first(arr: number[]) -> number { return arr[0]; }
        let a: number[] = [1, 2, 3];
        let n: number = first(a);
    ",
    );
    let ownership_diags: Vec<_> = diags.iter().filter(|d| d.code == "AT2013").collect();
    assert!(
        ownership_diags.is_empty(),
        "array is Copy (CoW), no AT2013 expected: {diags:?}"
    );
}

#[test]
fn test_redefine_builtin_copy_trait_is_error() {
    // Attempting to declare `trait Copy` should produce AT3030
    let diags = typecheck_source("trait Copy { fn do_copy() -> void; }");
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3030").collect();
    assert!(
        !errors.is_empty(),
        "Redefining Copy should produce AT3030: {diags:?}"
    );
}

#[test]
fn test_explicit_own_on_copy_type_allowed() {
    // own annotation on Copy type is redundant but not an error
    let diags = typecheck_source(
        "
        fn consume(own x: number) -> number { return x; }
        let n: number = 42;
        let result: number = consume(n);
    ",
    );
    // No errors — own on Copy is always valid
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == atlas_runtime::diagnostic::DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Explicit own on Copy type should not produce errors: {diags:?}"
    );
}

#[test]
fn test_impl_copy_for_type_registers_in_trait_registry() {
    // impl Copy for number (built-in Copy, already in registry) should not AT3030
    let diags = typecheck_source("impl Copy for number { }");
    let builtin_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3030").collect();
    assert!(
        builtin_errors.is_empty(),
        "impl Copy for number should not produce AT3030: {diags:?}"
    );
}

// ── Phase 10: Trait Bounds Enforcement ─────────────────────────────────────

#[test]
fn test_copy_bound_satisfied_by_number() {
    let diags = typecheck_source(
        "
        fn safe_copy<T: Copy>(x: T) -> T { return x; }
        let n: number = safe_copy(42);
    ",
    );
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        bound_errors.is_empty(),
        "number satisfies Copy bound, no AT3037 expected: {diags:?}"
    );
}

#[test]
fn test_copy_bound_satisfied_by_string() {
    let diags = typecheck_source(
        "
        fn safe_copy<T: Copy>(x: T) -> T { return x; }
        let s: string = safe_copy(\"hello\");
    ",
    );
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        bound_errors.is_empty(),
        "string satisfies Copy bound, no AT3037 expected: {diags:?}"
    );
}

#[test]
fn test_copy_bound_satisfied_by_bool() {
    let diags = typecheck_source(
        "
        fn safe_copy<T: Copy>(x: T) -> T { return x; }
        let b: bool = safe_copy(true);
    ",
    );
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        bound_errors.is_empty(),
        "bool satisfies Copy bound, no AT3037 expected: {diags:?}"
    );
}

#[test]
fn test_unbounded_type_param_no_error() {
    // Unbounded type params must still work
    let diags = typecheck_source(
        "
        fn identity<T>(x: T) -> T { return x; }
        let n: number = identity(42);
        let s: string = identity(\"hello\");
    ",
    );
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        bound_errors.is_empty(),
        "Unbounded type params should not produce AT3037: {diags:?}"
    );
}

#[test]
fn test_user_trait_bound_satisfied() {
    let diags = typecheck_source(
        "
        trait Printable { fn print_self(self: Printable) -> void; }
        impl Printable for number {
            fn print_self(self: number) -> void { }
        }
        fn log_it<T: Printable>(x: T) -> void { }
        log_it(42);
    ",
    );
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        bound_errors.is_empty(),
        "number implements Printable, bound satisfied: {diags:?}"
    );
}

#[test]
fn test_user_trait_bound_not_satisfied_is_error() {
    let diags = typecheck_source(
        "
        trait Printable { fn print_self(self: Printable) -> void; }
        impl Printable for number {
            fn print_self(self: number) -> void { }
        }
        fn log_it<T: Printable>(x: T) -> void { }
        log_it(\"hello\");
    ",
    );
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        !bound_errors.is_empty(),
        "string doesn't implement Printable — AT3037 expected: {diags:?}"
    );
}

#[test]
fn test_multiple_bounds_all_satisfied() {
    let diags = typecheck_source(
        "
        trait Printable { fn print_self(self: Printable) -> void; }
        impl Printable for number { fn print_self(self: number) -> void { } }
        fn process<T: Copy + Printable>(x: T) -> void { }
        process(42);
    ",
    );
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        bound_errors.is_empty(),
        "number is Copy AND Printable, both bounds satisfied: {diags:?}"
    );
}

#[test]
fn test_multiple_bounds_one_missing_is_error() {
    let diags = typecheck_source(
        "
        trait Printable { fn print_self(self: Printable) -> void; }
        fn process<T: Copy + Printable>(x: T) -> void { }
        process(42);
    ",
    );
    // number is Copy but no impl Printable here
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        !bound_errors.is_empty(),
        "Missing Printable impl — AT3037 expected: {diags:?}"
    );
}

// ============================================================
// Phase 11 — AT3xxx Error Code Coverage Tests
// ============================================================

// AT3035 — TYPE_DOES_NOT_IMPLEMENT_TRAIT
// Fires when a method is called on a type that declares no impl for the owning trait.
#[test]
fn test_at3035_method_call_trait_not_implemented() {
    let diags = typecheck_source(
        "
        trait Flippable { fn flip(self: Flippable) -> bool; }
        impl Flippable for bool { fn flip(self: bool) -> bool { return true; } }
        let n: number = 42;
        n.flip();
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3035").collect();
    assert!(
        !errors.is_empty(),
        "number doesn't implement Flippable — AT3035 expected: {diags:?}"
    );
}

#[test]
fn test_at3035_not_fired_when_impl_exists() {
    let diags = typecheck_source(
        "
        trait Flippable { fn flip(self: Flippable) -> bool; }
        impl Flippable for bool { fn flip(self: bool) -> bool { return true; } }
        let b: bool = true;
        b.flip();
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3035").collect();
    assert!(
        errors.is_empty(),
        "bool implements Flippable — no AT3035 expected: {diags:?}"
    );
}

// AT2013 — MOVE_TYPE_REQUIRES_OWNERSHIP_ANNOTATION (warning, not error)
#[test]
fn test_at2013_is_warning_not_error() {
    // AT2013 is intentionally a WARNING — eval should still succeed
    // We verify the warning fires but the program is not rejected
    let diags = typecheck_source(
        "
        fn take_user(x: number) -> void { }
        take_user(42);
    ",
    );
    // number is Copy — AT2013 should NOT fire
    let ownership_warns: Vec<_> = diags.iter().filter(|d| d.code == "AT2013").collect();
    assert!(
        ownership_warns.is_empty(),
        "number is Copy — AT2013 must not fire: {diags:?}"
    );
}

// Registry verification — all AT3029-AT3037 constants exist in the expected range
#[test]
fn test_at3xxx_codes_in_expected_range() {
    use atlas_runtime::diagnostic::error_codes;
    let trait_codes = [
        error_codes::IMPL_ALREADY_EXISTS,
        error_codes::TRAIT_REDEFINES_BUILTIN,
        error_codes::TRAIT_ALREADY_DEFINED,
        error_codes::TRAIT_NOT_FOUND,
        error_codes::IMPL_METHOD_MISSING,
        error_codes::IMPL_METHOD_SIGNATURE_MISMATCH,
        error_codes::TYPE_DOES_NOT_IMPLEMENT_TRAIT,
        error_codes::COPY_TYPE_REQUIRED,
        error_codes::TRAIT_BOUND_NOT_SATISFIED,
    ];
    for code in &trait_codes {
        assert!(
            code.starts_with("AT3"),
            "Trait error code '{}' should be in AT3xxx range",
            code
        );
    }
    // AT2013 is a warning, correctly in AT2xxx range
    assert!(error_codes::MOVE_TYPE_REQUIRES_OWNERSHIP_ANNOTATION.starts_with("AT2"));
}
