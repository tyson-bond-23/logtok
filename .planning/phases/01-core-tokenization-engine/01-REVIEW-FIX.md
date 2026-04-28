---
phase: 01-core-tokenization-engine
fixed_at: 2026-04-28
fix_scope: critical_warning
findings_in_scope: 6
fixed: 6
skipped: 0
iteration: 1
status: all_fixed
---

# Phase 01: Code Review Fix Report

**Fixed:** 2026-04-28
**Scope:** Critical + Warning (6 findings)
**Status:** all_fixed

## Fixes Applied

### CR-01: UTF-8 panic in `truncate_example` — FIXED
**File:** `src/processor.rs:267-274`
**Commit:** `fix(01): use char iterator in truncate_example to prevent UTF-8 panic (CR-01)`
**Change:** Replaced byte-index slice `&s[..max_len]` with `s.chars().take(max_len).collect()` to safely handle multi-byte UTF-8 characters in dry-run output.

### WR-01: `unwrap()` on `current_dir()` — FIXED
**File:** `src/main.rs` (tokenize, detokenize, and reset-store handlers)
**Commit:** `fix(01): replace unwrap on current_dir with context error (WR-01)`
**Change:** Replaced `unwrap()` with `.context("Cannot determine current directory for token store")?` in all three command handlers. Also fixed the tokenize path which had the same bare `?` pattern.

### WR-02: Unsigned underflow in `purge_expired` — FIXED
**File:** `src/tokenizer.rs:130`
**Commit:** `fix(01): use checked_sub in purge_expired to prevent unsigned underflow (WR-02)`
**Change:** Replaced `now - entry.created_at > ttl_seconds` with `now.checked_sub(entry.created_at).map_or(false, |age| age > ttl_seconds)`. Future-timestamped entries are now safely retained instead of triggering mass purge.

### WR-03: `ensure_gitignore` ignores store parameter — FIXED
**Files:** `src/store.rs`, `src/processor.rs`
**Commit:** `fix(01): use store directory from Store::dir() in ensure_gitignore (WR-03)`
**Change:** Added `pub fn dir(&self) -> &Path` to Store. Simplified `ensure_gitignore` to accept `store_dir: &Path` and use the actual store directory instead of hardcoded `current_dir()/.loktok`.

### WR-04: Regex recompiled per call — FIXED
**File:** `src/detokenizer.rs`
**Commit:** `fix(01): use static LazyLock for token regex in detokenizer (WR-04)`
**Change:** Moved regex compilation to a `static TOKEN_RE: LazyLock<Regex>` at module level. Pattern is compiled once on first use and reused across all invocations.

### WR-05: Unsafe env var mutation in store tests — FIXED
**Files:** `src/store.rs`, `tests/store_tests.rs`
**Commit:** `fix(01): add Store::with_passphrase to eliminate unsafe env var mutation in tests (WR-05)`
**Change:** Added `Store::with_passphrase(store_dir, passphrase)` constructor. Refactored `Store::new` to delegate to it. Replaced 12 `unsafe { std::env::set_var }` calls and removed `ENV_LOCK` mutex from store tests. Only `missing_logtok_key_produces_error` retains env var manipulation (necessary to test that specific error path).

## Info Findings (Not in Scope)

- **IN-01:** Unused `tracing`/`tracing-subscriber` deps — skipped (info)
- **IN-02:** Redundant `tempfile` in dev-deps — skipped (info)
- **IN-03:** PATH regex Unix-only — skipped (info)
