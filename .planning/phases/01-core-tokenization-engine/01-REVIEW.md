---
phase: 01-core-tokenization-engine
reviewed: 2026-04-19T12:00:00Z
depth: standard
files_reviewed: 23
files_reviewed_list:
  - src/cli.rs
  - src/clipboard.rs
  - src/compactor.rs
  - src/config.rs
  - src/detector.rs
  - src/detokenizer.rs
  - src/error.rs
  - src/json_processor.rs
  - src/lib.rs
  - src/main.rs
  - src/processor.rs
  - src/store.rs
  - src/tokenizer.rs
  - tests/cli_tests.rs
  - tests/compactor_tests.rs
  - tests/config_tests.rs
  - tests/integration_tests.rs
  - tests/json_tests.rs
  - tests/store_tests.rs
  - tests/unit_tests.rs
  - Cargo.toml
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
findings:
  critical: 1
  warning: 5
  info: 3
  total: 9
status: issues_found
---

# Phase 01: Code Review Report

**Reviewed:** 2026-04-19
**Depth:** standard
**Files Reviewed:** 23
**Status:** issues_found

## Summary

The codebase is well-structured with clean module separation, solid test coverage (unit, integration, CLI, and store-level), and correct use of authenticated encryption (AES-256-GCM + Argon2id) for the token store. Detection patterns are carefully ordered by priority with stable-sort overlap resolution. The processing pipeline (detect, tokenize, compact, output) is sound and the block-based architecture is ready for large file handling.

The main concerns are: (1) a panic-inducing byte-boundary string slice in `truncate_example`, (2) an `unwrap()` on `current_dir()` in the detokenize error path, (3) potential unsigned integer underflow in TTL expiry logic, (4) `ensure_gitignore` ignoring its store parameter, and (5) unsafe environment variable mutation in tests.

## Critical Issues

### CR-01: Panic on multi-byte UTF-8 in `truncate_example`

**File:** `src/processor.rs:273`
**Issue:** `&s[..max_len]` slices on byte offset, not character boundary. If a multi-byte UTF-8 character (e.g., accented name in log data, CJK characters) straddles the `max_len` boundary, this panics at runtime with `byte index N is not a char boundary`. This is reachable through the `--dry-run` flag with any log containing non-ASCII sensitive values.
**Fix:**
```rust
fn truncate_example(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len).collect();
        format!("{}...", truncated)
    }
}
```

## Warnings

### WR-01: `unwrap()` on `current_dir()` panics if CWD is unavailable

**File:** `src/main.rs:112`
**Issue:** `store_path.unwrap_or_else(|| std::env::current_dir().unwrap().join(".loktok"))` will panic if the working directory has been deleted or is otherwise inaccessible. This is the detokenize code path where a user might reasonably be in a cleaned-up directory or running from a removed temp directory.
**Fix:**
```rust
let store_dir = match store_path {
    Some(p) => p,
    None => std::env::current_dir()
        .context("Cannot determine current directory for token store")?
        .join(".loktok"),
};
```

### WR-02: Unsigned integer underflow in `purge_expired`

**File:** `src/tokenizer.rs:130`
**Issue:** `now - entry.created_at` performs unsigned subtraction on `u64`. If a token entry has a `created_at` timestamp in the future (e.g., clock skew between machines, manually edited store, or system clock adjusted backward), this wraps around to a very large number in release builds, causing unexpected mass-purge of all tokens. While clock-in-future is uncommon, it is a realistic scenario for a tool that persists encrypted state across sessions.
**Fix:**
```rust
.filter(|(_, entry)| {
    now.checked_sub(entry.created_at)
        .map_or(false, |age| age > ttl_seconds)
})
```

### WR-03: `ensure_gitignore` ignores its `_store` parameter, hardcodes CWD

