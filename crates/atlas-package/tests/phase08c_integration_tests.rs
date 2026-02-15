//! Integration tests for phase-08c: resolver + registry + lockfile + conflict + build order

use atlas_package::{
    BuildOrderComputer, ConflictResolver, Dependency, LockedPackage, LockedSource, Lockfile,
    PackageManifest, Resolution, ResolvedPackage, Resolver,
};
use rstest::rstest;
use semver::Version;
use std::collections::HashMap;

// Test helper: Create a simple manifest
fn create_test_manifest(name: &str, version: &str, deps: Vec<(&str, &str)>) -> PackageManifest {
    use atlas_package::manifest::PackageMetadata;

    let mut dependencies = HashMap::new();
    for (dep_name, dep_version) in deps {
        dependencies.insert(
            dep_name.to_string(),
            Dependency::Simple(dep_version.to_string()),
        );
    }

    PackageManifest {
        package: PackageMetadata {
            name: name.to_string(),
            version: version.parse().unwrap(),
            description: None,
            authors: vec![],
            license: None,
            repository: None,
            homepage: None,
            keywords: vec![],
            categories: vec![],
        },
        dependencies,
        dev_dependencies: HashMap::new(),
        build: None,
        lib: None,
        bin: vec![],
        workspace: None,
        features: HashMap::new(),
    }
}

// Test helper: Create a lockfile with packages
fn create_test_lockfile(packages: Vec<(&str, &str, Vec<(&str, &str)>)>) -> Lockfile {
    let mut lockfile = Lockfile::new();

    for (name, version, deps) in packages {
        let mut dependencies = HashMap::new();
        for (dep_name, dep_version) in deps {
            dependencies.insert(dep_name.to_string(), dep_version.parse().unwrap());
        }

        lockfile.add_package(LockedPackage {
            name: name.to_string(),
            version: version.parse().unwrap(),
            source: LockedSource::Registry { registry: None },
            checksum: Some("mock_checksum".to_string()),
            dependencies,
        });
    }

    lockfile
}

#[test]
fn test_full_resolution_pipeline() {
    // Manifest -> resolver -> resolution -> lockfile
    let manifest = create_test_manifest("myapp", "1.0.0", vec![("dep1", "^1.0")]);

    let mut resolver = Resolver::new();
    let resolution = resolver.resolve(&manifest).unwrap();

    // Should resolve the dependency
    assert!(resolution.packages.contains_key("dep1"));

    // Generate lockfile from resolution
    let lockfile = resolver.generate_lockfile(&resolution);
    assert_eq!(lockfile.packages.len(), resolution.packages.len());
    assert!(lockfile.get_package("dep1").is_some());
}

#[test]
fn test_resolve_with_existing_lockfile() {
    let manifest = create_test_manifest("myapp", "1.0.0", vec![("dep1", "^1.0")]);

    // Create a valid lockfile
    let lockfile = create_test_lockfile(vec![("dep1", "1.2.0", vec![])]);

    let mut resolver = Resolver::new();
    let resolution = resolver
        .resolve_with_lockfile(&manifest, Some(&lockfile))
        .unwrap();

    // Should use lockfile version (1.2.0)
    let dep1 = resolution.packages.get("dep1").unwrap();
    assert_eq!(dep1.version, Version::new(1, 2, 0));
}

#[test]
fn test_lockfile_regeneration_on_manifest_change() {
    let manifest = create_test_manifest("myapp", "1.0.0", vec![("dep1", "^1.0")]);

    // Create lockfile with incompatible version (2.0.0 doesn't match ^1.0)
    let lockfile = create_test_lockfile(vec![("dep1", "2.0.0", vec![])]);

    let mut resolver = Resolver::new();
    let resolution = resolver
        .resolve_with_lockfile(&manifest, Some(&lockfile))
        .unwrap();

    // Should re-resolve since lockfile is invalid
    let dep1 = resolution.packages.get("dep1").unwrap();
    // Fresh resolution would pick latest 1.x version
    assert!(dep1.version.major == 1);
}

#[test]
fn test_lockfile_missing_dependency() {
    let manifest = create_test_manifest("myapp", "1.0.0", vec![("dep1", "^1.0"), ("dep2", "^1.0")]);

    // Lockfile only has dep1, missing dep2
    let lockfile = create_test_lockfile(vec![("dep1", "1.0.0", vec![])]);

    let mut resolver = Resolver::new();
    let resolution = resolver
        .resolve_with_lockfile(&manifest, Some(&lockfile))
        .unwrap();

    // Should re-resolve since lockfile is incomplete
    assert!(resolution.packages.contains_key("dep1"));
    assert!(resolution.packages.contains_key("dep2"));
}

