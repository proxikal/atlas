//! Enhanced type inference
//!
//! Provides:
//! - Return type inference from function body analysis
//! - Bidirectional type checking (expected type guides inference)
//! - Variable type inference from initializers
//! - Least upper bound computation for multiple types
//! - Checking modes (synthesis vs checking)
//! - Let-polymorphism with value and monomorphism restrictions
//! - Type inference heuristics

use crate::ast::*;
use crate::types::Type;

// ============================================================================
// Checking Modes
// ============================================================================

/// Bidirectional type checking mode
///
/// In synthesis mode, the type of an expression is computed bottom-up.
/// In checking mode, the expression is validated against an expected type.
#[derive(Debug, Clone, PartialEq)]
pub enum CheckingMode {
    /// Synthesize (infer) the type of an expression bottom-up
    Synthesis,
    /// Check the expression against an expected type top-down
    Checking(Type),
}

impl CheckingMode {
    /// Get the expected type, if any
    pub fn expected_type(&self) -> Option<&Type> {
        match self {
            Self::Synthesis => None,
            Self::Checking(ty) => Some(ty),
        }
    }

    /// Whether this mode has an expected type
    pub fn has_expected(&self) -> bool {
        matches!(self, Self::Checking(_))
    }

    /// Switch to checking mode with the given expected type
    pub fn with_expected(ty: Type) -> Self {
        Self::Checking(ty)
    }

    /// Switch to synthesis mode
    pub fn synthesis() -> Self {
        Self::Synthesis
    }
}

// ============================================================================
// Bidirectional Checker
// ============================================================================

/// Bidirectional type checking engine
///
/// Combines synthesis and checking modes:
/// - Synthesis mode: infer type from expression, bottom-up
/// - Checking mode: validate expression against expected type, top-down
///
/// The expected type is propagated through expressions to guide inference
/// and reduce the need for explicit annotations.
pub struct BidirectionalChecker {
    mode: CheckingMode,
}

impl BidirectionalChecker {
    /// Create a checker in synthesis mode
    pub fn synthesis() -> Self {
        Self {
            mode: CheckingMode::Synthesis,
        }
    }

    /// Create a checker in checking mode
    pub fn checking(expected: Type) -> Self {
        Self {
            mode: CheckingMode::Checking(expected),
        }
    }

    /// Get the current mode
    pub fn mode(&self) -> &CheckingMode {
        &self.mode
    }

    /// Switch to synthesis mode (at expression boundaries)
    pub fn switch_to_synthesis(&mut self) {
        self.mode = CheckingMode::Synthesis;
    }

    /// Switch to checking mode with a new expected type
    pub fn switch_to_checking(&mut self, expected: Type) {
        self.mode = CheckingMode::Checking(expected);
    }

    /// Propagate expected type to a sub-expression.
    ///
    /// Returns a new checker configured appropriately for the sub-expression.
    /// The expected type is only propagated when it can meaningfully guide inference.
    pub fn propagate_expected(&self, sub_expr_ty: Option<&Type>) -> CheckingMode {
        match (&self.mode, sub_expr_ty) {
            (CheckingMode::Checking(expected), _) => {
                // If we already have an expected type, propagate it
                CheckingMode::Checking(expected.clone())
            }
            (CheckingMode::Synthesis, Some(hint)) => {
                // A hint from context: switch to checking
                CheckingMode::Checking(hint.clone())
            }
            (CheckingMode::Synthesis, None) => CheckingMode::Synthesis,
        }
    }

    /// Validate an inferred type against the expected type (if any).
    ///
    /// Returns `BidirectionalResult::Compatible` when the inferred type is
    /// acceptable in the current context.
    pub fn validate(&self, inferred: &Type) -> BidirectionalResult {
        match &self.mode {
            CheckingMode::Synthesis => BidirectionalResult::Compatible,
            CheckingMode::Checking(expected) => check_bidirectional(expected, inferred),
        }
    }
}

// ============================================================================
// Let-Polymorphism
// ============================================================================

