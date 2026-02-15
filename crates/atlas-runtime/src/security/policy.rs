//! Security policy definition and enforcement

use crate::security::permissions::{Permission, PermissionSet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Security policy errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum PolicyError {
    #[error("Policy parse error: {0}")]
    ParseError(String),

    #[error("Policy validation error: {0}")]
    ValidationError(String),

    #[error("Invalid policy field: {field} - {reason}")]
    InvalidField { field: String, reason: String },

    #[error("Policy not found: {0}")]
    NotFound(String),
}

/// Security policy loaded from configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Policy name/identifier
    pub name: String,

    /// Policy description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Default action (allow or deny)
    #[serde(default = "default_deny")]
    pub default_action: PolicyAction,

    /// Allow rules
    #[serde(default)]
    pub allow: Vec<PolicyRule>,

    /// Deny rules (higher priority than allow)
    #[serde(default)]
    pub deny: Vec<PolicyRule>,

    /// Inherited policies
    #[serde(default)]
    pub inherits: Vec<String>,

    /// Time-based permission grants
    #[serde(default)]
    pub time_based: Vec<TimeBasedGrant>,
}

fn default_deny() -> PolicyAction {
    PolicyAction::Deny
}

/// Policy action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyAction {
    Allow,
    Deny,
}

/// Policy rule for matching operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyRule {
    /// Resource type
    pub resource: ResourceType,

    /// Resource pattern (path, domain, etc.)
    pub pattern: String,

    /// Optional scope restrictions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// Rule description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Resource types for policy rules
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    #[serde(rename = "file-read")]
    FileRead,
    #[serde(rename = "file-write")]
    FileWrite,
    #[serde(rename = "file-delete")]
    FileDelete,
    #[serde(rename = "network-connect")]
    NetworkConnect,
    #[serde(rename = "network-listen")]
    NetworkListen,
    #[serde(rename = "ffi")]
    FFI,
    #[serde(rename = "process")]
    Process,
    #[serde(rename = "environment")]
    Environment,
    #[serde(rename = "reflection")]
    Reflection,
}

/// Time-based permission grant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBasedGrant {
    /// Permission to grant
    pub permission: String,

    /// Start time (Unix timestamp)
    pub start: Option<u64>,

    /// End time (Unix timestamp)
    pub end: Option<u64>,

    /// Days of week (0 = Sunday, 6 = Saturday)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub days_of_week: Option<Vec<u8>>,

    /// Hours of day (0-23)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours_of_day: Option<Vec<u8>>,
}