#[test]
fn test_generate_lockfile_with_metadata() {
    let manifest = create_test_manifest("myapp", "1.0.0", vec![("dep1", "^1.0")]);

    let mut resolver = Resolver::new();
    let resolution = resolver.resolve(&manifest).unwrap();
    let lockfile = resolver.generate_lockfile(&resolution);

    // Check metadata is populated
    assert!(lockfile.metadata.generated_at.is_some());
    assert!(lockfile.metadata.atlas_version.is_some());
}

#[test]
fn test_lockfile_preserves_dependencies() {
    let manifest = create_test_manifest("myapp", "1.0.0", vec![("dep1", "^1.0")]);

    let mut resolver = Resolver::new();
    let resolution = resolver.resolve(&manifest).unwrap();
    let lockfile = resolver.generate_lockfile(&resolution);

    // Verify lockfile structure
    assert_eq!(lockfile.version, Lockfile::VERSION);
    assert_eq!(lockfile.packages.len(), resolution.packages.len());

    for pkg in &lockfile.packages {
        assert!(resolution.packages.contains_key(&pkg.name));
    }
}

// Conflict Resolution Tests

#[test]
fn test_detect_simple_conflict() {
    use atlas_package::resolver::VersionConstraint;

    let mut resolver = ConflictResolver::new();
    let mut constraints = HashMap::new();

    // ^1.0 and ^2.0 conflict
    constraints.insert(
        "conflicting-pkg".to_string(),
        vec![
            VersionConstraint {
                requirement: "^1.0".parse().unwrap(),
                source: "pkg-a".to_string(),
            },
            VersionConstraint {
                requirement: "^2.0".parse().unwrap(),
                source: "pkg-b".to_string(),
            },
        ],
    );

    let conflicts = resolver.detect_conflicts(&constraints);
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].package, "conflicting-pkg");
    assert_eq!(conflicts[0].constraint_count(), 2);
}

#[test]
fn test_no_conflict_compatible_constraints() {
    use atlas_package::resolver::VersionConstraint;

    let mut resolver = ConflictResolver::new();
    let mut constraints = HashMap::new();

    // ^1.0 and ^1.1 are compatible (1.1.0 satisfies both)
    constraints.insert(
        "compatible-pkg".to_string(),
        vec![
            VersionConstraint {
                requirement: "^1.0".parse().unwrap(),
                source: "pkg-a".to_string(),
            },
            VersionConstraint {
                requirement: "^1.1".parse().unwrap(),
                source: "pkg-b".to_string(),
            },
        ],
    );

    let conflicts = resolver.detect_conflicts(&constraints);
    assert!(conflicts.is_empty());
}

#[test]
fn test_conflict_report_formatting() {
    use atlas_package::{Conflict, ConflictingConstraint};

    let conflict = Conflict::new(
        "test-pkg".to_string(),
        vec![
            ConflictingConstraint::new("^1.0".parse().unwrap(), "source-a".to_string()),
            ConflictingConstraint::new("^2.0".parse().unwrap(), "source-b".to_string()),
        ],
    );

    let report = conflict.report();
    assert!(report.contains("test-pkg"));
    assert!(report.contains("source-a"));
    assert!(report.contains("source-b"));
    assert!(report.contains("Possible solutions"));
}

#[test]
fn test_suggest_conflict_resolutions() {
    use atlas_package::{Conflict, ConflictingConstraint};

    let resolver = ConflictResolver::new();
    let conflict = Conflict::new(
        "test-pkg".to_string(),
        vec![
            ConflictingConstraint::new("^1.0".parse().unwrap(), "pkg-a".to_string()),
            ConflictingConstraint::new("^2.0".parse().unwrap(), "pkg-b".to_string()),
        ],
    );

    let suggestions = resolver.suggest_resolutions(&conflict);
    assert!(!suggestions.is_empty());
    assert!(suggestions.len() >= 3);
}

// Build Order Tests (also in build_order_tests.rs)

#[test]
fn test_build_order_from_resolution() {
    let mut resolution = Resolution::new();

    resolution.add_package(ResolvedPackage::with_dependencies(
        "pkg1".to_string(),
        Version::new(1, 0, 0),
        vec!["pkg2".to_string()],
    ));
    resolution.add_package(ResolvedPackage::with_dependencies(
        "pkg2".to_string(),
        Version::new(1, 0, 0),
        vec![],
    ));

    let computer = BuildOrderComputer::new(&resolution);
    let order = computer.compute_build_order().unwrap();

    // pkg2 should come before pkg1
    let pkg2_idx = order.iter().position(|p| p == "pkg2").unwrap();
    let pkg1_idx = order.iter().position(|p| p == "pkg1").unwrap();
    assert!(pkg2_idx < pkg1_idx);
}

