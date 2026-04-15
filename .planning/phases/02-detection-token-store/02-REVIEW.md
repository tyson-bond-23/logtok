---
phase: 02-detection-token-store
reviewed: 2026-04-15T00:00:00Z
depth: standard
files_reviewed: 14
files_reviewed_list:
  - Cargo.toml
  - src/cli.rs
  - src/config.rs
  - src/detector.rs
  - src/error.rs
  - src/lib.rs
  - src/main.rs
  - src/processor.rs
  - src/store.rs
  - src/tokenizer.rs
  - tests/config_tests.rs
  - tests/integration_tests.rs
  - tests/store_tests.rs
  - tests/unit_tests.rs
findings:
  critical: 1
  warning: 7
  info: 3
  total: 11
status: issues_found
---

# Phase 02: Code Review Report

**Reviewed:** 2026-04-15
**Depth:** standard
**Files Reviewed:** 14
**Status:** issues_found

## Summary

Phase 2 adds 19 detection categories, an encrypted token store (AES-256-GCM + Argon2id), config-driven detection, TTL-based expiry, dry-run mode, and block processing. The overall architecture is solid: the encryption design is correct, the Luhn post-validation for CC is appropriate, and the atomic save (temp-file + rename) is well-implemented.

The primary concern is a UTF-8 boundary panic in dry-run output that will crash on any log file containing non-ASCII characters. There are also two `unwrap()` calls on system clock arithmetic that can panic on exotic system configurations, a silent salt rotation on re-save of a corrupted store, and a dead parameter pair in `ensure_gitignore` that indicates an incomplete refactor. The test suite uses `unsafe` env-var mutation with a per-test mutex that does not actually prevent data races with other test threads.

---

## Critical Issues

### CR-01: UTF-8 Byte-Boundary Panic in `truncate_example`

**File:** `src/processor.rs:270`
**Issue:** `&s[..max_len]` indexes into a `&str` by byte offset, not character boundary. If `max_len` falls inside a multi-byte UTF-8 sequence (any log line with non-ASCII characters — accented names, CJK paths, emoji in log messages) the process panics with `byte index N is not a char boundary`. This is a runtime crash in dry-run mode whenever real-world non-ASCII log data is analysed.

**Fix:**
```rust
fn truncate_example(s: &str, max_len: usize) -> String {
    // Use char-aware truncation to avoid splitting multi-byte sequences
    let mut boundary = 0;
    for (i, _) in s.char_indices() {
        if i >= max_len {
            break;
        }
        boundary = i;
    }
    // Advance boundary past the last included char
    if s.len() <= max_len {
        s.to_string()
    } else {
        // Find the end of the last included character
        let cutoff = s
            .char_indices()
            .take_while(|(i, _)| *i < max_len)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        format!("{}...", &s[..cutoff])
    }
}
```

---

## Warnings

### WR-01: Integer Underflow Panic in `purge_expired`

**File:** `src/tokenizer.rs:130`
**Issue:** `now - entry.created_at` is unsigned (`u64`) subtraction. If `entry.created_at > now` (possible with clock skew, NTP adjustment, or a manually set clock), this wraps to a huge number in release builds (making entries appear immortal) or panics in debug builds. The test in `unit_tests.rs:565` manually sets `created_at = 0` which works, but the production path is unguarded.

**Fix:**
```rust
.filter(|(_, entry)| now.saturating_sub(entry.created_at) > ttl_seconds)
```

### WR-02: `unwrap()` on System Clock Can Panic

**File:** `src/tokenizer.rs:77-79` and `src/tokenizer.rs:123-125`
**Issue:** Both `get_or_insert` and `purge_expired` call `SystemTime::now().duration_since(UNIX_EPOCH).unwrap()`. This panics if the system clock is set before January 1, 1970 (possible on misconfigured VMs, containers with broken time, or embedded environments). It is also the same root as WR-01 — both callers of `duration_since` should handle the error consistently.

**Fix:**
```rust
let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()  // 0 seconds is a safe fallback for token timestamps
    .as_secs();
```

### WR-03: Silent Salt Rotation on Corrupted Store Re-Save

**File:** `src/store.rs:119-128`
**Issue:** In `save()`, when the existing store file is shorter than `5 + SALT_LEN` bytes (corrupt partial write, truncated file, or version mismatch), a new random salt is silently generated. On the next `load()`, the newly derived key will not match the key used to save any previous data. The old tokens are silently lost with no error or warning. This is a data-loss path that is easy to trigger during development (e.g., a failed previous save leaves a partial file).

**Fix:** Fail explicitly when the existing file exists but is malformed:
```rust
} else {
    return Err(TokeniserError::StoreError {
        message: format!(
            "Store file exists but is too short to read salt ({} bytes). \
             Run --reset-store to start fresh.",
            existing.len()
        ),
    });
}
```

### WR-04: `--reset-store` Requires `LOGTOK_KEY` to Be Set

**File:** `src/main.rs:30-34`
**Issue:** `--reset-store` calls `store::Store::new()`, which immediately fails if `LOGTOK_KEY` is not set. The most common reason to reset the store is precisely that you no longer have (or want to use) the old key. An operator trying to recover from a lost key cannot reset the store without also setting the environment variable.

**Fix:** Handle the reset path without going through `Store::new()`. The store is just a file; deletion does not require the passphrase:
```rust
if cli.reset_store {
    let store_path = std::env::current_dir()?.join(".logtok").join("store.enc");
    if store_path.exists() {
        std::fs::remove_file(&store_path)
            .with_context(|| format!("Cannot delete store file: {}", store_path.display()))?;
    }
    eprintln!("logtok: token store reset");
    return Ok(());
}
```

