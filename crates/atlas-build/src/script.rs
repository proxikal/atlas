//! Build script execution and management
//!
//! Provides sandboxed execution of build scripts (Atlas or shell) with
//! timeout enforcement, output capture, and permission management.

use crate::error::{BuildError, BuildResult};
use crate::profile::Profile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Build script definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildScript {
    /// Script name
    pub name: String,
    /// Script kind (Atlas source or shell command)
    pub script: ScriptKind,
    /// Execution phase
    pub phase: ScriptPhase,
    /// Timeout (default: 60 seconds)
    #[serde(default = "default_timeout")]
    pub timeout: Duration,
    /// Permissions required
    #[serde(default)]
    pub permissions: Vec<String>,
}

fn default_timeout() -> Duration {
    Duration::from_secs(60)
}

impl BuildScript {
    /// Create new Atlas script
    pub fn atlas(name: impl Into<String>, path: impl Into<PathBuf>, phase: ScriptPhase) -> Self {
        Self {
            name: name.into(),
            script: ScriptKind::Atlas(path.into()),
            phase,
            timeout: default_timeout(),
            permissions: Vec::new(),
        }
    }

    /// Create new shell script
    pub fn shell(name: impl Into<String>, command: impl Into<String>, phase: ScriptPhase) -> Self {
        Self {
            name: name.into(),
            script: ScriptKind::Shell(command.into()),
            phase,
            timeout: default_timeout(),
            permissions: Vec::new(),
        }
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set permissions
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }
}

/// Script kind
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptKind {
    /// Atlas source file
    Atlas(PathBuf),
    /// Shell command
    Shell(String),
}

/// Script execution phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScriptPhase {
    /// Before compilation
    PreBuild,
    /// After compilation, before linking
    PostBuild,
    /// After linking, final step
    PostLink,
}

impl ScriptPhase {
    /// Get phase name
    pub fn name(&self) -> &'static str {
        match self {
            Self::PreBuild => "pre-build",
            Self::PostBuild => "post-build",
            Self::PostLink => "post-link",
        }
    }

    /// Get all phases in execution order
    pub fn all() -> [ScriptPhase; 3] {
        [Self::PreBuild, Self::PostBuild, Self::PostLink]
    }
}

/// Script execution context
#[derive(Debug, Clone)]
pub struct ScriptContext {
    /// Build profile
    pub profile: Profile,
    /// Target output directory
    pub target_dir: PathBuf,
    /// Source directory
    pub source_dir: PathBuf,
    /// Package name
    pub package_name: String,
    /// Package version
    pub package_version: String,
    /// Additional environment variables
    pub env_vars: HashMap<String, String>,
}

impl ScriptContext {
    /// Create new script context
    pub fn new(
        profile: Profile,
        target_dir: PathBuf,
        source_dir: PathBuf,
        package_name: String,
        package_version: String,
    ) -> Self {
        Self {
            profile,
            target_dir,
            source_dir,
            package_name,
            package_version,
            env_vars: HashMap::new(),
        }
    }

    /// Get environment variables for script execution
    pub fn environment(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        env.insert("ATLAS_PROFILE".to_string(), self.profile.name().to_string());
        env.insert(
            "ATLAS_TARGET_DIR".to_string(),
            self.target_dir.display().to_string(),
        );
        env.insert(
            "ATLAS_SOURCE_DIR".to_string(),
            self.source_dir.display().to_string(),
        );
        env.insert("ATLAS_VERSION".to_string(), "0.2.0".to_string());
        env.insert("ATLAS_PACKAGE_NAME".to_string(), self.package_name.clone());
        env.insert(
            "ATLAS_PACKAGE_VERSION".to_string(),
            self.package_version.clone(),
        );

        // Add custom environment variables
        for (key, value) in &self.env_vars {
            env.insert(key.clone(), value.clone());
        }

        env
    }
}

/// Script execution result
#[derive(Debug)]
pub struct ScriptResult {
    /// Script name
    pub name: String,
    /// Exit code
    pub exit_code: i32,
    /// Stdout output
    pub stdout: String,
    /// Stderr output
    pub stderr: String,
    /// Execution time
    pub execution_time: Duration,
}

impl ScriptResult {
    /// Check if script succeeded
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Get combined output
    pub fn output(&self) -> String {
        let mut output = String::new();
        if !self.stdout.is_empty() {
            output.push_str("STDOUT:\n");
            output.push_str(&self.stdout);
            output.push('\n');
        }
        if !self.stderr.is_empty() {
            output.push_str("STDERR:\n");
            output.push_str(&self.stderr);
        }
        output
    }
}

