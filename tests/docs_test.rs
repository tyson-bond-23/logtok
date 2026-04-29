use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn docs_generates_html_file() {
    let tmp = TempDir::new().unwrap();
    let output = tmp.path().join("test-docs.html");

    Command::cargo_bin("logtok")
        .unwrap()
        .args(["docs", "-o"])
        .arg(&output)
        .assert()
        .success()
        .stderr(predicate::str::contains("documentation written to"));

    let html = fs::read_to_string(&output).unwrap();
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("logtok"));
}

#[test]
fn docs_contains_all_subcommands() {
    let tmp = TempDir::new().unwrap();
    let output = tmp.path().join("test-docs.html");

    Command::cargo_bin("logtok")
        .unwrap()
        .args(["docs", "-o"])
        .arg(&output)
        .assert()
        .success();

    let html = fs::read_to_string(&output).unwrap();
    // All real subcommands present
    assert!(html.contains("tokenize"), "Missing tokenize command");
    assert!(html.contains("detokenize"), "Missing detokenize command");
    assert!(html.contains("reset-store"), "Missing reset-store command");
    // docs subcommand filtered out
    assert!(
        !html.contains("<h3>logtok docs</h3>") && !html.contains("id=\"cmd-docs\""),
        "docs subcommand should not appear in its own output"
    );
}

#[test]
fn docs_contains_token_categories() {
    let tmp = TempDir::new().unwrap();
    let output = tmp.path().join("test-docs.html");

    Command::cargo_bin("logtok")
        .unwrap()
        .args(["docs", "-o"])
        .arg(&output)
        .assert()
        .success();

    let html = fs::read_to_string(&output).unwrap();
    // Spot-check several token categories
    assert!(html.contains("IP"), "Missing IP category");
    assert!(html.contains("EMAIL"), "Missing EMAIL category");
    assert!(html.contains("JWT"), "Missing JWT category");
    assert!(html.contains("PEM"), "Missing PEM category");
    assert!(html.contains("CUSTOM"), "Missing CUSTOM category");
}

#[test]
fn docs_is_self_contained() {
    let tmp = TempDir::new().unwrap();
    let output = tmp.path().join("test-docs.html");

    Command::cargo_bin("logtok")
        .unwrap()
        .args(["docs", "-o"])
        .arg(&output)
        .assert()
        .success();

    let html = fs::read_to_string(&output).unwrap();
    // No external CSS or JS references
    assert!(
        !html.contains("<link rel=\"stylesheet\""),
        "External CSS found"
    );
    assert!(!html.contains("<script src="), "External JS found");
    // Has embedded CSS and JS
    assert!(html.contains("<style>"), "Missing embedded CSS");
    assert!(html.contains("<script>"), "Missing embedded JS");
}

#[test]
fn docs_quiet_suppresses_output() {
    let tmp = TempDir::new().unwrap();
    let output = tmp.path().join("test-docs.html");

    Command::cargo_bin("logtok")
        .unwrap()
        .args(["--quiet", "docs", "-o"])
        .arg(&output)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    assert!(output.exists());
}

#[test]
fn docs_help_shows_usage() {
    Command::cargo_bin("logtok")
        .unwrap()
        .args(["docs", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate HTML documentation"));
}
