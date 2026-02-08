//! CLI Integration Tests for FORMA compiler
//!
//! Tests the `forma` binary end-to-end using `std::process::Command`.

use std::path::PathBuf;
use std::process::Command;

/// Get the path to the forma binary (debug build).
fn forma_bin() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("forma");
    path
}

/// Get the path to a test fixture file.
fn fixture(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    path
}

#[test]
fn test_cli_run_hello() {
    let output = Command::new(forma_bin())
        .args(["run", "--allow-all"])
        .arg(fixture("hello.forma"))
        .output()
        .expect("failed to execute forma");
    assert!(
        output.status.success(),
        "forma run hello.forma should exit 0"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello"), "stdout should contain 'hello'");
}

#[test]
fn test_cli_run_syntax_error() {
    let output = Command::new(forma_bin())
        .args(["run"])
        .arg(fixture("syntax_error.forma"))
        .output()
        .expect("failed to execute forma");
    assert!(
        !output.status.success(),
        "forma run syntax_error.forma should exit nonzero"
    );
}

#[test]
fn test_cli_run_syntax_error_json() {
    let output = Command::new(forma_bin())
        .args(["--error-format", "json", "run"])
        .arg(fixture("syntax_error.forma"))
        .output()
        .expect("failed to execute forma");
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\"error\"") || stdout.contains("\"errors\""),
        "JSON output should contain error key, got: {}",
        stdout
    );
}

#[test]
fn test_cli_check_hello() {
    let output = Command::new(forma_bin())
        .args(["check"])
        .arg(fixture("hello.forma"))
        .output()
        .expect("failed to execute forma");
    assert!(
        output.status.success(),
        "forma check hello.forma should exit 0"
    );
}

#[test]
fn test_cli_check_type_error() {
    let output = Command::new(forma_bin())
        .args(["check"])
        .arg(fixture("type_error.forma"))
        .output()
        .expect("failed to execute forma");
    assert!(
        !output.status.success(),
        "forma check type_error.forma should exit nonzero"
    );
}

#[test]
fn test_cli_lex_hello() {
    let output = Command::new(forma_bin())
        .args(["lex"])
        .arg(fixture("hello.forma"))
        .output()
        .expect("failed to execute forma");
    assert!(
        output.status.success(),
        "forma lex hello.forma should exit 0"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Lex output should contain token names
    assert!(
        stdout.contains("Fn") || stdout.contains("fn") || stdout.contains("Ident"),
        "lex output should contain token names, got: {}",
        stdout
    );
}

#[test]
fn test_cli_parse_hello() {
    let output = Command::new(forma_bin())
        .args(["parse"])
        .arg(fixture("hello.forma"))
        .output()
        .expect("failed to execute forma");
    assert!(
        output.status.success(),
        "forma parse hello.forma should exit 0"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse output should contain AST-related terms
    assert!(
        stdout.contains("main") || stdout.contains("Function") || stdout.contains("item"),
        "parse output should contain AST terms, got: {}",
        stdout
    );
}

#[test]
fn test_cli_fmt_hello() {
    let output = Command::new(forma_bin())
        .args(["fmt"])
        .arg(fixture("hello.forma"))
        .output()
        .expect("failed to execute forma");
    assert!(
        output.status.success(),
        "forma fmt hello.forma should exit 0"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("main"),
        "fmt output should contain formatted code with 'main'"
    );
}

#[test]
fn test_cli_fmt_json_error() {
    let output = Command::new(forma_bin())
        .args(["--error-format", "json", "fmt"])
        .arg(fixture("syntax_error.forma"))
        .output()
        .expect("failed to execute forma");
    assert!(
        !output.status.success(),
        "forma fmt syntax_error.forma should exit nonzero"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\"error\"") || stdout.contains("\"errors\""),
        "JSON fmt error should contain error key, got: {}",
        stdout
    );
}

#[test]
fn test_cli_run_env_denied() {
    let output = Command::new(forma_bin())
        .args(["run"])
        .arg(fixture("env_usage.forma"))
        .output()
        .expect("failed to execute forma");
    assert!(
        !output.status.success(),
        "forma run env_usage.forma without --allow-env should exit nonzero"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("capability") || stderr.contains("Capability"),
        "error should mention capability, got: {}",
        stderr
    );
}

#[test]
fn test_cli_run_env_allowed() {
    let output = Command::new(forma_bin())
        .args(["run", "--allow-env"])
        .arg(fixture("env_usage.forma"))
        .output()
        .expect("failed to execute forma");
    assert!(
        output.status.success(),
        "forma run --allow-env env_usage.forma should exit 0"
    );
}

#[test]
fn test_cli_run_allow_all() {
    let output = Command::new(forma_bin())
        .args(["run", "--allow-all"])
        .arg(fixture("env_usage.forma"))
        .output()
        .expect("failed to execute forma");
    assert!(
        output.status.success(),
        "forma run --allow-all env_usage.forma should exit 0"
    );
}

#[test]
fn test_cli_run_no_check_contracts() {
    let output = Command::new(forma_bin())
        .args(["run", "--no-check-contracts"])
        .arg(fixture("contract_fail.forma"))
        .output()
        .expect("failed to execute forma");
    assert!(
        output.status.success(),
        "forma run --no-check-contracts contract_fail.forma should exit 0 (contracts skipped)"
    );
}

#[test]
fn test_cli_run_contract_violation() {
    let output = Command::new(forma_bin())
        .args(["run"])
        .arg(fixture("contract_fail.forma"))
        .output()
        .expect("failed to execute forma");
    assert!(
        !output.status.success(),
        "forma run contract_fail.forma should exit nonzero (contract violation)"
    );
}
