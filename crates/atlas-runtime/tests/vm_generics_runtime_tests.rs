//! Runtime tests for generic functions (VM)
//!
//! BLOCKER 02-C: Monomorphization infrastructure
//!
//! These tests verify VM parity with interpreter for generic function support.
//! Full generic function execution tests will be added in BLOCKER 02-D when
//! Option<T> and Result<T,E> are implemented.

use atlas_runtime::typechecker::generics::Monomorphizer;
use atlas_runtime::types::{Type, TypeParamDef};

// VM uses the same monomorphization infrastructure as interpreter
// These tests verify the infrastructure works identically

#[test]
fn test_vm_monomorphizer_basic() {
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
fn test_vm_monomorphizer_multiple_types() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![TypeParamDef {
        name: "T".to_string(),
        bound: None,
    }];

    // Test number
    mono.get_substitutions("f", &type_params, &[Type::Number])
        .unwrap();

    // Test string
    mono.get_substitutions("f", &type_params, &[Type::String])
        .unwrap();

    // Test bool
    mono.get_substitutions("f", &type_params, &[Type::Bool])
        .unwrap();

    // Test array
    let array_type = Type::Array(Box::new(Type::Number));
    mono.get_substitutions("f", &type_params, &[array_type])
        .unwrap();

    // Should have 4 instances
    assert_eq!(mono.instance_count(), 4);
}

#[test]
fn test_vm_name_mangling() {
    // VM uses mangled names for function dispatch

    // Basic types
    assert_eq!(
        Monomorphizer::mangle_name("identity", &[Type::Number]),
        "identity$number"
    );
    assert_eq!(
        Monomorphizer::mangle_name("identity", &[Type::String]),
        "identity$string"
    );
    assert_eq!(
        Monomorphizer::mangle_name("identity", &[Type::Bool]),
        "identity$bool"
    );

    // Multiple type parameters
    assert_eq!(
        Monomorphizer::mangle_name("map", &[Type::String, Type::Number]),
        "map$string$number"
    );
    assert_eq!(
        Monomorphizer::mangle_name("fold", &[Type::Number, Type::String, Type::Bool]),
        "fold$number$string$bool"
    );
}

#[test]
fn test_vm_name_mangling_arrays() {
    let array_number = Type::Array(Box::new(Type::Number));
    assert_eq!(
        Monomorphizer::mangle_name("process", &[array_number]),
        "process$number[]"
    );

    let array_string = Type::Array(Box::new(Type::String));
    assert_eq!(
        Monomorphizer::mangle_name("process", &[array_string]),
        "process$string[]"
    );

    // Nested arrays
    let array_array = Type::Array(Box::new(Type::Array(Box::new(Type::Number))));
    assert_eq!(
        Monomorphizer::mangle_name("flatten", &[array_array]),
        "flatten$number[][]"
    );
}

#[test]
fn test_vm_monomorphizer_cache_efficiency() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![TypeParamDef {
        name: "T".to_string(),
        bound: None,
    }];
    let type_args = vec![Type::Number];

    // Multiple calls with same types should reuse cache
    for _ in 0..10 {
        mono.get_substitutions("identity", &type_params, &type_args)
            .unwrap();
    }

    // Should only have 1 cached instance
    assert_eq!(mono.instance_count(), 1);
}

#[test]
fn test_vm_monomorphizer_different_functions() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![TypeParamDef {
        name: "T".to_string(),
        bound: None,
    }];
    let type_args = vec![Type::Number];

    // Different functions with same type args should create separate instances
    mono.get_substitutions("identity", &type_params, &type_args)
        .unwrap();
    mono.get_substitutions("clone", &type_params, &type_args)
        .unwrap();
    mono.get_substitutions("process", &type_params, &type_args)
        .unwrap();

    assert_eq!(mono.instance_count(), 3);
}

#[test]
fn test_vm_generic_type_substitution() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![
        TypeParamDef {
            name: "T".to_string(),
            bound: None,
        },
        TypeParamDef {
            name: "E".to_string(),
            bound: None,
        },
    ];

    // Result<number, string>
    let type_args = vec![Type::Number, Type::String];

    let subst = mono
        .get_substitutions("result_map", &type_params, &type_args)
        .unwrap();

    assert_eq!(subst.get("T"), Some(&Type::Number));
    assert_eq!(subst.get("E"), Some(&Type::String));

    // Result<string, bool>
    let type_args2 = vec![Type::String, Type::Bool];

    let subst2 = mono
        .get_substitutions("result_map", &type_params, &type_args2)
        .unwrap();

    assert_eq!(subst2.get("T"), Some(&Type::String));
    assert_eq!(subst2.get("E"), Some(&Type::Bool));

    // Should have 2 instances
    assert_eq!(mono.instance_count(), 2);
}

#[test]
fn test_vm_complex_mangling() {
    // Generic types in mangling
    let option_type = Type::Generic {
        name: "Option".to_string(),
        type_args: vec![Type::Number],
    };

    assert_eq!(
        Monomorphizer::mangle_name("unwrap", &[option_type]),
        "unwrap$Option<number>"
    );

    // Nested generics
    let result_type = Type::Generic {
        name: "Result".to_string(),
        type_args: vec![Type::Number, Type::String],
    };
    let option_result = Type::Generic {
        name: "Option".to_string(),
        type_args: vec![result_type],
    };

    assert_eq!(
        Monomorphizer::mangle_name("process", &[option_result]),
        "process$Option<Result<number, string>>"
    );
}

// Full VM execution tests with bytecode generation will be added in BLOCKER 02-D
// when the complete generic function pipeline is integrated.
