use atlas_package::*;
use std::collections::HashMap;

mod manifest_parsing {
    use super::*;

    #[test]
    fn test_parse_manifest_with_all_fields() {
        let toml = r#"
            [package]
            name = "full-package"
            version = "1.2.3"
            description = "A complete package"
            authors = ["Alice <alice@example.com>", "Bob <bob@example.com>"]
            license = "MIT OR Apache-2.0"
            repository = "https://github.com/example/repo"
            homepage = "https://example.com"
            keywords = ["test", "example"]
            categories = ["development-tools"]

            [dependencies]
            foo = "1.0"
            bar = { version = "2.0", optional = true }

            [dev-dependencies]
            test-utils = "0.1"

            [build]
            optimize = "size"
            target = "wasm32"

            [lib]
            path = "src/lib.atl"
            name = "mylib"

            [[bin]]
            name = "mybinary"
            path = "src/main.atl"

            [features]
            default = { dependencies = [], default = true }
            extra = { dependencies = ["bar/feature"], default = false }
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        assert_eq!(manifest.package.name, "full-package");
        assert_eq!(manifest.package.version.to_string(), "1.2.3");
        assert_eq!(manifest.package.description, Some("A complete package".to_string()));
        assert_eq!(manifest.package.authors.len(), 2);
        assert_eq!(manifest.package.license, Some("MIT OR Apache-2.0".to_string()));
        assert_eq!(manifest.dependencies.len(), 2);
        assert_eq!(manifest.dev_dependencies.len(), 1);
        assert!(manifest.build.is_some());
        assert!(manifest.lib.is_some());
        assert_eq!(manifest.bin.len(), 1);
        assert_eq!(manifest.features.len(), 2);
    }

    #[test]
    fn test_parse_git_dependency() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"

            [dependencies]
            git-dep = { git = "https://github.com/example/repo", branch = "main" }
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        assert!(matches!(
            manifest.dependencies.get("git-dep"),
            Some(Dependency::Detailed(_))
        ));
    }

    #[test]
    fn test_parse_path_dependency() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"

            [dependencies]
            local = { path = "../local-package" }
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        assert!(matches!(
            manifest.dependencies.get("local"),
            Some(Dependency::Detailed(_))
        ));
    }

    #[test]
    fn test_parse_renamed_dependency() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"

            [dependencies]
            new-name = { version = "1.0", package = "old-name" }
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        if let Some(Dependency::Detailed(dep)) = manifest.dependencies.get("new-name") {
            assert_eq!(dep.rename, Some("old-name".to_string()));
        } else {
            panic!("Expected detailed dependency");
        }
    }

    #[test]
    fn test_parse_dependency_with_features() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"

            [dependencies]
            foo = { version = "1.0", features = ["feature1", "feature2"], default-features = false }
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        if let Some(Dependency::Detailed(dep)) = manifest.dependencies.get("foo") {
            assert_eq!(dep.features, Some(vec!["feature1".to_string(), "feature2".to_string()]));
            assert_eq!(dep.default_features, Some(false));
        } else {
            panic!("Expected detailed dependency");
        }
    }

    #[test]
    fn test_parse_workspace() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"

            [workspace]
            members = ["packages/*", "tools/cli"]
            exclude = ["packages/experimental"]

            [workspace.dependencies]
            shared-dep = "1.0"
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        let workspace = manifest.workspace.unwrap();
        assert_eq!(workspace.members.len(), 2);
        assert_eq!(workspace.exclude.len(), 1);
        assert_eq!(workspace.dependencies.len(), 1);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"

            [dependencies]
            foo = "1.0"
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        let serialized = manifest.to_string().unwrap();
        let deserialized = PackageManifest::from_str(&serialized).unwrap();
        assert_eq!(manifest, deserialized);
    }
}

mod version_constraints {
    use super::*;

    #[test]
    fn test_wildcard_version() {
        let constraint = VersionConstraint::parse("*").unwrap();
        assert!(matches!(constraint, VersionConstraint::Wildcard));
        assert!(constraint.matches(&semver::Version::new(0, 0, 1)));
        assert!(constraint.matches(&semver::Version::new(999, 999, 999)));
    }

