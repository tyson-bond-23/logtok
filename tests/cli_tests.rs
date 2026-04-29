use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_help_flag() {
    Command::cargo_bin("logtok")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tokenize sensitive data"))
        .stdout(predicate::str::contains("tokenize"))
        .stdout(predicate::str::contains("detokenize"))
        .stdout(predicate::str::contains("reset-store"));
}

#[test]
fn test_version_flag() {
    Command::cargo_bin("logtok")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("logtok"));
}

#[test]
fn test_missing_subcommand() {
    Command::cargo_bin("logtok")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn test_nonexistent_file() {
    Command::cargo_bin("logtok")
        .unwrap()
        .arg("tokenize")
        .arg("nonexistent_file_that_does_not_exist.log")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot access file"));
}

#[test]
fn test_valid_file_accepted() {
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "some log line").unwrap();

    Command::cargo_bin("logtok")
        .unwrap()
        .arg("tokenize")
        .arg(tmp.path())
        .assert()
        .success();
}

#[test]
fn test_invalid_block_size_too_small() {
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "some log line").unwrap();

    Command::cargo_bin("logtok")
        .unwrap()
        .arg("tokenize")
        .arg(tmp.path())
        .arg("--block-size")
        .arg("100")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid block size"));
}

#[test]
fn test_tokenize_help() {
    Command::cargo_bin("logtok")
        .unwrap()
        .arg("tokenize")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--clipboard"))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--block-size"))
        .stdout(predicate::str::contains("--dry-run"));
}

#[test]
fn test_detokenize_help() {
    Command::cargo_bin("logtok")
        .unwrap()
        .arg("detokenize")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--detailed"))
        .stdout(predicate::str::contains("--store"));
}

#[test]
fn test_no_color_compliance() {
    let output = Command::cargo_bin("logtok")
        .unwrap()
        .arg("--help")
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // ANSI escape codes start with ESC[ (0x1b 0x5b)
    assert!(
        !stdout.contains("\x1b["),
        "Help output should not contain ANSI escape codes when NO_COLOR=1 is set. Found: {}",
        stdout
    );
}

#[test]
fn test_colored_output_with_clicolor_force() {
    let output = Command::cargo_bin("logtok")
        .unwrap()
        .arg("--help")
        .env("CLICOLOR_FORCE", "1")
        .env_remove("NO_COLOR")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // With CLICOLOR_FORCE=1, ANSI codes should be present even when piped
    assert!(
        stdout.contains("\x1b["),
        "Help output should contain ANSI escape codes when CLICOLOR_FORCE=1. Got: {}",
        stdout
    );
}

#[test]
fn test_root_help_has_examples() {
    Command::cargo_bin("logtok")
        .unwrap()
        .arg("--help")
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("logtok tokenize server.log"))
        .stdout(predicate::str::contains("logtok detokenize response.txt"));
}

#[test]
fn test_tokenize_help_has_examples() {
    Command::cargo_bin("logtok")
        .unwrap()
        .args(["tokenize", "--help"])
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains("logtok tokenize server.log"));
}

#[test]
fn test_detokenize_help_has_examples() {
    Command::cargo_bin("logtok")
        .unwrap()
        .args(["detokenize", "--help"])
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains("logtok detokenize response.txt"));
}

#[test]
fn test_reset_store_help_has_long_description() {
    Command::cargo_bin("logtok")
        .unwrap()
        .args(["reset-store", "--help"])
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Removes the .logtok/store.enc"))
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains("logtok reset-store"));
}

#[test]
fn test_piped_help_no_ansi_codes() {
    // assert_cmd captures stdout via pipe, so anstream should strip ANSI
    let output = Command::cargo_bin("logtok")
        .unwrap()
        .arg("--help")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("NO_COLOR")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("\x1b["),
        "Piped help output should not contain ANSI escape codes. Got: {}",
        stdout
    );
}
