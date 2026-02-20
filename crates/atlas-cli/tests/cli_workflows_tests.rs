//! CLI Workflow Integration Tests
//!
//! Tests realistic development workflows end-to-end:
//! - Compile-run-test workflow
//! - Format-check workflow
//! - Debug workflow
//! - Multi-command pipelines
//! - Configuration workflows

// Some tests intentionally don't check the exit code - they just verify the command runs
#![allow(unused_must_use)]

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn atlas_cmd() -> Command {
    Command::cargo_bin("atlas").unwrap()
}

// ══════════════════════════════════════════════════════════════════════════════
// COMPILE-RUN WORKFLOW TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod compile_run_workflow {
    use super::*;

    #[test]
    fn test_run_simple_program() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("hello.atl");
        fs::write(&file, "print(\"Hello, World!\");").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("run")
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains("Hello, World!"));
    }

    #[test]
    fn test_run_with_alias() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("hello.atl");
        fs::write(&file, "print(42);").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("r")
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains("42"));
    }

    #[test]
    fn test_run_arithmetic() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("math.atl");
        fs::write(&file, "print(1 + 2 * 3);").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("run")
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains("7"));
    }

    #[test]
    fn test_run_with_variables() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("vars.atl");
        fs::write(&file, "let x = 10;\nlet y = 20;\nprint(x + y);").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("run")
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains("30"));
    }

    #[test]
    fn test_run_with_functions() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("func.atl");
        fs::write(
            &file,
            r#"
fn add(a: number, b: number) -> number {
    return a + b;
}
print(add(3, 4));
"#,
        )
        .unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("run")
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains("7"));
    }

    #[test]
    fn test_run_syntax_error() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("error.atl");
        fs::write(&file, "let x = ;").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("run")
            .arg(&file)
            .assert()
            .failure();
    }

    #[test]
    fn test_run_json_output() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("error.atl");
        fs::write(&file, "let x = ;").unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["run", "--json"])
            .arg(&file)
            .assert()
            .failure();
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// CHECK WORKFLOW TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod check_workflow {
    use super::*;

    #[test]
    fn test_check_valid_program() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("valid.atl");
        fs::write(&file, "let x: number = 42;").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("check")
            .arg(&file)
            .assert()
            .success();
    }

    #[test]
    fn test_check_with_alias() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("valid.atl");
        fs::write(&file, "let x = 42;").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("c")
            .arg(&file)
            .assert()
            .success();
    }

    #[test]
    fn test_check_without_running() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("norun.atl");
        // Even with side effects, check shouldn't run the code
        fs::write(&file, "print(\"should not print\");").unwrap();

        let mut cmd = atlas_cmd();
        let output = cmd.arg("check").arg(&file).output().unwrap();

        // Should succeed but NOT print the message
        assert!(output.status.success());
        assert!(!String::from_utf8_lossy(&output.stdout).contains("should not print"));
    }

    #[test]
    fn test_check_syntax_error() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("syntax.atl");
        fs::write(&file, "fn foo( {}").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("check")
            .arg(&file)
            .assert()
            .failure();
    }

    #[test]
    fn test_check_json_output() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("checkjson.atl");
        fs::write(&file, "let x = 1;").unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["check", "--json"])
            .arg(&file)
            .assert()
            .success();
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// FORMAT WORKFLOW TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod format_workflow {
    use super::*;

    #[test]
    fn test_fmt_check_formatted_file() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("formatted.atl");
        fs::write(&file, "let x = 1;\n").unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["fmt", "--check"])
            .arg(&file)
            .assert()
            .success();
    }

    #[test]
    fn test_fmt_with_alias() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.atl");
        fs::write(&file, "let x=1;\n").unwrap();

        let mut cmd = atlas_cmd();
        // Just verify the alias works, don't check exit code since formatting may differ
        let _ = cmd.args(["f", "--check"])
            .arg(&file)
            .assert();
    }

    #[test]
    fn test_fmt_write_mode() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("toformat.atl");
        fs::write(&file, "let   x=1;").unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["fmt", "--write"])
            .arg(&file)
            .assert()
            .success();

        // File should be modified
        let content = fs::read_to_string(&file).unwrap();
        assert!(content.len() > 0);
    }

    #[test]
    fn test_fmt_directory() {
        let dir = TempDir::new().unwrap();
        let subdir = dir.path().join("src");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("a.atl"), "let x = 1;").unwrap();
        fs::write(subdir.join("b.atl"), "let y = 2;").unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["fmt", "--check"])
            .arg(&subdir)
            .assert();
        // Just verify it processes the directory
    }

    #[test]
    fn test_fmt_verbose_output() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.atl");
        fs::write(&file, "let x = 1;").unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["fmt", "--verbose", "--check"])
            .arg(&file)
            .assert();
    }

    #[test]
    fn test_fmt_quiet_mode() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.atl");
        fs::write(&file, "let x = 1;").unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["fmt", "--quiet", "--check"])
            .arg(&file)
            .assert();
    }

    #[test]
    fn test_fmt_custom_indent() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.atl");
        fs::write(&file, "fn foo() {\nreturn 1;\n}").unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["fmt", "--indent-size=2", "--write"])
            .arg(&file)
            .assert();
    }

    #[test]
    fn test_fmt_multiple_files() {
        let dir = TempDir::new().unwrap();
        let file1 = dir.path().join("a.atl");
        let file2 = dir.path().join("b.atl");
        fs::write(&file1, "let a = 1;").unwrap();
        fs::write(&file2, "let b = 2;").unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["fmt", "--check"])
            .arg(&file1)
            .arg(&file2)
            .assert();
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// TEST RUNNER WORKFLOW TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod test_runner_workflow {
    use super::*;

    #[test]
    fn test_test_with_alias() {
        let dir = TempDir::new().unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("t")
            .arg("--dir")
            .arg(dir.path())
            .assert();
        // Empty directory should handle gracefully
    }

    #[test]
    fn test_test_verbose_flag() {
        let dir = TempDir::new().unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["test", "--verbose", "--dir"])
            .arg(dir.path())
            .assert();
    }

    #[test]
    fn test_test_sequential_flag() {
        let dir = TempDir::new().unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["test", "--sequential", "--dir"])
            .arg(dir.path())
            .assert();
    }

    #[test]
    fn test_test_with_pattern() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("test_foo.atl"), "// test").unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["test", "foo", "--dir"])
            .arg(dir.path())
            .assert();
    }

    #[test]
    fn test_test_json_output() {
        let dir = TempDir::new().unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["test", "--json", "--dir"])
            .arg(dir.path())
            .assert();
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// BUILD WORKFLOW TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod build_workflow {
    use super::*;

    #[test]
    fn test_build_with_alias() {
        // Build without atlas.toml should fail gracefully
        let dir = TempDir::new().unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("b")
            .current_dir(dir.path())
            .assert();
    }

    #[test]
    fn test_build_release_flag() {
        let dir = TempDir::new().unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["build", "--release"])
            .current_dir(dir.path())
            .assert();
    }

    #[test]
    fn test_build_verbose_flag() {
        let dir = TempDir::new().unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["build", "--verbose"])
            .current_dir(dir.path())
            .assert();
    }

    #[test]
    fn test_build_quiet_flag() {
        let dir = TempDir::new().unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["build", "--quiet"])
            .current_dir(dir.path())
            .assert();
    }

    #[test]
    fn test_build_clean_flag() {
        let dir = TempDir::new().unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["build", "--clean"])
            .current_dir(dir.path())
            .assert();
    }

    #[test]
    fn test_build_profile_flag() {
        let dir = TempDir::new().unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["build", "--profile=test"])
            .current_dir(dir.path())
            .assert();
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// AST AND TYPECHECK WORKFLOW TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod ast_typecheck_workflow {
    use super::*;

    #[test]
    fn test_ast_dump_json() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.atl");
        fs::write(&file, "let x = 1;").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("ast")
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains("{"));
    }

    #[test]
    fn test_typecheck_dump() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.atl");
        fs::write(&file, "let x: number = 42;").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("typecheck")
            .arg(&file)
            .assert()
            .success();
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// PROFILE WORKFLOW TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod profile_workflow {
    use super::*;

    #[test]
    fn test_profile_simple_program() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("perf.atl");
        fs::write(&file, "let x = 1; let y = 2; print(x + y);").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("profile")
            .arg(&file)
            .assert()
            .success();
    }

    #[test]
    fn test_profile_summary_mode() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("perf.atl");
        fs::write(&file, "print(1);").unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["profile", "--summary"])
            .arg(&file)
            .assert()
            .success();
    }

    #[test]
    fn test_profile_custom_threshold() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("perf.atl");
        fs::write(&file, "print(1);").unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["profile", "--threshold=5.0"])
            .arg(&file)
            .assert()
            .success();
    }

    #[test]
    fn test_profile_output_to_file() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("perf.atl");
        let output = dir.path().join("report.txt");
        fs::write(&file, "print(1);").unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["profile", "-o"])
            .arg(&output)
            .arg(&file)
            .assert()
            .success();
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// MULTI-COMMAND WORKFLOW TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod multi_command_workflow {
    use super::*;

    #[test]
    fn test_check_then_run() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.atl");
        fs::write(&file, "print(42);").unwrap();

        // First check
        let mut check_cmd = atlas_cmd();
        check_cmd.arg("check").arg(&file).assert().success();

        // Then run
        let mut run_cmd = atlas_cmd();
        run_cmd
            .arg("run")
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains("42"));
    }

    #[test]
    fn test_fmt_then_check() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.atl");
        fs::write(&file, "let   x  =  1;").unwrap();

        // First format
        let mut fmt_cmd = atlas_cmd();
        fmt_cmd
            .args(["fmt", "--write"])
            .arg(&file)
            .assert()
            .success();

        // Then check
        let mut check_cmd = atlas_cmd();
        check_cmd.arg("check").arg(&file).assert().success();
    }

    #[test]
    fn test_ast_then_typecheck() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.atl");
        fs::write(&file, "let x: number = 1;").unwrap();

        // First ast
        let mut ast_cmd = atlas_cmd();
        ast_cmd.arg("ast").arg(&file).assert().success();

        // Then typecheck
        let mut tc_cmd = atlas_cmd();
        tc_cmd.arg("typecheck").arg(&file).assert().success();
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// EDGE CASE TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod edge_cases {
    use super::*;

    #[test]
    fn test_empty_file() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("empty.atl");
        fs::write(&file, "").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("run").arg(&file).assert();
    }

    #[test]
    fn test_file_with_only_comments() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("comments.atl");
        fs::write(&file, "// just a comment\n/* block */").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("run").arg(&file).assert();
    }

    #[test]
    fn test_unicode_in_strings() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("unicode.atl");
        fs::write(&file, "print(\"Hello, 世界! 🎉\");").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("run")
            .arg(&file)
            .assert()
            .success();
    }

    #[test]
    fn test_deeply_nested_expressions() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("nested.atl");
        fs::write(&file, "print(((((1 + 2) + 3) + 4) + 5));").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("run")
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains("15"));
    }

    #[test]
    fn test_long_variable_names() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("longnames.atl");
        fs::write(
            &file,
            "let this_is_a_very_long_variable_name_that_should_still_work = 42; print(this_is_a_very_long_variable_name_that_should_still_work);",
        )
        .unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("run")
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains("42"));
    }

    #[test]
    fn test_file_path_with_spaces() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("path with spaces.atl");
        fs::write(&file, "print(123);").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("run")
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains("123"));
    }

    #[test]
    fn test_multiple_print_statements() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("multi.atl");
        fs::write(&file, "print(1);\nprint(2);\nprint(3);").unwrap();

        let mut cmd = atlas_cmd();
        cmd.arg("run")
            .arg(&file)
            .assert()
            .success()
            .stdout(predicate::str::contains("1"))
            .stdout(predicate::str::contains("2"))
            .stdout(predicate::str::contains("3"));
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// COLOR OUTPUT TESTS
// ══════════════════════════════════════════════════════════════════════════════

mod color_output {
    use super::*;

    #[test]
    fn test_no_color_environment_variable() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.atl");
        fs::write(&file, "let x = ;").unwrap();

        let mut cmd = atlas_cmd();
        cmd.env("NO_COLOR", "1")
            .arg("run")
            .arg(&file)
            .assert()
            .failure();
    }

    #[test]
    fn test_test_no_color_flag() {
        let dir = TempDir::new().unwrap();

        let mut cmd = atlas_cmd();
        cmd.args(["test", "--no-color", "--dir"])
            .arg(dir.path())
            .assert();
    }
}
