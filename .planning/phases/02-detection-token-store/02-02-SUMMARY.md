---
phase: 02-detection-token-store
plan: 02
subsystem: config, encryption
tags: [toml, aes-gcm, argon2, config-discovery, encrypted-store]

# Dependency graph
requires:
  - phase: 02-detection-token-store/01
    provides: "DetectionConfig, CustomPatternDef, TokenMapData, TokenEntry structs"
provides:
  - "TOML config parsing with walk-up discovery (src/config.rs)"
  - "AES-256-GCM encrypted token store with Argon2 key derivation (src/store.rs)"
  - "ConfigError and StoreError variants in error.rs"
affects: [02-detection-token-store/03, cli-integration, store-persistence]

# Tech tracking
tech-stack:
  added: [toml 0.8, aes-gcm 0.10.3, argon2 0.5.3, rand 0.9]
  patterns: [walk-up config discovery, AEAD encrypted file format with magic/version header, atomic file write via temp+rename, env var key derivation]

key-files:
  created: [src/config.rs, src/store.rs, tests/config_tests.rs, tests/store_tests.rs]
  modified: [src/error.rs, src/lib.rs, Cargo.toml]

key-decisions:
  - "Used find_config_from(path) as testable entry point, find_config() wraps it with CWD"
  - "Store file format: 4-byte magic LTOK + 1-byte version + 16-byte salt + 12-byte nonce + ciphertext"
  - "Salt persists across saves (read from existing file) so same passphrase derives same key"
  - "Mutex-based test serialization for store tests that share LOGTOK_KEY env var"

patterns-established:
  - "Config walk-up: search from start dir upward for .logtok.toml"
  - "Encrypted file format: magic bytes + version + salt + nonce + AEAD ciphertext"
  - "Atomic file write: write to .enc.tmp then rename to .enc"
  - "Env var gating: LOGTOK_KEY required, clear error message if missing"

requirements-completed: [DET-05, TOK-03, TOK-04]

# Metrics
duration: 97min
completed: 2026-04-15
---

# Phase 2 Plan 2: Config and Encrypted Store Summary

**TOML config module with walk-up discovery and AES-256-GCM encrypted token store using Argon2id key derivation from LOGTOK_KEY env var**

## Performance

- **Duration:** 97 min
- **Started:** 2026-04-15T08:12:32Z
- **Completed:** 2026-04-15T09:49:36Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- TOML config module with walk-up discovery, disabled categories, custom patterns, and configurable TTL
- AES-256-GCM encrypted store with LTOK magic header, Argon2id key derivation, persistent salt, fresh nonce per save
- Atomic file write prevents corruption on crash, clear error on missing LOGTOK_KEY
- 20 new tests (10 config + 10 store), full suite of 110 tests passes

## Task Commits

Each task was committed atomically:

1. **Task 1: Create TOML configuration module with discovery and parsing** - `29ed255` (feat)
2. **Task 2: Create encrypted token store module with Argon2 + AES-256-GCM** - `7f59bf7` (feat)

_Both tasks followed TDD: RED (failing tests) then GREEN (implementation)._

## Files Created/Modified
- `src/config.rs` - TOML config parsing, walk-up discovery, LoktokConfig/DetectionSection/StoreSection structs
- `src/store.rs` - AES-256-GCM encrypted store with save/load/reset, Argon2 key derivation
- `src/error.rs` - Added ConfigError and StoreError variants
- `src/lib.rs` - Added pub mod config and pub mod store
- `Cargo.toml` - Added toml, aes-gcm, argon2, rand dependencies
- `tests/config_tests.rs` - 10 tests for config discovery, parsing, defaults, conversion
- `tests/store_tests.rs` - 10 tests for encryption round-trip, nonce freshness, wrong key, reset, atomic write

## Decisions Made
- Used `find_config_from(path)` as the testable entry point instead of `find_config()` which uses CWD -- enables deterministic testing with tempdir
- Store file format uses 33-byte header (4 magic + 1 version + 16 salt + 12 nonce) before ciphertext -- enables format validation and future versioning
- Salt persists across saves by reading from existing file header -- same passphrase always derives same key within a session
- Store tests use `std::sync::Mutex` to serialize env var access since `set_var`/`remove_var` affect the whole process

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added Debug derive to Store struct**
- **Found during:** Task 2 (store tests)
- **Issue:** `unwrap_err()` on `Result<Store, TokeniserError>` requires `Store: Debug`
- **Fix:** Added `#[derive(Debug)]` to `Store` struct
- **Files modified:** src/store.rs
- **Verification:** All store tests compile and pass
- **Committed in:** 7f59bf7 (Task 2 commit)

**2. [Rule 1 - Bug] Added mutex serialization for store tests**
- **Found during:** Task 2 (store tests)
- **Issue:** Tests that manipulate LOGTOK_KEY env var race when run in parallel, causing `same_passphrase_same_salt_derives_same_key` to fail
- **Fix:** Added `static ENV_LOCK: Mutex<()>` and `let _guard = ENV_LOCK.lock()` in each test
- **Files modified:** tests/store_tests.rs
- **Verification:** All 10 store tests pass reliably
- **Committed in:** 7f59bf7 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Both fixes necessary for correctness. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Config and store modules are standalone, ready for Plan 03 to wire them into the CLI and processor pipeline
- `LoktokConfig::to_detection_config()` bridges directly to `DetectionPatterns::from_config()`
- `Store::load()`/`Store::save()` work with `TokenMapData` from tokenizer.rs
- LOGTOK_KEY env var must be set before using store features

## Self-Check: PASSED

- All 7 files verified present on disk
- Both task commits verified in git log (29ed255, 7f59bf7)
- Key content markers verified in source files

---
*Phase: 02-detection-token-store*
*Completed: 2026-04-15*
