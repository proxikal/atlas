//! Configuration Loader
//!
//! Handles loading and merging configuration from multiple sources with proper precedence.

use crate::global::GlobalConfig;
use crate::project::ProjectConfig;
use crate::{ConfigError, ConfigResult};
use std::env;
use std::path::{Path, PathBuf};

/// Configuration loader
///
/// Loads configuration from multiple sources and merges them with proper precedence:
/// 1. Global config (~/.atlas/config.toml) - lowest priority
/// 2. Project config (./atlas.toml) - overrides global
/// 3. Environment variables (ATLAS_*) - overrides project
/// 4. CLI flags - highest priority (handled by caller)
pub struct ConfigLoader {
    /// Cached global config path
    global_config_path: Option<PathBuf>,
}

/// Merged configuration result
#[derive(Debug, Clone)]
pub struct Config {
    /// Project configuration
    pub project: ProjectConfig,

    /// Global configuration
    pub global: GlobalConfig,

    /// Project root directory (where atlas.toml was found)
    pub project_root: Option<PathBuf>,
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Self {
        Self {
            global_config_path: None,
        }
    }

    /// Load configuration starting from the given directory
    ///
    /// Walks up the directory tree to find atlas.toml, then loads and merges
    /// global config if it exists.
    pub fn load_from_directory(&mut self, start_dir: &Path) -> ConfigResult<Config> {
        // Find project root (directory containing atlas.toml)
        let (project_root, project_config) = self.find_project_config(start_dir)?;

        // Load global config (optional)
        let global_config = self.load_global_config().unwrap_or_default();

        // Apply environment variable overrides
        let project_config = self.apply_env_overrides(project_config)?;

        Ok(Config {
            project: project_config,
            global: global_config,
            project_root,
        })
    }

    /// Load configuration from a specific project config file
    pub fn load_from_file(&mut self, config_path: &Path) -> ConfigResult<Config> {
        let project_config = ProjectConfig::load_from_file(config_path)?;
        let global_config = self.load_global_config().unwrap_or_default();

        let project_root = config_path.parent().map(|p| p.to_path_buf());

        Ok(Config {
            project: project_config,
            global: global_config,
            project_root,
        })
    }

