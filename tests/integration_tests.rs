use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

fn logtok_cmd() -> Command {
    Command::cargo_bin("logtok").expect("binary should exist")
}

#[test]
fn plain_text_has_tokens_no_raw_ips() {
    let output = logtok_cmd()
        .arg("tests/fixtures/sample_plain.log")
        .arg("--quiet")
        .output()
        .expect("failed to run logtok");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain IP tokens
    assert!(stdout.contains("[IP_"), "Output should contain IP tokens");
    // Should NOT contain raw IPs
    assert!(!stdout.contains("192.168.1.100"), "Output should not contain raw IP 192.168.1.100");
    assert!(!stdout.contains("10.0.0.55"), "Output should not contain raw IP 10.0.0.55");
    assert!(!stdout.contains("172.16.0.42"), "Output should not contain raw IP 172.16.0.42");
}

#[test]
fn plain_text_has_no_raw_emails() {
    let output = logtok_cmd()
        .arg("tests/fixtures/sample_plain.log")
        .arg("--quiet")
        .output()
        .expect("failed to run logtok");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[EMAIL_"), "Output should contain EMAIL tokens");
    assert!(!stdout.contains("admin@example.com"), "Output should not contain raw email");
    assert!(!stdout.contains("user@company.org"), "Output should not contain raw email");
}

#[test]
fn json_output_is_valid_json_with_tokens() {
    let output = logtok_cmd()
        .arg("tests/fixtures/sample_json.log")
        .arg("--quiet")
        .output()
        .expect("failed to run logtok");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Each non-empty line should be valid JSON (compaction may add [xN] prefix)
    for line in stdout.lines() {
        if line.is_empty() {
            continue;
        }
        // Strip compaction prefix if present
        let json_str = if line.starts_with("[x") {
            if let Some(pos) = line.find("] {") {
                &line[pos + 2..]
            } else {
                line
            }
        } else {
            line
        };
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_str);
        assert!(parsed.is_ok(), "Line should be valid JSON: {}", json_str);
    }
    // Should contain tokens in values
    assert!(stdout.contains("[IP_"), "JSON output should contain IP tokens");
}

#[test]
fn compaction_reduces_duplicate_lines() {
    let output = logtok_cmd()
        .arg("tests/fixtures/sample_plain.log")
        .arg("--quiet")
        .output()
        .expect("failed to run logtok");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // The fixture has 3 consecutive identical lines (Connected to db-primary.internal:5432)
    // and 3 consecutive notification lines -- compaction should produce [x3] prefixes
    assert!(stdout.contains("[x3]"), "Output should contain [x3] compaction prefix");
    // Fixture has 2 consecutive "Connection refused" lines
    assert!(stdout.contains("[x2]"), "Output should contain [x2] compaction prefix");
}

#[test]
fn output_flag_writes_to_file() {
    let output_file = NamedTempFile::new().expect("create temp file");
    let output_path = output_file.path().to_str().unwrap();

    let result = logtok_cmd()
        .arg("tests/fixtures/sample_plain.log")
        .arg("--output")
        .arg(output_path)
        .arg("--quiet")
        .assert()
        .success();

    let content = fs::read_to_string(output_path).expect("read output file");
    assert!(content.contains("[IP_"), "Output file should contain IP tokens");
    assert!(!content.is_empty(), "Output file should not be empty");
}

#[test]
fn quiet_flag_suppresses_progress() {
    let output = logtok_cmd()
        .arg("tests/fixtures/sample_plain.log")
        .arg("--quiet")
        .output()
        .expect("failed to run logtok");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // With --quiet, stderr should only contain the summary line, no progress bar
    assert!(!stderr.contains("=>"), "Progress bar characters should not appear with --quiet");
}

#[test]
fn determinism_same_ip_gets_same_token() {
    let output = logtok_cmd()
        .arg("tests/fixtures/sample_plain.log")
        .arg("--quiet")
        .output()
        .expect("failed to run logtok");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // 192.168.1.100 appears in lines 1 and 13 of the fixture
    // Both should produce the same token [IP_001]
    let lines: Vec<&str> = stdout.lines().collect();
    // Find lines that would have had 192.168.1.100 (first and "Request from" lines)
    let first_line = &lines[0]; // "Server started on [IP_001]:8080"
    let request_line = lines.iter().find(|l| l.contains("Request from")).unwrap();
    // Both should reference the same IP token
    assert!(first_line.contains("[IP_001]"), "First line should have [IP_001]");
    assert!(request_line.contains("[IP_001]"), "Request line should have same [IP_001] token");
}

#[test]
fn token_determinism_across_blocks() {
    // Use a very small block size to force multiple blocks
    // The same IP should get the same token regardless of which block it's in
    let output = logtok_cmd()
        .arg("tests/fixtures/sample_plain.log")
        .arg("--block-size")
        .arg("1024") // Force many small blocks
        .arg("--quiet")
        .output()
        .expect("failed to run logtok");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // 192.168.1.100 appears in both early and late lines
    // With small blocks, they'll be in different blocks
    // Both should produce the same token
    let ip_token_count = stdout.matches("[IP_001]").count();
    assert!(ip_token_count >= 2, "Same IP should produce same token across blocks, found {} occurrences", ip_token_count);
}
