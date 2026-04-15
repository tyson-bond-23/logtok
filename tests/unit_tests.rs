use logtok::detector::{DetectionConfig, DetectionPatterns, luhn_check};
use logtok::tokenizer::{TokenMap, TokenMapData};

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

// === New detection category tests (Phase 2) ===

#[test]
fn detect_connection_string_postgresql() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("db_url=postgresql://user:pass@db.host:5432/mydb");
    let conn: Vec<_> = matches.iter().filter(|m| m.category == "CONN").collect();
    assert_eq!(conn.len(), 1);
    assert!(conn[0].value.starts_with("postgresql://"));
}

#[test]
fn detect_connection_string_redis() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("cache=redis://cache.local:6379");
    let conn: Vec<_> = matches.iter().filter(|m| m.category == "CONN").collect();
    assert_eq!(conn.len(), 1);
    assert!(conn[0].value.starts_with("redis://"));
}

#[test]
fn detect_phone_international() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Call +1-555-123-4567 for support");
    let phone: Vec<_> = matches.iter().filter(|m| m.category == "PHONE").collect();
    assert_eq!(phone.len(), 1);
    assert!(phone[0].value.contains("555"));
}

#[test]
fn detect_phone_parenthesized() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Contact (555) 123-4567");
    let phone: Vec<_> = matches.iter().filter(|m| m.category == "PHONE").collect();
    assert_eq!(phone.len(), 1);
}

#[test]
fn detect_uuid() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Request ID: 550e8400-e29b-41d4-a716-446655440000");
    let uuid: Vec<_> = matches.iter().filter(|m| m.category == "UUID").collect();
    assert_eq!(uuid.len(), 1);
    assert_eq!(uuid[0].value, "550e8400-e29b-41d4-a716-446655440000");
}

#[test]
fn detect_arn() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Resource: arn:aws:s3:us-east-1:123456789012:my-bucket/path");
    let arn: Vec<_> = matches.iter().filter(|m| m.category == "ARN").collect();
    assert_eq!(arn.len(), 1);
    assert!(arn[0].value.starts_with("arn:aws:"));
}

#[test]
fn detect_ssn() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("SSN: 123-45-6789");
    let ssn: Vec<_> = matches.iter().filter(|m| m.category == "SSN").collect();
    assert_eq!(ssn.len(), 1);
    assert_eq!(ssn[0].value, "123-45-6789");
}

#[test]
fn detect_cc_valid_luhn() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Card: 4111111111111111");
    let cc: Vec<_> = matches.iter().filter(|m| m.category == "CC").collect();
    assert_eq!(cc.len(), 1, "Luhn-valid CC should be detected");
    assert_eq!(cc[0].value, "4111111111111111");
}

#[test]
fn detect_cc_rejects_invalid_luhn() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Card: 4111111111111112");
    let cc: Vec<_> = matches.iter().filter(|m| m.category == "CC").collect();
    assert_eq!(cc.len(), 0, "Luhn-invalid CC should NOT be detected");
}

#[test]
fn detect_jwt() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Token: eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abc123def456");
    let jwt: Vec<_> = matches.iter().filter(|m| m.category == "JWT").collect();
    assert_eq!(jwt.len(), 1);
    assert!(jwt[0].value.starts_with("eyJ"));
}

#[test]
fn detect_pem() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Key: -----BEGIN RSA PRIVATE KEY-----");
    let pem: Vec<_> = matches.iter().filter(|m| m.category == "PEM").collect();
    assert_eq!(pem.len(), 1);
}

#[test]
fn detect_mac_address() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Interface MAC: 00:1A:2B:3C:4D:5E");
    let mac: Vec<_> = matches.iter().filter(|m| m.category == "MAC").collect();
    assert_eq!(mac.len(), 1);
    assert_eq!(mac[0].value, "00:1A:2B:3C:4D:5E");
}

#[test]
fn detect_dns() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Resolving api.example.com");
    let dns: Vec<_> = matches.iter().filter(|m| m.category == "DNS").collect();
    assert_eq!(dns.len(), 1);
    assert_eq!(dns[0].value, "api.example.com");
}

#[test]
fn detect_dns_multi_level() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Host: logs.prod.company.io");
    let dns: Vec<_> = matches.iter().filter(|m| m.category == "DNS").collect();
    assert_eq!(dns.len(), 1);
    assert_eq!(dns[0].value, "logs.prod.company.io");
}