### WR-05: `ensure_gitignore` Ignores Its Own Parameters — Wrong Directory Risk

**File:** `src/processor.rs:185-193`
**Issue:** Both parameters `_input_path` and `_store` are dead (prefixed `_`). The function hardcodes `std::env::current_dir()?.join(".logtok")` as the target directory. The `Store` already knows its own path (`self.path`), so the gitignore should be placed next to the store file. If the store was created in a directory other than CWD (e.g., a project with a different working directory), the gitignore is written to the wrong location — the actual store directory remains unprotected.

**Fix:** Expose the store directory from `Store` and use it, or pass the store path:
```rust
// In Store: add a method
pub fn dir(&self) -> &Path {
    self.path.parent().unwrap_or(Path::new("."))
}

// In ensure_gitignore:
fn ensure_gitignore(store: &Store) -> Result<()> {
    let store_dir = store.dir();
    let gitignore_path = store_dir.join(".gitignore");
    if !gitignore_path.exists() {
        fs::create_dir_all(store_dir)?;
        fs::write(&gitignore_path, "*\n")?;
    }
    Ok(())
}
```

### WR-06: Overlap Resolution Priority Is Incorrect for Same-Start Ties

**File:** `src/detector.rs:280-293`
**Issue:** `refined` is built by iterating patterns in priority order (index 0 = highest priority), so for two patterns matching at the same `start` position, the higher-priority pattern is pushed first. After `sort_by_key(|m| m.start)`, Rust's sort is stable, so equal-start entries retain their relative order — meaning the higher-priority entry still appears first. The overlap filter then keeps the first entry and discards the second. This works correctly for same-start ties.

However, for the case where a lower-priority pattern's `start` is less than a higher-priority pattern's `start` but both are non-overlapping with `last_end`, both are accepted. The real issue is when a higher-priority pattern starts *later* but overlaps a lower-priority pattern that started earlier and was already accepted. In this case the lower-priority early match wins, which may not be the intent.

Example: A `PHONE` match starting at byte 5 and a `SSN` match starting at byte 10 where SSN is fully contained in PHONE's range. SSN has higher priority (index 8) vs PHONE (index 13), but PHONE's earlier start position causes PHONE to win. The comment says "more specific patterns first" which would suggest SSN should win here, but the algorithm does not implement that.

**Fix:** Document this as intentional position-wins-over-priority behaviour, or change the algorithm to sort by `(start, pattern_index)` to break start-position ties by priority:
```rust
refined.sort_by(|a, b| {
    a.start.cmp(&b.start)
        .then_with(|| {
            // Earlier pattern index = higher priority; keep that ordering
            // (The original push order reflects priority; stable sort preserves it)
            std::cmp::Ordering::Equal
        })
});
```
The current stable sort already handles start-position ties correctly; the real documentation debt is that position always wins over priority for non-overlapping but interleaved matches.

### WR-07: `FileReadError` Is Unreachable via `?` Due to `#[from]` on `WriteError`

**File:** `src/error.rs:9-25`
**Issue:** `WriteError(#[from] std::io::Error)` generates a `From<std::io::Error> for TokeniserError` impl. `FileReadError { source: std::io::Error }` does *not* have `#[from]`, so it cannot be reached via `?` from an `std::io::Error`. Any `?` on a file-read operation producing `std::io::Error` will produce `WriteError`, not `FileReadError`. The `FileReadError` variant is effectively dead as an auto-conversion target, making it misleading in error messages.

**Fix:** Either remove `FileReadError` and consolidate on `WriteError` (rename to `IoError`), or remove `#[from]` from `WriteError` and use explicit `.map_err()` at each site to pick the correct variant:
```rust
#[error("I/O error: {0}")]
IoError(#[from] std::io::Error),
```

---

## Info

### IN-01: Unsafe Env-Var Mutation in Test Suite

**File:** `tests/store_tests.rs:33,52,68,88,91,109,130,147,163,170`
**Issue:** The tests use `unsafe { std::env::set_var(...) }` and `unsafe { std::env::remove_var(...) }`. In Rust 2024 edition (which this project uses per `Cargo.toml:4`), modifying environment variables is `unsafe` due to potential data races in multi-threaded test binaries. The `ENV_LOCK: Mutex<()>` only protects the test bodies against each other — it does not prevent races with any other thread in the test binary that reads the environment concurrently. The correct fix is to pass the key through a different mechanism (e.g., constructor parameter) rather than relying on process-global env state in tests.

**Fix:** Pass the passphrase directly to `Store::new_with_key(store_dir, passphrase)` for testability, keeping `Store::new()` as the production entry point that reads from env. This removes the need for env mutation in tests entirely.

### IN-02: Struct Name Typo — `LoktokConfig` vs Project Name `logtok`

**File:** `src/config.rs:8`
**Issue:** The struct is named `LoktokConfig` (Loktok) while the binary, crate, and config file are all `logtok` / `.logtok.toml`. This is an internal inconsistency that will cause confusion when reading code.

**Fix:** Rename to `LogtokConfig` throughout `config.rs` and its callers in `main.rs`.

### IN-03: `to_detection_config` Clones Entire Vec on Every Call

**File:** `src/config.rs:88-101`
**Issue:** `to_detection_config` clones all strings in `disabled_categories` and all fields of each `CustomPatternDef` on every invocation. Currently called once in `main.rs`, but if called in a hot path or in tests it performs unnecessary allocations. Minor, but the method name suggests a cheap conversion.

**Fix:** Accept this as-is for current usage, or cache the `DetectionConfig` at config-load time rather than reconstructing it on demand.

---

_Reviewed: 2026-04-15_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