    /// Find project configuration by walking up directory tree
    ///
    /// Returns (project_root, project_config) or error if not found
    fn find_project_config(
        &self,
        start_dir: &Path,
    ) -> ConfigResult<(Option<PathBuf>, ProjectConfig)> {
        let mut current = start_dir.to_path_buf();

        loop {
            let config_path = current.join("atlas.toml");

            if config_path.exists() {
                let project_config = ProjectConfig::load_from_file(&config_path)?;
                return Ok((Some(current), project_config));
            }

            // Try parent directory
            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => {
                    // Reached filesystem root without finding atlas.toml
                    // Return default config with no project root
                    return Ok((None, ProjectConfig::default()));
                }
            }
        }
    }

    /// Load global configuration from ~/.atlas/config.toml
    fn load_global_config(&mut self) -> ConfigResult<GlobalConfig> {
        // Get or cache global config path
        if self.global_config_path.is_none() {
            self.global_config_path = Some(GlobalConfig::global_config_path()?);
        }

        let path = self.global_config_path.as_ref().unwrap();

        // Global config is optional - if it doesn't exist, return default
        if !path.exists() {
            return Ok(GlobalConfig::default());
        }

        GlobalConfig::load_from_file(path)
    }

    /// Apply environment variable overrides to project config
    ///
    /// Environment variables follow the pattern: ATLAS_<SECTION>_<KEY>
    /// Example: ATLAS_COMPILER_OPTIMIZE=true
    fn apply_env_overrides(&self, mut config: ProjectConfig) -> ConfigResult<ProjectConfig> {
        // Check for ATLAS_EDITION
        if let Ok(edition) = env::var("ATLAS_EDITION") {
            if let Some(pkg) = config.package.as_mut() {
                pkg.edition = Some(edition);
            }
        }

        // Check for ATLAS_OPTIMIZE
        if let Ok(optimize) = env::var("ATLAS_OPTIMIZE") {
            let optimize_bool = matches!(optimize.to_lowercase().as_str(), "true" | "1" | "yes");
            if config.compiler.is_none() {
                config.compiler = Some(Default::default());
            }
            if let Some(compiler) = config.compiler.as_mut() {
                compiler.optimize = Some(optimize_bool);
            }
        }

        // Check for ATLAS_DEBUG
        if let Ok(debug) = env::var("ATLAS_DEBUG") {
            let debug_bool = matches!(debug.to_lowercase().as_str(), "true" | "1" | "yes");
            if config.compiler.is_none() {
                config.compiler = Some(Default::default());
            }
            if let Some(compiler) = config.compiler.as_mut() {
                compiler.debug = Some(debug_bool);
            }
        }

        Ok(config)
    }

    /// Get the global configuration directory (~/.atlas)
    pub fn global_config_dir() -> ConfigResult<PathBuf> {
        let home = dirs::home_dir().ok_or(ConfigError::HomeNotFound)?;
        Ok(home.join(".atlas"))
    }

    /// Ensure global configuration directory exists
    pub fn ensure_global_config_dir() -> ConfigResult<PathBuf> {
        let dir = Self::global_config_dir()?;
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Get the effective edition (project > global > default)
    pub fn edition(&self) -> &str {
        self.project
            .edition()
            .or_else(|| self.global.default_edition())
            .unwrap_or("2026")
    }

    /// Get the project root directory
    pub fn project_root(&self) -> Option<&Path> {
        self.project_root.as_deref()
    }

    /// Get the package name
    pub fn package_name(&self) -> Option<&str> {
        self.project.package_name()
    }

    /// Check if this is a project (has atlas.toml)
    pub fn is_project(&self) -> bool {
        self.project_root.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    fn create_config_file(dir: &Path, content: &str) -> PathBuf {
        let config_path = dir.join("atlas.toml");
        fs::write(&config_path, content).unwrap();
        config_path
    }

    #[test]
    fn test_load_project_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_content = r#"
[package]
name = "test-project"
version = "1.0.0"
"#;
        create_config_file(temp_dir.path(), config_content);

        let mut loader = ConfigLoader::new();
        let config = loader.load_from_directory(temp_dir.path()).unwrap();

        assert_eq!(config.package_name(), Some("test-project"));
        assert!(config.is_project());
    }

    #[test]
    fn test_find_config_in_parent() {
        let temp_dir = TempDir::new().unwrap();
        let config_content = r#"
[package]
name = "parent-project"
version = "1.0.0"
"#;
        create_config_file(temp_dir.path(), config_content);

        // Create subdirectory
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        let mut loader = ConfigLoader::new();
        let config = loader.load_from_directory(&sub_dir).unwrap();

        assert_eq!(config.package_name(), Some("parent-project"));
        assert_eq!(config.project_root(), Some(temp_dir.path()));
    }

    #[test]
    fn test_no_project_config() {
        let temp_dir = TempDir::new().unwrap();

        let mut loader = ConfigLoader::new();
        let config = loader.load_from_directory(temp_dir.path()).unwrap();

        assert_eq!(config.package_name(), None);
        assert!(!config.is_project());
    }

    #[test]
    #[serial]
    fn test_env_override_edition() {
        let temp_dir = TempDir::new().unwrap();
        let config_content = r#"
[package]
name = "test"
version = "1.0.0"
edition = "2026"
"#;
        create_config_file(temp_dir.path(), config_content);

        env::set_var("ATLAS_EDITION", "2027");

        let mut loader = ConfigLoader::new();
        let config = loader.load_from_directory(temp_dir.path()).unwrap();

        assert_eq!(config.edition(), "2027");

        env::remove_var("ATLAS_EDITION");
    }

    #[test]
    #[serial]
    fn test_env_override_optimize() {
        let temp_dir = TempDir::new().unwrap();
        let config_content = r#"
[package]
name = "test"
version = "1.0.0"

[compiler]
optimize = false
"#;
        create_config_file(temp_dir.path(), config_content);

        env::set_var("ATLAS_OPTIMIZE", "true");

        let mut loader = ConfigLoader::new();
        let config = loader.load_from_directory(temp_dir.path()).unwrap();

        assert_eq!(
            config.project.compiler.as_ref().unwrap().optimize,
            Some(true)
        );

        env::remove_var("ATLAS_OPTIMIZE");
    }

    #[test]
    fn test_default_edition() {
        let config = Config {
            project: ProjectConfig::default(),
            global: GlobalConfig::default(),
            project_root: None,
        };

        assert_eq!(config.edition(), "2026"); // Default edition
    }

    #[test]
    fn test_load_from_specific_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_content = r#"
[package]
name = "specific-file"
version = "2.0.0"
"#;
        let config_path = create_config_file(temp_dir.path(), config_content);

        let mut loader = ConfigLoader::new();
        let config = loader.load_from_file(&config_path).unwrap();

        assert_eq!(config.package_name(), Some("specific-file"));
    }
}