impl SecurityPolicy {
    /// Create new empty policy with default-deny
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            default_action: PolicyAction::Deny,
            allow: Vec::new(),
            deny: Vec::new(),
            inherits: Vec::new(),
            time_based: Vec::new(),
        }
    }

    /// Load policy from TOML string
    pub fn from_toml(content: &str) -> Result<Self, PolicyError> {
        toml::from_str(content).map_err(|e| PolicyError::ParseError(e.to_string()))
    }

    /// Load policy from JSON string
    pub fn from_json(content: &str) -> Result<Self, PolicyError> {
        serde_json::from_str(content).map_err(|e| PolicyError::ParseError(e.to_string()))
    }

    /// Validate policy
    pub fn validate(&self) -> Result<(), PolicyError> {
        // Check name is not empty
        if self.name.is_empty() {
            return Err(PolicyError::ValidationError(
                "Policy name cannot be empty".to_string(),
            ));
        }

        // Validate allow rules
        for rule in &self.allow {
            self.validate_rule(rule)?;
        }

        // Validate deny rules
        for rule in &self.deny {
            self.validate_rule(rule)?;
        }

        // Validate time-based grants
        for grant in &self.time_based {
            self.validate_time_based_grant(grant)?;
        }

        Ok(())
    }

    fn validate_rule(&self, rule: &PolicyRule) -> Result<(), PolicyError> {
        // Check pattern is not empty
        if rule.pattern.is_empty() {
            return Err(PolicyError::InvalidField {
                field: "pattern".to_string(),
                reason: "Pattern cannot be empty".to_string(),
            });
        }

        // Validate resource-specific patterns
        match rule.resource {
            ResourceType::FileRead | ResourceType::FileWrite | ResourceType::FileDelete => {
                // File paths should be absolute or contain wildcards
                if !rule.pattern.starts_with('/') && !rule.pattern.contains('*') {
                    return Err(PolicyError::InvalidField {
                        field: "pattern".to_string(),
                        reason: "File path must be absolute or contain wildcards".to_string(),
                    });
                }
            }
            ResourceType::NetworkConnect | ResourceType::NetworkListen => {
                // Network patterns should be valid domain or IP
                // Basic validation - in production would use more sophisticated checks
                if rule.pattern.contains("..") {
                    return Err(PolicyError::InvalidField {
                        field: "pattern".to_string(),
                        reason: "Invalid network pattern".to_string(),
                    });
                }
            }
            _ => {
                // Other resource types - basic validation
            }
        }

        Ok(())
    }

    fn validate_time_based_grant(&self, grant: &TimeBasedGrant) -> Result<(), PolicyError> {
        // Validate days of week
        if let Some(ref days) = grant.days_of_week {
            for day in days {
                if *day > 6 {
                    return Err(PolicyError::InvalidField {
                        field: "days_of_week".to_string(),
                        reason: format!("Invalid day: {} (must be 0-6)", day),
                    });
                }
            }
        }

        // Validate hours of day
        if let Some(ref hours) = grant.hours_of_day {
            for hour in hours {
                if *hour > 23 {
                    return Err(PolicyError::InvalidField {
                        field: "hours_of_day".to_string(),
                        reason: format!("Invalid hour: {} (must be 0-23)", hour),
                    });
                }
            }
        }

        Ok(())
    }

    /// Convert policy to permission set
    pub fn to_permission_set(&self) -> PermissionSet {
        let mut perms = PermissionSet::new();

        // Process allow rules
        for rule in &self.allow {
            if let Some(perm) = self.rule_to_permission(rule) {
                perms.grant(perm);
            }
        }

        perms
    }

    fn rule_to_permission(&self, rule: &PolicyRule) -> Option<Permission> {
        match rule.resource {
            ResourceType::FileRead => Some(Permission::FilesystemRead {
                path: PathBuf::from(&rule.pattern),
                recursive: rule.scope.as_deref() == Some("recursive"),
            }),
            ResourceType::FileWrite => Some(Permission::FilesystemWrite {
                path: PathBuf::from(&rule.pattern),
                recursive: rule.scope.as_deref() == Some("recursive"),
            }),
            ResourceType::NetworkConnect => Some(Permission::Network {
                host: rule.pattern.clone(),
            }),
            ResourceType::Process => Some(Permission::Process {
                command: rule.pattern.clone(),
            }),
            ResourceType::Environment => Some(Permission::Environment {
                var: rule.pattern.clone(),
            }),
            // TODO: Add support for other resource types when they're added to Permission enum
            _ => None,
        }
    }

    /// Check if policy allows operation
    pub fn allows(&self, resource: &ResourceType, pattern: &str) -> bool {
        // Check deny rules first (higher priority)
        for rule in &self.deny {
            if rule.resource == *resource && self.pattern_matches(&rule.pattern, pattern) {
                return false;
            }
        }

        // Check allow rules
        for rule in &self.allow {
            if rule.resource == *resource && self.pattern_matches(&rule.pattern, pattern) {
                return true;
            }
        }

        // Apply default action
        self.default_action == PolicyAction::Allow
    }

    fn pattern_matches(&self, pattern: &str, target: &str) -> bool {
        // Simple wildcard matching
        if pattern == "*" {
            return true;
        }

        if pattern.contains('*') {
            // Simplified wildcard matching
            if let Some(suffix) = pattern.strip_prefix('*') {
                return target.ends_with(suffix);
            } else if let Some(prefix) = pattern.strip_suffix('*') {
                return target.starts_with(prefix);
            }
        }

        // Exact match
        pattern == target
    }
}

/// Policy manager for loading and managing multiple policies
#[derive(Debug, Default)]
pub struct PolicyManager {
    policies: HashMap<String, SecurityPolicy>,
}

