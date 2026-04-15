use logtok::config::{find_config_from, load_config, LoktokConfig};
use std::fs;
use tempfile::TempDir;

#[test]
fn find_config_in_cwd() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join(".logtok.toml"), "").unwrap();
    let result = find_config_from(dir.path());
    assert!(result.is_some());
    assert_eq!(result.unwrap(), dir.path().join(".logtok.toml"));
}

#[test]
fn find_config_in_parent_directory() {
    let parent = TempDir::new().unwrap();
    fs::write(parent.path().join(".logtok.toml"), "").unwrap();
    let child = parent.path().join("subdir");
    fs::create_dir(&child).unwrap();
    let result = find_config_from(&child);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), parent.path().join(".logtok.toml"));
}

#[test]
fn find_config_returns_none_when_missing() {
    let dir = TempDir::new().unwrap();
    // No .logtok.toml created
    let result = find_config_from(dir.path());
    assert!(result.is_none());
}

#[test]
fn load_config_parses_full_toml() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join(".logtok.toml");
    fs::write(
        &config_path,
        r#"
[detection]
disabled = ["IP", "HOST"]

[[detection.custom_patterns]]
name = "CUSTOM_ID"
pattern = "CID-\\d{6}"

[store]
ttl_days = 7
"#,
    )
    .unwrap();
    let config = load_config(&config_path).unwrap();
    assert_eq!(config.detection.disabled, vec!["IP", "HOST"]);
    assert_eq!(config.detection.custom_patterns.len(), 1);
    assert_eq!(config.detection.custom_patterns[0].name, "CUSTOM_ID");
    assert_eq!(config.store.ttl_days, 7);
}

#[test]
fn load_config_returns_defaults_for_empty_toml() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join(".logtok.toml");
    fs::write(&config_path, "").unwrap();
    let config = load_config(&config_path).unwrap();
    assert!(config.detection.disabled.is_empty());
    assert!(config.detection.custom_patterns.is_empty());
    assert_eq!(config.store.ttl_days, 30);
}

#[test]
fn load_config_returns_defaults_for_store_only_toml() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join(".logtok.toml");
    fs::write(&config_path, "[store]\nttl_days = 14\n").unwrap();
    let config = load_config(&config_path).unwrap();
    assert!(config.detection.disabled.is_empty());
    assert!(config.detection.custom_patterns.is_empty());
    assert_eq!(config.store.ttl_days, 14);
}

#[test]
fn to_detection_config_converts_correctly() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join(".logtok.toml");
    fs::write(
        &config_path,
        r#"
[detection]
disabled = ["EMAIL"]

[[detection.custom_patterns]]
name = "TICKET"
pattern = "TICK-\\d+"
capture_group = 0
"#,
    )
    .unwrap();
    let config = load_config(&config_path).unwrap();
    let det = config.to_detection_config();
    assert_eq!(det.disabled_categories, vec!["EMAIL"]);
    assert_eq!(det.custom_patterns.len(), 1);
    assert_eq!(det.custom_patterns[0].name, "TICKET");
    assert_eq!(det.custom_patterns[0].capture_group, Some(0));
}

#[test]
fn load_config_invalid_toml_produces_error() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join(".logtok.toml");
    fs::write(&config_path, "this is not valid toml [[[").unwrap();
    let result = load_config(&config_path);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Config error"));
}

#[test]
fn default_ttl_is_30_days() {
    let config = LoktokConfig::default();
    assert_eq!(config.store.ttl_days, 30);
    assert_eq!(config.ttl_seconds(), 2592000);
}

#[test]
fn custom_ttl_overrides_default() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join(".logtok.toml");
    fs::write(&config_path, "[store]\nttl_days = 7\n").unwrap();
    let config = load_config(&config_path).unwrap();
    assert_eq!(config.store.ttl_days, 7);
    assert_eq!(config.ttl_seconds(), 604800);
}
