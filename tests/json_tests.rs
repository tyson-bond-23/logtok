use logtok::detector::DetectionPatterns;
use logtok::json_processor::{is_json_line, tokenize_json_line};
use logtok::tokenizer::TokenMap;

#[test]
fn json_string_value_with_ip_is_tokenized() {
    let patterns = DetectionPatterns::new();
    let mut map = TokenMap::new();
    let input = r#"{"server_ip": "192.168.1.1", "status": "ok"}"#;
    let result = tokenize_json_line(input, &mut map, &patterns).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["server_ip"], "[IP_001]");
    assert_eq!(parsed["status"], "ok");
}

#[test]
fn json_keys_never_tokenized() {
    let patterns = DetectionPatterns::new();
    let mut map = TokenMap::new();
    let input = r#"{"server_ip": "192.168.1.1"}"#;
    let result = tokenize_json_line(input, &mut map, &patterns).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    // The key "server_ip" must remain unchanged (D-12)
    assert!(parsed.get("server_ip").is_some(), "Key must not be tokenized");
}

#[test]
fn nested_json_values_tokenized_recursively() {
    let patterns = DetectionPatterns::new();
    let mut map = TokenMap::new();
    let input = r#"{"conn": {"source": "10.0.0.1", "dest": "10.0.0.2"}}"#;
    let result = tokenize_json_line(input, &mut map, &patterns).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    // Both nested values should be tokenized (order depends on serde_json map iteration)
    let source = parsed["conn"]["source"].as_str().unwrap();
    let dest = parsed["conn"]["dest"].as_str().unwrap();
    assert!(source.starts_with("[IP_") && source.ends_with(']'), "source should be tokenized: {}", source);
    assert!(dest.starts_with("[IP_") && dest.ends_with(']'), "dest should be tokenized: {}", dest);
    assert_ne!(source, dest, "Different IPs should get different tokens");
    assert!(!source.contains("10.0.0"), "Original IP should not appear");
    assert!(!dest.contains("10.0.0"), "Original IP should not appear");
}

#[test]
fn json_array_elements_tokenized() {
    let patterns = DetectionPatterns::new();
    let mut map = TokenMap::new();
    let input = r#"{"hosts": ["db-primary.internal", "cache.internal"]}"#;
    let result = tokenize_json_line(input, &mut map, &patterns).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    let hosts = parsed["hosts"].as_array().unwrap();
    assert_eq!(hosts[0], "[HOST_001]");
    assert_eq!(hosts[1], "[HOST_002]");
}

#[test]
fn json_null_bool_number_unchanged() {
    let patterns = DetectionPatterns::new();
    let mut map = TokenMap::new();
    let input = r#"{"count": 42, "active": true, "extra": null}"#;
    let result = tokenize_json_line(input, &mut map, &patterns).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["count"], 42);
    assert_eq!(parsed["active"], true);
    assert!(parsed["extra"].is_null());
}

#[test]
fn json_string_with_multiple_sensitive_values() {
    let patterns = DetectionPatterns::new();
    let mut map = TokenMap::new();
    let input = r#"{"msg": "Connection from 192.168.1.1 to db-primary.internal"}"#;
    let result = tokenize_json_line(input, &mut map, &patterns).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    let msg = parsed["msg"].as_str().unwrap();
    assert!(msg.contains("[IP_001]"));
    assert!(msg.contains("[HOST_001]"));
    assert!(!msg.contains("192.168.1.1"));
    assert!(!msg.contains("db-primary.internal"));
}

#[test]
fn is_json_line_detects_json_object() {
    assert!(is_json_line(r#"{"key": "value"}"#));
    assert!(is_json_line(r#"  {"key": "value"}"#)); // leading whitespace
}

#[test]
fn is_json_line_detects_json_array() {
    assert!(is_json_line(r#"["a", "b"]"#));
}

#[test]
fn is_json_line_rejects_plain_text() {
    assert!(!is_json_line("2024-01-15 ERROR Connection refused"));
    assert!(!is_json_line("plain text log line"));
}

#[test]
fn json_output_is_valid_parseable_json() {
    let patterns = DetectionPatterns::new();
    let mut map = TokenMap::new();
    let input = r#"{"email": "admin@example.com", "url": "https://api.internal/v1"}"#;
    let result = tokenize_json_line(input, &mut map, &patterns).unwrap();
    // Must parse back successfully
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(parsed.is_object());
}