/// The kind of binding being generalized
#[derive(Debug, Clone, PartialEq)]
pub enum BindingKind {
    /// A syntactic value (literal, lambda, …) – eligible for generalization
    SyntacticValue,
    /// A non-value (function call, variable reference, …) – restricted by the
    /// monomorphism restriction from being generalized.
    NonValue,
    /// A mutable binding – cannot be generalized (value restriction)
    Mutable,
}

/// Let-polymorphism analysis
///
/// Determines whether a let binding's type can be generalized to a polymorphic
/// type scheme, respecting:
/// - Value restriction: only syntactic values can be generalized
/// - Monomorphism restriction: mutable bindings are not generalized
/// - Safety: recursive bindings require care
pub struct LetPolymorphism;

impl LetPolymorphism {
    /// Classify an expression into its binding kind for generalization purposes
    pub fn classify_expr(expr: &Expr) -> BindingKind {
        match expr {
            // Syntactic values that can be safely generalized
            Expr::Literal(_, _) => BindingKind::SyntacticValue,
            Expr::ArrayLiteral(_) => BindingKind::SyntacticValue,
            // Closures/lambdas would be here too if Atlas had anonymous functions
            // For now, group expressions are transparent
            Expr::Group(g) => Self::classify_expr(&g.expr),
            // Everything else is a non-value
            _ => BindingKind::NonValue,
        }
    }

    /// Decide whether a binding should be generalized.
    ///
    /// `inferred_ty` is the type inferred for the binding, `mutable` indicates
    /// whether the binding is declared with `mut`.
    pub fn is_generalizable(inferred_ty: &Type, expr: &Expr, mutable: bool) -> bool {
        // Mutable bindings cannot be generalized (value restriction)
        if mutable {
            return false;
        }

        // Non-value expressions cannot be generalized (monomorphism restriction)
        if Self::classify_expr(expr) == BindingKind::NonValue {
            return false;
        }

        // Types with free type parameters can potentially be generalized
        has_type_parameters(inferred_ty)
    }

    /// Quantify the free type parameters in a type.
    ///
    /// Returns the set of free type parameter names that would be universally
    /// quantified in the polymorphic type scheme.
    pub fn quantify_free_vars(ty: &Type) -> Vec<String> {
        let mut vars = Vec::new();
        collect_type_params(ty, &mut vars);
        vars.sort();
        vars.dedup();
        vars
    }

    /// Apply value restriction: if the expression is not a syntactic value,
    /// return the type unchanged (no generalization).
    pub fn apply_value_restriction(ty: Type, _expr: &Expr) -> Type {
        ty
    }
}

/// Check whether a type contains free type parameters
fn has_type_parameters(ty: &Type) -> bool {
    match ty {
        Type::TypeParameter { .. } => true,
        Type::Array(elem) => has_type_parameters(elem),
        Type::Function {
            type_params,
            params,
            return_type,
        } => {
            !type_params.is_empty()
                || params.iter().any(has_type_parameters)
                || has_type_parameters(return_type)
        }
        Type::Generic { type_args, .. } => type_args.iter().any(has_type_parameters),
        Type::Alias {
            type_args, target, ..
        } => type_args.iter().any(has_type_parameters) || has_type_parameters(target),
        Type::Structural { members } => members.iter().any(|m| has_type_parameters(&m.ty)),
        Type::Union(members) | Type::Intersection(members) => {
            members.iter().any(has_type_parameters)
        }
        _ => false,
    }
}

/// Collect all type parameter names in a type
fn collect_type_params(ty: &Type, vars: &mut Vec<String>) {
    match ty {
        Type::TypeParameter { name } => vars.push(name.clone()),
        Type::Array(elem) => collect_type_params(elem, vars),
        Type::Function {
            params,
            return_type,
            ..
        } => {
            for p in params {
                collect_type_params(p, vars);
            }
            collect_type_params(return_type, vars);
        }
        Type::Generic { type_args, .. } => {
            for a in type_args {
                collect_type_params(a, vars);
            }
        }
        Type::Alias {
            type_args, target, ..
        } => {
            for a in type_args {
                collect_type_params(a, vars);
            }
            collect_type_params(target, vars);
        }
        Type::Structural { members } => {
            for m in members {
                collect_type_params(&m.ty, vars);
            }
        }
        Type::Union(members) | Type::Intersection(members) => {
            for m in members {
                collect_type_params(m, vars);
            }
        }
        _ => {}
    }
}

