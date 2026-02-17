//! Build profile management
//!
//! Provides build configuration profiles (dev, release, test, custom) with
//! optimization levels, debug settings, and environment customization.

use crate::builder::OptLevel;
use crate::error::{BuildError, BuildResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Build profile
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    /// Development profile (default)
    Dev,
    /// Release profile (optimized)
    Release,
    /// Test profile
    Test,
    /// Custom profile
    Custom(String),
}

impl Profile {
    /// Parse profile from string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> BuildResult<Self> {
        match s.to_lowercase().as_str() {
            "dev" => Ok(Self::Dev),
            "release" => Ok(Self::Release),
            "test" => Ok(Self::Test),
            custom => Ok(Self::Custom(custom.to_string())),
        }
    }

    /// Get profile name
    pub fn name(&self) -> &str {
        match self {
            Self::Dev => "dev",
            Self::Release => "release",
            Self::Test => "test",
            Self::Custom(name) => name,
        }
    }

    /// Check if this is a built-in profile
    pub fn is_builtin(&self) -> bool {
        matches!(self, Self::Dev | Self::Release | Self::Test)
    }

    /// Get default configuration for this profile
    pub fn default_config(&self) -> ProfileConfig {
        match self {
            Self::Dev => ProfileConfig {
                name: "dev".to_string(),
                optimization_level: OptLevel::O0,
                debug_info: true,
                inline_threshold: 25,
                parallel: true,
                incremental: true,
                dependencies: DependencyProfile::Dev,
                env_vars: HashMap::new(),
            },
            Self::Release => ProfileConfig {
                name: "release".to_string(),
                optimization_level: OptLevel::O2,
                debug_info: false,
                inline_threshold: 200,
                parallel: true,
                incremental: false,
                dependencies: DependencyProfile::Release,
                env_vars: HashMap::new(),
            },
            Self::Test => ProfileConfig {
                name: "test".to_string(),
                optimization_level: OptLevel::O0,
                debug_info: true,
                inline_threshold: 25,
                parallel: true,
                incremental: true,
                dependencies: DependencyProfile::Dev,
                env_vars: {
                    let mut env = HashMap::new();
                    env.insert("ATLAS_TEST".to_string(), "1".to_string());
                    env
                },
            },
            Self::Custom(name) => ProfileConfig {
                name: name.clone(),
                optimization_level: OptLevel::O0,
                debug_info: true,
                inline_threshold: 25,
                parallel: true,
                incremental: true,
                dependencies: DependencyProfile::Dev,
                env_vars: HashMap::new(),
            },
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for Profile {
    fn default() -> Self {
        Self::Dev
    }
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Profile name
    pub name: String,
    /// Optimization level
    #[serde(default)]
    pub optimization_level: OptLevel,
    /// Include debug information
    #[serde(default)]
    pub debug_info: bool,
    /// Inline threshold (functions with fewer instructions may be inlined)
    #[serde(default = "default_inline_threshold")]
    pub inline_threshold: usize,
    /// Enable parallel compilation
    #[serde(default = "default_true")]
    pub parallel: bool,
    /// Enable incremental compilation
    #[serde(default = "default_true")]
    pub incremental: bool,
    /// Dependency profile to use
    #[serde(default)]
    pub dependencies: DependencyProfile,
    /// Environment variables
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
}

fn default_inline_threshold() -> usize {
    25
}

fn default_true() -> bool {
    true
}

impl ProfileConfig {
    /// Create from profile with defaults
    pub fn from_profile(profile: &Profile) -> Self {
        profile.default_config()
    }

    /// Merge with manifest profile configuration
    pub fn merge_with_manifest(&mut self, manifest: &ManifestProfileConfig) {
        if let Some(opt) = manifest.opt_level {
            self.optimization_level = opt;
        }
        if let Some(debug) = manifest.debug_info {
            self.debug_info = debug;
        }
        if let Some(threshold) = manifest.inline_threshold {
            self.inline_threshold = threshold;
        }
        if let Some(parallel) = manifest.parallel {
            self.parallel = parallel;
        }
        if let Some(incremental) = manifest.incremental {
            self.incremental = incremental;
        }
        // Merge environment variables
        for (key, value) in &manifest.env_vars {
            self.env_vars.insert(key.clone(), value.clone());
        }
    }

    /// Create from custom profile with inheritance
    pub fn from_custom(
        name: String,
        manifest: &ManifestProfileConfig,
        base_profile: Option<&Profile>,
    ) -> Self {
        let mut config = if let Some(base) = base_profile {
            base.default_config()
        } else {
            Profile::Dev.default_config()
        };

        config.name = name;
        config.merge_with_manifest(manifest);
        config
    }

    /// Get cache key suffix for this profile
    pub fn cache_key_suffix(&self) -> String {
        format!(
            "{}-{:?}-{}",
            self.name,
            self.optimization_level,
            if self.debug_info { "debug" } else { "nodebug" }
        )
    }
}

/// Manifest profile configuration (from atlas.toml)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestProfileConfig {
    /// Optimization level (0-3)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_level: Option<OptLevel>,
    /// Include debug information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_info: Option<bool>,
    /// Inline threshold
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_threshold: Option<usize>,
    /// Enable parallel compilation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel: Option<bool>,
    /// Enable incremental compilation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incremental: Option<bool>,
    /// Inherit from base profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherits: Option<String>,
    /// Environment variables
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
}

/// Dependency profile - how to build dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyProfile {
    /// Use development versions of dependencies
    Dev,
    /// Use release versions of dependencies
    Release,
}

