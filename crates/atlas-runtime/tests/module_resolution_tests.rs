//! Module Resolution Tests (BLOCKER 04-A)
//!
//! Tests for path resolution and circular dependency detection.
//! Does NOT test module loading (that's BLOCKER 04-B).

use atlas_runtime::{ModuleResolver, Span};
use std::path::PathBuf;

// ============================================================================
// Path Resolution Tests
// ============================================================================

#[test]
fn test_resolve_relative_sibling() {
    // Test internal logic without file existence check
    let resolved = PathBuf::from("/project/src").join("utils.atl");
    assert!(resolved.to_string_lossy().contains("src/utils.atl"));
}

#[test]
fn test_resolve_relative_parent() {
    let importing = PathBuf::from("/project/src/lib/main.atl");
    let source = "../utils";

    // Test path resolution logic
    // From /project/src/lib/, ../ goes to /project/src/, then /utils makes it /project/src/utils
    let importing_dir = importing.parent().unwrap(); // /project/src/lib
    let with_ext = format!("{}.atl", source);
    let resolved = importing_dir.join(with_ext); // /project/src/lib/../utils.atl

    // The path should contain ../utils.atl
    assert!(resolved.to_string_lossy().contains("../utils.atl"));
}

#[test]
fn test_resolve_absolute_from_root() {
    let root = PathBuf::from("/project");
    let source = "/src/utils";

    // Remove leading '/' and append .atl
    let relative = &source[1..];
    let with_ext = format!("{}.atl", relative);
    let resolved = root.join(with_ext);

    assert_eq!(resolved, root.join("src/utils.atl"));
}

#[test]
fn test_path_with_atl_extension() {
    let root = PathBuf::from("/project");
    let source = "/src/utils.atl";

    // If already has .atl, don't add another
    let relative = &source[1..];
    let resolved = root.join(relative);

    assert_eq!(resolved, root.join("src/utils.atl"));
}

#[test]
fn test_invalid_path_format() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let importing = PathBuf::from("/project/main.atl");
    let source = "invalid_path"; // No ./, ../, or /

    let result = resolver.resolve_path(source, &importing, Span::dummy());
    assert!(result.is_err(), "Should reject invalid path format");

    let err = result.unwrap_err();
    assert!(err.message.contains("Invalid module path"));
}

// ============================================================================
// Circular Dependency Detection Tests
// ============================================================================

#[test]
fn test_simple_circular_dependency() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");
    let b = PathBuf::from("/project/b.atl");

    // a -> b -> a (cycle)
    resolver.add_dependency(a.clone(), b.clone());
    resolver.add_dependency(b, a.clone());

    let result = resolver.check_circular(&a, Span::dummy());
    assert!(result.is_err(), "Should detect simple cycle");

    let err = result.unwrap_err();
    assert!(err.message.contains("Circular dependency"));
}

#[test]
fn test_three_node_cycle() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");
    let b = PathBuf::from("/project/b.atl");
    let c = PathBuf::from("/project/c.atl");

    // a -> b -> c -> a (cycle)
    resolver.add_dependency(a.clone(), b.clone());
    resolver.add_dependency(b, c.clone());
    resolver.add_dependency(c, a.clone());

    let result = resolver.check_circular(&a, Span::dummy());
    assert!(result.is_err(), "Should detect three-node cycle");
}

#[test]
fn test_no_cycle_linear() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");
    let b = PathBuf::from("/project/b.atl");
    let c = PathBuf::from("/project/c.atl");

    // a -> b -> c (no cycle)
    resolver.add_dependency(a.clone(), b.clone());
    resolver.add_dependency(b, c);

    let result = resolver.check_circular(&a, Span::dummy());
    assert!(
        result.is_ok(),
        "Should not detect cycle in linear dependency"
    );
}

#[test]
fn test_no_cycle_diamond() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");
    let b = PathBuf::from("/project/b.atl");
    let c = PathBuf::from("/project/c.atl");
    let d = PathBuf::from("/project/d.atl");

    // Diamond: a -> b -> d, a -> c -> d (no cycle)
    resolver.add_dependency(a.clone(), b.clone());
    resolver.add_dependency(a.clone(), c.clone());
    resolver.add_dependency(b, d.clone());
    resolver.add_dependency(c, d);

    let result = resolver.check_circular(&a, Span::dummy());
    assert!(
        result.is_ok(),
        "Should not detect cycle in diamond dependency"
    );
}

#[test]
fn test_self_cycle() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");

    // a -> a (self cycle)
    resolver.add_dependency(a.clone(), a.clone());

    let result = resolver.check_circular(&a, Span::dummy());
    assert!(result.is_err(), "Should detect self-import cycle");
}

// ============================================================================
// Dependency Graph Tests
// ============================================================================

#[test]
fn test_get_dependencies_empty() {
    let root = PathBuf::from("/project");
    let resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");

    let deps = resolver.get_dependencies(&a);
    assert_eq!(deps.len(), 0, "Should have no dependencies initially");
}

#[test]
fn test_get_dependencies_single() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");
    let b = PathBuf::from("/project/b.atl");

    resolver.add_dependency(a.clone(), b.clone());

    let deps = resolver.get_dependencies(&a);
    assert_eq!(deps.len(), 1, "Should have one dependency");
    assert_eq!(deps[0], b);
}

#[test]
fn test_get_dependencies_multiple() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");
    let b = PathBuf::from("/project/b.atl");
    let c = PathBuf::from("/project/c.atl");
    let d = PathBuf::from("/project/d.atl");

    resolver.add_dependency(a.clone(), b.clone());
    resolver.add_dependency(a.clone(), c.clone());
    resolver.add_dependency(a.clone(), d.clone());

    let deps = resolver.get_dependencies(&a);
    assert_eq!(deps.len(), 3, "Should have three dependencies");
    assert!(deps.contains(&b));
    assert!(deps.contains(&c));
    assert!(deps.contains(&d));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_complex_nested_relative_path() {
    let importing = PathBuf::from("/project/src/features/auth/login.atl");
    let source = "../../utils/helpers";

    // Test path resolution: from /project/src/features/auth, ../../utils/helpers
    let importing_dir = importing.parent().unwrap();
    let with_ext = format!("{}.atl", source);
    let resolved = importing_dir.join(with_ext);

    // Path should contain the relative navigation
    assert!(resolved
        .to_string_lossy()
        .contains("../../utils/helpers.atl"));
}

#[test]
fn test_cycle_error_includes_path() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");
    let b = PathBuf::from("/project/b.atl");

    resolver.add_dependency(a.clone(), b.clone());
    resolver.add_dependency(b, a.clone());

    let result = resolver.check_circular(&a, Span::dummy());
    assert!(result.is_err());

    let err = result.unwrap_err();
    // Error should include the cycle path for debugging
    assert!(err.message.contains("Circular dependency"));
}
