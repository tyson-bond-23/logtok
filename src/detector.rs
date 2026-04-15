use regex::Regex;

use crate::error::TokeniserError;

/// A single detection match with its category, matched value, and byte position.
pub struct DetectionMatch {
    pub category: String,
    pub value: String,
    pub start: usize,
    pub end: usize,
}

/// How a pattern match is extracted from text.
#[derive(Clone, Debug)]
pub enum MatchMode {
    /// Use `find_iter` -- full match is the value
    Full,
    /// Use `captures_iter` -- extract the given capture group
    CaptureGroup(usize),
    /// Use `find_iter` then post-validate with Luhn algorithm
    LuhnValidated,
}

/// Configuration for detection: which categories are disabled and any custom patterns.
pub struct DetectionConfig {
    pub disabled_categories: Vec<String>,
    pub custom_patterns: Vec<CustomPatternDef>,
}

/// A user-defined custom detection pattern.
pub struct CustomPatternDef {
    pub name: String,
    pub pattern: String,
    pub capture_group: Option<usize>,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            disabled_categories: vec![],
            custom_patterns: vec![],
        }
    }
}

/// Returns all 19 built-in pattern definitions as (category, regex_str, MatchMode).
/// Order determines priority for overlapping matches -- more specific patterns first.
pub fn builtin_patterns() -> Vec<(&'static str, &'static str, MatchMode)> {
    vec![
        // 1. PEM -- very specific header
        ("PEM", r"-----BEGIN [A-Z ]*PRIVATE KEY-----", MatchMode::Full),
        // 2. JWT -- three base64url segments
        (
            "JWT",
            r"eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+",
            MatchMode::Full,
        ),
        // 3. CONN -- database/message broker connection strings (before URL to prevent URL match)
        (
            "CONN",
            r#"(?:postgresql|postgres|mysql|mongodb|mongodb\+srv|redis|rediss|amqp|amqps|kafka)://[^\s"'>]+"#,
            MatchMode::Full,
        ),
        // 4. EMAIL
        (
            "EMAIL",
            r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
            MatchMode::Full,
        ),
        // 5. URL
        ("URL", r#"https?://[^\s"\]>]+"#, MatchMode::Full),
        // 6. KEY (capture group 1 -- the secret value)
        (
            "KEY",
            r#"(?i)(?:api[_-]?key|token|secret|bearer|authorization)\s*[=:]\s*['"]?([a-zA-Z0-9_\-/.+=]{16,})['"]?"#,
            MatchMode::CaptureGroup(1),
        ),
        // 7. PASS (capture group 1 -- the password value)
        (
            "PASS",
            r#"(?i)(?:password|passwd|pwd)\s*[=:]\s*['"]?(\S+)['"]?"#,
            MatchMode::CaptureGroup(1),
        ),
        // 8. CC -- credit card with Luhn post-validation
        (
            "CC",
            r"\b(?:4\d{3}|5[1-5]\d{2}|3[47]\d{2}|6011)[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b",
            MatchMode::LuhnValidated,
        ),
        // 9. SSN
        ("SSN", r"\b\d{3}-\d{2}-\d{4}\b", MatchMode::Full),
        // 10. ARN
        (
            "ARN",
            r#"arn:aws:[a-zA-Z0-9-]+:[a-z0-9-]*:\d{12}:[^\s"]+"#,
            MatchMode::Full,
        ),
        // 11. NAME (capture group 1 -- structured user/name patterns only)
        // Handles: user=jdoe, username: admin, "username": "admin", name='bob'
        (
            "NAME",
            r#"(?i)(?:user(?:name)?|author|name)['"]?\s*[=:]\s*['"]?([a-zA-Z][a-zA-Z0-9._-]{1,30})['"]?"#,
            MatchMode::CaptureGroup(1),
        ),
        // 12. UUID
        (
            "UUID",
            r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
            MatchMode::Full,
        ),
        // 13. MAC
        (
            "MAC",
            r"(?:[0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}",
            MatchMode::Full,
        ),
        // 14. PHONE
        (
            "PHONE",
            r"(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}",
            MatchMode::Full,
        ),
        // 15. IP
        (
            "IP",
            r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b",
            MatchMode::Full,
        ),
        // 16. HOST -- internal hostnames only (per D-04)
        (
            "HOST",
            r"\b[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.(?:internal|local|corp|intra|private|lan)\b",
            MatchMode::Full,
        ),
        // 17. DNS -- broader domain matching
        (
            "DNS",
            r"\b[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+\.[a-zA-Z]{2,}\b",
            MatchMode::Full,
        ),
        // 18. OS
        (
            "OS",
            r"(?:Linux|Darwin|Windows NT|Ubuntu|CentOS|Red Hat|Debian|Alpine)\s+[\d.]+",
            MatchMode::Full,
        ),
        // 19. PATH
        ("PATH", r"(?:/[a-zA-Z0-9._-]+){3,}", MatchMode::Full),
    ]
}

/// Validates a number string using the Luhn algorithm.
/// Returns true if the number passes the Luhn check (valid credit card number).
pub fn luhn_check(number: &str) -> bool {
    let digits: Vec<u32> = number
        .chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c.to_digit(10).unwrap())
        .collect();
    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }
    let sum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &d)| {
            if i % 2 == 1 {
                let doubled = d * 2;
                if doubled > 9 {
                    doubled - 9
                } else {
                    doubled
                }
            } else {
                d
            }
        })
        .sum();
    sum % 10 == 0
}

