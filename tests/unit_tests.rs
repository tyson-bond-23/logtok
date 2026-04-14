use logtok::detector::DetectionPatterns;
use logtok::tokenizer::TokenMap;

// === TokenMap determinism tests ===

#[test]
fn token_map_first_ip_gets_001() {
    let mut map = TokenMap::new();
    let token = map.get_or_insert("192.168.1.1", "IP");
    assert_eq!(token, "[IP_001]");
}

#[test]
fn token_map_second_different_ip_gets_002() {
    let mut map = TokenMap::new();
    map.get_or_insert("192.168.1.1", "IP");
    let token = map.get_or_insert("10.0.0.1", "IP");
    assert_eq!(token, "[IP_002]");
}

#[test]
fn token_map_same_ip_returns_same_token() {
    let mut map = TokenMap::new();
    let first = map.get_or_insert("192.168.1.1", "IP");
    let second = map.get_or_insert("192.168.1.1", "IP");
    assert_eq!(first, second);
    assert_eq!(first, "[IP_001]");
}

#[test]
fn token_map_independent_category_counters() {
    let mut map = TokenMap::new();
    let ip_token = map.get_or_insert("192.168.1.1", "IP");
    let email_token = map.get_or_insert("user@example.com", "EMAIL");
    assert_eq!(ip_token, "[IP_001]");
    assert_eq!(email_token, "[EMAIL_001]");
}

#[test]
fn token_map_counter_overflow_past_999() {
    let mut map = TokenMap::new();
    for i in 1..=1000 {
        let value = format!("192.168.1.{}", i);
        let token = map.get_or_insert(&value, "IP");
        if i == 999 {
            assert_eq!(token, "[IP_999]");
        }
        if i == 1000 {
            assert_eq!(token, "[IP_1000]");
        }
    }
}

// === DetectionPatterns detection tests ===

#[test]
fn detect_ip_address() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Connection from 192.168.1.1 established");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].category, "IP");
    assert_eq!(matches[0].value, "192.168.1.1");
}

#[test]
fn detect_email() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("User user@example.com logged in");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].category, "EMAIL");
    assert_eq!(matches[0].value, "user@example.com");
}

#[test]
fn detect_url() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Calling https://internal.corp.com/api/v2/users endpoint");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].category, "URL");
    assert_eq!(matches[0].value, "https://internal.corp.com/api/v2/users");
}

#[test]
fn detect_api_key() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Config: api_key=sk_live_abc123def456xyz789");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].category, "KEY");
    assert_eq!(matches[0].value, "sk_live_abc123def456xyz789");
}

#[test]
fn detect_password() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Login attempt password=mysecret123 failed");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].category, "PASS");
    assert_eq!(matches[0].value, "mysecret123");
}

#[test]
fn detect_internal_hostname() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Connected to db-primary.internal on port 5432");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].category, "HOST");
    assert_eq!(matches[0].value, "db-primary.internal");
}

#[test]
fn detect_file_path() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Error reading /var/log/app/server.log");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].category, "PATH");
    assert_eq!(matches[0].value, "/var/log/app/server.log");
}

// === Line tokenization tests ===

#[test]
fn tokenize_line_replaces_all_sensitive_values() {
    let patterns = DetectionPatterns::new();
    let mut map = TokenMap::new();
    let line = "Connection from 192.168.1.1 to db-primary.internal by user@example.com";
    let result = map.tokenize_line(line, &patterns);
    assert!(result.contains("[IP_001]"));
    assert!(result.contains("[HOST_001]"));
    assert!(result.contains("[EMAIL_001]"));
    assert!(!result.contains("192.168.1.1"));
    assert!(!result.contains("db-primary.internal"));
    assert!(!result.contains("user@example.com"));
}

#[test]
fn tokenize_line_deterministic_across_calls() {
    let patterns = DetectionPatterns::new();
    let mut map = TokenMap::new();
    let line1 = "Connection from 192.168.1.1";
    let line2 = "Another connection from 192.168.1.1";
    let result1 = map.tokenize_line(line1, &patterns);
    let result2 = map.tokenize_line(line2, &patterns);
    // Same IP should get the same token in both lines
    assert!(result1.contains("[IP_001]"));
    assert!(result2.contains("[IP_001]"));
}

#[test]
fn overlap_priority_url_before_host() {
    // URL pattern should consume the full URL, preventing HOST from matching the hostname inside it.
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Calling https://api-server.internal/v2/users");
    // URL should be detected, and HOST should NOT be separately detected within the URL
    assert_eq!(matches.len(), 1, "Only URL match, no separate HOST");
    assert_eq!(matches[0].category, "URL");
}

#[test]
fn overlap_email_and_ip_separate() {
    // When email and IP appear at different positions, both should be detected
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("User admin@example.com from 10.0.0.1");
    let categories: Vec<&str> = matches.iter().map(|m| m.category.as_str()).collect();
    assert!(categories.contains(&"EMAIL"), "Should detect email");
    assert!(categories.contains(&"IP"), "Should detect IP");
}

#[test]
fn tokens_not_re_tokenized() {
    let patterns = DetectionPatterns::new();
    let mut map = TokenMap::new();
    // First tokenization
    let line = "Server 192.168.1.1 responded";
    let result1 = map.tokenize_line(line, &patterns);
    assert!(result1.contains("[IP_001]"));

    // Tokenizing the result again should NOT create new tokens for [IP_001]
    let result2 = map.tokenize_line(&result1, &patterns);
    assert_eq!(result1, result2, "Already-tokenized output should not be re-tokenized");
}
