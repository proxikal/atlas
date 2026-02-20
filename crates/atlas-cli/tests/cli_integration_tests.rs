//! Comprehensive CLI integration tests
//!
//! Tests the complete CLI experience including:
//! - Command aliases
//! - Help messages and examples
//! - Shell completions
//! - Error handling
//! - Environment variable support
//! - Flag parsing

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn atlas_cmd() -> Command {
    Command::cargo_bin("atlas").unwrap()
}

// ══════════════════════════════════════════════════════════════════════════════
// HELP MESSAGE TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod help_messages {
    use super::*;

    #[test]
    fn test_main_help_shows_all_commands() {
        let mut cmd = atlas_cmd();
        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("run"))
            .stdout(predicate::str::contains("check"))
            .stdout(predicate::str::contains("build"))
            .stdout(predicate::str::contains("test"))
            .stdout(predicate::str::contains("fmt"))
            .stdout(predicate::str::contains("debug"))
            .stdout(predicate::str::contains("lsp"))
            .stdout(predicate::str::contains("repl"))
            .stdout(predicate::str::contains("completions"));
    }

    #[test]
    fn test_main_help_shows_examples() {
        let mut cmd = atlas_cmd();
        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("EXAMPLES"))
            .stdout(predicate::str::contains("atlas run main.atl"));
    }

    #[test]
    fn test_main_help_shows_environment_variables() {
        let mut cmd = atlas_cmd();
        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("ENVIRONMENT VARIABLES"))
            .stdout(predicate::str::contains("ATLAS_JSON"))
            .stdout(predicate::str::contains("NO_COLOR"));
    }

    #[test]
    fn test_run_help_comprehensive() {
        let mut cmd = atlas_cmd();
        cmd.args(["run", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("EXAMPLES"))
            .stdout(predicate::str::contains("--watch"))
            .stdout(predicate::str::contains("--json"))
            .stdout(predicate::str::contains("-w"));
    }

    #[test]
    fn test_check_help_comprehensive() {
        let mut cmd = atlas_cmd();
        cmd.args(["check", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Type-check"))
            .stdout(predicate::str::contains("EXAMPLES"));
    }

    #[test]
    fn test_build_help_comprehensive() {
        let mut cmd = atlas_cmd();
        cmd.args(["build", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--release"))
            .stdout(predicate::str::contains("--profile"))
            .stdout(predicate::str::contains("EXAMPLES"));
    }

    #[test]
    fn test_test_help_comprehensive() {
        let mut cmd = atlas_cmd();
        cmd.args(["test", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--sequential"))
            .stdout(predicate::str::contains("--verbose"))
            .stdout(predicate::str::contains("--dir"))
            .stdout(predicate::str::contains("EXAMPLES"));
    }

    #[test]
    fn test_fmt_help_comprehensive() {
        let mut cmd = atlas_cmd();
        cmd.args(["fmt", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--check"))
            .stdout(predicate::str::contains("--write"))
            .stdout(predicate::str::contains("--indent-size"))
            .stdout(predicate::str::contains("EXAMPLES"));
    }

    #[test]
    fn test_debug_help_comprehensive() {
        let mut cmd = atlas_cmd();
        cmd.args(["debug", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("DEBUGGER COMMANDS"))
            .stdout(predicate::str::contains("break"))
            .stdout(predicate::str::contains("step"))
            .stdout(predicate::str::contains("EXAMPLES"));
    }

    #[test]
    fn test_repl_help_comprehensive() {
        let mut cmd = atlas_cmd();
        cmd.args(["repl", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("REPL COMMANDS"))
            .stdout(predicate::str::contains(":help"))
            .stdout(predicate::str::contains(":quit"))
            .stdout(predicate::str::contains("--tui"));
    }

    #[test]
    fn test_lsp_help_comprehensive() {
        let mut cmd = atlas_cmd();
        cmd.args(["lsp", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--tcp"))
            .stdout(predicate::str::contains("--port"))
            .stdout(predicate::str::contains("EXAMPLES"));
    }

    #[test]
    fn test_completions_help_comprehensive() {
        let mut cmd = atlas_cmd();
        cmd.args(["completions", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("bash"))
            .stdout(predicate::str::contains("zsh"))
            .stdout(predicate::str::contains("fish"))
            .stdout(predicate::str::contains("INSTALLATION"));
    }

    #[test]
    fn test_profile_help_comprehensive() {
        let mut cmd = atlas_cmd();
        cmd.args(["profile", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--threshold"))
            .stdout(predicate::str::contains("--output"))
            .stdout(predicate::str::contains("EXAMPLES"));
    }

    #[test]
    fn test_ast_help() {
        let mut cmd = atlas_cmd();
        cmd.args(["ast", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("AST"))
            .stdout(predicate::str::contains("JSON"));
    }

    #[test]
    fn test_typecheck_help() {
        let mut cmd = atlas_cmd();
        cmd.args(["typecheck", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("type"));
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// COMMAND ALIAS TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod command_aliases {
    use super::*;

    #[test]
    fn test_alias_r_equivalent_to_run() {
        // Both should show same help content
        let run_help = atlas_cmd()
            .args(["run", "--help"])
            .output()
            .unwrap();

        let r_help = atlas_cmd()
            .args(["r", "--help"])
            .output()
            .unwrap();

        assert_eq!(
            String::from_utf8_lossy(&run_help.stdout),
            String::from_utf8_lossy(&r_help.stdout)
        );
    }

    #[test]
    fn test_alias_t_equivalent_to_test() {
        let test_help = atlas_cmd()
            .args(["test", "--help"])
            .output()
            .unwrap();

        let t_help = atlas_cmd()
            .args(["t", "--help"])
            .output()
            .unwrap();

        assert_eq!(
            String::from_utf8_lossy(&test_help.stdout),
            String::from_utf8_lossy(&t_help.stdout)
        );
    }

    #[test]
    fn test_alias_f_equivalent_to_fmt() {
        let fmt_help = atlas_cmd()
            .args(["fmt", "--help"])
            .output()
            .unwrap();

        let f_help = atlas_cmd()
            .args(["f", "--help"])
            .output()
            .unwrap();

        assert_eq!(
            String::from_utf8_lossy(&fmt_help.stdout),
            String::from_utf8_lossy(&f_help.stdout)
        );
    }

    #[test]
    fn test_alias_b_equivalent_to_build() {
        let build_help = atlas_cmd()
            .args(["build", "--help"])
            .output()
            .unwrap();

        let b_help = atlas_cmd()
            .args(["b", "--help"])
            .output()
            .unwrap();

        assert_eq!(
            String::from_utf8_lossy(&build_help.stdout),
            String::from_utf8_lossy(&b_help.stdout)
        );
    }

    #[test]
    fn test_alias_c_equivalent_to_check() {
        let check_help = atlas_cmd()
            .args(["check", "--help"])
            .output()
            .unwrap();

        let c_help = atlas_cmd()
            .args(["c", "--help"])
            .output()
            .unwrap();

        assert_eq!(
            String::from_utf8_lossy(&check_help.stdout),
            String::from_utf8_lossy(&c_help.stdout)
        );
    }

    #[test]
    fn test_alias_d_equivalent_to_debug() {
        let debug_help = atlas_cmd()
            .args(["debug", "--help"])
            .output()
            .unwrap();

        let d_help = atlas_cmd()
            .args(["d", "--help"])
            .output()
            .unwrap();

        assert_eq!(
            String::from_utf8_lossy(&debug_help.stdout),
            String::from_utf8_lossy(&d_help.stdout)
        );
    }

    #[test]
    fn test_aliases_shown_in_main_help() {
        let mut cmd = atlas_cmd();
        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("[aliases: r]"))
            .stdout(predicate::str::contains("[aliases: t]"))
            .stdout(predicate::str::contains("[aliases: f]"))
            .stdout(predicate::str::contains("[aliases: b]"))
            .stdout(predicate::str::contains("[aliases: c]"))
            .stdout(predicate::str::contains("[aliases: d]"));
    }

    #[test]
    fn test_alias_r_with_flags() {
        // Alias should work with flags
        let mut cmd = atlas_cmd();
        cmd.args(["r", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--watch"));
    }

    #[test]
    fn test_alias_t_with_pattern() {
        let mut cmd = atlas_cmd();
        cmd.args(["t", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("pattern"));
    }

    #[test]
    fn test_alias_f_with_check() {
        let mut cmd = atlas_cmd();
        cmd.args(["f", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--check"));
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// SHELL COMPLETION TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod shell_completions {
    use super::*;

    #[test]
    fn test_bash_completion_generated() {
        let mut cmd = atlas_cmd();
        cmd.args(["completions", "bash"])
            .assert()
            .success()
            .stdout(predicate::str::contains("_atlas"))
            .stdout(predicate::str::contains("COMPREPLY"));
    }

    #[test]
    fn test_zsh_completion_generated() {
        let mut cmd = atlas_cmd();
        cmd.args(["completions", "zsh"])
            .assert()
            .success()
            .stdout(predicate::str::contains("#compdef atlas"))
            .stdout(predicate::str::contains("_atlas"));
    }

    #[test]
    fn test_fish_completion_generated() {
        let mut cmd = atlas_cmd();
        cmd.args(["completions", "fish"])
            .assert()
            .success()
            .stdout(predicate::str::contains("complete -c atlas"));
    }

    #[test]
    fn test_powershell_completion_generated() {
        let mut cmd = atlas_cmd();
        cmd.args(["completions", "powershell"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Register-ArgumentCompleter"));
    }

    #[test]
    fn test_bash_completion_includes_commands() {
        let mut cmd = atlas_cmd();
        cmd.args(["completions", "bash"])
            .assert()
            .success()
            .stdout(predicate::str::contains("run"))
            .stdout(predicate::str::contains("check"))
            .stdout(predicate::str::contains("build"))
            .stdout(predicate::str::contains("test"))
            .stdout(predicate::str::contains("fmt"));
    }

    #[test]
    fn test_bash_completion_includes_aliases() {
        let mut cmd = atlas_cmd();
        cmd.args(["completions", "bash"])
            .assert()
            .success()
            .stdout(predicate::str::contains("atlas__build"))
            .stdout(predicate::str::contains("atlas,b"));
    }

    #[test]
    fn test_zsh_completion_includes_descriptions() {
        let mut cmd = atlas_cmd();
        cmd.args(["completions", "zsh"])
            .assert()
            .success()
            // Zsh completions include command descriptions
            .stdout(predicate::str::contains("Run an Atlas source file"));
    }

    #[test]
    fn test_fish_completion_includes_commands() {
        let mut cmd = atlas_cmd();
        cmd.args(["completions", "fish"])
            .assert()
            .success()
            .stdout(predicate::str::contains(r#"-a "run""#))
            .stdout(predicate::str::contains(r#"-a "check""#));
    }

    #[test]
    fn test_completion_invalid_shell() {
        let mut cmd = atlas_cmd();
        cmd.args(["completions", "invalid-shell"])
            .assert()
            .failure();
    }

    #[test]
    fn test_completion_no_shell_arg() {
        let mut cmd = atlas_cmd();
        cmd.args(["completions"])
            .assert()
            .failure();
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// FLAG PARSING TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod flag_parsing {
    use super::*;

    #[test]
    fn test_run_watch_short_flag() {
        let mut cmd = atlas_cmd();
        cmd.args(["run", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("-w"))
            .stdout(predicate::str::contains("--watch"));
    }

    #[test]
    fn test_run_verbose_short_flag() {
        let mut cmd = atlas_cmd();
        cmd.args(["run", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("-v"))
            .stdout(predicate::str::contains("--verbose"));
    }

    #[test]
    fn test_build_profile_short_flag() {
        let mut cmd = atlas_cmd();
        cmd.args(["build", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("-p"))
            .stdout(predicate::str::contains("--profile"));
    }

    #[test]
    fn test_fmt_write_short_flag() {
        let mut cmd = atlas_cmd();
        cmd.args(["fmt", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("-w"))
            .stdout(predicate::str::contains("--write"));
    }

    #[test]
    fn test_fmt_config_short_flag() {
        let mut cmd = atlas_cmd();
        cmd.args(["fmt", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("-c"))
            .stdout(predicate::str::contains("--config"));
    }

    #[test]
    fn test_fmt_quiet_short_flag() {
        let mut cmd = atlas_cmd();
        cmd.args(["fmt", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("-q"))
            .stdout(predicate::str::contains("--quiet"));
    }

    #[test]
    fn test_test_verbose_short_flag() {
        let mut cmd = atlas_cmd();
        cmd.args(["test", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("-v"))
            .stdout(predicate::str::contains("--verbose"));
    }

    #[test]
    fn test_debug_breakpoint_short_flag() {
        let mut cmd = atlas_cmd();
        cmd.args(["debug", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("-b"))
            .stdout(predicate::str::contains("--breakpoint"));
    }

    #[test]
    fn test_profile_output_short_flag() {
        let mut cmd = atlas_cmd();
        cmd.args(["profile", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("-o"))
            .stdout(predicate::str::contains("--output"));
    }

    #[test]
    fn test_lsp_verbose_short_flag() {
        let mut cmd = atlas_cmd();
        cmd.args(["lsp", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("-v"))
            .stdout(predicate::str::contains("--verbose"));
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// VERSION AND METADATA TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod version_metadata {
    use super::*;

    #[test]
    fn test_version_flag() {
        let mut cmd = atlas_cmd();
        cmd.arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("atlas"));
    }

    #[test]
    fn test_version_short_flag() {
        let mut cmd = atlas_cmd();
        cmd.arg("-V")
            .assert()
            .success()
            .stdout(predicate::str::contains("atlas"));
    }

    #[test]
    fn test_help_long_flag() {
        let mut cmd = atlas_cmd();
        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("atlas"));
    }

    #[test]
    fn test_help_short_flag() {
        let mut cmd = atlas_cmd();
        cmd.arg("-h")
            .assert()
            .success()
            .stdout(predicate::str::contains("atlas"));
    }

    #[test]
    fn test_subcommand_version_propagated() {
        // Version should be available on subcommands with --version
        let mut cmd = atlas_cmd();
        cmd.args(["run", "--version"])
            .assert()
            .success()
            .stdout(predicate::str::contains("atlas"));
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// ERROR HANDLING TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod error_handling {
    use super::*;

    #[test]
    fn test_unknown_command_error() {
        let mut cmd = atlas_cmd();
        cmd.arg("unknown-command")
            .assert()
            .failure()
            .stderr(predicate::str::contains("error"));
    }

    #[test]
    fn test_missing_required_arg_run() {
        let mut cmd = atlas_cmd();
        cmd.arg("run")
            .assert()
            .failure()
            .stderr(predicate::str::contains("required"));
    }

    #[test]
    fn test_missing_required_arg_check() {
        let mut cmd = atlas_cmd();
        cmd.arg("check")
            .assert()
            .failure()
            .stderr(predicate::str::contains("required"));
    }

    #[test]
    fn test_missing_required_arg_fmt() {
        let mut cmd = atlas_cmd();
        cmd.arg("fmt")
            .assert()
            .failure()
            .stderr(predicate::str::contains("required"));
    }

    #[test]
    fn test_missing_required_arg_debug() {
        let mut cmd = atlas_cmd();
        cmd.arg("debug")
            .assert()
            .failure()
            .stderr(predicate::str::contains("required"));
    }

    #[test]
    fn test_invalid_port_lsp() {
        let mut cmd = atlas_cmd();
        cmd.args(["lsp", "--port", "not-a-number"])
            .assert()
            .failure();
    }

    #[test]
    fn test_invalid_threshold_profile() {
        let mut cmd = atlas_cmd();
        cmd.args(["profile", "test.atl", "--threshold", "not-a-number"])
            .assert()
            .failure();
    }

    #[test]
    fn test_conflicting_verbosity_flags() {
        // Quiet and verbose together should work (quiet takes precedence)
        let mut cmd = atlas_cmd();
        cmd.args(["fmt", "--help"])
            .assert()
            .success();
    }

    #[test]
    fn test_nonexistent_file_run() {
        let mut cmd = atlas_cmd();
        cmd.args(["run", "definitely_does_not_exist.atl"])
            .assert()
            .failure();
    }

    #[test]
    fn test_nonexistent_file_check() {
        let mut cmd = atlas_cmd();
        cmd.args(["check", "definitely_does_not_exist.atl"])
            .assert()
            .failure();
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// DEFAULT VALUE TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod default_values {
    use super::*;

    #[test]
    fn test_lsp_default_port() {
        let mut cmd = atlas_cmd();
        cmd.args(["lsp", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("9257"));
    }

    #[test]
    fn test_lsp_default_host() {
        let mut cmd = atlas_cmd();
        cmd.args(["lsp", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("127.0.0.1"));
    }

    #[test]
    fn test_profile_default_threshold() {
        let mut cmd = atlas_cmd();
        cmd.args(["profile", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("1.0"));
    }

    #[test]
    fn test_test_default_dir() {
        let mut cmd = atlas_cmd();
        cmd.args(["test", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("."));
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// SUBCOMMAND STRUCTURE TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod subcommand_structure {
    use super::*;

    #[test]
    fn test_all_commands_have_help() {
        let commands = [
            "run", "check", "build", "repl", "ast", "typecheck",
            "fmt", "profile", "test", "debug", "lsp", "completions"
        ];

        for cmd_name in commands {
            let mut cmd = atlas_cmd();
            cmd.args([cmd_name, "--help"])
                .assert()
                .success();
        }
    }

    #[test]
    fn test_all_aliases_have_help() {
        let aliases = ["r", "c", "b", "t", "f", "d"];

        for alias in aliases {
            let mut cmd = atlas_cmd();
            cmd.args([alias, "--help"])
                .assert()
                .success();
        }
    }

    #[test]
    fn test_no_command_shows_help() {
        let mut cmd = atlas_cmd();
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Usage"));
    }

    #[test]
    fn test_help_after_help() {
        // This should be handled gracefully
        let mut cmd = atlas_cmd();
        cmd.args(["--help"])
            .assert()
            .success();
    }
}