impl PolicyManager {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }

    /// Load policy
    pub fn load_policy(&mut self, policy: SecurityPolicy) -> Result<(), PolicyError> {
        policy.validate()?;
        self.policies.insert(policy.name.clone(), policy);
        Ok(())
    }

    /// Get policy by name
    pub fn get_policy(&self, name: &str) -> Option<&SecurityPolicy> {
        self.policies.get(name)
    }

    /// Get permission set from policy with inheritance
    pub fn get_permissions(&self, policy_name: &str) -> Result<PermissionSet, PolicyError> {
        let policy = self
            .policies
            .get(policy_name)
            .ok_or_else(|| PolicyError::NotFound(policy_name.to_string()))?;

        let mut perms = PermissionSet::new();

        // Process inherited policies first
        for inherited in &policy.inherits {
            let inherited_perms = self.get_permissions(inherited)?;
            perms.merge(&inherited_perms);
        }

        // Merge current policy permissions
        perms.merge(&policy.to_permission_set());

        Ok(perms)
    }

    /// Load policies from directory
    pub fn load_from_directory(&mut self, _path: &std::path::Path) -> Result<(), PolicyError> {
        // TODO: Implement directory scanning for policy files
        // For now, return Ok - this will be implemented in a future phase
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_policy_from_toml() {
        let toml = r#"
            name = "test-policy"
            description = "Test policy"
            default_action = "deny"

            [[allow]]
            resource = "file-read"
            pattern = "/data/*"
            scope = "recursive"

            [[deny]]
            resource = "file-write"
            pattern = "/etc/*"
        "#;

        let policy = SecurityPolicy::from_toml(toml).unwrap();
        assert_eq!(policy.name, "test-policy");
        assert_eq!(policy.allow.len(), 1);
        assert_eq!(policy.deny.len(), 1);
    }

    #[test]
    fn test_policy_validation() {
        let mut policy = SecurityPolicy::new("test".to_string());

        // Valid policy
        assert!(policy.validate().is_ok());

        // Invalid policy - empty pattern
        policy.allow.push(PolicyRule {
            resource: ResourceType::FileRead,
            pattern: "".to_string(),
            scope: None,
            description: None,
        });

        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_default_deny_policy() {
        let policy = SecurityPolicy::new("test".to_string());

        assert!(!policy.allows(&ResourceType::FileRead, "/data/file.txt"));
        assert!(!policy.allows(&ResourceType::NetworkConnect, "example.com"));
    }

    #[test]
    fn test_whitelist_specific_paths() {
        let mut policy = SecurityPolicy::new("test".to_string());
        policy.allow.push(PolicyRule {
            resource: ResourceType::FileRead,
            pattern: "/data/*".to_string(),
            scope: None,
            description: None,
        });

        assert!(policy.allows(&ResourceType::FileRead, "/data/file.txt"));
        assert!(!policy.allows(&ResourceType::FileRead, "/etc/passwd"));
    }

    #[test]
    fn test_blacklist_dangerous_operations() {
        let mut policy = SecurityPolicy::new("test".to_string());
        policy.default_action = PolicyAction::Allow;
        policy.deny.push(PolicyRule {
            resource: ResourceType::FileWrite,
            pattern: "/etc/*".to_string(),
            scope: None,
            description: None,
        });

        assert!(!policy.allows(&ResourceType::FileWrite, "/etc/passwd"));
        assert!(policy.allows(&ResourceType::FileWrite, "/tmp/file.txt"));
    }

    #[test]
    fn test_policy_inheritance() {
        let mut manager = PolicyManager::new();

        // Base policy
        let mut base = SecurityPolicy::new("base".to_string());
        base.allow.push(PolicyRule {
            resource: ResourceType::FileRead,
            pattern: "/data/*".to_string(),
            scope: None,
            description: None,
        });
        manager.load_policy(base).unwrap();

        // Derived policy
        let mut derived = SecurityPolicy::new("derived".to_string());
        derived.inherits.push("base".to_string());
        derived.allow.push(PolicyRule {
            resource: ResourceType::FileWrite,
            pattern: "/tmp/*".to_string(),
            scope: None,
            description: None,
        });
        manager.load_policy(derived).unwrap();

        let perms = manager.get_permissions("derived").unwrap();
        
        // Should have both base and derived permissions
        assert!(perms.len() >= 2);
    }

    #[test]
    fn test_invalid_policy_detection() {
        let toml = r#"
            name = ""
            default_action = "deny"
        "#;

        let policy = SecurityPolicy::from_toml(toml).unwrap();
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_time_based_validation() {
        let mut policy = SecurityPolicy::new("test".to_string());
        
        // Invalid day of week
        policy.time_based.push(TimeBasedGrant {
            permission: "file-read:/data/*".to_string(),
            start: None,
            end: None,
            days_of_week: Some(vec![7]), // Invalid - must be 0-6
            hours_of_day: None,
        });

        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_policy_to_permission_set() {
        let mut policy = SecurityPolicy::new("test".to_string());
        policy.allow.push(PolicyRule {
            resource: ResourceType::FileRead,
            pattern: "/data".to_string(),
            scope: Some("recursive".to_string()),
            description: None,
        });

        let perms = policy.to_permission_set();
        assert_eq!(perms.len(), 1);
    }
}