// ============================================================================
// Inference Heuristics
// ============================================================================

/// Type inference heuristics
///
/// When type inference is ambiguous (multiple solutions exist), heuristics
/// guide the solver toward the most useful type without sacrificing soundness.
pub struct InferenceHeuristics;

impl InferenceHeuristics {
    /// From a set of candidate types, prefer the simplest one.
    ///
    /// Simple types are primitive types (number, string, bool, null)
    /// rather than compound types (arrays, generics, functions).
    pub fn prefer_simple(types: &[Type]) -> Option<Type> {
        if types.is_empty() {
            return None;
        }
        // Prefer primitive types first
        let primitives = [Type::Number, Type::String, Type::Bool, Type::Null];
        for primitive in &primitives {
            if types.iter().any(|t| t.normalized() == *primitive) {
                return Some(primitive.clone());
            }
        }
        // Fall back to first type
        Some(types[0].clone())
    }

    /// From a set of candidate types, prefer primitive types when ambiguous.
    ///
    /// Prefers: number > string > bool > null > compound types
    pub fn prefer_primitive(types: &[Type]) -> Option<Type> {
        let priority_order = [Type::Number, Type::String, Type::Bool, Type::Null];
        for candidate in &priority_order {
            if types.iter().any(|t| &t.normalized() == candidate) {
                return Some(candidate.clone());
            }
        }
        // No primitive found – return first
        types.first().cloned()
    }

    /// Infer a type for a variable assigned across multiple branches.
    ///
    /// - All same: return that type
    /// - Mix: form a union (removing duplicates and Never)
    pub fn infer_union_from_branches(branch_types: Vec<Type>) -> Type {
        if branch_types.is_empty() {
            return Type::Unknown;
        }

        let first_norm = branch_types[0].normalized();
        let all_same = branch_types.iter().all(|t| t.normalized() == first_norm);

        if all_same {
            branch_types[0].clone()
        } else {
            // Filter out Never and Unknown before forming union
            let members: Vec<Type> = branch_types
                .into_iter()
                .filter(|t| {
                    let n = t.normalized();
                    n != Type::Never && n != Type::Unknown
                })
                .collect();
            if members.is_empty() {
                return Type::Unknown;
            }
            Type::union(members)
        }
    }

    /// Infer a literal type from an expression (if it's a literal).
    ///
    /// Returns `None` when the expression is not a literal.
    pub fn infer_literal_type(expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Literal(Literal::Number(_), _) => Some(Type::Number),
            Expr::Literal(Literal::String(_), _) => Some(Type::String),
            Expr::Literal(Literal::Bool(_), _) => Some(Type::Bool),
            Expr::Literal(Literal::Null, _) => Some(Type::Null),
            Expr::Group(g) => Self::infer_literal_type(&g.expr),
            _ => None,
        }
    }

    /// Minimize type variables: replace free type parameters with Unknown.
    ///
    /// Used when inference cannot determine a concrete type; replaces each
    /// unresolved type parameter with Unknown to produce a concrete type.
    pub fn minimize_type_variables(ty: &Type) -> Type {
        match ty {
            Type::TypeParameter { .. } => Type::Unknown,
            Type::Array(elem) => Type::Array(Box::new(Self::minimize_type_variables(elem))),
            Type::Function {
                type_params,
                params,
                return_type,
            } => Type::Function {
                type_params: type_params.clone(),
                params: params.iter().map(Self::minimize_type_variables).collect(),
                return_type: Box::new(Self::minimize_type_variables(return_type)),
            },
            Type::Generic { name, type_args } => Type::Generic {
                name: name.clone(),
                type_args: type_args
                    .iter()
                    .map(Self::minimize_type_variables)
                    .collect(),
            },
            Type::Structural { members } => Type::Structural {
                members: members
                    .iter()
                    .map(|m| crate::types::StructuralMemberType {
                        name: m.name.clone(),
                        ty: Self::minimize_type_variables(&m.ty),
                    })
                    .collect(),
            },
            Type::Union(members) => {
                Type::union(members.iter().map(Self::minimize_type_variables).collect())
            }
            Type::Intersection(members) => {
                Type::intersection(members.iter().map(Self::minimize_type_variables).collect())
            }
            _ => ty.clone(),
        }
    }
}

