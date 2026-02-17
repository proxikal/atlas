//! Generic constraint checking

use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::typechecker::generics::TypeInferer;
use crate::typechecker::methods::MethodTable;
use crate::types::{StructuralMemberType, Type, TypeParamDef};

/// Validate inferred type arguments against declared constraints.
pub fn check_constraints(
    type_params: &[TypeParamDef],
    inferer: &TypeInferer,
    method_table: &MethodTable,
    diagnostics: &mut Vec<Diagnostic>,
    span: Span,
) -> bool {
    let mut ok = true;

    for param in type_params {
        let Some(bound) = &param.bound else {
            continue;
        };
        let Some(actual) = inferer.get_substitution(&param.name) else {
            continue;
        };

        let bound = inferer.apply_substitutions(bound).normalized();
        if bound == Type::Never {
            diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3001",
                    format!(
                        "Conflicting constraints for type parameter '{}'",
                        param.name
                    ),
                    span,
                )
                .with_label("unsatisfiable constraint")
                .with_help("remove or simplify the conflicting constraints"),
            );
            ok = false;
            continue;
        }

        if let Err(details) = satisfies_constraint(actual, &bound, method_table) {
            diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3001",
                    format!(
                        "Type argument '{}' must satisfy constraint {}, found {}",
                        param.name,
                        bound.display_name(),
                        actual.display_name()
                    ),
                    span,
                )
                .with_label(format!("violates constraint {}", bound.display_name()))
                .with_help(details),
            );
            ok = false;
        }
    }

    ok
}

fn satisfies_constraint(
    actual: &Type,
    bound: &Type,
    method_table: &MethodTable,
) -> Result<(), String> {
    match bound.normalized() {
        Type::Union(members) => {
            for member in members {
                if satisfies_constraint(actual, &member, method_table).is_ok() {
                    return Ok(());
                }
            }
            Err(format!(
                "type {} does not satisfy any union member",
                actual.display_name()
            ))
        }
        Type::Intersection(members) => {
            for member in members {
                satisfies_constraint(actual, &member, method_table)?;
            }
            Ok(())
        }
        Type::Structural { members } => {
            satisfies_structural_constraint(actual, &members, method_table)
        }
        bound => {
            if actual.is_assignable_to(&bound) {
                Ok(())
            } else {
                Err(format!(
                    "expected {}, found {}",
                    bound.display_name(),
                    actual.display_name()
                ))
            }
        }
    }
}

fn satisfies_structural_constraint(
    actual: &Type,
    members: &[StructuralMemberType],
    method_table: &MethodTable,
) -> Result<(), String> {
    if let Type::Structural {
        members: actual_members,
    } = actual.normalized()
    {
        for required in members {
            let Some(found) = actual_members.iter().find(|m| m.name == required.name) else {
                return Err(format!("missing member '{}'", required.name));
            };
            if !found.ty.is_assignable_to(&required.ty) {
                return Err(format!(
                    "member '{}' must be {}, found {}",
                    required.name,
                    required.ty.display_name(),
                    found.ty.display_name()
                ));
            }
        }
        return Ok(());
    }

    // Treat function-typed members as method requirements when possible.
    for required in members {
        if let Type::Function {
            params,
            return_type,
            ..
        } = &required.ty
        {
            let Some(method_sig) = method_table.lookup(actual, &required.name) else {
                return Err(format!("missing method '{}'", required.name));
            };

            if method_sig.arg_types.len() != params.len() {
                return Err(format!(
                    "method '{}' expects {} argument(s), found {}",
                    required.name,
                    params.len(),
                    method_sig.arg_types.len()
                ));
            }

            for (idx, required_arg) in params.iter().enumerate() {
                let actual_arg = &method_sig.arg_types[idx];
                if !required_arg.is_assignable_to(actual_arg) {
                    return Err(format!(
                        "method '{}' argument {} must be {}, found {}",
                        required.name,
                        idx + 1,
                        required_arg.display_name(),
                        actual_arg.display_name()
                    ));
                }
            }

            if !method_sig.return_type.is_assignable_to(return_type) {
                return Err(format!(
                    "method '{}' must return {}, found {}",
                    required.name,
                    return_type.display_name(),
                    method_sig.return_type.display_name()
                ));
            }
            continue;
        }

        if matches!(actual.normalized(), Type::JsonValue) {
            let actual_field_type = Type::JsonValue;
            if !actual_field_type.is_assignable_to(&required.ty) {
                return Err(format!(
                    "field '{}' must be {}, found {}",
                    required.name,
                    required.ty.display_name(),
                    actual_field_type.display_name()
                ));
            }
            continue;
        }

        return Err(format!(
            "type {} does not support structural member '{}'",
            actual.display_name(),
            required.name
        ));
    }

    Ok(())
}