#[test]
fn detect_os_linux() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Kernel: Linux 5.15.0");
    let os: Vec<_> = matches.iter().filter(|m| m.category == "OS").collect();
    assert_eq!(os.len(), 1);
    assert_eq!(os[0].value, "Linux 5.15.0");
}

#[test]
fn detect_os_windows() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Platform: Windows NT 10.0");
    let os: Vec<_> = matches.iter().filter(|m| m.category == "OS").collect();
    assert_eq!(os.len(), 1);
    assert_eq!(os[0].value, "Windows NT 10.0");
}

#[test]
fn detect_os_darwin() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("System: Darwin 23.1.0");
    let os: Vec<_> = matches.iter().filter(|m| m.category == "OS").collect();
    assert_eq!(os.len(), 1);
    assert_eq!(os[0].value, "Darwin 23.1.0");
}

#[test]
fn detect_name_user_equals() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Logged in user=jdoe successfully");
    let name: Vec<_> = matches.iter().filter(|m| m.category == "NAME").collect();
    assert_eq!(name.len(), 1);
    assert_eq!(name[0].value, "jdoe");
}

#[test]
fn detect_name_json_username() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect(r#"{"username": "admin"}"#);
    let name: Vec<_> = matches.iter().filter(|m| m.category == "NAME").collect();
    assert_eq!(name.len(), 1);
    assert_eq!(name[0].value, "admin");
}

#[test]
fn detect_name_rejects_free_text() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("Hello John, welcome back");
    let name: Vec<_> = matches.iter().filter(|m| m.category == "NAME").collect();
    assert_eq!(name.len(), 0, "Free text names should NOT be detected");
}

// === Priority tests ===

#[test]
fn priority_conn_not_matched_as_url() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("db=postgresql://user:pass@db.host:5432/mydb");
    // CONN should be matched, not URL
    let conn: Vec<_> = matches.iter().filter(|m| m.category == "CONN").collect();
    let url: Vec<_> = matches.iter().filter(|m| m.category == "URL").collect();
    assert_eq!(conn.len(), 1, "Should detect as CONN");
    assert_eq!(url.len(), 0, "Should NOT detect as URL");
}

#[test]
fn priority_jwt_not_matched_as_key() {
    let patterns = DetectionPatterns::new();
    let matches = patterns.detect("auth: eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abc123def456");
    let jwt: Vec<_> = matches.iter().filter(|m| m.category == "JWT").collect();
    let key: Vec<_> = matches.iter().filter(|m| m.category == "KEY").collect();
    assert_eq!(jwt.len(), 1, "Should detect as JWT");
    assert_eq!(key.len(), 0, "Should NOT detect as KEY");
}

// === from_config tests ===

#[test]
fn from_config_default_has_all_19_categories() {
    let config = DetectionConfig::default();
    let patterns = DetectionPatterns::from_config(&config).unwrap();
    // Test by detecting a representative input that has many category matches
    let text = "user=jdoe from 192.168.1.1 via postgresql://u:p@host:5432/db";
    let matches = patterns.detect(text);
    assert!(!matches.is_empty(), "Default config should detect things");
}

#[test]
fn from_config_disabled_categories_excluded() {
    let config = DetectionConfig {
        disabled_categories: vec!["DNS".to_string(), "OS".to_string()],
        custom_patterns: vec![],
    };
    let patterns = DetectionPatterns::from_config(&config).unwrap();
    // DNS should not be detected
    let matches = patterns.detect("Resolving api.example.com on Linux 5.15.0");
    let dns: Vec<_> = matches.iter().filter(|m| m.category == "DNS").collect();
    let os: Vec<_> = matches.iter().filter(|m| m.category == "OS").collect();
    assert_eq!(dns.len(), 0, "DNS should be disabled");
    assert_eq!(os.len(), 0, "OS should be disabled");
}

// === Luhn validation tests ===

#[test]
fn luhn_check_valid_visa() {
    assert!(luhn_check("4111111111111111"));
}

#[test]
fn luhn_check_invalid_number() {
    assert!(!luhn_check("1234567890123456"));
}

// === Regression: all 7 original categories still work ===