#[allow(clippy::derivable_impls)]
impl Default for DependencyProfile {
    fn default() -> Self {
        Self::Dev
    }
}

/// Profile manager - handles profile loading and configuration
pub struct ProfileManager {
    /// Available profiles
    profiles: HashMap<String, ProfileConfig>,
}

impl ProfileManager {
    /// Create new profile manager
    pub fn new() -> Self {
        let mut profiles = HashMap::new();

        // Register built-in profiles
        profiles.insert("dev".to_string(), Profile::Dev.default_config());
        profiles.insert("release".to_string(), Profile::Release.default_config());
        profiles.insert("test".to_string(), Profile::Test.default_config());

        Self { profiles }
    }

    /// Load profiles from manifest
    pub fn load_from_manifest(
        &mut self,
        manifest_profiles: &HashMap<String, ManifestProfileConfig>,
    ) -> BuildResult<()> {
        for (name, manifest_config) in manifest_profiles {
            let profile = if name == "dev" || name == "release" || name == "test" {
                // Override built-in profile
                let mut config = Profile::from_str(name)?.default_config();
                config.merge_with_manifest(manifest_config);
                config
            } else {
                // Custom profile - check for inheritance
                let base_profile = if let Some(ref inherits) = manifest_config.inherits {
                    Some(Profile::from_str(inherits)?)
                } else {
                    None
                };

                ProfileConfig::from_custom(name.clone(), manifest_config, base_profile.as_ref())
            };

            self.profiles.insert(name.clone(), profile);
        }

        Ok(())
    }

    /// Get profile configuration
    pub fn get(&self, profile: &Profile) -> BuildResult<ProfileConfig> {
        let name = profile.name();
        self.profiles
            .get(name)
            .cloned()
            .ok_or_else(|| BuildError::ProfileNotFound(name.to_string()))
    }

    /// Check if profile exists
    pub fn has_profile(&self, name: &str) -> bool {
        self.profiles.contains_key(name)
    }

    /// List all available profiles
    pub fn list_profiles(&self) -> Vec<String> {
        let mut names: Vec<_> = self.profiles.keys().cloned().collect();
        names.sort();
        names
    }
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_from_str() {
        assert_eq!(Profile::from_str("dev").unwrap(), Profile::Dev);
        assert_eq!(Profile::from_str("release").unwrap(), Profile::Release);
        assert_eq!(Profile::from_str("test").unwrap(), Profile::Test);
        assert_eq!(
            Profile::from_str("custom").unwrap(),
            Profile::Custom("custom".to_string())
        );
    }

