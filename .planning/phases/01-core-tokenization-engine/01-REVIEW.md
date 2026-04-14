---
phase: 01-core-tokenization-engine
reviewed: 2026-04-14T00:00:00Z
depth: standard
files_reviewed: 15
files_reviewed_list:
  - Cargo.toml
  - src/cli.rs
  - src/compactor.rs
  - src/detector.rs
  - src/error.rs
  - src/json_processor.rs
  - src/lib.rs
  - src/main.rs
  - src/processor.rs
  - src/tokenizer.rs
  - tests/cli_tests.rs
  - tests/compactor_tests.rs
  - tests/integration_tests.rs
  - tests/json_tests.rs
  - tests/unit_tests.rs
findings:
  critical: 0
  warning: 3
  info: 3
  total: 6
status: issues_found
---

# Phase 1: Code Review Report

**Reviewed:** 2026-04-14
**Depth:** standard
**Files Reviewed:** 15
**Status:** issues_found

## Summary

The core tokenization engine is well-structured with clean module boundaries, deterministic token assignment, and solid test coverage across unit, integration, and CLI levels. The code follows Rust idioms correctly, uses appropriate error handling with `anyhow`/`thiserror`, and the processing pipeline (detect -> tokenize -> compact -> output) is sound.

No critical security issues were found. The key concerns are: (1) the IP address regex over-matches invalid addresses, (2) cross-platform byte counting is inaccurate on Windows due to CRLF handling, and (3) the overlap resolution for detection matches at identical start positions may not respect intended pattern priority.

## Warnings

### WR-01: IP regex matches invalid addresses (false positives)

**File:** `src/detector.rs:34`
**Issue:** The IP pattern `\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b` will match syntactically invalid IP addresses like `999.999.999.999` or version strings like `1.22.333.4444`. In log files, version numbers (e.g., software versions) commonly appear in dot-separated numeric format and would be incorrectly tokenized as IP addresses, corrupting non-sensitive data.
**Fix:** Either validate octets are 0-255 in the regex, or add a post-match validation step:
```rust
("IP", r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b"),
```

### WR-02: Block byte tracking inaccurate on Windows (CRLF)

**File:** `src/processor.rs:69`
**Issue:** `line.len() + 1` assumes Unix-style `\n` line endings. On Windows, files commonly use `\r\n` (2 bytes). `BufReader::lines()` strips both `\r\n` and `\n`, so the actual bytes consumed per line may be `line.len() + 2`. This means `block_bytes` underestimates bytes read from CRLF files, causing: (1) blocks to be larger than the configured `block_size`, and (2) the progress bar to not reach 100% at completion (the total is `file_size` from metadata, but accumulated `block_bytes` will be less).
**Fix:** Read the raw byte length before stripping, or account for platform line endings:
```rust
// After the progress bar finishes the final block, call pb.set_position(file_size)
// to ensure it reaches 100%, regardless of line-ending byte counting.
if let Some(ref pb) = progress {
    pb.set_position(file_size);
}
```
For block size accuracy, consider using `BufRead::read_line()` instead of `lines()`, which preserves the line terminator and gives accurate byte counts.

### WR-03: Detection overlap resolution is fragile at identical start positions

**File:** `src/detector.rs:88-99`
**Issue:** The comment on line 88 says "ties broken by pattern order (earlier = higher priority)" but this relies on `sort_by_key` being stable (it is in Rust) AND on matches being pushed in pattern-definition order. For KEY and PASS patterns, matches are pushed based on capture group 1 position (line 72: `captured.start()`), not the full regex match position. If a KEY's captured value starts at the same byte offset as another pattern's match, the priority depends on insertion order during iteration. For example, if a line contains `token=admin@example.com`, the KEY pattern would match with capture group starting at `admin@...`, and the EMAIL pattern would also match at the same position. The KEY match would be inserted first (earlier in pattern list), so it wins -- but the KEY's full match spans `token=admin@example.com` while only the capture `admin@example.com` is recorded. The EMAIL match at the same start is correctly suppressed, but the `last_end` is set to the EMAIL-length portion, potentially allowing later overlapping matches to slip through.
**Fix:** For KEY/PASS, use the full match span for overlap resolution while still only tokenizing the captured group:
```rust
// In the KEY/PASS branch, track full match bounds for overlap resolution
if let Some(captured) = caps.get(1) {
    let full_match = caps.get(0).unwrap();
    refined.push(DetectionMatch {
        category: category.clone(),
        value: captured.as_str().to_string(),
        start: captured.start(),
        end: full_match.end(), // Use full match end for overlap exclusion
    });
}
```

## Info

### IN-01: Redundant character in KEY regex character class

**File:** `src/detector.rs:28`
**Issue:** The regex `['""]?` contains a duplicated double-quote character in the character class. The bytes confirm two identical `0x22` characters. This is functionally harmless (regex engines deduplicate) but suggests a possible intent to match smart/curly quotes that was not implemented.
**Fix:** Remove the duplicate: `['"]?`

### IN-02: `#[allow(dead_code)]` on entire error enum

**File:** `src/error.rs:5-6`
**Issue:** The `#[allow(dead_code)]` attribute on `TokeniserError` suppresses warnings for all variants. Currently, `FileNotFound`, `FileReadError`, and `JsonParseError` appear unused. If these are planned for future use, consider adding a TODO comment. If not, remove the unused variants to keep the API surface clean.
**Fix:** Either remove the attribute and unused variants, or add `#[allow(dead_code)]` only to specific unused variants with a comment explaining they are planned:
```rust
#[derive(Error, Debug)]
pub enum TokeniserError {
    // Used
    #[error("Invalid block size: {size} (must be between 1024 and 104857600)")]
    InvalidBlockSize { size: usize },

    #[error("Write error: {0}")]
    WriteError(#[from] std::io::Error),

    // Planned for structured error handling in future phases
    #[allow(dead_code)]
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },
    // ...
}
```

### IN-03: PATH pattern misses short sensitive paths

**File:** `src/detector.rs:39`
**Issue:** The PATH regex `(?:/[a-zA-Z0-9._-]+){3,}` requires 3+ path segments, so paths like `/etc/passwd`, `/etc/shadow`, or `/tmp/secrets` (2 segments) are not detected. These are commonly sensitive file paths in log output.
**Fix:** Consider reducing the minimum to 2 segments, or adding a separate pattern for known sensitive short paths:
```rust
("PATH", r"(?:/[a-zA-Z0-9._-]+){2,}"),
```
Note: Lowering to 2 may increase false positives on strings like `/api/users` in URLs that are already captured by the URL pattern. The overlap resolution should handle this, but test coverage for this edge case would be prudent.

---

_Reviewed: 2026-04-14_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
