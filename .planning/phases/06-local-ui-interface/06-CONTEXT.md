# Phase 6: Local UI Interface - Context

**Gathered:** 2026-04-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a `logtok ui` subcommand that starts a local web server and opens an interactive dashboard in the browser. The dashboard combines documentation, tokenize/detokenize panels, token store browser, and config editor in a polished, responsive interface with dark/light themes and English/Hebrew language support.

</domain>

<decisions>
## Implementation Decisions

### Interface Type
- **D-01:** Local web server approach. `logtok ui` starts an axum HTTP server on `127.0.0.1`, auto-opens the default browser. Interactive dashboard served from embedded assets in the single binary.
- **D-02:** This supersedes the PROJECT.md "Web UI or dashboard — Out of Scope" entry. The UI is a localhost-only developer tool, not a hosted web dashboard.

### Feature Scope
- **D-03:** Five dashboard sections: Docs (carried from Phase 5), Tokenize panel, Detokenize panel, Token Store browser, Config editor.
- **D-04:** Config editor includes blacklist management (names, variables to always tokenize) and custom pattern editing. Visual form view by default with raw TOML toggle for advanced users.

### Navigation & Layout
- **D-05:** Top tab bar navigation: Tokenize | Detokenize | Token Store | Config | Docs. Default landing tab is Tokenize (primary action).
- **D-06:** Top nav bar includes: theme toggle button (dark/light), language selector (English/Hebrew).

### Language & RTL
- **D-07:** Two languages at launch: English and Hebrew.
- **D-08:** Full RTL layout flip when Hebrew is selected — navigation, panels, text direction all mirror. This is correct UX for Hebrew speakers.

### Visual Design — Dark Theme (fresh design)
- **D-09:** Modern dev tool aesthetic (VS Code / Linear / Vercel style). Not carrying forward the Phase 5 docs palette.
- **D-10:** Dark theme palette:
  - Background: #0f0f10 (near black)
  - Surface: #1c1c1e (dark gray)
  - Border: #2a2a2c (subtle)
  - Primary: #6366f1 (indigo)
  - Success: #22c55e (green)
  - Error: #ef4444 (red)
  - Text: #e4e4e7 (light gray)
  - Muted: #71717a (zinc)

