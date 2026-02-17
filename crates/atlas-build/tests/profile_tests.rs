//! Build profile tests

use atlas_build::{
    DependencyProfile, ManifestProfileConfig, OptLevel, Profile, ProfileConfig, ProfileManager,
};
use std::collections::HashMap;

#[test]
fn test_dev_profile_settings() {
    let config = Profile::Dev.default_config();
    assert_eq!(config.name, "dev");
    assert_eq!(config.optimization_level, OptLevel::O0);
    assert!(config.debug_info);
    assert!(config.parallel);
    assert!(config.incremental);
    assert_eq!(config.dependencies, DependencyProfile::Dev);
}

#[test]
fn test_release_profile_optimization() {
    let config = Profile::Release.default_config();
    assert_eq!(config.name, "release");
    assert_eq!(config.optimization_level, OptLevel::O2);
    assert!(!config.debug_info);
    assert!(!config.incremental); // Clean builds for release
    assert_eq!(config.dependencies, DependencyProfile::Release);
}

#[test]
fn test_custom_profile_from_manifest() {
    let manifest = ManifestProfileConfig {
        opt_level: Some(OptLevel::O3),
        debug_info: Some(false),
        inline_threshold: Some(500),
        parallel: None,
        incremental: None,
        inherits: Some("release".to_string()),
        env_vars: {
            let mut env = HashMap::new();
            env.insert("BENCH".to_string(), "1".to_string());
            env
        },
    };

    let config =
        ProfileConfig::from_custom("bench".to_string(), &manifest, Some(&Profile::Release));

    assert_eq!(config.name, "bench");
    assert_eq!(config.optimization_level, OptLevel::O3);
    assert!(!config.debug_info);
    assert_eq!(config.inline_threshold, 500);
    assert_eq!(config.env_vars.get("BENCH"), Some(&"1".to_string()));
}

#[test]
fn test_cli_override_profile() {
    // Simulates: atlas build --profile=release
    let profile = Profile::from_str("release").unwrap();
    assert_eq!(profile, Profile::Release);

    let config = profile.default_config();
    assert_eq!(config.optimization_level, OptLevel::O2);
}

#[test]
fn test_profile_specific_dependencies() {
    let dev_config = Profile::Dev.default_config();
    assert_eq!(dev_config.dependencies, DependencyProfile::Dev);

    let release_config = Profile::Release.default_config();
    assert_eq!(release_config.dependencies, DependencyProfile::Release);
}

#[test]
fn test_profile_manager_builtin() {
    let manager = ProfileManager::new();
    assert!(manager.has_profile("dev"));
    assert!(manager.has_profile("release"));
    assert!(manager.has_profile("test"));
}

#[test]
fn test_profile_manager_custom() {
    let mut manager = ProfileManager::new();
    let mut manifest_profiles = HashMap::new();

    let bench_config = ManifestProfileConfig {
        opt_level: Some(OptLevel::O3),
        debug_info: Some(false),
        inline_threshold: None,
        parallel: None,
        incremental: None,
        inherits: Some("release".to_string()),
        env_vars: HashMap::new(),
    };

    manifest_profiles.insert("bench".to_string(), bench_config);
    manager.load_from_manifest(&manifest_profiles).unwrap();

    assert!(manager.has_profile("bench"));
    let config = manager.get(&Profile::Custom("bench".to_string())).unwrap();
    assert_eq!(config.optimization_level, OptLevel::O3);
}

#[test]
fn test_cache_key_suffix_different_profiles() {
    let dev_config = Profile::Dev.default_config();
    let release_config = Profile::Release.default_config();

    assert_ne!(
        dev_config.cache_key_suffix(),
        release_config.cache_key_suffix()
    );
}

#[test]
fn test_profile_env_vars() {
    let test_config = Profile::Test.default_config();
    assert_eq!(
        test_config.env_vars.get("ATLAS_TEST"),
        Some(&"1".to_string())
    );

    let dev_config = Profile::Dev.default_config();
    assert!(dev_config.env_vars.is_empty());
}

#[test]
fn test_custom_profile_inheritance() {
    let manifest = ManifestProfileConfig {
        opt_level: Some(OptLevel::O3),
        debug_info: None, // Should inherit from base
        inline_threshold: None,
        parallel: None,
        incremental: None,
        inherits: Some("release".to_string()),
        env_vars: HashMap::new(),
    };

    let config =
        ProfileConfig::from_custom("bench".to_string(), &manifest, Some(&Profile::Release));

    // O3 from manifest
    assert_eq!(config.optimization_level, OptLevel::O3);
    // debug_info inherited from release (false)
    assert!(!config.debug_info);
}