    #[test]
    fn test_caret_major_zero() {
        let constraint = VersionConstraint::parse("^0.1.2").unwrap();
        assert!(constraint.matches(&semver::Version::new(0, 1, 2)));
        assert!(constraint.matches(&semver::Version::new(0, 1, 9)));
        assert!(!constraint.matches(&semver::Version::new(1, 0, 0)));
    }

    #[test]
    fn test_tilde_patch_versions() {
        let constraint = VersionConstraint::parse("~1.2.3").unwrap();
        assert!(constraint.matches(&semver::Version::new(1, 2, 3)));
        assert!(constraint.matches(&semver::Version::new(1, 2, 10)));
        assert!(!constraint.matches(&semver::Version::new(1, 3, 0)));
    }

    #[test]
    fn test_range_constraints() {
        let constraint = VersionConstraint::parse(">=1.0.0, <2.0.0").unwrap();
        assert!(matches!(constraint, VersionConstraint::Range(_)));
        assert!(constraint.matches(&semver::Version::new(1, 0, 0)));
        assert!(constraint.matches(&semver::Version::new(1, 9, 9)));
        assert!(!constraint.matches(&semver::Version::new(2, 0, 0)));
    }

    #[test]
    fn test_greater_than_constraint() {
        let constraint = VersionConstraint::parse(">1.0.0").unwrap();
        assert!(!constraint.matches(&semver::Version::new(1, 0, 0)));
        assert!(constraint.matches(&semver::Version::new(1, 0, 1)));
        assert!(constraint.matches(&semver::Version::new(2, 0, 0)));
    }

    #[test]
    fn test_less_than_or_equal() {
        let constraint = VersionConstraint::parse("<=1.5.0").unwrap();
        assert!(constraint.matches(&semver::Version::new(1, 0, 0)));
        assert!(constraint.matches(&semver::Version::new(1, 5, 0)));
        assert!(!constraint.matches(&semver::Version::new(1, 5, 1)));
    }

    #[test]
    fn test_exact_version_matching() {
        let constraint = VersionConstraint::parse("1.2.3").unwrap();
        assert!(constraint.matches(&semver::Version::new(1, 2, 3)));
        assert!(!constraint.matches(&semver::Version::new(1, 2, 4)));
        assert!(!constraint.matches(&semver::Version::new(1, 3, 3)));
    }

    #[test]
    fn test_invalid_version_constraint() {
        assert!(VersionConstraint::parse("not-a-version").is_err());
        assert!(VersionConstraint::parse("1.2.3.4.5").is_err());
    }
}

mod validation {
    use super::*;

    #[test]
    fn test_validate_complete_manifest() {
        let toml = r#"
            [package]
            name = "valid-package"
            version = "1.0.0"

            [dependencies]
            foo = "1.0"
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        assert!(Validator::validate(&manifest).is_ok());
    }

    #[test]
    fn test_package_name_length_limit() {
        let long_name = "a".repeat(65);
        assert!(Validator::validate_package_name(&long_name).is_err());

        let valid_name = "a".repeat(64);
        assert!(Validator::validate_package_name(&valid_name).is_ok());
    }

    #[test]
    fn test_package_name_special_characters() {
        assert!(Validator::validate_package_name("my-package").is_ok());
        assert!(Validator::validate_package_name("my_package").is_ok());
        assert!(Validator::validate_package_name("my.package").is_err());
        assert!(Validator::validate_package_name("my@package").is_err());
        assert!(Validator::validate_package_name("my space").is_err());
    }

    #[test]
    fn test_dependency_no_source() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"

