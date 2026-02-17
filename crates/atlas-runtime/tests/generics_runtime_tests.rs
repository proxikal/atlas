//! Runtime tests for generic functions (Interpreter)
//!
//! BLOCKER 02-C: Monomorphization infrastructure
//!
//! These tests verify that the monomorphization infrastructure is in place
//! and ready for full generic function execution (which requires integration
//! with the full compilation pipeline in later phases).

use atlas_runtime::typechecker::generics::Monomorphizer;
use atlas_runtime::types::{Type, TypeParamDef};

#[test]
fn test_monomorphizer_substitutions() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![TypeParamDef {
        name: "T".to_string(),
        bound: None,
    }];
    let type_args = vec![Type::Number];

    let subst = mono
        .get_substitutions("identity", &type_params, &type_args)
        .unwrap();

    assert_eq!(subst.len(), 1);
    assert_eq!(subst.get("T"), Some(&Type::Number));
}

#[test]
fn test_monomorphizer_multiple_instantiations() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![TypeParamDef {
        name: "T".to_string(),
        bound: None,
    }];

    // identity<number>
    let subst1 = mono
        .get_substitutions("identity", &type_params, &[Type::Number])
        .unwrap();
    assert_eq!(subst1.get("T"), Some(&Type::Number));

    // identity<string>
    let subst2 = mono
        .get_substitutions("identity", &type_params, &[Type::String])
        .unwrap();
    assert_eq!(subst2.get("T"), Some(&Type::String));

    // identity<bool>
    let subst3 = mono
        .get_substitutions("identity", &type_params, &[Type::Bool])
        .unwrap();
    assert_eq!(subst3.get("T"), Some(&Type::Bool));

    // Should have 3 cached instances
    assert_eq!(mono.instance_count(), 3);
}

#[test]
fn test_monomorphizer_complex_types() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![TypeParamDef {
        name: "T".to_string(),
        bound: None,
    }];

    // Array types
    let array_number = Type::Array(Box::new(Type::Number));
    let subst = mono
        .get_substitutions("process", &type_params, std::slice::from_ref(&array_number))
        .unwrap();

    assert_eq!(subst.get("T"), Some(&array_number));
}

#[test]
fn test_monomorphizer_multiple_type_params() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![
        TypeParamDef {
            name: "K".to_string(),
            bound: None,
        },
        TypeParamDef {
            name: "V".to_string(),
            bound: None,
        },
    ];
    let type_args = vec![Type::String, Type::Number];

    let subst = mono
        .get_substitutions("map", &type_params, &type_args)
        .unwrap();

    assert_eq!(subst.len(), 2);
    assert_eq!(subst.get("K"), Some(&Type::String));
    assert_eq!(subst.get("V"), Some(&Type::Number));
}

#[test]
fn test_name_mangling() {
    // No type args
    assert_eq!(Monomorphizer::mangle_name("foo", &[]), "foo");

    // Single type arg
    assert_eq!(
        Monomorphizer::mangle_name("identity", &[Type::Number]),
        "identity$number"
    );

    // Multiple type args
    assert_eq!(
        Monomorphizer::mangle_name("map", &[Type::String, Type::Number]),
        "map$string$number"
    );

    // Complex types
    let array_type = Type::Array(Box::new(Type::Bool));
    assert_eq!(
        Monomorphizer::mangle_name("process", &[array_type]),
        "process$bool[]"
    );
}

#[test]
fn test_monomorphizer_caching() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![TypeParamDef {
        name: "T".to_string(),
        bound: None,
    }];
    let type_args = vec![Type::Number];

    // First call - should create new instance
    mono.get_substitutions("identity", &type_params, &type_args)
        .unwrap();
    assert_eq!(mono.instance_count(), 1);

    // Second call with same args - should use cache
    mono.get_substitutions("identity", &type_params, &type_args)
        .unwrap();
    assert_eq!(mono.instance_count(), 1); // Still 1

    // Clear cache
    mono.clear_cache();
    assert_eq!(mono.instance_count(), 0);

    // After clear, should create new instance again
    mono.get_substitutions("identity", &type_params, &type_args)
        .unwrap();
    assert_eq!(mono.instance_count(), 1);
}

#[test]
fn test_monomorphizer_generic_types() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![TypeParamDef {
        name: "T".to_string(),
        bound: None,
    }];

    // Option<number>
    let option_number = Type::Generic {
        name: "Option".to_string(),
        type_args: vec![Type::Number],
    };

    let subst = mono
        .get_substitutions("unwrap", &type_params, std::slice::from_ref(&option_number))
        .unwrap();

    assert_eq!(subst.get("T"), Some(&option_number));
}

#[test]
fn test_monomorphizer_nested_generics() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![TypeParamDef {
        name: "T".to_string(),
        bound: None,
    }];

    // Option<Result<number, string>>
    let result_type = Type::Generic {
        name: "Result".to_string(),
        type_args: vec![Type::Number, Type::String],
    };
    let option_result = Type::Generic {
        name: "Option".to_string(),
        type_args: vec![result_type.clone()],
    };

    let subst = mono
        .get_substitutions(
            "process",
            &type_params,
            std::slice::from_ref(&option_result),
        )
        .unwrap();

    assert_eq!(subst.get("T"), Some(&option_result));
}

// Additional integration tests will be added in BLOCKER 02-D
// when Option<T> and Result<T,E> are fully implemented.
