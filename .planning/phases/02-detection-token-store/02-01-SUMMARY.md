---
phase: 02-detection-token-store
plan: 01
subsystem: detection
tags: [regex, luhn, serde, detection, tokenizer, pii, credentials, infrastructure]

# Dependency graph
requires:
  - phase: 01-core-tokenization
    provides: "DetectionPatterns with 7 categories, TokenMap with deterministic get_or_insert"
provides:
  - "19-category detection with config-driven from_config() constructor"
  - "Serializable TokenMapData with reverse map and TTL metadata"
  - "Luhn-validated credit card detection"
  - "DetectionConfig for disabling categories and adding custom patterns"
  - "MatchMode enum (Full, CaptureGroup, LuhnValidated) for pattern extraction"
  - "merge() and purge_expired() for token store lifecycle"
affects: [02-02-PLAN (config), 02-03-PLAN (encrypted store)]

# Tech tracking
tech-stack:
  added: []
  patterns: [config-driven detection, match mode enum, TDD red-green]

key-files:
  created: []
  modified:
    - src/detector.rs
    - src/tokenizer.rs
    - src/error.rs
    - tests/unit_tests.rs

key-decisions:
  - "NAME regex extended with optional trailing quote on key to handle JSON 'username': 'X' patterns"
  - "Merge operates on stored data (load-then-merge pattern), not independent session merging"
  - "to_data_mut() exposed for testing and internal TTL manipulation"

patterns-established:
  - "MatchMode enum: Full, CaptureGroup(usize), LuhnValidated for extensible extraction modes"
  - "builtin_patterns() function separates pattern definitions from compilation for reuse"
  - "TDD red-green with separate commits for failing tests and implementation"

requirements-completed: [DET-01, DET-02, DET-03]

# Metrics
duration: 7min
completed: 2026-04-15
---

# Phase 2 Plan 1: Detection Expansion & Serializable TokenMap Summary

**19-category regex detection with Luhn CC validation, config-driven from_config(), and serde-serializable TokenMap with reverse map, merge, and TTL purge**

## Performance

- **Duration:** 7 min
- **Started:** 2026-04-15T08:02:36Z
- **Completed:** 2026-04-15T08:09:43Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Expanded detection from 7 to 19 categories covering credentials (JWT, PEM, CONN), PII (CC, SSN, PHONE, NAME, UUID), and infrastructure (DNS, MAC, ARN, OS)
- Luhn algorithm validates credit card numbers before detection (prevents false positives on arbitrary 16-digit numbers)
- TokenMap is fully serializable via serde with reverse map (token_to_value), merge support, and TTL-based purge
- Config-driven architecture: from_config() supports disabling categories and adding custom patterns
- All 51 tests pass (17 original + 34 new) with zero regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: Expand DetectionPatterns to 19 categories** - `de9bc2a` (test: RED), `73fd263` (feat: GREEN)
2. **Task 2: Make TokenMap serializable with reverse map and TTL** - `0cd3a22` (test: RED), `f85904c` (feat: GREEN)

_TDD tasks have separate commits for failing tests and implementation._

## Files Created/Modified
- `src/detector.rs` - 19-category detection with MatchMode enum, builtin_patterns(), from_config(), Luhn validation, DetectionConfig
- `src/tokenizer.rs` - TokenEntry, TokenMapData with Serialize/Deserialize, reverse map, merge(), purge_expired(), to_data()/from_data()
- `src/error.rs` - Added PatternCompileError variant
- `tests/unit_tests.rs` - 51 tests covering all 19 categories, priority, config, Luhn, serialization, merge, TTL

## Decisions Made
- NAME regex extended with optional quote after key name (`['"]?` before `\s*[=:]`) to handle JSON `"username": "admin"` patterns where the key is quoted
- Merge designed for load-then-merge pattern (loading a previously saved store into a new session), not merging two independent sessions
- Exposed `to_data_mut()` for testing TTL purge (setting created_at to epoch for deterministic expiration tests)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] NAME regex did not match JSON-quoted keys**
- **Found during:** Task 1 (GREEN phase)
- **Issue:** NAME regex `(?i)(?:user(?:name)?|author|name)\s*[=:]` did not match `"username": "admin"` because the closing `"` after `username` was not handled before the `\s*[=:]`
- **Fix:** Added optional quote `['"]?` between the keyword and `\s*[=:]` to handle JSON-style keys
- **Files modified:** src/detector.rs
- **Verification:** detect_name_json_username test passes
- **Committed in:** 73fd263 (Task 1 feat commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Essential for correctness -- JSON username patterns are a primary use case for NAME detection. No scope creep.

## Issues Encountered
- Windows linker contention (LNK1104) from parallel agent builds sharing the target directory -- resolved by retrying after brief delay. Not a code issue.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Detection patterns ready for config-driven operation via `.logtok.toml` (Plan 02)
- TokenMapData serializable and ready for encrypted persistence (Plan 03)
- from_config() ready to accept parsed TOML configuration
- merge() and purge_expired() ready for store load/save lifecycle

---
*Phase: 02-detection-token-store*
*Completed: 2026-04-15*
