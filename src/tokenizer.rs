use std::collections::HashMap;

use crate::detector::DetectionPatterns;

/// Deterministic token map with per-category counters.
/// The same input value always maps to the same token (TOK-01).
/// Tokens follow the format [CATEGORY_NNN] per D-01 through D-05.
pub struct TokenMap {
    value_to_token: HashMap<String, String>,
    category_counters: HashMap<String, u32>,
}

impl TokenMap {
    pub fn new() -> Self {
        Self {
            value_to_token: HashMap::new(),
            category_counters: HashMap::new(),
        }
    }

    /// Get existing token or create new one. Format: [CATEGORY_NNN] per D-01/D-02/D-03/D-04.
    /// Counter overflows past 999 gracefully to 4+ digits.
    pub fn get_or_insert(&mut self, value: &str, category: &str) -> String {
        if let Some(token) = self.value_to_token.get(value) {
            return token.clone();
        }
        let counter = self
            .category_counters
            .entry(category.to_string())
            .or_insert(0);
        *counter += 1;
        let token = format!("[{}_{:03}]", category, counter);
        self.value_to_token.insert(value.to_string(), token.clone());
        token
    }

    /// Tokenize a plain text line: detect sensitive values and replace with tokens.
    /// Replacements applied right-to-left to preserve byte positions.
    pub fn tokenize_line(&mut self, line: &str, patterns: &DetectionPatterns) -> String {
        let matches = patterns.detect(line);

        if matches.is_empty() {
            return line.to_string();
        }

        // Build result by replacing matches right-to-left to preserve byte offsets
        let mut result = line.to_string();
        for m in matches.iter().rev() {
            let token = self.get_or_insert(&m.value, &m.category);
            result.replace_range(m.start..m.end, &token);
        }

        result
    }

    /// Returns the number of unique tokens created
    pub fn len(&self) -> usize {
        self.value_to_token.len()
    }

    /// Check if no tokens have been created
    pub fn is_empty(&self) -> bool {
        self.value_to_token.is_empty()
    }
}

impl Default for TokenMap {
    fn default() -> Self {
        Self::new()
    }
}
