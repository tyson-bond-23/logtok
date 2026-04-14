use regex::Regex;

/// A single detection match with its category, matched value, and byte position.
pub struct DetectionMatch {
    pub category: String,
    pub value: String,
    pub start: usize,
    pub end: usize,
}

/// Compiled regex patterns for detecting 7 categories of sensitive data.
/// Pattern order determines priority for overlapping matches.
pub struct DetectionPatterns {
    patterns: Vec<(String, Regex)>,
}

impl DetectionPatterns {
    pub fn new() -> Self {
        // ORDER MATTERS: more specific patterns first to handle overlapping matches
        // EMAIL before IP (emails can contain IP-like segments)
        // URL before HOST (URLs contain hostnames)
        // KEY/PASS before general patterns
        let pattern_defs: Vec<(&str, &str)> = vec![
            ("EMAIL", r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"),
            ("URL", r#"https?://[^\s"\]>]+"#),
            (
                "KEY",
                r#"(?i)(?:api[_-]?key|token|secret|bearer|authorization)\s*[=:]\s*['""]?([a-zA-Z0-9_\-/.+=]{16,})['""]?"#,
            ),
            (
                "PASS",
                r#"(?i)(?:password|passwd|pwd)\s*[=:]\s*['"]?(\S+)['"]?"#,
            ),
            ("IP", r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b"),
            (
                "HOST",
                r"\b[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.(?:internal|local|corp|intra|private|lan)\b",
            ),
            ("PATH", r"(?:/[a-zA-Z0-9._-]+){3,}"),
        ];

        Self {
            patterns: pattern_defs
                .into_iter()
                .map(|(cat, pat)| {
                    (
                        cat.to_string(),
                        Regex::new(pat).expect("Invalid regex pattern"),
                    )
                })
                .collect(),
        }
    }

    /// Find all matches in text, ordered by position, with priority-based overlap resolution.
    /// Returns non-overlapping matches sorted by start position.
    ///
    /// For KEY and PASS patterns, only the captured value (the secret itself) is returned,
    /// not the full `key=value` match. This ensures only the sensitive portion is tokenized.
    pub fn detect(&self, text: &str) -> Vec<DetectionMatch> {
        let mut refined: Vec<DetectionMatch> = Vec::new();

        for (category, regex) in &self.patterns {
            if category == "KEY" || category == "PASS" {
                // For KEY and PASS, extract the captured group (the actual secret value)
                for caps in regex.captures_iter(text) {
                    if let Some(captured) = caps.get(1) {
                        refined.push(DetectionMatch {
                            category: category.clone(),
                            value: captured.as_str().to_string(),
                            start: captured.start(),
                            end: captured.end(),
                        });
                    }
                }
            } else {
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
