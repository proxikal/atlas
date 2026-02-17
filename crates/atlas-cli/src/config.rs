//! CLI configuration via environment variables
//!
//! Atlas uses environment variables for optional configuration.
//! This keeps the CLI simple while allowing customization.

use std::env;
use std::path::PathBuf;

/// CLI configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    /// Default to JSON diagnostic output (ATLAS_DIAGNOSTICS=json)
    pub default_json: bool,
    /// Disable colored output (ATLAS_NO_COLOR=1 or NO_COLOR=1)
    #[allow(dead_code)]
    pub no_color: bool,
    /// Custom history file path (ATLAS_HISTORY_FILE=/path/to/file)
    pub history_file: Option<PathBuf>,
    /// Disable history by default (ATLAS_NO_HISTORY=1)
    pub no_history: bool,
    /// Enable automatic type display in REPL (ATLAS_REPL_SHOW_TYPES, defaults to true)
    pub show_types: bool,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            default_json: env::var("ATLAS_DIAGNOSTICS")
                .map(|v| v.to_lowercase() == "json")
                .unwrap_or(false),
            no_color: env::var("ATLAS_NO_COLOR").is_ok() || env::var("NO_COLOR").is_ok(),
            history_file: env::var("ATLAS_HISTORY_FILE").ok().map(PathBuf::from),
            no_history: env::var("ATLAS_NO_HISTORY").is_ok(),
            show_types: env::var("ATLAS_REPL_SHOW_TYPES")
                .map(|v| {
                    let lower = v.to_lowercase();
                    !(lower == "0" || lower == "false" || lower == "off")
                })
                .unwrap_or(true),
        }
    }

    /// Get the history file path
    ///
    /// Returns:
    /// 1. ATLAS_HISTORY_FILE if set
    /// 2. ~/.atlas/history if home directory exists
    /// 3. None otherwise
    pub fn get_history_path(&self) -> Option<PathBuf> {
        if let Some(ref path) = self.history_file {
            return Some(path.clone());
        }
        dirs::home_dir().map(|home| home.join(".atlas").join("history"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_defaults() {
        // Clear environment variables for this test
        env::remove_var("ATLAS_DIAGNOSTICS");
        env::remove_var("ATLAS_NO_COLOR");
        env::remove_var("NO_COLOR");
        env::remove_var("ATLAS_HISTORY_FILE");
        env::remove_var("ATLAS_NO_HISTORY");
        env::remove_var("ATLAS_REPL_SHOW_TYPES");

        let config = Config::from_env();
        assert!(!config.default_json);
        assert!(!config.no_color);
        assert!(config.history_file.is_none());
        assert!(!config.no_history);
        assert!(config.show_types);
    }

    #[test]
    fn test_config_json_diagnostics() {
        env::set_var("ATLAS_DIAGNOSTICS", "json");
        let config = Config::from_env();
        assert!(config.default_json);
        env::remove_var("ATLAS_DIAGNOSTICS");
    }

    #[test]
    fn test_config_no_color() {
        env::set_var("ATLAS_NO_COLOR", "1");
        let config = Config::from_env();
        assert!(config.no_color);
        env::remove_var("ATLAS_NO_COLOR");

        // Also test NO_COLOR (standard)
        env::set_var("NO_COLOR", "1");
        let config = Config::from_env();
        assert!(config.no_color);
        env::remove_var("NO_COLOR");
    }

    #[test]
    fn test_config_custom_history() {
        env::set_var("ATLAS_HISTORY_FILE", "/tmp/custom_history");
        let config = Config::from_env();
        assert_eq!(
            config.history_file,
            Some(PathBuf::from("/tmp/custom_history"))
        );
        env::remove_var("ATLAS_HISTORY_FILE");
    }

    #[test]
    fn test_config_no_history() {
        env::set_var("ATLAS_NO_HISTORY", "1");
        let config = Config::from_env();
        assert!(config.no_history);
        env::remove_var("ATLAS_NO_HISTORY");
    }

    #[test]
    fn test_repl_show_types_flag() {
        env::set_var("ATLAS_REPL_SHOW_TYPES", "0");
        let config = Config::from_env();
        assert!(!config.show_types);

        env::set_var("ATLAS_REPL_SHOW_TYPES", "false");
        let config = Config::from_env();
        assert!(!config.show_types);
        env::remove_var("ATLAS_REPL_SHOW_TYPES");
    }

    #[test]
    fn test_get_history_path_custom() {
        env::set_var("ATLAS_HISTORY_FILE", "/tmp/custom");
        let config = Config::from_env();
        assert_eq!(
            config.get_history_path(),
            Some(PathBuf::from("/tmp/custom"))
        );
        env::remove_var("ATLAS_HISTORY_FILE");
    }

    #[test]
    fn test_get_history_path_default() {
        env::remove_var("ATLAS_HISTORY_FILE");
        let config = Config::from_env();
        let path = config.get_history_path();
        // Should be Some(~/.atlas/history) if home directory exists
        if let Some(home) = dirs::home_dir() {
            assert_eq!(path, Some(home.join(".atlas").join("history")));
        }
    }
}
