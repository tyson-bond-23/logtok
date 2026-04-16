---
phase: 03-diagnosis-delivery
plan: 01
subsystem: cli
tags: [clap, subcommands, detokenize, clipboard, regex]

# Dependency graph
requires:
  - phase: 02-detection-store
    provides: "Encrypted token store with AES-256-GCM, detection patterns, TokenMapData"
provides:
  - "Subcommand-based CLI (tokenize/detokenize/reset-store)"
  - "De-tokenization engine replacing [CATEGORY_NNN] tokens with real values"
  - "Clipboard copy support for tokenized output"
  - "Progress bar with throughput display"
affects: [03-02, 03-03]

# Tech tracking
tech-stack:
  added: [cli-clipboard]
  patterns: [subcommand-based CLI routing, regex-based token replacement, stdin/file input duality]

key-files:
  created: [src/detokenizer.rs, src/clipboard.rs]
  modified: [src/cli.rs, src/main.rs, src/error.rs, src/lib.rs, src/processor.rs, Cargo.toml, tests/integration_tests.rs, tests/cli_tests.rs]

key-decisions:
  - "Clipboard only on tokenize command (not detokenize) to prevent real values reaching clipboard"
  - "Unresolved tokens left as-is during detokenization for graceful degradation"
  - "tempfile used for clipboard capture path to avoid holding full output in memory twice"

patterns-established:
  - "Subcommand routing: match cli.command with Commands::Variant destructuring"
  - "Stdin detection: is_terminal() check to distinguish pipe from interactive"
  - "Detailed output: --detailed flag writes markdown report with stats footer"

requirements-completed: [DIA-02, DIA-03, DIA-04, DIA-05]

# Metrics
duration: 5min
completed: 2026-04-16
---

# Phase 3 Plan 1: CLI Subcommands + De-tokenization Engine Summary

**Subcommand-based CLI with detokenize engine replacing [CATEGORY_NNN] tokens from encrypted store, clipboard copy on tokenize, and progress bar throughput display**

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-16T12:19:32Z
- **Completed:** 2026-04-16T12:24:16Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments
- Restructured CLI from flat args to subcommands (tokenize/detokenize/reset-store) with global --quiet and --config flags
- Implemented de-tokenization engine with regex-based [CATEGORY_NNN] token replacement, stdin/file input, stdout/markdown output
- Added cross-platform clipboard support for tokenize output via cli-clipboard crate
- Enhanced progress bar with throughput display (bytes_per_sec)
- All 113 tests pass including 7 new tests for detokenize and CLI subcommands

## Task Commits

Each task was committed atomically:

1. **Task 1: CLI restructure to subcommands + detokenizer module + clipboard module** - `6a6422a` (feat)
2. **Task 2: Wire subcommands into main.rs + update all tests** - `de71367` (feat)

## Files Created/Modified
- `src/cli.rs` - Subcommand-based CLI with Tokenize/Detokenize/ResetStore enum
- `src/detokenizer.rs` - De-tokenization engine (read_input, detokenize, write_output)
- `src/clipboard.rs` - Cross-platform clipboard copy wrapper
- `src/main.rs` - Subcommand routing with detokenize/clipboard integration
- `src/error.rs` - Added DetokenizeError variant
- `src/lib.rs` - Registered clipboard and detokenizer modules
- `src/processor.rs` - Enhanced progress bar template with bytes_per_sec
- `Cargo.toml` - Added cli-clipboard and tempfile dependencies
- `tests/integration_tests.rs` - Updated to subcommand syntax, added 5 detokenize tests
- `tests/cli_tests.rs` - Updated to subcommand syntax, added tokenize/detokenize help tests

## Decisions Made
- Clipboard only available on tokenize (not detokenize) per threat model T-03-01 to prevent real values reaching clipboard
- Unresolved tokens left as-is during detokenization -- graceful degradation rather than error
- Used tempfile crate for clipboard capture path to avoid holding output in memory twice
- Global --config and --quiet flags shared across all subcommands

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- De-tokenization engine complete, ready for Claude API integration (plan 03-02)
- CLI subcommand structure extensible for future commands
- Store persistence verified across tokenize/detokenize cycle

---
*Phase: 03-diagnosis-delivery*
*Completed: 2026-04-16*