### Visual Design — Light Theme
- **D-11:** Warm white light theme (#fafaf5 range). Same accent colors adapted for light backgrounds.

### Theme Default
- **D-12:** Follow OS preference via `prefers-color-scheme` media query. If OS is dark, UI starts dark. If OS is light, UI starts light.

### Tech Stack
- **D-13:** Backend: axum (HTTP server, built on tokio already in project).
- **D-14:** Templating: Askama (already used in Phase 5 for HTML docs).
- **D-15:** Asset embedding: rust-embed (embeds HTML/CSS/JS into binary at compile time).
- **D-16:** Frontend: HTMX (server-driven interactivity) + Alpine.js (client-side reactivity — tabs, theme toggle, RTL). ~30KB total, no build step needed.
- **D-17:** Middleware: tower-http for static file serving and middleware.

### File Input (Tokenize Panel)
- **D-18:** Four input methods: drag-and-drop zone, file picker button, paste text area, and file path input field (server reads file directly — best for large files).

### Security
- **D-19:** Encryption key is auto-generated per session. No manual key entry.
- **D-20:** Server binds to 127.0.0.1 only. Not accessible from the network.

### Server Lifecycle
- **D-21:** `logtok ui` starts server, auto-opens browser to the dashboard.
- **D-22:** Server auto-stops when the browser tab is closed (WebSocket heartbeat detection).

### Persistence (localStorage)
- **D-23:** Language preference persists across sessions.
- **D-24:** Last active tab persists across sessions.
- **D-25:** Recent file paths persist for quick re-selection.
- **D-26:** Theme does NOT persist — follows OS preference each time.

### Config Editor
- **D-27:** Dual-mode config editor: form-based view by default (toggles per category, text fields for patterns/blacklist), with raw TOML editor toggle for power users. Saves to .logtok.toml on submit.

### Claude's Discretion
- Exact CSS values (spacing, font sizes, border radius) for light theme adaptation
- Responsive breakpoints for mobile/tablet
- Warm white exact hex values within the #fafaf5 range
- WebSocket heartbeat interval for auto-stop
- Default port selection strategy (8080 or next available)
- Internal HTML structure and component organization

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Context
- `.planning/PROJECT.md` — Core value, constraints, current milestone goals
- `.planning/REQUIREMENTS.md` — Requirements traceability (Phase 6 requirements TBD)
- `.planning/ROADMAP.md` — Phase 6 goal and dependencies

### Prior Phase Context
- `.planning/phases/05-html-documentation/05-CONTEXT.md` — Askama templating pattern, docs content structure, dark theme decisions (Phase 6 uses fresh design but same templating approach)
- `.planning/phases/04-colored-cli-help/04-CONTEXT.md` — CLI color palette (warm professional), clap styles

### Source Code
- `src/cli.rs` — Current CLI definition with all subcommands (new `Ui` variant needed)
- `src/docs.rs` — Existing HTML docs generation (Askama + clap introspection pattern to reuse)
- `src/main.rs` — Command dispatch (new `Commands::Ui` match arm)
- `src/config.rs` — Config loading/parsing (API endpoints needed for config editor)
- `src/store.rs` — Token store (API endpoints needed for store browser)
- `src/processor.rs` — Tokenization processing (API endpoint needed for tokenize panel)
- `src/detokenizer.rs` — Detokenization logic (API endpoint needed for detokenize panel)
- `Cargo.toml` — Current dependencies (axum, rust-embed, tower-http to be added)
- `templates/docs.html` — Existing Askama HTML template (reference for new UI templates)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/docs.rs` — Askama template pattern with `CommandInfo`, `ArgInfo`, `TokenCategory` structs. Same introspection approach can feed the Docs tab.
- `templates/docs.html` — Existing HTML template. Content can be embedded as a tab in the new dashboard.
- `STYLES` const in `cli.rs` — CLI color palette reference (dashboard uses fresh design but same brand identity).
- `src/processor.rs::process_file_with_config()` — Core tokenization function to expose via API.
- `src/detokenizer.rs::detokenize()` — Core detokenization function to expose via API.
- `src/store.rs::Store` — Token store with load/save/reset, expose via API for store browser.
- `src/config.rs::LoktokConfig` — Config struct with serde, expose via API for config editor.

### Established Patterns
- Askama compile-time templating — used in Phase 5, reuse for dashboard templates
- clap derive macros for CLI — add new `Ui` subcommand variant
- Single binary philosophy — rust-embed keeps assets compiled in
- tokio async runtime — axum is built on tokio, no runtime conflict

### Integration Points
- New `Ui` variant in `Commands` enum in `cli.rs` with `--port` optional flag
- New `ui.rs` module for axum server setup, routes, and handlers
- New `templates/ui/` directory for dashboard Askama templates (or embedded HTML)
- API routes: `POST /api/tokenize`, `POST /api/detokenize`, `GET /api/store`, `GET /api/config`, `PUT /api/config`
- WebSocket endpoint for heartbeat auto-stop detection

</code_context>

<specifics>
## Specific Ideas

- Dashboard should feel like a modern dev tool (VS Code, Linear, Vercel) — not a generic web app
- Hebrew support must be full RTL layout flip, not just translated labels
- Config editor should make it easy for non-technical users to add names/variables to the blacklist without knowing TOML syntax
- File path input in Tokenize panel is key for large files — server reads directly instead of browser upload
- Auto-stop via WebSocket heartbeat keeps things clean — no orphan servers

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 06-local-ui-interface*
*Context gathered: 2026-04-29*
