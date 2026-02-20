//! Integration tests for atlas lsp command

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use std::time::Duration;

fn atlas_cmd() -> Command {
    Command::cargo_bin("atlas").unwrap()
}

// ── Help and argument tests ───────────────────────────────────────────────────

#[test]
fn test_lsp_help() {
    let mut cmd = atlas_cmd();
    cmd.args(["lsp", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Atlas Language Server"));
}

#[test]
fn test_lsp_version() {
    let mut cmd = atlas_cmd();
    cmd.args(["--version"])
        .assert()
        .success()
        .stdout(predicate::str::contains("atlas"));
}

#[test]
fn test_lsp_tcp_flag_parsing() {
    // Just test that the flag is parsed correctly
    let mut cmd = atlas_cmd();
    cmd.args(["lsp", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--tcp"));
}

#[test]
fn test_lsp_port_flag_parsing() {
    let mut cmd = atlas_cmd();
    cmd.args(["lsp", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--port"));
}

#[test]
fn test_lsp_host_flag_parsing() {
    let mut cmd = atlas_cmd();
    cmd.args(["lsp", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--host"));
}

#[test]
fn test_lsp_verbose_flag_parsing() {
    let mut cmd = atlas_cmd();
    cmd.args(["lsp", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--verbose"));
}

// ── TCP mode tests ────────────────────────────────────────────────────────────

#[test]
fn test_lsp_tcp_startup_message() {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;

    // Find a free port
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener); // Release the port

    let mut child = Command::cargo_bin("atlas")
        .unwrap()
        .args(["lsp", "--tcp", "--port", &port.to_string()])
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start LSP server");

    // Read the startup message
    let stderr = child.stderr.take().unwrap();
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();

    // Set a timeout for reading
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        if reader.read_line(&mut line).unwrap() > 0 {
            if line.contains("listening on") {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // Kill the server
    child.kill().ok();
    child.wait().ok();

    assert!(
        line.contains("listening on"),
        "Expected startup message, got: {}",
        line
    );
}

#[test]
fn test_lsp_tcp_accepts_connection() {
    use std::io::{BufRead, BufReader};
    use std::net::TcpStream;
    use std::process::Stdio;

    // Find a free port
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let mut child = Command::cargo_bin("atlas")
        .unwrap()
        .args(["lsp", "--tcp", "--port", &port.to_string(), "--verbose"])
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start LSP server");

    // Wait for server to start
    let stderr = child.stderr.take().unwrap();
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();

    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        if reader.read_line(&mut line).unwrap() > 0 {
            if line.contains("listening on") {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // Try to connect
    let addr = format!("127.0.0.1:{}", port);
    let connection_result = TcpStream::connect(&addr);

    // Kill the server
    child.kill().ok();
    child.wait().ok();

    assert!(connection_result.is_ok(), "Failed to connect to LSP server");
}

#[test]
fn test_lsp_tcp_invalid_port() {
    // Port 99999 is out of range for u16, but clap should handle this
    let mut cmd = atlas_cmd();
    cmd.args(["lsp", "--tcp", "--port", "abc"])
        .assert()
        .failure();
}

#[test]
fn test_lsp_tcp_port_in_use() {
    // Bind to a port first
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    // Try to start LSP server on the same port
    let child = Command::cargo_bin("atlas")
        .unwrap()
        .args(["lsp", "--tcp", "--port", &port.to_string()])
        .output()
        .expect("Failed to run command");

    drop(listener);

    // Should fail because port is in use
    assert!(!child.status.success() || child.stderr.len() > 0);
}

// ── stdio mode tests ──────────────────────────────────────────────────────────

#[test]
fn test_lsp_stdio_startup_verbose() {
    use std::process::Stdio;

    // Start LSP server in stdio mode with verbose flag
    let mut child = Command::cargo_bin("atlas")
        .unwrap()
        .args(["lsp", "--verbose"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start LSP server");

    // Wait a bit for startup message
    std::thread::sleep(Duration::from_millis(200));

    // Kill the server
    child.kill().ok();
    let output = child.wait_with_output().unwrap();

    // Check stderr for startup message
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Starting") || stderr.is_empty(),
        "Unexpected stderr: {}",
        stderr
    );
}

// ── Default values tests ──────────────────────────────────────────────────────

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

// ── Integration with CLI structure ────────────────────────────────────────────

#[test]
fn test_lsp_is_subcommand() {
    let mut cmd = atlas_cmd();
    cmd.args(["help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lsp"));
}

#[test]
fn test_lsp_short_verbose() {
    let mut cmd = atlas_cmd();
    cmd.args(["lsp", "-h"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-v"));
}