**File:** `src/processor.rs:186`
**Issue:** The function signature is `fn ensure_gitignore(_input_path: &Path, _store: &Store)` but both parameters are unused (prefixed with `_`). The function hardcodes `std::env::current_dir()?.join(".loktok")`. If the store directory is in a non-CWD location (future `--store` flag on tokenize, or any invocation where CWD differs from the store location), the `.gitignore` is written to the wrong directory.
**Fix:** Either expose the store directory path from `Store` and use it, or accept `store_dir: &Path` as a parameter:
```rust
fn ensure_gitignore(store_dir: &Path) -> Result<()> {
    let gitignore_path = store_dir.join(".gitignore");
    if !gitignore_path.exists() {
        fs::create_dir_all(store_dir)?;
        fs::write(&gitignore_path, "*\n")?;
    }
    Ok(())
}
```

### WR-04: Regex recompiled on every `detokenize` call

**File:** `src/detokenizer.rs:35`
**Issue:** `Regex::new(r"\[([A-Z]+_\d{3,})\]").unwrap()` compiles the regex on every invocation of `detokenize()`. While the `unwrap()` is safe here (the pattern is a static literal), recompilation is wasteful. More importantly, if this function is ever called in a loop (batch mode, streaming), the cost compounds. Use a static `LazyLock`.
**Fix:**
```rust
use std::sync::LazyLock;

static TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[([A-Z]+_\d{3,})\]").unwrap()
});

pub fn detokenize(text: &str, token_to_value: &HashMap<String, String>) -> DetokenizeResult {
    let re = &*TOKEN_RE;
    // ... rest unchanged
}
```

### WR-05: Unsafe environment variable mutation in store tests

**File:** `tests/store_tests.rs:33,52,69,91,109,132,163,173`
**Issue:** Multiple test functions use `unsafe { std::env::set_var("LOGTOK_KEY", ...) }` to mutate the process environment. The `ENV_LOCK` mutex serializes these specific tests, but does not prevent other threads (Rust test harness runs tests in parallel by default) from reading `LOGTOK_KEY` concurrently. Rust 2024 edition correctly marks `set_var` as `unsafe` because this is inherently racy in multi-threaded programs.
**Fix:** Refactor `Store::new` to accept the passphrase as a parameter (or via a builder/config struct) instead of reading from the environment directly. Keep the env var as the default source in `main.rs` for backward compatibility:
```rust
impl Store {
    pub fn with_passphrase(store_dir: &Path, passphrase: String) -> Result<Self, TokeniserError> {
        // ... current logic without env::var
    }

    pub fn new(store_dir: &Path) -> Result<Self, TokeniserError> {
        let passphrase = std::env::var("LOGTOK_KEY").map_err(|_| ...)?;
        Self::with_passphrase(store_dir, passphrase)
    }
}
```
Tests then call `Store::with_passphrase()` directly, eliminating all `unsafe` blocks.

## Info

### IN-01: Unused dependencies: `tracing` and `tracing-subscriber`

**File:** `Cargo.toml:20-21`
**Issue:** `tracing = "0.1.44"` and `tracing-subscriber = "0.3"` are listed as dependencies but are never imported or used anywhere in the source code. They add unnecessary compile time and binary size.
**Fix:** Remove both from `[dependencies]` until structured logging is implemented.

### IN-02: `tempfile` listed in both `[dependencies]` and `[dev-dependencies]`

**File:** `Cargo.toml:17,24`
**Issue:** `tempfile = "3"` appears in both `[dependencies]` and `[dev-dependencies]`. It is used in `main.rs` for clipboard temp file handling (production code) so the `[dependencies]` entry is correct, but the `[dev-dependencies]` entry is redundant since dev-dependencies automatically include regular dependencies.
**Fix:** Remove `tempfile = "3"` from `[dev-dependencies]`.

### IN-03: PATH regex only matches Unix-style paths

**File:** `src/detector.rs:148`
**Issue:** The PATH pattern `(?:/[a-zA-Z0-9._-]+){3,}` only matches forward-slash Unix paths. On Windows, paths like `C:\Users\app\logs\file.log` will not be detected. This may be intentional (log files typically originate from Linux servers), but is worth noting given the project's cross-platform binary distribution goal.
**Fix:** If Windows path detection is desired in a future iteration, add an alternative pattern or document Unix-only scope explicitly.

---

_Reviewed: 2026-04-19_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
