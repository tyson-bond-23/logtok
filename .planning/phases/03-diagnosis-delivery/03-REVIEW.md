---
phase: 03-diagnosis-delivery
reviewed: 2026-04-16T15:30:00Z
depth: standard
files_reviewed: 14
files_reviewed_list:
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
  - CLAUDE.md
  - Cargo.toml
  - README.md
  - src/cli.rs
  - src/clipboard.rs
  - src/detokenizer.rs
  - src/error.rs
  - src/lib.rs
  - src/main.rs
  - src/processor.rs
  - tests/cli_tests.rs
  - tests/integration_tests.rs
findings:
  critical: 1
  warning: 5
  info: 3
  total: 9
status: issues_found
---

# Phase 3: Code Review Report

**Reviewed:** 2026-04-16T15:30:00Z
**Depth:** standard
**Files Reviewed:** 14
**Status:** issues_found

## Summary

Reviewed all source files for the logtok CLI tool -- a Rust-based log tokenizer with encrypted store, detokenization, and clipboard support. The codebase is generally well-structured with good error handling patterns. The encryption implementation (AES-256-GCM with Argon2id) follows best practices. Key concerns include a potential panic from `unwrap()` on `current_dir()` in a user-facing code path, a hardcoded CI secret, and a `truncate_example` function that can panic on multi-byte UTF-8 input. Several minor code quality items were also identified.

## Critical Issues

### CR-01: `truncate_example` panics on multi-byte UTF-8 strings

**File:** `src/processor.rs:273`
**Issue:** The `truncate_example` function slices the string using byte index (`&s[..max_len]`), but `s.len()` returns byte length while the slice assumes character alignment. If a multi-byte UTF-8 character straddles the `max_len` boundary, `&s[..max_len]` will panic at runtime with "byte index is not a char boundary". Log files from international applications routinely contain multi-byte characters (e.g., CJK, Cyrillic, emoji in hostnames or usernames).
**Fix:**
```rust
fn truncate_example(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // Find a valid char boundary at or before max_len
        let mut end = max_len;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &s[..end])
    }
}
```

## Warnings

### WR-01: `unwrap()` on `current_dir()` in user-facing code path

**File:** `src/main.rs:112`
**Issue:** The detokenize command uses `std::env::current_dir().unwrap()` in the fallback for `--store`. If the current working directory has been deleted or is otherwise inaccessible (not uncommon in containerized or scripted environments), this panics instead of producing a user-friendly error.
**Fix:**
```rust
let store_dir = match store_path {
    Some(p) => p,
    None => std::env::current_dir()
        .context("Cannot determine current directory for token store")?
        .join(".loktok"),
};
```

### WR-02: `ensure_gitignore` ignores the `Store` parameter and hardcodes CWD

**File:** `src/processor.rs:185-192`
**Issue:** The function `ensure_gitignore` accepts `_input_path` and `_store` parameters but ignores both, instead hardcoding `std::env::current_dir()?.join(".loktok")`. If the store is in a non-CWD location (e.g., via future `--store` flag on tokenize), the `.gitignore` will be written to the wrong directory. The underscore-prefixed unused parameters also mask this bug.
**Fix:** Derive the store directory from the `Store` instance or accept the store path directly:
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

### WR-03: Hardcoded test passphrase in CI workflow

**File:** `.github/workflows/ci.yml:23`
**Issue:** `LOGTOK_KEY: "ci-test-passphrase"` is hardcoded directly in the workflow file. While this is a CI-only test passphrase and not a production secret, it sets a poor pattern. If a contributor copies this pattern to a production workflow or a deploy step, real secrets could be hardcoded. The value should use a GitHub Actions secret or at minimum be clearly marked as test-only.
**Fix:** Use a GitHub secret or add a clear comment:
```yaml
env:
  # Test-only passphrase -- NOT a real secret. Used only for integration tests.
  LOGTOK_KEY: "ci-test-passphrase"
```
Alternatively, use `${{ secrets.LOGTOK_TEST_KEY }}` with a fallback.

### WR-04: `tempfile` is both a runtime and dev dependency

**File:** `Cargo.toml:17,24`
**Issue:** `tempfile = "3"` appears in both `[dependencies]` and `[dev-dependencies]`. It is used at runtime in `main.rs` for clipboard capture (via `tempfile::NamedTempFile`). Having it in both sections is redundant, but more importantly, `tempfile` as a runtime dependency means the clipboard path creates a temporary file on disk for every tokenize-with-clipboard operation. This is a design smell -- the tokenized output could be captured in-memory instead.
**Fix:** Remove from `[dev-dependencies]` since it is already in `[dependencies]`. Consider refactoring the clipboard path to use an in-memory buffer instead of a temp file:
```rust
// Instead of writing to temp file then reading back:
let mut buffer = Vec::new();
// ... write tokenized output to buffer ...
let tokenized = String::from_utf8(buffer)?;
```

### WR-05: Detokenizer regex compiled on every call

**File:** `src/detokenizer.rs:35`
**Issue:** `Regex::new(r"\[([A-Z]+_\d{3,})\]").unwrap()` is called inside `detokenize()`, meaning the regex is recompiled every time the function is invoked. While this function is typically called once per run, it is a public API and could be called in a loop by library consumers. The `unwrap()` is safe here (the pattern is a compile-time constant), but the recompilation is wasteful.
**Fix:** Use `std::sync::LazyLock` (stable in Rust 1.80+):
```rust
use std::sync::LazyLock;

static TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[([A-Z]+_\d{3,})\]").unwrap()
});
```

## Info

### IN-01: Dual module declarations in `lib.rs` and `main.rs`

**File:** `src/lib.rs:1-11`, `src/main.rs:1-11`
**Issue:** All modules are declared in both `lib.rs` (as `pub mod`) and `main.rs` (as `mod`). The `main.rs` declarations shadow the library, meaning `main.rs` does not actually use the library crate at all. Integration tests using `logtok::` would use the `lib.rs` path, while `main.rs` compiles its own copy. This is a common Rust project structure issue that can lead to confusing behavior if types from the two compilation units are mixed.
**Fix:** In `main.rs`, import from the library crate instead of re-declaring modules:
```rust
use logtok::{cli, clipboard, config, detokenizer, processor, store};
```
This requires making the necessary types and functions `pub` in the library modules.

### IN-02: `edition = "2024"` in Cargo.toml

**File:** `Cargo.toml:4`
**Issue:** The Rust 2024 edition is very new (stabilized in Rust 1.85). This limits contributors to a very recent toolchain. Not a bug, but worth noting for portability awareness, especially since the CI workflow uses `dtolnay/rust-toolchain@stable` without pinning a minimum version.
**Fix:** No change needed if all contributors are on 1.85+. Consider adding a `rust-version` field to Cargo.toml to make the MSRV explicit:
```toml
rust-version = "1.85"
```

### IN-03: Unused `_block_size` parameter in `run_dry_run`

**File:** `src/processor.rs:200`
**Issue:** The `_block_size` parameter is accepted but unused, indicated by the underscore prefix. If dry-run is intended to simulate block processing, this parameter should be used. Otherwise, remove it from the function signature.
**Fix:** Remove the parameter if not needed, or add a comment explaining future intent.

---

_Reviewed: 2026-04-16T15:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