    #[test]
    fn test_profile_name() {
        assert_eq!(Profile::Dev.name(), "dev");
        assert_eq!(Profile::Release.name(), "release");
        assert_eq!(Profile::Test.name(), "test");
        assert_eq!(Profile::Custom("bench".to_string()).name(), "bench");
    }

    #[test]
    fn test_profile_is_builtin() {
        assert!(Profile::Dev.is_builtin());
        assert!(Profile::Release.is_builtin());
        assert!(Profile::Test.is_builtin());
        assert!(!Profile::Custom("bench".to_string()).is_builtin());
    }

    #[test]
    fn test_dev_profile_config() {
        let config = Profile::Dev.default_config();
        assert_eq!(config.name, "dev");
        assert_eq!(config.optimization_level, OptLevel::O0);
        assert!(config.debug_info);
        assert!(config.parallel);
        assert!(config.incremental);
        assert_eq!(config.dependencies, DependencyProfile::Dev);
    }

    #[test]
    fn test_release_profile_config() {
        let config = Profile::Release.default_config();
        assert_eq!(config.name, "release");
        assert_eq!(config.optimization_level, OptLevel::O2);
        assert!(!config.debug_info);
        assert!(config.parallel);
        assert!(!config.incremental); // Clean builds for release
        assert_eq!(config.dependencies, DependencyProfile::Release);
    }

    #[test]
    fn test_test_profile_config() {
        let config = Profile::Test.default_config();
        assert_eq!(config.name, "test");
        assert_eq!(config.optimization_level, OptLevel::O0);
        assert!(config.debug_info);
        assert!(config.parallel);
        assert!(config.incremental);
        assert_eq!(config.env_vars.get("ATLAS_TEST"), Some(&"1".to_string()));
    }

    #[test]
    fn test_custom_profile_default_config() {
        let config = Profile::Custom("bench".to_string()).default_config();
        assert_eq!(config.name, "bench");
        assert_eq!(config.optimization_level, OptLevel::O0);
        assert!(config.debug_info);
    }

    #[test]
    fn test_profile_manager_builtin_profiles() {
        let manager = ProfileManager::new();
        assert!(manager.has_profile("dev"));
        assert!(manager.has_profile("release"));
        assert!(manager.has_profile("test"));
        assert!(!manager.has_profile("custom"));
    }

    #[test]
    fn test_profile_manager_get() {
        let manager = ProfileManager::new();
        let config = manager.get(&Profile::Dev).unwrap();
        assert_eq!(config.name, "dev");
        assert_eq!(config.optimization_level, OptLevel::O0);
    }

    #[test]
    fn test_manifest_profile_merge() {
        let mut config = Profile::Dev.default_config();
        let manifest = ManifestProfileConfig {
            opt_level: Some(OptLevel::O2),
            debug_info: Some(false),
            inline_threshold: None,
            parallel: None,
            incremental: Some(false),
            inherits: None,
            env_vars: {
                let mut env = HashMap::new();
                env.insert("FOO".to_string(), "bar".to_string());
                env
            },
        };

        config.merge_with_manifest(&manifest);
        assert_eq!(config.optimization_level, OptLevel::O2);
        assert!(!config.debug_info);
        assert!(!config.incremental);
        assert_eq!(config.env_vars.get("FOO"), Some(&"bar".to_string()));
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
    fn test_profile_manager_load_custom() {
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
    fn test_cache_key_suffix() {
        let dev_config = Profile::Dev.default_config();
        assert_eq!(dev_config.cache_key_suffix(), "dev-O0-debug");

        let release_config = Profile::Release.default_config();
        assert_eq!(release_config.cache_key_suffix(), "release-O2-nodebug");
    }
}
