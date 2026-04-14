use serde_json::Value;

use crate::detector::DetectionPatterns;
use crate::tokenizer::TokenMap;

/// Recursively tokenize all string values in a JSON Value tree.
/// Keys are never tokenized (per D-12). Output remains valid JSON (per D-11).
fn tokenize_json_value(value: &mut Value, token_map: &mut TokenMap, patterns: &DetectionPatterns) {
    match value {
        Value::String(s) => {
            let tokenized = token_map.tokenize_line(s, patterns);
            *s = tokenized;
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                tokenize_json_value(item, token_map, patterns);
            }
        }
        Value::Object(map) => {
            for (_key, val) in map.iter_mut() {
                tokenize_json_value(val, token_map, patterns);
            }
        }
        _ => {} // Null, Bool, Number -- leave unchanged
    }
}

/// Tokenize a single JSON line. Parses as JSON, tokenizes string values,
/// re-serializes to a compact JSON string.
/// Returns Ok(tokenized_json_string) or Err if not valid JSON.
pub fn tokenize_json_line(
    line: &str,
    token_map: &mut TokenMap,
    patterns: &DetectionPatterns,
) -> Result<String, serde_json::Error> {
    let mut value: Value = serde_json::from_str(line)?;
    tokenize_json_value(&mut value, token_map, patterns);
    serde_json::to_string(&value)
}

/// Detect whether a line is likely JSON (starts with '{' or '[' after trimming whitespace).
pub fn is_json_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}