            [dependencies]
            foo = {}
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        let result = Validator::validate(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_git_dependency_multiple_references() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"

            [dependencies]
            foo = { git = "https://github.com/example/foo", branch = "main", tag = "v1.0" }
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        let result = Validator::validate(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_feature_dependency() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"

            [features]
            test = { dependencies = ["unknown/feature"], default = false }
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        let result = Validator::validate(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_workspace_empty_members() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"

            [workspace]
            members = []
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        let result = Validator::validate(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_workspace_valid() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"

            [workspace]
            members = ["packages/foo"]
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        assert!(Validator::validate(&manifest).is_ok());
    }
}

mod lockfile_tests {
    use super::*;

    #[test]
    fn test_lockfile_version() {
        let lockfile = Lockfile::new();
        assert_eq!(lockfile.version, Lockfile::VERSION);
    }

    #[test]
    fn test_add_multiple_packages() {
        let mut lockfile = Lockfile::new();

        lockfile.add_package(LockedPackage {
            name: "pkg1".to_string(),
            version: semver::Version::new(1, 0, 0),
            source: LockedSource::Registry { registry: None },
            checksum: None,
            dependencies: HashMap::new(),
        });

        lockfile.add_package(LockedPackage {
            name: "pkg2".to_string(),
            version: semver::Version::new(2, 0, 0),
            source: LockedSource::Registry { registry: None },
            checksum: None,
            dependencies: HashMap::new(),
        });

        assert_eq!(lockfile.packages.len(), 2);
        // Packages should be sorted by name
        assert_eq!(lockfile.packages[0].name, "pkg1");
        assert_eq!(lockfile.packages[1].name, "pkg2");
    }

    #[test]
    fn test_update_existing_package() {
        let mut lockfile = Lockfile::new();

        lockfile.add_package(LockedPackage {
            name: "pkg".to_string(),
            version: semver::Version::new(1, 0, 0),
            source: LockedSource::Registry { registry: None },
            checksum: None,
            dependencies: HashMap::new(),
        });

        lockfile.add_package(LockedPackage {
            name: "pkg".to_string(),
            version: semver::Version::new(2, 0, 0),
            source: LockedSource::Registry { registry: None },
            checksum: Some("new-checksum".to_string()),
            dependencies: HashMap::new(),
        });

        assert_eq!(lockfile.packages.len(), 1);
        assert_eq!(lockfile.get_package("pkg").unwrap().version.to_string(), "2.0.0");
        assert_eq!(lockfile.get_package("pkg").unwrap().checksum, Some("new-checksum".to_string()));
    }

    #[test]
    fn test_lockfile_with_dependencies() {
        let mut deps = HashMap::new();
        deps.insert("dep1".to_string(), semver::Version::new(1, 0, 0));
        deps.insert("dep2".to_string(), semver::Version::new(2, 0, 0));

        let mut lockfile = Lockfile::new();
        lockfile.add_package(LockedPackage {
            name: "pkg".to_string(),
            version: semver::Version::new(1, 0, 0),
            source: LockedSource::Registry { registry: None },
            checksum: None,
            dependencies: deps,
        });

        let pkg = lockfile.get_package("pkg").unwrap();
        assert_eq!(pkg.dependencies.len(), 2);
    }

    #[test]
    fn test_lockfile_roundtrip() {
        let mut lockfile = Lockfile::new();

        lockfile.add_package(LockedPackage {
            name: "pkg".to_string(),
            version: semver::Version::new(1, 0, 0),
            source: LockedSource::Git {
                url: "https://github.com/example/repo".to_string(),
                rev: "abc123".to_string(),
            },
            checksum: Some("checksum123".to_string()),
            dependencies: HashMap::new(),
        });

        let toml = lockfile.to_string().unwrap();
        let parsed = Lockfile::from_str(&toml).unwrap();

        assert_eq!(lockfile, parsed);
    }

    #[test]
    fn test_lockfile_verify_newer_version() {
        let mut lockfile = Lockfile::new();
        lockfile.version = Lockfile::VERSION + 1;

        assert!(lockfile.verify().is_err());
    }

    #[test]
    fn test_lockfile_verify_valid() {
        let lockfile = Lockfile::new();
        assert!(lockfile.verify().is_ok());
    }

    #[test]
    fn test_locked_source_registry_serialization() {
        let source = LockedSource::Registry {
            registry: Some("https://registry.example.com".to_string()),
        };

        let mut lockfile = Lockfile::new();
        lockfile.add_package(LockedPackage {
            name: "pkg".to_string(),
            version: semver::Version::new(1, 0, 0),
            source,
            checksum: None,
            dependencies: HashMap::new(),
        });

        let toml = lockfile.to_string().unwrap();
        assert!(toml.contains("registry = \"https://registry.example.com\""));
    }
}

mod dependency_resolution {
    use super::*;

    #[test]
    fn test_resolve_simple_dependency() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"

            [dependencies]
            foo = "1.2.3"
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        let lockfile = Resolver::resolve(&manifest).unwrap();

        assert_eq!(lockfile.packages.len(), 1);
        assert_eq!(lockfile.packages[0].name, "foo");
        assert_eq!(lockfile.packages[0].version.to_string(), "1.2.3");
    }

    #[test]
    fn test_resolve_caret_version() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"

            [dependencies]
            foo = "^1.2.3"
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        let lockfile = Resolver::resolve(&manifest).unwrap();

        assert_eq!(lockfile.packages.len(), 1);
        assert_eq!(lockfile.packages[0].version.to_string(), "1.2.3");
    }

    #[test]
    fn test_resolve_tilde_version() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"

            [dependencies]
            foo = "~1.2.3"
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        let lockfile = Resolver::resolve(&manifest).unwrap();

        assert_eq!(lockfile.packages.len(), 1);
        assert_eq!(lockfile.packages[0].version.to_string(), "1.2.3");
    }

