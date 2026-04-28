use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::detector::DetectionPatterns;

/// A single token entry with metadata for persistence and TTL.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenEntry {
    pub token: String,
    pub category: String,
    pub created_at: u64, // Unix timestamp seconds
}

/// Serializable data backing the TokenMap. Can be persisted to/from JSON for encrypted storage.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct TokenMapData {
    pub value_to_token: HashMap<String, String>,
    pub token_to_value: HashMap<String, String>,
    pub category_counters: HashMap<String, u32>,
    pub entries: HashMap<String, TokenEntry>, // keyed by original value
}

/// Deterministic token map with per-category counters.
/// The same input value always maps to the same token (TOK-01).
/// Tokens follow the format [CATEGORY_NNN] per D-01 through D-05.
pub struct TokenMap {
    data: TokenMapData,
}

impl TokenMap {
    pub fn new() -> Self {
        Self {
            data: TokenMapData::default(),
        }
    }

    /// Construct a TokenMap from deserialized data (for loading from store).
    pub fn from_data(data: TokenMapData) -> Self {
        Self { data }
    }

    /// Access the underlying serializable data (for saving to store).
    pub fn to_data(&self) -> &TokenMapData {
        &self.data
    }

    /// Mutable access to the underlying data (for testing and internal adjustments).
    pub fn to_data_mut(&mut self) -> &mut TokenMapData {
        &mut self.data
    }

    /// Get existing token or create new one. Format: [CATEGORY_NNN] per D-01/D-02/D-03/D-04.
    /// Counter overflows past 999 gracefully to 4+ digits.
    /// Populates both forward and reverse maps, and creates a TokenEntry with timestamp.
    pub fn get_or_insert(&mut self, value: &str, category: &str) -> String {
        if let Some(token) = self.data.value_to_token.get(value) {
            return token.clone();
        }
        let counter = self
            .data
            .category_counters
            .entry(category.to_string())
            .or_insert(0);
        *counter += 1;
        let token = format!("[{}_{:03}]", category, counter);

        self.data
            .value_to_token
            .insert(value.to_string(), token.clone());
        self.data
            .token_to_value
            .insert(token.clone(), value.to_string());

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.data.entries.insert(
            value.to_string(),
            TokenEntry {
                token: token.clone(),
                category: category.to_string(),
                created_at: now,
            },
        );

        token
    }

    /// Merge another TokenMapData into this map.
    /// Existing tokens are preserved; new tokens are added.
    /// Counters are advanced to the maximum seen value.
    pub fn merge(&mut self, other: TokenMapData) {
        for (value, entry) in other.entries {
            if !self.data.value_to_token.contains_key(&value) {
                self.data
                    .value_to_token
                    .insert(value.clone(), entry.token.clone());
                self.data
                    .token_to_value
                    .insert(entry.token.clone(), value.clone());
                self.data.entries.insert(value, entry.clone());
            }
            // Always update counter to max seen
            let counter = self
                .data
                .category_counters
                .entry(entry.category.clone())
                .or_insert(0);
            if let Some(num) = parse_counter_from_token(&entry.token) {
                if num > *counter {
                    *counter = num;
                }
            }
        }
    }

    /// Remove entries older than `ttl_seconds` from both maps.
    pub fn purge_expired(&mut self, ttl_seconds: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expired_values: Vec<String> = self
            .data
            .entries
            .iter()
            .filter(|(_, entry)| {
                now.checked_sub(entry.created_at)
                    .map_or(false, |age| age > ttl_seconds)
            })
            .map(|(value, _)| value.clone())
            .collect();
        for value in expired_values {
            if let Some(entry) = self.data.entries.remove(&value) {
                self.data.value_to_token.remove(&value);
                self.data.token_to_value.remove(&entry.token);
            }
        }
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
        self.data.value_to_token.len()
    }

    /// Check if no tokens have been created
    pub fn is_empty(&self) -> bool {
        self.data.value_to_token.is_empty()
    }
}

impl Default for TokenMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse the numeric counter from a token string like "[IP_003]".
/// Returns Some(3) for "[IP_003]", None if format doesn't match.
fn parse_counter_from_token(token: &str) -> Option<u32> {
    // Token format: [CATEGORY_NNN]
    let inner = token.trim_start_matches('[').trim_end_matches(']');
    let parts: Vec<&str> = inner.rsplitn(2, '_').collect();
    if parts.len() == 2 {
        parts[0].parse().ok()
    } else {
        None
    }
}