/// Infer the return type of a function from its body.
///
/// Collects all `return` statement expression types and computes their
/// least upper bound. Returns `None` if no return statements found
/// (implies void), or `Some(type)` with the inferred return type.
pub fn infer_return_type(body: &Block) -> InferredReturn {
    let mut return_types = Vec::new();
    let mut has_implicit_void = false;

    collect_return_types(&body.statements, &mut return_types, &mut has_implicit_void);

    if return_types.is_empty() {
        return InferredReturn::Void;
    }

    // Check if all return types are the same
    let first = &return_types[0];
    let first_norm = first.normalized();
    let all_same = return_types.iter().all(|t| t.normalized() == first_norm);

    if all_same {
        if has_implicit_void && first_norm != Type::Void {
            // Some paths return a value, some fall through (void)
            InferredReturn::Inconsistent {
                types: return_types,
                has_void_path: true,
            }
        } else {
            InferredReturn::Uniform(first.clone())
        }
    } else {
        InferredReturn::Inconsistent {
            types: return_types,
            has_void_path: has_implicit_void,
        }
    }
}

/// Result of return type inference
#[derive(Debug, Clone, PartialEq)]
pub enum InferredReturn {
    /// All paths return void (no explicit returns, or only `return;`)
    Void,
    /// All explicit returns have the same type
    Uniform(Type),
    /// Returns have different types (error)
    Inconsistent {
        types: Vec<Type>,
        has_void_path: bool,
    },
}

/// Collect return types from statements recursively.
fn collect_return_types(
    stmts: &[Stmt],
    return_types: &mut Vec<Type>,
    has_implicit_void: &mut bool,
) {
    for stmt in stmts {
        match stmt {
            Stmt::Return(ret) => {
                let ty = if let Some(value) = &ret.value {
                    infer_expr_type(value)
                } else {
                    Type::Void
                };
                return_types.push(ty);
            }
            Stmt::If(if_stmt) => {
                collect_return_types(
                    &if_stmt.then_block.statements,
                    return_types,
                    has_implicit_void,
                );
                if let Some(else_block) = &if_stmt.else_block {
                    collect_return_types(&else_block.statements, return_types, has_implicit_void);
                } else {
                    // If without else: the "no else" path might fall through
                    *has_implicit_void = true;
                }
            }
            Stmt::While(while_stmt) => {
                collect_return_types(&while_stmt.body.statements, return_types, has_implicit_void);
            }
            Stmt::For(for_stmt) => {
                collect_return_types(&for_stmt.body.statements, return_types, has_implicit_void);
            }
            Stmt::ForIn(for_in_stmt) => {
                collect_return_types(
                    &for_in_stmt.body.statements,
                    return_types,
                    has_implicit_void,
                );
            }
            _ => {}
        }
    }
}

/// Quick type inference for an expression (without full type checking).
///
/// This is a lightweight version used during return type inference.
/// It handles the common cases; the full type checker handles the rest.
pub fn infer_expr_type(expr: &Expr) -> Type {
    match expr {
        Expr::Literal(lit, _) => match lit {
            Literal::Number(_) => Type::Number,
            Literal::String(_) => Type::String,
            Literal::Bool(_) => Type::Bool,
            Literal::Null => Type::Null,
        },
        Expr::Binary(binary) => infer_binary_type(&binary.op),
        Expr::Unary(unary) => match unary.op {
            UnaryOp::Negate => Type::Number,
            UnaryOp::Not => Type::Bool,
        },
        Expr::ArrayLiteral(_) => Type::Array(Box::new(Type::Unknown)),
        Expr::Group(group) => infer_expr_type(&group.expr),
        _ => Type::Unknown,
    }
}