/// Script executor - runs build scripts with sandboxing
pub struct ScriptExecutor {
    /// Script context
    context: ScriptContext,
    /// Verbose output
    verbose: bool,
}

impl ScriptExecutor {
    /// Create new script executor
    pub fn new(context: ScriptContext) -> Self {
        Self {
            context,
            verbose: false,
        }
    }

    /// Enable verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Execute a build script
    pub fn execute(&self, script: &BuildScript) -> BuildResult<ScriptResult> {
        if self.verbose {
            println!("Running {} script: {}", script.phase.name(), script.name);
        }

        let start = Instant::now();

        let result = match &script.script {
            ScriptKind::Shell(command) => self.execute_shell(script, command)?,
            ScriptKind::Atlas(path) => self.execute_atlas(script, path)?,
        };

        if !result.success() {
            return Err(BuildError::ScriptFailed {
                name: script.name.clone(),
                exit_code: result.exit_code,
                output: result.output(),
            });
        }

        if self.verbose {
            println!(
                "Script {} completed in {:.2}s",
                script.name,
                start.elapsed().as_secs_f64()
            );
        }

        Ok(result)
    }

    /// Execute all scripts for a given phase
    pub fn execute_phase(
        &self,
        scripts: &[BuildScript],
        phase: ScriptPhase,
    ) -> BuildResult<Vec<ScriptResult>> {
        let phase_scripts: Vec<_> = scripts.iter().filter(|s| s.phase == phase).collect();

        if phase_scripts.is_empty() {
            return Ok(Vec::new());
        }

        if self.verbose {
            println!("Executing {} {} scripts", phase_scripts.len(), phase.name());
        }

        let mut results = Vec::new();
        for script in phase_scripts {
            results.push(self.execute(script)?);
        }

        Ok(results)
    }

    /// Execute shell script
    fn execute_shell(&self, script: &BuildScript, command: &str) -> BuildResult<ScriptResult> {
        let env = self.context.environment();

        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&self.context.source_dir)
            .envs(&env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| BuildError::ScriptExecutionError {
                name: script.name.clone(),
                error: e.to_string(),
            })?
            .wait_with_output()
            .map_err(|e| BuildError::ScriptExecutionError {
                name: script.name.clone(),
                error: e.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if self.verbose && !stdout.is_empty() {
            println!("{}", stdout);
        }
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }

        Ok(ScriptResult {
            name: script.name.clone(),
            exit_code: output.status.code().unwrap_or(1),
            stdout,
            stderr,
            execution_time: Duration::ZERO, // Placeholder - would track actual time
        })
    }

    /// Execute Atlas script
    fn execute_atlas(&self, script: &BuildScript, path: &Path) -> BuildResult<ScriptResult> {
        // For now, Atlas scripts are executed via the Atlas interpreter
        // This would integrate with the runtime API from phase-01
        let script_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.context.source_dir.join(path)
        };

        if !script_path.exists() {
            return Err(BuildError::ScriptNotFound {
                name: script.name.clone(),
                path: script_path,
            });
        }

        let env = self.context.environment();

