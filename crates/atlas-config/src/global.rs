//! Global Configuration (~/.atlas/config.toml)
//!
//! Handles user-level configuration stored in `~/.atlas/config.toml`.

use crate::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Global user configuration from ~/.atlas/config.toml
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct GlobalConfig {
    /// Default settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defaults: Option<DefaultsConfig>,

    /// Formatting preferences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatting: Option<GlobalFormattingConfig>,

    /// Permission defaults
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<PermissionsConfig>,

    /// LSP settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lsp: Option<LspConfig>,
}

/// Default settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DefaultsConfig {
    /// Default edition for new projects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,

    /// Default author for new projects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Default license for new projects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

/// Global formatting preferences
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GlobalFormattingConfig {
    /// Indentation size (default: 4)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent: Option<usize>,

    /// Maximum line length (default: 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_line_length: Option<usize>,

    /// Use tabs instead of spaces
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_tabs: Option<bool>,
}

/// Permission defaults
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PermissionsConfig {
    /// Network access ("allow", "deny", "prompt")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,

    /// Filesystem access ("allow", "deny", "prompt")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesystem: Option<String>,

    /// Environment variables ("allow", "deny", "prompt")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
}

/// LSP server settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LspConfig {
    /// Enable diagnostics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<bool>,

    /// Enable code completion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<bool>,

    /// Enable hover information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover: Option<bool>,
}

impl GlobalConfig {
    /// Load global configuration from a file
    pub fn load_from_file(path: &Path) -> ConfigResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ConfigError::NotFound(path.to_path_buf())
            } else {
                ConfigError::IoError(e)
            }
        })?;

        let config: Self = toml::from_str(&content).map_err(|e| ConfigError::TomlParseError {
            file: path.to_path_buf(),
            error: e,
        })?;

        config.validate()?;
        Ok(config)
    }

    /// Validate the global configuration
    pub fn validate(&self) -> ConfigResult<()> {
        // Validate permission values
        if let Some(perms) = &self.permissions {
            if let Some(network) = &perms.network {
                validate_permission_value("permissions.network", network)?;
            }
            if let Some(filesystem) = &perms.filesystem {
                validate_permission_value("permissions.filesystem", filesystem)?;
            }
            if let Some(env) = &perms.env {
                validate_permission_value("permissions.env", env)?;
            }
        }

        // Validate edition if present
        if let Some(defaults) = &self.defaults {
            if let Some(edition) = &defaults.edition {
                if !is_valid_edition(edition) {
                    return Err(ConfigError::InvalidValue {
                        field: "defaults.edition".to_string(),
                        reason: format!("invalid edition '{}'", edition),
                    });
                }
            }
        }

        Ok(())
    }

    /// Get the global config file path (~/.atlas/config.toml)
    pub fn global_config_path() -> ConfigResult<PathBuf> {
        let home = dirs::home_dir().ok_or(ConfigError::HomeNotFound)?;
        Ok(home.join(".atlas").join("config.toml"))
    }

    /// Get the default edition
    pub fn default_edition(&self) -> Option<&str> {
        self.defaults.as_ref().and_then(|d| d.edition.as_deref())
    }

    /// Merge another global config into this one
    /// Other config takes precedence for non-None values
    pub fn merge(&mut self, other: &GlobalConfig) {
        if other.defaults.is_some() {
            self.defaults = other.defaults.clone();
        }
        if other.formatting.is_some() {
            self.formatting = other.formatting.clone();
        }
        if other.permissions.is_some() {
            self.permissions = other.permissions.clone();
        }
        if other.lsp.is_some() {
            self.lsp = other.lsp.clone();
        }
    }
}

/// Validate permission value
fn validate_permission_value(field: &str, value: &str) -> ConfigResult<()> {
    if !matches!(value, "allow" | "deny" | "prompt") {
        return Err(ConfigError::InvalidValue {
            field: field.to_string(),
            reason: format!("must be 'allow', 'deny', or 'prompt', got '{}'", value),
        });
    }
    Ok(())
}

/// Check if edition is valid
fn is_valid_edition(edition: &str) -> bool {
    matches!(edition, "2026" | "2027" | "2028")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_global_config() {
        let toml = r#"
[defaults]
edition = "2026"
"#;

        let config: GlobalConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.default_edition(), Some("2026"));
    }

    #[test]
    fn test_parse_full_global_config() {
        let toml = r#"
[defaults]
edition = "2026"
author = "Alice <alice@example.com>"
license = "MIT"

[formatting]
indent = 4
max_line_length = 100
use_tabs = false

[permissions]
network = "deny"
filesystem = "prompt"
env = "allow"

[lsp]
diagnostics = true
completion = true
hover = true
"#;

        let config: GlobalConfig = toml::from_str(toml).unwrap();
        assert!(config.validate().is_ok());
        assert_eq!(config.default_edition(), Some("2026"));
        assert!(config.permissions.is_some());
    }

    #[test]
    fn test_invalid_permission_value() {
        let config = GlobalConfig {
            permissions: Some(PermissionsConfig {
                network: Some("invalid".to_string()),
                filesystem: None,
                env: None,
            }),
            ..Default::default()
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_merge_configs() {
        let mut base = GlobalConfig::default();
        let override_config = GlobalConfig {
            defaults: Some(DefaultsConfig {
                edition: Some("2027".to_string()),
                author: None,
                license: None,
            }),
            ..Default::default()
        };

        base.merge(&override_config);
        assert_eq!(base.default_edition(), Some("2027"));
    }
}
