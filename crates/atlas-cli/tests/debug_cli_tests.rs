//! Integration tests for atlas debug command

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_test_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();
    file
}

fn atlas_cmd() -> Command {
    Command::from(assert_cmd::cargo::cargo_bin_cmd!("atlas"))
}

// ── Basic launch tests ────────────────────────────────────────────────────────

#[test]
fn test_debug_help() {
    let mut cmd = atlas_cmd();
    cmd.args(["debug", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Debug an Atlas program"));
}

#[test]
fn test_debug_missing_file() {
    let mut cmd = atlas_cmd();
    cmd.args(["debug", "nonexistent.atlas"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to read"));
}

#[test]
fn test_debug_syntax_error() {
    let file = create_test_file("let x = ;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_debug_launch_and_quit() {
    let file = create_test_file("let x = 1;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Atlas Debugger"))
        .stdout(predicate::str::contains("Debugger exited"));
}

#[test]
fn test_debug_shows_source() {
    let file = create_test_file("let x = 42;\nlet y = x + 1;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("let x = 42"));
}

// ── Breakpoint tests ──────────────────────────────────────────────────────────

#[test]
fn test_debug_set_breakpoint() {
    let file = create_test_file("let x = 1;\nlet y = 2;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("break 1\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Breakpoint"));
}

#[test]
fn test_debug_set_breakpoint_with_flag() {
    let file = create_test_file("let x = 1;\nlet y = 2;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap(), "-b", "1"])
        .write_stdin("quit\n")
        .assert()
        .success();
}

#[test]
fn test_debug_list_breakpoints() {
    let file = create_test_file("let x = 1;\nlet y = 2;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("break 1\nbp\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Breakpoints:"));
}

#[test]
fn test_debug_delete_breakpoint() {
    let file = create_test_file("let x = 1;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("break 1\ndelete 1\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("deleted"));
}

#[test]
fn test_debug_delete_all_breakpoints() {
    let file = create_test_file("let x = 1;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("break 1\ndelete all\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("cleared"));
}

#[test]
fn test_debug_no_breakpoints_message() {
    let file = create_test_file("let x = 1;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("bp\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("No breakpoints"));
}

// ── Execution tests ───────────────────────────────────────────────────────────

#[test]
fn test_debug_run_command() {
    let file = create_test_file("let x = 1 + 2;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("run\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Starting program"));
}

#[test]
fn test_debug_step_command() {
    let file = create_test_file("let x = 1;\nlet y = 2;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("step\nquit\n")
        .assert()
        .success();
}

#[test]
fn test_debug_next_command() {
    let file = create_test_file("let x = 1;\nlet y = 2;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("next\nquit\n")
        .assert()
        .success();
}

#[test]
fn test_debug_continue_command() {
    let file = create_test_file("let x = 1;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("continue\nquit\n")
        .assert()
        .success();
}

// ── Inspection tests ──────────────────────────────────────────────────────────

#[test]
fn test_debug_vars_command() {
    let file = create_test_file("let x = 1;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("run\nvars\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Variables"));
}

#[test]
fn test_debug_backtrace_command() {
    let file = create_test_file("let x = 1;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("backtrace\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Call Stack").or(predicate::str::contains("No stack")));
}

#[test]
fn test_debug_print_expression() {
    let file = create_test_file("let x = 42;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("run\nprint 1 + 1\nquit\n")
        .assert()
        .success();
}

#[test]
fn test_debug_location_command() {
    let file = create_test_file("let x = 1;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("location\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("At"));
}

// ── Source listing tests ──────────────────────────────────────────────────────

#[test]
fn test_debug_list_command() {
    let file = create_test_file("let a = 1;\nlet b = 2;\nlet c = 3;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("list\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("let a = 1"));
}

#[test]
fn test_debug_list_with_line() {
    let file = create_test_file("let a = 1;\nlet b = 2;\nlet c = 3;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("list 2\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("let b = 2"));
}

// ── Help and error handling tests ─────────────────────────────────────────────

#[test]
fn test_debug_help_command() {
    let file = create_test_file("let x = 1;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("help\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Debugger Commands"));
}

#[test]
fn test_debug_unknown_command() {
    let file = create_test_file("let x = 1;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("foobar\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Unknown command"));
}

#[test]
fn test_debug_break_without_args() {
    let file = create_test_file("let x = 1;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("break\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn test_debug_print_without_args() {
    let file = create_test_file("let x = 1;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("print\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn test_debug_delete_without_args() {
    let file = create_test_file("let x = 1;");
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("delete\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

// ── Command aliases ───────────────────────────────────────────────────────────

#[test]
fn test_debug_quit_aliases() {
    let file = create_test_file("let x = 1;");

    // Test 'q'
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("q\n")
        .assert()
        .success();

    // Test 'exit'
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("exit\n")
        .assert()
        .success();
}

#[test]
fn test_debug_help_aliases() {
    let file = create_test_file("let x = 1;");

    // Test 'h'
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("h\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Debugger Commands"));

    // Test '?'
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("?\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Debugger Commands"));
}

#[test]
fn test_debug_step_aliases() {
    let file = create_test_file("let x = 1;");

    // Test 's' alias
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("s\nquit\n")
        .assert()
        .success();
}

#[test]
fn test_debug_next_aliases() {
    let file = create_test_file("let x = 1;");

    // Test 'n' alias
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("n\nquit\n")
        .assert()
        .success();
}

#[test]
fn test_debug_continue_aliases() {
    let file = create_test_file("let x = 1;");

    // Test 'c' alias
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("c\nquit\n")
        .assert()
        .success();
}

#[test]
fn test_debug_run_aliases() {
    let file = create_test_file("let x = 1;");

    // Test 'r' alias
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("r\nquit\n")
        .assert()
        .success();
}

#[test]
fn test_debug_breakpoint_aliases() {
    let file = create_test_file("let x = 1;");

    // Test 'b' alias
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("b 1\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Breakpoint"));
}

#[test]
fn test_debug_delete_aliases() {
    let file = create_test_file("let x = 1;");

    // Test 'd' alias
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("b 1\nd 1\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("deleted"));
}

#[test]
fn test_debug_vars_aliases() {
    let file = create_test_file("let x = 1;");

    // Test 'v' alias
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("v\nquit\n")
        .assert()
        .success();

    // Test 'locals' alias
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("locals\nquit\n")
        .assert()
        .success();
}

#[test]
fn test_debug_print_aliases() {
    let file = create_test_file("let x = 1;");

    // Test 'p' alias
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("p 1\nquit\n")
        .assert()
        .success();

    // Test 'inspect' alias
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("inspect 1\nquit\n")
        .assert()
        .success();
}

#[test]
fn test_debug_backtrace_aliases() {
    let file = create_test_file("let x = 1;");

    // Test 'bt' alias
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("bt\nquit\n")
        .assert()
        .success();

    // Test 'where' alias
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("where\nquit\n")
        .assert()
        .success();
}

#[test]
fn test_debug_list_aliases() {
    let file = create_test_file("let x = 1;");

    // Test 'l' alias
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("l\nquit\n")
        .assert()
        .success();
}

#[test]
fn test_debug_location_aliases() {
    let file = create_test_file("let x = 1;");

    // Test 'loc' alias
    let mut cmd = atlas_cmd();
    cmd.args(["debug", file.path().to_str().unwrap()])
        .write_stdin("loc\nquit\n")
        .assert()
        .success();
}