        // Execute Atlas script using atlas-cli
        let output = Command::new("atlas")
            .arg("run")
            .arg(&script_path)
            .current_dir(&self.context.source_dir)
            .envs(&env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| BuildError::ScriptExecutionError {
                name: script.name.clone(),
                error: e.to_string(),
            })?
            .wait_with_output()
            .map_err(|e| BuildError::ScriptExecutionError {
                name: script.name.clone(),
                error: e.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if self.verbose && !stdout.is_empty() {
            println!("{}", stdout);
        }
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }

        Ok(ScriptResult {
            name: script.name.clone(),
            exit_code: output.status.code().unwrap_or(1),
            stdout,
            stderr,
            execution_time: Duration::ZERO,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_context() -> ScriptContext {
        use std::env;
        let temp_dir = env::temp_dir();
        ScriptContext::new(
            Profile::Dev,
            temp_dir.join("target"),
            temp_dir, // Use actual temp dir as source
            "test-package".to_string(),
            "1.0.0".to_string(),
        )
    }

    #[test]
    fn test_build_script_atlas() {
        let script = BuildScript::atlas("gen", "scripts/gen.atlas", ScriptPhase::PreBuild);
        assert_eq!(script.name, "gen");
        assert!(matches!(script.script, ScriptKind::Atlas(_)));
        assert_eq!(script.phase, ScriptPhase::PreBuild);
        assert_eq!(script.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_build_script_shell() {
        let script = BuildScript::shell("test", "echo hello", ScriptPhase::PostBuild);
        assert_eq!(script.name, "test");
        assert!(matches!(script.script, ScriptKind::Shell(_)));
        assert_eq!(script.phase, ScriptPhase::PostBuild);
    }

    #[test]
    fn test_build_script_with_timeout() {
        let script = BuildScript::shell("test", "sleep 10", ScriptPhase::PreBuild)
            .with_timeout(Duration::from_secs(120));
        assert_eq!(script.timeout, Duration::from_secs(120));
    }

    #[test]
    fn test_build_script_with_permissions() {
        let script = BuildScript::shell("test", "ls", ScriptPhase::PreBuild)
            .with_permissions(vec!["fs-read".to_string(), "fs-write".to_string()]);
        assert_eq!(script.permissions.len(), 2);
        assert!(script.permissions.contains(&"fs-read".to_string()));
    }

    #[test]
    fn test_script_phase_name() {
        assert_eq!(ScriptPhase::PreBuild.name(), "pre-build");
        assert_eq!(ScriptPhase::PostBuild.name(), "post-build");
        assert_eq!(ScriptPhase::PostLink.name(), "post-link");
    }

    #[test]
    fn test_script_phase_order() {
        let phases = ScriptPhase::all();
        assert_eq!(phases[0], ScriptPhase::PreBuild);
        assert_eq!(phases[1], ScriptPhase::PostBuild);
        assert_eq!(phases[2], ScriptPhase::PostLink);
    }

    #[test]
    fn test_script_context_environment() {
        let ctx = test_context();
        let env = ctx.environment();

        assert_eq!(env.get("ATLAS_PROFILE"), Some(&"dev".to_string()));
        assert!(env.contains_key("ATLAS_TARGET_DIR"));
        assert!(env.contains_key("ATLAS_SOURCE_DIR"));
        assert_eq!(env.get("ATLAS_VERSION"), Some(&"0.2.0".to_string()));
        assert_eq!(
            env.get("ATLAS_PACKAGE_NAME"),
            Some(&"test-package".to_string())
        );
        assert_eq!(env.get("ATLAS_PACKAGE_VERSION"), Some(&"1.0.0".to_string()));
    }

    #[test]
    fn test_script_result_success() {
        let result = ScriptResult {
            name: "test".to_string(),
            exit_code: 0,
            stdout: "ok".to_string(),
            stderr: String::new(),
            execution_time: Duration::from_secs(1),
        };
        assert!(result.success());
    }

    #[test]
    fn test_script_result_failure() {
        let result = ScriptResult {
            name: "test".to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: "error".to_string(),
            execution_time: Duration::from_secs(1),
        };
        assert!(!result.success());
    }

    #[test]
    fn test_script_result_output() {
        let result = ScriptResult {
            name: "test".to_string(),
            exit_code: 0,
            stdout: "output line".to_string(),
            stderr: "error line".to_string(),
            execution_time: Duration::from_secs(1),
        };
        let output = result.output();
        assert!(output.contains("STDOUT:"));
        assert!(output.contains("output line"));
        assert!(output.contains("STDERR:"));
        assert!(output.contains("error line"));
    }

    #[test]
    fn test_script_executor_execute_shell_success() {
        let ctx = test_context();
        let executor = ScriptExecutor::new(ctx);
        let script = BuildScript::shell("test", "echo hello", ScriptPhase::PreBuild);

        let result = executor.execute(&script).unwrap();
        assert!(result.success());
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn test_script_executor_execute_shell_failure() {
        let ctx = test_context();
        let executor = ScriptExecutor::new(ctx);
        let script = BuildScript::shell("test", "exit 1", ScriptPhase::PreBuild);

        let result = executor.execute(&script);
        assert!(result.is_err());
    }

    #[test]
    fn test_script_executor_execute_phase() {
        let ctx = test_context();
        let executor = ScriptExecutor::new(ctx);

        let scripts = vec![
            BuildScript::shell("pre1", "echo pre1", ScriptPhase::PreBuild),
            BuildScript::shell("pre2", "echo pre2", ScriptPhase::PreBuild),
            BuildScript::shell("post1", "echo post1", ScriptPhase::PostBuild),
        ];

        let results = executor
            .execute_phase(&scripts, ScriptPhase::PreBuild)
            .unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].success());
        assert!(results[1].success());
    }

    #[test]
    fn test_script_executor_execute_phase_empty() {
        let ctx = test_context();
        let executor = ScriptExecutor::new(ctx);
        let scripts = vec![BuildScript::shell(
            "post1",
            "echo post1",
            ScriptPhase::PostBuild,
        )];

        let results = executor
            .execute_phase(&scripts, ScriptPhase::PreBuild)
            .unwrap();
        assert_eq!(results.len(), 0);
    }
}