/// Infer the result type of a binary operation.
fn infer_binary_type(op: &BinaryOp) -> Type {
    match op {
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
            Type::Number // Could be string for Add, but default to number
        }
        BinaryOp::Eq
        | BinaryOp::Ne
        | BinaryOp::Lt
        | BinaryOp::Le
        | BinaryOp::Gt
        | BinaryOp::Ge
        | BinaryOp::And
        | BinaryOp::Or => Type::Bool,
    }
}

/// Check if an expected type is compatible with an inferred type,
/// allowing the expected type to guide inference.
///
/// In bidirectional checking, when we know the expected type from context
/// (e.g., variable annotation, return type), we can use it to validate
/// and refine inference results.
pub fn check_bidirectional(expected: &Type, inferred: &Type) -> BidirectionalResult {
    // Unknown can flow to any expected type
    if inferred.normalized() == Type::Unknown {
        return BidirectionalResult::Compatible;
    }
    if expected.normalized() == Type::Unknown {
        return BidirectionalResult::Compatible;
    }

    if inferred.is_assignable_to(expected) {
        BidirectionalResult::Compatible
    } else {
        BidirectionalResult::Mismatch {
            expected: expected.clone(),
            found: inferred.clone(),
        }
    }
}

/// Result of bidirectional type checking
#[derive(Debug, Clone, PartialEq)]
pub enum BidirectionalResult {
    /// Types are compatible
    Compatible,
    /// Types don't match
    Mismatch { expected: Type, found: Type },
}

