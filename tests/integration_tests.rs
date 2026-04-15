use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;
use tempfile::TempDir;

fn logtok_cmd() -> Command {
    Command::cargo_bin("logtok").expect("binary should exist")
}

/// Return absolute path to a test fixture file.
fn fixture_path(name: &str) -> std::path::PathBuf {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("tests").join("fixtures").join(name)
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

    let _result = logtok_cmd()
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

// ============================================================
// New integration tests for Phase 2 Plan 03
// ============================================================

#[test]
fn dry_run_shows_summary_no_output() {
    let output = logtok_cmd()
        .arg("--dry-run")
        .arg("tests/fixtures/sample_plain.log")
        .arg("--quiet")
        .output()
        .expect("failed to run logtok");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // stdout must be empty (no tokenized output written)
    assert!(stdout.is_empty(), "Dry-run stdout should be empty, got: {}", stdout);

    // stderr should contain dry-run summary
    assert!(stderr.contains("dry-run:"), "Stderr should contain 'dry-run:' marker");
    assert!(
        stderr.contains("sensitive values detected"),
        "Stderr should contain 'sensitive values detected'"
    );
    assert!(stderr.contains("Category"), "Stderr should contain 'Category' header");
}

#[test]
fn dry_run_hides_key_values() {
    let output = logtok_cmd()
        .arg("--dry-run")
        .arg("tests/fixtures/sample_plain.log")
        .arg("--quiet")
        .output()
        .expect("failed to run logtok");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // KEY and PASS categories should show "(values hidden)" per T-02-12
    assert!(
        stderr.contains("(values hidden)"),
        "Stderr should contain '(values hidden)' for sensitive categories"
    );
}

#[test]
fn config_flag_loads_custom_config() {
    // Create a temp config that disables IP detection
    let tmp_dir = TempDir::new().expect("create temp dir");
    let config_path = tmp_dir.path().join(".logtok.toml");
    fs::write(
        &config_path,
        "[detection]\ndisabled = [\"IP\"]\n",
    )
    .expect("write config");

    let output = logtok_cmd()
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("tests/fixtures/sample_plain.log")
        .arg("--quiet")
        .output()
        .expect("failed to run logtok");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // With IP disabled, output should NOT contain [IP_ tokens
    assert!(
        !stdout.contains("[IP_"),
        "Output should not contain IP tokens when IP category is disabled"
    );
    // But raw IPs should appear since they are not tokenized
    assert!(
        stdout.contains("192.168.1.100"),
        "Output should contain raw IP when IP detection is disabled"
    );
}

#[test]
fn reset_store_flag() {
    let tmp_dir = TempDir::new().expect("create temp dir");
    let store_dir = tmp_dir.path().join(".logtok");
    fs::create_dir_all(&store_dir).expect("create store dir");
    let store_file = store_dir.join("store.enc");
    fs::write(&store_file, b"dummy data").expect("write dummy store");

    assert!(store_file.exists(), "Store file should exist before reset");

    let _result = logtok_cmd()
        .arg("--reset-store")
        .env("LOGTOK_KEY", "testkey123")
        .current_dir(tmp_dir.path())
        .assert()
        .success();

    assert!(
        !store_file.exists(),
        "Store file should be deleted after --reset-store"
    );
}

#[test]
fn new_categories_detected_in_output() {
    let output = logtok_cmd()
        .arg("tests/fixtures/sample_plain.log")
        .arg("--quiet")
        .output()
        .expect("failed to run logtok");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify new category tokens from the added fixture lines
    assert!(
        stdout.contains("[JWT_"),
        "Output should contain JWT tokens from fixture"
    );
    assert!(
        stdout.contains("[CONN_"),
        "Output should contain CONN tokens from fixture"
    );
    assert!(
        stdout.contains("[MAC_"),
        "Output should contain MAC tokens from fixture"
    );
    assert!(
        stdout.contains("[UUID_"),
        "Output should contain UUID tokens from fixture"
    );
    assert!(
        stdout.contains("[OS_"),
        "Output should contain OS tokens from fixture"
    );
}

#[test]
fn store_persistence_across_runs() {
    let tmp_dir = TempDir::new().expect("create temp dir");
    let output1_file = tmp_dir.path().join("output1.log");
    let output2_file = tmp_dir.path().join("output2.log");
    let fixture = fixture_path("sample_plain.log");

    // Run 1: tokenize with LOGTOK_KEY set
    logtok_cmd()
        .arg(fixture.to_str().unwrap())
        .arg("--quiet")
        .arg("--output")
        .arg(output1_file.to_str().unwrap())
        .env("LOGTOK_KEY", "testkey123")
        .current_dir(tmp_dir.path())
        .assert()
        .success();

    // Verify store was created
    let store_file = tmp_dir.path().join(".logtok").join("store.enc");
    assert!(
        store_file.exists(),
        "Store file should be created after first run"
    );

    // Run 2: tokenize again with same key
    logtok_cmd()
        .arg(fixture.to_str().unwrap())
        .arg("--quiet")
        .arg("--output")
        .arg(output2_file.to_str().unwrap())
        .env("LOGTOK_KEY", "testkey123")
        .current_dir(tmp_dir.path())
        .assert()
        .success();

    // Both outputs should be identical (same tokens assigned via store)
    let out1 = fs::read_to_string(&output1_file).expect("read output1");
    let out2 = fs::read_to_string(&output2_file).expect("read output2");
    assert_eq!(out1, out2, "Both runs should produce identical token assignments via store persistence");
}