    #[test]
    fn test_resolve_git_dependency() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"

            [dependencies]
            foo = { git = "https://github.com/example/foo", rev = "abc123" }
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        let lockfile = Resolver::resolve(&manifest).unwrap();

        // Git dependencies without version are skipped in current implementation
        assert!(lockfile.packages.is_empty());
    }

    #[test]
    fn test_lockfile_metadata() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        let lockfile = Resolver::resolve(&manifest).unwrap();

        assert!(lockfile.metadata.atlas_version.is_some());
        assert!(lockfile.metadata.generated_at.is_some());
    }
}

mod error_handling {
    use super::*;

    #[test]
    fn test_parse_invalid_toml() {
        let toml = "not valid toml { [[ ]]";
        assert!(PackageManifest::from_str(toml).is_err());
    }

    #[test]
    fn test_parse_missing_package_section() {
        let toml = r#"
            [dependencies]
            foo = "1.0"
        "#;

        assert!(PackageManifest::from_str(toml).is_err());
    }

    #[test]
    fn test_parse_missing_name() {
        let toml = r#"
            [package]
            version = "1.0.0"
        "#;

        assert!(PackageManifest::from_str(toml).is_err());
    }

    #[test]
    fn test_parse_missing_version() {
        let toml = r#"
            [package]
            name = "my-package"
        "#;

        assert!(PackageManifest::from_str(toml).is_err());
    }

    #[test]
    fn test_parse_invalid_version_format() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "not-a-version"
        "#;

        assert!(PackageManifest::from_str(toml).is_err());
    }

    #[test]
    fn test_lockfile_parse_invalid_toml() {
        let toml = "not valid { [[ ]]";
        assert!(Lockfile::from_str(toml).is_err());
    }
}

mod file_operations {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_manifest_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("atlas.toml");

        let toml = r#"
            [package]
            name = "my-package"
            version = "1.0.0"
        "#;

        fs::write(&manifest_path, toml).unwrap();

        let manifest = PackageManifest::from_file(&manifest_path).unwrap();
        assert_eq!(manifest.package.name, "my-package");
    }

    #[test]
    fn test_write_lockfile_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let lockfile_path = temp_dir.path().join("atlas.lock");

        let mut lockfile = Lockfile::new();
        lockfile.add_package(LockedPackage {
            name: "pkg".to_string(),
            version: semver::Version::new(1, 0, 0),
            source: LockedSource::Registry { registry: None },
            checksum: None,
            dependencies: HashMap::new(),
        });

        lockfile.write_to_file(&lockfile_path).unwrap();
        assert!(lockfile_path.exists());

        let loaded = Lockfile::from_file(&lockfile_path).unwrap();
        assert_eq!(loaded.packages.len(), 1);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = PackageManifest::from_file(std::path::Path::new("/nonexistent/path.toml"));
        assert!(result.is_err());
    }
}