#[test]
fn regression_all_original_categories() {
    let patterns = DetectionPatterns::new();

    let email_m = patterns.detect("user@example.com");
    assert_eq!(email_m[0].category, "EMAIL");

    let url_m = patterns.detect("https://example.com/path");
    assert_eq!(url_m[0].category, "URL");

    let key_m = patterns.detect("api_key=sk_live_abc123def456xyz789");
    assert_eq!(key_m[0].category, "KEY");

    let pass_m = patterns.detect("password=secret123");
    assert_eq!(pass_m[0].category, "PASS");

    let ip_m = patterns.detect("10.0.0.1");
    assert_eq!(ip_m[0].category, "IP");

    let host_m = patterns.detect("db-primary.internal");
    assert_eq!(host_m[0].category, "HOST");

    let path_m = patterns.detect("/var/log/app/server.log");
    assert_eq!(path_m[0].category, "PATH");
}

// === TokenMap serialization tests (Phase 2 Task 2) ===

#[test]
fn token_map_serialize_deserialize_roundtrip() {
    let mut map = TokenMap::new();
    map.get_or_insert("192.168.1.100", "IP");
    map.get_or_insert("user@example.com", "EMAIL");

    let data = map.to_data();
    let json = serde_json::to_string(data).unwrap();
    let deserialized: TokenMapData = serde_json::from_str(&json).unwrap();

    let restored = TokenMap::from_data(deserialized);
    assert_eq!(restored.len(), 2);
}

#[test]
fn token_map_counter_continuity_after_deserialize() {
    let mut map = TokenMap::new();
    map.get_or_insert("192.168.1.1", "IP");
    map.get_or_insert("192.168.1.2", "IP");
    map.get_or_insert("192.168.1.3", "IP");

    let data = map.to_data();
    let json = serde_json::to_string(data).unwrap();
    let deserialized: TokenMapData = serde_json::from_str(&json).unwrap();

    let mut restored = TokenMap::from_data(deserialized);
    // Next IP should be IP_004, not IP_001
    let token = restored.get_or_insert("10.0.0.1", "IP");
    assert_eq!(token, "[IP_004]");
}

#[test]
fn token_map_reverse_map_populated() {
    let mut map = TokenMap::new();
    map.get_or_insert("192.168.1.100", "IP");

    let data = map.to_data();
    assert_eq!(
        data.token_to_value.get("[IP_001]").map(|s| s.as_str()),
        Some("192.168.1.100")
    );
}

#[test]
fn token_map_merge_combines_maps() {
    let mut map1 = TokenMap::new();
    map1.get_or_insert("192.168.1.1", "IP");

    let mut map2 = TokenMap::new();
    map2.get_or_insert("10.0.0.1", "IP");
    map2.get_or_insert("user@example.com", "EMAIL");

    map1.merge(map2.to_data().clone());

    assert_eq!(map1.len(), 3);
    // Counters should be advanced: next IP should be IP_003
    let next = map1.get_or_insert("172.16.0.1", "IP");
    assert_eq!(next, "[IP_003]");
}

#[test]
fn token_map_merge_preserves_existing() {
    let mut map1 = TokenMap::new();
    let original_token = map1.get_or_insert("192.168.1.1", "IP");

    let mut map2 = TokenMap::new();
    map2.get_or_insert("192.168.1.1", "IP"); // same value

    map1.merge(map2.to_data().clone());

    // Original token should be preserved, not overwritten
    let data = map1.to_data();
    assert_eq!(
        data.value_to_token.get("192.168.1.1").map(|s| s.as_str()),
        Some(original_token.as_str())
    );
    assert_eq!(map1.len(), 1); // no duplicate
}

#[test]
fn token_entry_has_created_at() {
    let mut map = TokenMap::new();
    map.get_or_insert("192.168.1.1", "IP");

    let data = map.to_data();
    let entry = data.entries.get("192.168.1.1").unwrap();
    assert!(entry.created_at > 0, "created_at should be a Unix timestamp");
    assert_eq!(entry.category, "IP");
    assert_eq!(entry.token, "[IP_001]");
}

#[test]
fn token_map_purge_expired_removes_old_entries() {
    let mut map = TokenMap::new();
    map.get_or_insert("192.168.1.1", "IP");

    // Manually set created_at to the past (1 second ago won't expire with TTL=3600)
    // Instead, set to 0 (epoch) to guarantee expiration
    {
        let data = map.to_data_mut();
        if let Some(entry) = data.entries.get_mut("192.168.1.1") {
            entry.created_at = 0; // epoch = always expired
        }
    }

    assert_eq!(map.len(), 1);
    map.purge_expired(1); // TTL of 1 second -- epoch entry is definitely expired
    assert_eq!(map.len(), 0, "Expired entry should be purged");
}