#[test]
fn test_parallel_build_groups_from_resolution() {
    let mut resolution = Resolution::new();

    // Independent packages
    resolution.add_package(ResolvedPackage::with_dependencies(
        "pkg1".to_string(),
        Version::new(1, 0, 0),
        vec![],
    ));
    resolution.add_package(ResolvedPackage::with_dependencies(
        "pkg2".to_string(),
        Version::new(1, 0, 0),
        vec![],
    ));
    resolution.add_package(ResolvedPackage::with_dependencies(
        "pkg3".to_string(),
        Version::new(1, 0, 0),
        vec![],
    ));

    let computer = BuildOrderComputer::new(&resolution);
    let groups = computer.parallel_build_groups().unwrap();

    // All three should be in the same group (parallel)
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].len(), 3);
}

#[test]
fn test_end_to_end_resolution_to_lockfile_to_build_order() {
    // Full integration: manifest -> resolution -> lockfile -> build order
    let manifest = create_test_manifest("myapp", "1.0.0", vec![("dep1", "^1.0"), ("dep2", "^1.0")]);

    // Resolve
    let mut resolver = Resolver::new();
    let resolution = resolver.resolve(&manifest).unwrap();

    // Generate lockfile
    let lockfile = resolver.generate_lockfile(&resolution);
    assert_eq!(lockfile.packages.len(), resolution.packages.len());

    // Compute build order
    let computer = BuildOrderComputer::new(&resolution);
    let order = computer.compute_build_order().unwrap();
    assert!(!order.is_empty());

    // Verify lockfile can be parsed back
    let toml = lockfile.to_string().unwrap();
    let parsed_lockfile = Lockfile::from_str(&toml).unwrap();
    assert_eq!(parsed_lockfile.packages.len(), lockfile.packages.len());
}

#[rstest]
#[case("^1.0", "1.0.0", true)]
#[case("^1.0", "1.5.0", true)]
#[case("^1.0", "2.0.0", false)]
#[case("~1.2", "1.2.5", true)]
#[case("~1.2", "1.3.0", false)]
fn test_lockfile_version_compatibility(
    #[case] constraint: &str,
    #[case] locked_version: &str,
    #[case] should_be_valid: bool,
) {
    let manifest = create_test_manifest("myapp", "1.0.0", vec![("dep1", constraint)]);
    let lockfile = create_test_lockfile(vec![("dep1", locked_version, vec![])]);

    let mut resolver = Resolver::new();
    let resolution = resolver
        .resolve_with_lockfile(&manifest, Some(&lockfile))
        .unwrap();

    let dep1 = resolution.packages.get("dep1").unwrap();

    if should_be_valid {
        // Should use lockfile version
        assert_eq!(dep1.version.to_string(), locked_version);
    } else {
        // Should re-resolve (lockfile invalid)
        // Fresh resolution would pick latest compatible version
        assert!(dep1.version.to_string() != locked_version);
    }
}

#[test]
fn test_lockfile_integrity_verification() {
    let mut lockfile = Lockfile::new();

    lockfile.add_package(LockedPackage {
        name: "pkg1".to_string(),
        version: Version::new(1, 0, 0),
        source: LockedSource::Registry { registry: None },
        checksum: Some("abc123".to_string()),
        dependencies: HashMap::new(),
    });

    // Should verify successfully
    assert!(lockfile.verify().is_ok());

    // Add duplicate (manually to bypass add_package deduplication)
    lockfile.packages.push(LockedPackage {
        name: "pkg1".to_string(),
        version: Version::new(2, 0, 0),
        source: LockedSource::Registry { registry: None },
        checksum: Some("def456".to_string()),
        dependencies: HashMap::new(),
    });

    // Should fail verification
    assert!(lockfile.verify().is_err());
}

#[test]
fn test_transitive_dependency_in_lockfile() {
    // Create resolution with transitive dependency
    // myapp -> dep1 -> dep2
    let mut resolution = Resolution::new();

    resolution.add_package(ResolvedPackage::with_dependencies(
        "dep2".to_string(),
        Version::new(1, 0, 0),
        vec![],
    ));
    resolution.add_package(ResolvedPackage::with_dependencies(
        "dep1".to_string(),
        Version::new(1, 0, 0),
        vec!["dep2".to_string()],
    ));

    let resolver = Resolver::new();
    let lockfile = resolver.generate_lockfile(&resolution);

    // Both should be in lockfile
    assert!(lockfile.get_package("dep1").is_some());
    assert!(lockfile.get_package("dep2").is_some());

    // dep1 should list dep2 in its dependencies
    let dep1_locked = lockfile.get_package("dep1").unwrap();
    assert!(dep1_locked.dependencies.contains_key("dep2"));
    assert_eq!(
        dep1_locked.dependencies.get("dep2").unwrap(),
        &Version::new(1, 0, 0)
    );
}