/// Compiled regex patterns for detecting 19 categories of sensitive data.
/// Pattern order determines priority for overlapping matches.
pub struct DetectionPatterns {
    patterns: Vec<(String, Regex, MatchMode)>,
}

impl DetectionPatterns {
    /// Create with all 19 built-in patterns (backward compatible).
    pub fn new() -> Self {
        Self::from_config(&DetectionConfig::default())
            .expect("Built-in patterns must compile")
    }

    /// Create from configuration, filtering disabled categories and adding custom patterns.
    pub fn from_config(config: &DetectionConfig) -> Result<Self, TokeniserError> {
        let mut compiled: Vec<(String, Regex, MatchMode)> = Vec::new();

        for (cat, pat, mode) in builtin_patterns() {
            if config
                .disabled_categories
                .iter()
                .any(|d| d.eq_ignore_ascii_case(cat))
            {
                continue;
            }
            let regex = Regex::new(pat).map_err(|e| TokeniserError::PatternCompileError {
                pattern: pat.to_string(),
                source: e,
            })?;
            compiled.push((cat.to_string(), regex, mode));
        }

        // Append custom patterns
        for custom in &config.custom_patterns {
            let mode = match custom.capture_group {
                Some(g) => MatchMode::CaptureGroup(g),
                None => MatchMode::Full,
            };
            let regex =
                Regex::new(&custom.pattern).map_err(|e| TokeniserError::PatternCompileError {
                    pattern: custom.pattern.clone(),
                    source: e,
                })?;
            compiled.push((custom.name.clone(), regex, mode));
        }

        Ok(Self { patterns: compiled })
    }

    /// Find all matches in text, ordered by position, with priority-based overlap resolution.
    /// Returns non-overlapping matches sorted by start position.
    ///
    /// For KEY, PASS, and NAME patterns, only the captured value is returned,
    /// not the full `key=value` match. This ensures only the sensitive portion is tokenized.
    /// For CC, matches are post-validated with the Luhn algorithm.
    pub fn detect(&self, text: &str) -> Vec<DetectionMatch> {
        let mut refined: Vec<DetectionMatch> = Vec::new();

        for (category, regex, mode) in &self.patterns {
            match mode {
                MatchMode::CaptureGroup(group) => {
                    for caps in regex.captures_iter(text) {
                        if let Some(captured) = caps.get(*group) {
                            refined.push(DetectionMatch {
                                category: category.clone(),
                                value: captured.as_str().to_string(),
                                start: captured.start(),
                                end: captured.end(),
                            });
                        }
                    }
                }
                MatchMode::LuhnValidated => {
                    for m in regex.find_iter(text) {
                        if luhn_check(m.as_str()) {
                            refined.push(DetectionMatch {
                                category: category.clone(),
                                value: m.as_str().to_string(),
                                start: m.start(),
                                end: m.end(),
                            });
                        }
                    }
                }
                MatchMode::Full => {
                    for m in regex.find_iter(text) {
                        refined.push(DetectionMatch {
                            category: category.clone(),
                            value: m.as_str().to_string(),
                            start: m.start(),
                            end: m.end(),
                        });
                    }
                }
            }
        }

        // Sort by start position; ties broken by pattern order (earlier = higher priority)
        refined.sort_by_key(|m| m.start);

        // Remove overlapping matches: keep the first (highest priority due to pattern order)
        let mut result: Vec<DetectionMatch> = Vec::new();
        let mut last_end = 0;
        for m in refined {
            if m.start >= last_end {
                last_end = m.end;
                result.push(m);
            }
        }

        result
    }
}

impl Default for DetectionPatterns {
    fn default() -> Self {
        Self::new()
    }
}
