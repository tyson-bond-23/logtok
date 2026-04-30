use assert_cmd::Command;

#[test]
fn ui_help_shows_subcommand() {
    // Verify the ui subcommand is registered and shows help
    let mut cmd = Command::cargo_bin("logtok").unwrap();
    let output = cmd.arg("ui").arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Start the interactive dashboard"));
    assert!(stdout.contains("--port"));
}

#[test]
fn ui_version_in_help() {
    // Verify the main help lists the ui subcommand
    let mut cmd = Command::cargo_bin("logtok").unwrap();
    let output = cmd.arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ui"));
}
