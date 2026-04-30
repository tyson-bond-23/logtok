---
phase: "06"
plan: "03"
subsystem: ui-tokenize-detokenize
tags: [api, htmx, tokenize, detokenize, multipart, templates]
dependency_graph:
  requires: ["06-01"]
  provides: ["api_tokenize", "api_detokenize", "tokenize-panel", "detokenize-panel"]
  affects: ["src/ui/api_tokenize.rs", "templates/ui/tokenize.html", "templates/ui/detokenize.html", "Cargo.toml"]
tech_stack:
  added: ["axum multipart feature"]
  patterns: ["spawn_blocking for CPU work", "multipart form parsing", "HTMX hx-post with multipart encoding", "Alpine.js drag-drop state", "localStorage recent files"]
key_files:
  created: []
  modified:
    - src/ui/api_tokenize.rs
    - templates/ui/tokenize.html
    - templates/ui/detokenize.html
    - Cargo.toml
decisions:
  - "Used manual html_escape function instead of askama::filters::escape for inline HTML strings"
  - "Added axum multipart feature to Cargo.toml (required for Multipart extractor)"
  - "Tokenize panel uses input mode tabs (paste/filepath/upload) instead of all-visible layout for cleaner UX"
  - "Token count in stats uses rough line-based estimation since core engine writes to file"
metrics:
  duration: 333s
  completed: "2026-04-30T07:19:11Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 4
requirements:
  - UI-06
  - UI-07
  - UI-13
---

# Phase 06 Plan 03: Tokenize & Detokenize Panels Summary

Real tokenize/detokenize API handlers with 3 input sources and HTMX-powered panel templates with drag-drop, file picker, paste, and file path input

## What Was Done

### Task 1: Implement tokenize and detokenize API handlers
Replaced stub `api_tokenize.rs` with full implementations. The `api_tokenize` handler accepts multipart form data with three input sources: file path (server reads directly), pasted text content, and uploaded file bytes. Processing uses `spawn_blocking` to avoid blocking the async runtime (Pitfall 1). The handler loads config via `find_config()`/`load_config()`, creates a `Store` with the session key from `AppState` (D-19), and calls `process_file_with_config` for tokenization. File paths are validated with `is_file()` (T-06-02). All output is HTML-escaped (T-06-06). Successful file path tokenization sends an `HX-Trigger` header for Alpine.js recent files update (D-25).

The `api_detokenize` handler accepts content, loads the token store, and calls `detokenizer::detokenize()` to restore real values. Both handlers return HTML fragments for HTMX swap.

### Task 2: Create tokenize and detokenize panel templates
Replaced stub templates with full HTMX-powered panels. The tokenize panel features input mode tabs (Paste Text, File Path, Upload File) for clean UX. The paste mode has a textarea. The file path mode has a text input with recent files dropdown from localStorage (D-25). The upload mode has a drag-and-drop zone with visual feedback via Alpine.js plus a file picker button. All modes submit via `hx-post="/api/tokenize"` with `multipart/form-data` encoding.

The detokenize panel has a textarea for pasting tokenized text and submits via `hx-post="/api/detokenize"`. Both panels include loading indicators and result areas.

## Commits

| # | Hash | Message | Files |
|---|------|---------|-------|
| 1 | 8458313 | feat(06-03): implement tokenize and detokenize API handlers | src/ui/api_tokenize.rs, Cargo.toml |
| 2 | 7fd8001 | feat(06-03): create tokenize and detokenize panel templates | templates/ui/tokenize.html, templates/ui/detokenize.html |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added axum multipart feature**
- **Found during:** Task 1
- **Issue:** `axum::extract::Multipart` requires the `multipart` feature flag which was not enabled
- **Fix:** Added `"multipart"` to axum features in Cargo.toml
- **Files modified:** Cargo.toml
- **Commit:** 8458313

**2. [Rule 1 - Bug] Fixed multipart field type inference**
- **Found during:** Task 1
- **Issue:** Rust compiler could not infer types for multipart field `.text()` and `.bytes()` return values
- **Fix:** Added explicit type annotations for `String` and `axum::body::Bytes` in match arms
- **Files modified:** src/ui/api_tokenize.rs
- **Commit:** 8458313

## Verification

- `cargo check` succeeds with no errors (8 warnings from other modules)
- `api_tokenize` handles file_path, content, and file input sources
- `api_detokenize` handles content input
- Both use `spawn_blocking` for CPU-intensive work
- Both use `Store::with_passphrase` with session key
- File path validated with `is_file()` (T-06-02)
- Output HTML-escaped (T-06-06)
- Templates contain all required HTMX attributes and input methods
- Drag-and-drop zone with Alpine.js reactive state

## Self-Check: PASSED
