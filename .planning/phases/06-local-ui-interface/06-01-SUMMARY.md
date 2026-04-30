---
phase: 06-local-ui-interface
plan: 01
name: "Server Foundation"
subsystem: ui
tags: [axum, websocket, rust-embed, cli, server]
dependency_graph:
  requires: []
  provides: [ui-module-tree, axum-server, websocket-heartbeat, static-assets, cli-ui-command]
  affects: [Cargo.toml, src/main.rs, src/cli.rs]
tech_stack:
  added: [axum-0.8, askama_web-0.15, rust-embed-8, tower-http-0.6, tower-0.5, open-5, hex-0.4, mime_guess-2, tokio-1]
  patterns: [axum-state-extraction, rust-embed-static-serving, websocket-heartbeat-auto-stop, tokio-runtime-on-demand]
key_files:
  created:
    - src/ui/mod.rs
    - src/ui/routes.rs
    - src/ui/handlers.rs
    - src/ui/api_tokenize.rs
    - src/ui/api_store.rs
    - src/ui/api_config.rs
    - src/ui/ws.rs
    - src/ui/assets.rs
    - src/ui/i18n.rs
    - templates/ui/base.html
    - templates/ui/tokenize.html
    - templates/ui/detokenize.html
    - templates/ui/store.html
    - templates/ui/docs.html
    - templates/ui/config.html
    - static/styles.css
    - static/app.js
    - tests/ui_server.rs
  modified:
    - Cargo.toml
    - src/cli.rs
    - src/main.rs
decisions:
  - "Used tokio::runtime::Runtime::new() for ui command only, keeping sync main unchanged"
  - "Used unsafe block for env::set_var per Rust 2024 edition requirements"
  - "Session key uses hex-encoded 32 random bytes via rand + hex crates"
metrics:
  duration: 354s
  completed: "2026-04-30T07:09:00Z"
  tasks_completed: 3
  tasks_total: 3
---

# Phase 06 Plan 01: Server Foundation Summary

Axum HTTP server with WebSocket heartbeat auto-stop, rust-embed static assets, Askama template rendering, and full CLI integration via `logtok ui` command

## What Was Built

### Server Infrastructure (src/ui/mod.rs)
- `start_server(port)` async function: binds to 127.0.0.1, scans ports if busy, opens browser, graceful shutdown via oneshot channel
- `find_available_port()`: scans 100-port range from preferred port
- `generate_session_key()`: 32 random bytes hex-encoded, sets LOGTOK_KEY env var for Store interop
- AppState struct shared via Arc with shutdown channel and session key

### WebSocket Heartbeat (src/ui/ws.rs)
- Client expected to send ping every 5 seconds
- Server triggers graceful shutdown after 15 seconds of silence (browser tab closed)
- Uses axum WebSocketUpgrade extractor with State for shutdown channel access

### Route Registration (src/ui/routes.rs)
- `GET /` -- dashboard page (Askama template)
- `POST /api/tokenize`, `POST /api/detokenize` -- stub handlers
- `GET /api/store`, `GET /api/docs` -- stub handlers
- `GET /api/config`, `PUT /api/config` -- stub handlers
- `GET /ws/heartbeat` -- WebSocket auto-stop
- `GET /static/{*path}` -- rust-embed static files
- CompressionLayer middleware applied

### Static Asset Serving (src/ui/assets.rs)
- rust-embed `#[derive(Embed)]` with `#[folder = "static/"]`
- MIME type detection via mime_guess
- Cache-Control header (1 hour)

### i18n (src/ui/i18n.rs)
- 34 translation keys for English and Hebrew
- HashMap-based lookup, sufficient for 2 languages

### CLI Integration
- `Ui { port: Option<u16> }` variant added to Commands enum
- `Commands::Ui` match arm in main.rs creates tokio Runtime on demand
- Keeps synchronous main() unchanged for all other commands

### Template and Static Stubs
- 6 HTML templates in templates/ui/ (base.html with version variable, 5 panel placeholders)
- CSS placeholder with dark theme background color
- JS placeholder with Alpine.js app() stub

## Commits

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Add dependencies and file scaffolding | d271bd3 | Cargo.toml, templates/ui/*, static/* |
| 2 | Create src/ui/ Rust modules and CLI integration | f12e70f | src/ui/*.rs, src/cli.rs, src/main.rs |
| 3 | Verify build and CLI integration tests | a02a576 | tests/ui_server.rs |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added unsafe block for env::set_var**
- **Found during:** Task 2
- **Issue:** Rust 2024 edition (edition = "2024") marks `std::env::set_var` as unsafe
- **Fix:** Wrapped in `unsafe { std::env::set_var(...) }` with `#[allow(deprecated)]` annotation
- **Files modified:** src/ui/mod.rs

## Known Stubs

| File | Description | Resolved By |
|------|-------------|-------------|
| src/ui/api_tokenize.rs | Returns placeholder HTML, not wired to core engine | Plan 03 |
| src/ui/api_store.rs | Returns placeholder HTML, not wired to Store | Plan 04 |
| src/ui/api_config.rs | Returns placeholder HTML, not wired to Config | Plan 04 |
| src/ui/handlers.rs | Dashboard renders minimal base.html with version only | Plan 02 |
| templates/ui/base.html | Placeholder page, no tabs/theme/RTL | Plan 02 |
| static/styles.css | Minimal dark background only | Plan 02 |
| static/app.js | Stub app() function | Plan 02 |

All stubs are intentional scaffolding that subsequent plans (02-04) replace. The plan's goal (compilable server infrastructure) is fully achieved.

## Threat Surface

No new threat surface beyond what the plan's threat model already documents. Server binds 127.0.0.1 only (T-06-01 mitigated). Session key never sent to browser (T-06-04 mitigated).

## Self-Check: PASSED

All 13 created files verified on disk. All 3 commit hashes verified in git log.
