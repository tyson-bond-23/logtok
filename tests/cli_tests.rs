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