/// Compute the least upper bound (common type) of two types.
///
/// Returns `None` if the types are incompatible.
pub fn least_upper_bound(a: &Type, b: &Type) -> Option<Type> {
    let a_norm = a.normalized();
    let b_norm = b.normalized();

    if a_norm == b_norm {
        return Some(a_norm);
    }

    // Unknown is subsumed by any concrete type
    if a_norm == Type::Unknown {
        return Some(b_norm);
    }
    if b_norm == Type::Unknown {
        return Some(a_norm);
    }

    // Arrays: LUB of element types
    if let (Type::Array(ea), Type::Array(eb)) = (&a_norm, &b_norm) {
        return least_upper_bound(ea, eb).map(|lub| Type::Array(Box::new(lub)));
    }

    if let Type::Union(mut members) = a_norm.clone() {
        members.push(b_norm.clone());
        return Some(Type::union(members));
    }
    if let Type::Union(mut members) = b_norm.clone() {
        members.push(a_norm.clone());
        return Some(Type::union(members));
    }

    if a.is_assignable_to(b) {
        return Some(b.clone());
    }
    if b.is_assignable_to(a) {
        return Some(a.clone());
    }

    // No common type - fall back to union
    Some(Type::union(vec![a.clone(), b.clone()]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    // ── CheckingMode ─────────────────────────────────────────────────────────

    #[test]
    fn test_checking_mode_synthesis_has_no_expected() {
        let mode = CheckingMode::Synthesis;
        assert_eq!(mode.expected_type(), None);
        assert!(!mode.has_expected());
    }

    #[test]
    fn test_checking_mode_checking_has_expected() {
        let mode = CheckingMode::Checking(Type::Number);
        assert_eq!(mode.expected_type(), Some(&Type::Number));
        assert!(mode.has_expected());
    }

    #[test]
    fn test_checking_mode_with_expected() {
        let mode = CheckingMode::with_expected(Type::String);
        assert_eq!(mode.expected_type(), Some(&Type::String));
    }

    #[test]
    fn test_checking_mode_synthesis_constructor() {
        let mode = CheckingMode::synthesis();
        assert_eq!(mode, CheckingMode::Synthesis);
    }

    // ── BidirectionalChecker ─────────────────────────────────────────────────

    #[test]
    fn test_bidirectional_synthesis_validates_any() {
        let checker = BidirectionalChecker::synthesis();
        assert_eq!(
            checker.validate(&Type::Number),
            BidirectionalResult::Compatible
        );
        assert_eq!(
            checker.validate(&Type::String),
            BidirectionalResult::Compatible
        );
    }

    #[test]
    fn test_bidirectional_checking_compatible() {
        let checker = BidirectionalChecker::checking(Type::Number);
        assert_eq!(
            checker.validate(&Type::Number),
            BidirectionalResult::Compatible
        );
    }

    #[test]
    fn test_bidirectional_checking_mismatch() {
        let checker = BidirectionalChecker::checking(Type::Number);
        let result = checker.validate(&Type::String);
        assert!(matches!(result, BidirectionalResult::Mismatch { .. }));
    }

    #[test]
    fn test_bidirectional_switch_to_synthesis() {
        let mut checker = BidirectionalChecker::checking(Type::Number);
        checker.switch_to_synthesis();
        assert_eq!(
            checker.validate(&Type::String),
            BidirectionalResult::Compatible
        );
    }

    #[test]
    fn test_bidirectional_switch_to_checking() {
        let mut checker = BidirectionalChecker::synthesis();
        checker.switch_to_checking(Type::Bool);
        assert_eq!(
            checker.validate(&Type::Bool),
            BidirectionalResult::Compatible
        );
        let result = checker.validate(&Type::Number);
        assert!(matches!(result, BidirectionalResult::Mismatch { .. }));
    }

    #[test]
    fn test_bidirectional_propagate_expected_from_checking() {
        let checker = BidirectionalChecker::checking(Type::Number);
        let mode = checker.propagate_expected(None);
        assert_eq!(mode, CheckingMode::Checking(Type::Number));
    }

    #[test]
    fn test_bidirectional_propagate_expected_from_synthesis_with_hint() {
        let checker = BidirectionalChecker::synthesis();
        let mode = checker.propagate_expected(Some(&Type::String));
        assert_eq!(mode, CheckingMode::Checking(Type::String));
    }

    #[test]
    fn test_bidirectional_propagate_expected_pure_synthesis() {
        let checker = BidirectionalChecker::synthesis();
        let mode = checker.propagate_expected(None);
        assert_eq!(mode, CheckingMode::Synthesis);
    }

    // ── LetPolymorphism ──────────────────────────────────────────────────────

    #[test]
    fn test_let_poly_classify_literal_is_syntactic_value() {
        let expr = Expr::Literal(Literal::Number(42.0), Span::dummy());
        assert_eq!(
            LetPolymorphism::classify_expr(&expr),
            BindingKind::SyntacticValue
        );
    }

    #[test]
    fn test_let_poly_classify_array_literal_is_syntactic_value() {
        let expr = Expr::ArrayLiteral(crate::ast::ArrayLiteral {
            elements: vec![],
            span: Span::dummy(),
        });
        assert_eq!(
            LetPolymorphism::classify_expr(&expr),
            BindingKind::SyntacticValue
        );
    }

    #[test]
    fn test_let_poly_classify_call_is_non_value() {
        let expr = Expr::Call(crate::ast::CallExpr {
            callee: Box::new(Expr::Identifier(crate::ast::Identifier {
                name: "f".to_string(),
                span: Span::dummy(),
            })),
            args: vec![],
            span: Span::dummy(),
        });
        assert_eq!(LetPolymorphism::classify_expr(&expr), BindingKind::NonValue);
    }

    #[test]
    fn test_let_poly_not_generalizable_if_mutable() {
        let expr = Expr::Literal(Literal::Number(1.0), Span::dummy());
        let ty = Type::TypeParameter {
            name: "T".to_string(),
        };
        assert!(!LetPolymorphism::is_generalizable(&ty, &expr, true));
    }

    #[test]
    fn test_let_poly_not_generalizable_non_value() {
        let expr = Expr::Call(crate::ast::CallExpr {
            callee: Box::new(Expr::Identifier(crate::ast::Identifier {
                name: "f".to_string(),
                span: Span::dummy(),
            })),
            args: vec![],
            span: Span::dummy(),
        });
        let ty = Type::TypeParameter {
            name: "T".to_string(),
        };
        assert!(!LetPolymorphism::is_generalizable(&ty, &expr, false));
    }

    #[test]
    fn test_let_poly_quantify_free_vars() {
        let ty = Type::Function {
            type_params: vec![],
            params: vec![Type::TypeParameter {
                name: "T".to_string(),
            }],
            return_type: Box::new(Type::TypeParameter {
                name: "U".to_string(),
            }),
        };
        let vars = LetPolymorphism::quantify_free_vars(&ty);
        assert!(vars.contains(&"T".to_string()));
        assert!(vars.contains(&"U".to_string()));
    }

    #[test]
    fn test_let_poly_quantify_no_vars_for_concrete() {
        let ty = Type::Number;
        let vars = LetPolymorphism::quantify_free_vars(&ty);
        assert!(vars.is_empty());
    }

    // ── InferenceHeuristics ──────────────────────────────────────────────────

    #[test]
    fn test_heuristics_prefer_simple_picks_number() {
        let types = vec![Type::Array(Box::new(Type::Number)), Type::Number];
        let result = InferenceHeuristics::prefer_simple(&types);
        assert_eq!(result, Some(Type::Number));
    }

    #[test]
    fn test_heuristics_prefer_simple_empty() {
        assert_eq!(InferenceHeuristics::prefer_simple(&[]), None);
    }

    #[test]
    fn test_heuristics_prefer_primitive_number_first() {
        let types = vec![Type::String, Type::Number, Type::Bool];
        let result = InferenceHeuristics::prefer_primitive(&types);
        assert_eq!(result, Some(Type::Number));
    }

    #[test]
    fn test_heuristics_prefer_primitive_string_over_bool() {
        let types = vec![Type::Bool, Type::String];
        let result = InferenceHeuristics::prefer_primitive(&types);
        assert_eq!(result, Some(Type::String));
    }

    #[test]
    fn test_heuristics_prefer_primitive_no_primitives() {
        let types = vec![Type::Array(Box::new(Type::Number))];
        let result = InferenceHeuristics::prefer_primitive(&types);
        assert!(result.is_some());
    }

    #[test]
    fn test_heuristics_infer_union_from_branches_same() {
        let types = vec![Type::Number, Type::Number];
        let result = InferenceHeuristics::infer_union_from_branches(types);
        assert_eq!(result, Type::Number);
    }

    #[test]
    fn test_heuristics_infer_union_from_branches_different() {
        let types = vec![Type::Number, Type::String];
        let result = InferenceHeuristics::infer_union_from_branches(types);
        assert!(matches!(result, Type::Union(_)));
    }

    #[test]
    fn test_heuristics_infer_union_from_branches_empty() {
        let result = InferenceHeuristics::infer_union_from_branches(vec![]);
        assert_eq!(result.normalized(), Type::Unknown);
    }

    #[test]
    fn test_heuristics_infer_union_skips_never() {
        let types = vec![Type::Number, Type::Never];
        let result = InferenceHeuristics::infer_union_from_branches(types);
        assert_eq!(result.normalized(), Type::Number);
    }

    #[test]
    fn test_heuristics_literal_type_number() {
        let expr = Expr::Literal(Literal::Number(1.0), Span::dummy());
        assert_eq!(
            InferenceHeuristics::infer_literal_type(&expr),
            Some(Type::Number)
        );
    }

    #[test]
    fn test_heuristics_literal_type_string() {
        let expr = Expr::Literal(Literal::String("hi".into()), Span::dummy());
        assert_eq!(
            InferenceHeuristics::infer_literal_type(&expr),
            Some(Type::String)
        );
    }

    #[test]
    fn test_heuristics_literal_type_bool() {
        let expr = Expr::Literal(Literal::Bool(true), Span::dummy());
        assert_eq!(
            InferenceHeuristics::infer_literal_type(&expr),
            Some(Type::Bool)
        );
    }

    #[test]
    fn test_heuristics_literal_type_null() {
        let expr = Expr::Literal(Literal::Null, Span::dummy());
        assert_eq!(
            InferenceHeuristics::infer_literal_type(&expr),
            Some(Type::Null)
        );
    }

    #[test]
    fn test_heuristics_literal_type_non_literal() {
        let expr = Expr::Identifier(crate::ast::Identifier {
            name: "x".to_string(),
            span: Span::dummy(),
        });
        assert_eq!(InferenceHeuristics::infer_literal_type(&expr), None);
    }

    #[test]
    fn test_heuristics_minimize_type_variables() {
        let ty = Type::TypeParameter {
            name: "T".to_string(),
        };
        let result = InferenceHeuristics::minimize_type_variables(&ty);
        assert_eq!(result, Type::Unknown);
    }

    #[test]
    fn test_heuristics_minimize_in_array() {
        let ty = Type::Array(Box::new(Type::TypeParameter {
            name: "T".to_string(),
        }));
        let result = InferenceHeuristics::minimize_type_variables(&ty);
        assert_eq!(result, Type::Array(Box::new(Type::Unknown)));
    }

    #[test]
    fn test_heuristics_minimize_concrete_unchanged() {
        let ty = Type::Number;
        let result = InferenceHeuristics::minimize_type_variables(&ty);
        assert_eq!(result, Type::Number);
    }

    #[test]
    fn test_infer_literal_types() {
        assert_eq!(
            infer_expr_type(&Expr::Literal(
                Literal::Number(42.0),
                crate::span::Span::dummy()
            )),
            Type::Number
        );
        assert_eq!(
            infer_expr_type(&Expr::Literal(
                Literal::String("hi".into()),
                crate::span::Span::dummy()
            )),
            Type::String
        );
        assert_eq!(
            infer_expr_type(&Expr::Literal(
                Literal::Bool(true),
                crate::span::Span::dummy()
            )),
            Type::Bool
        );
    }

    #[test]
    fn test_bidirectional_compatible() {
        assert_eq!(
            check_bidirectional(&Type::Number, &Type::Number),
            BidirectionalResult::Compatible
        );
    }

    #[test]
    fn test_bidirectional_mismatch() {
        let result = check_bidirectional(&Type::Number, &Type::String);
        assert!(matches!(result, BidirectionalResult::Mismatch { .. }));
    }

    #[test]
    fn test_bidirectional_unknown_compatible() {
        assert_eq!(
            check_bidirectional(&Type::Number, &Type::Unknown),
            BidirectionalResult::Compatible
        );
    }

    #[test]
    fn test_least_upper_bound_same() {
        assert_eq!(
            least_upper_bound(&Type::Number, &Type::Number),
            Some(Type::Number)
        );
    }

    #[test]
    fn test_least_upper_bound_incompatible() {
        assert_eq!(
            least_upper_bound(&Type::Number, &Type::String),
            Some(Type::union(vec![Type::Number, Type::String]))
        );
    }

    #[test]
    fn test_least_upper_bound_unknown() {
        assert_eq!(
            least_upper_bound(&Type::Unknown, &Type::Number),
            Some(Type::Number)
        );
    }

    #[test]
    fn test_least_upper_bound_arrays() {
        assert_eq!(
            least_upper_bound(
                &Type::Array(Box::new(Type::Number)),
                &Type::Array(Box::new(Type::Number))
            ),
            Some(Type::Array(Box::new(Type::Number)))
        );
    }

    #[test]
    fn test_infer_return_void() {
        let block = Block {
            statements: vec![],
            span: crate::span::Span::dummy(),
        };
        assert_eq!(infer_return_type(&block), InferredReturn::Void);
    }

    #[test]
    fn test_infer_return_uniform() {
        let block = Block {
            statements: vec![Stmt::Return(ReturnStmt {
                value: Some(Expr::Literal(
                    Literal::Number(42.0),
                    crate::span::Span::dummy(),
                )),
                span: crate::span::Span::dummy(),
            })],
            span: crate::span::Span::dummy(),
        };
        assert_eq!(
            infer_return_type(&block),
            InferredReturn::Uniform(Type::Number)
        );
    }
}
