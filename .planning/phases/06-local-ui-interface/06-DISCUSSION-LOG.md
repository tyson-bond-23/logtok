# Phase 6: Local UI Interface - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-29
**Phase:** 06-local-ui-interface
**Areas discussed:** Interface type, Feature scope, Navigation & interaction, Visual design, File upload handling, Security model, Server lifecycle, Config editor details

---

## Interface Type

| Option | Description | Selected |
|--------|-------------|----------|
| TUI | Terminal UI using ratatui, runs in terminal with panels | |
| Local web dashboard | localhost web server serving HTML dashboard in browser | |
| Enhanced CLI wizard | Interactive CLI wizard using dialoguer/inquire prompts | |

**User's choice:** Local server — user wanted the best UX and docs-style visual quality. Clarified they wanted to combine docs with interactive UI. Browser-based approach was confirmed as best practice (Grafana, Jupyter, Vite pattern).
**Notes:** PROJECT.md listed "Web UI or dashboard" as out of scope, but user chose to proceed with localhost-only dashboard. The todo referenced "uipro-cli" skill.

---

## Feature Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Docs (carry forward) | Existing HTML docs becomes a tab in dashboard | ✓ |
| Tokenize panel | Paste/upload log file, tokenize, see results | ✓ |
| Detokenize panel | Paste Claude response, detokenize, see restored output | ✓ |
| Token store browser | Searchable table of all token mappings | ✓ |

**User's choice:** All four options selected, plus user added: config editor with blacklist management (names/variables) and .logtok.toml editing.
**Notes:** Config editor was user-initiated addition — not in original options.

---

## Navigation & Interaction

| Option | Description | Selected |
|--------|-------------|----------|
| Top tab bar | Horizontal tabs at top, content below | ✓ |
| Sidebar navigation | Vertical sidebar on left | |
| Single-page scroll | All sections on one scrollable page | |

**User's choice:** Top tab bar. User specified: add dark/light theme toggle button and Hebrew language option in the top nav bar.
**Notes:** User directly stated their preference rather than selecting from options.

---

## Visual Design — Dark Theme

| Option | Description | Selected |
|--------|-------------|----------|
| Carry forward Phase 5 | #1a1a2e dark background, yellow/green/cyan accents | |
| Fresh design | New color scheme for dashboard | ✓ |
| You decide | Claude picks | |

**User's choice:** Fresh design. User provided exact color palette:
- Background: #0f0f10, Surface: #1c1c1e, Border: #2a2a2c
- Primary: #6366f1, Success: #22c55e, Error: #ef4444
- Text: #e4e4e7, Muted: #71717a

---

## Visual Design — Light Theme

| Option | Description | Selected |
|--------|-------------|----------|
| Clean white | White/light gray background | |
| Warm white | Slightly warm/cream background (#fafaf5 range) | ✓ |
| You decide | Claude picks | |

**User's choice:** Warm white

---

## Visual Design — Default Theme

| Option | Description | Selected |
|--------|-------------|----------|
| Follow OS preference | Uses prefers-color-scheme media query | ✓ |
| Always dark first | Dark regardless of OS | |
| Always light first | Light regardless of OS | |

**User's choice:** Follow OS preference

---

## RTL Support

| Option | Description | Selected |
|--------|-------------|----------|
| Full RTL | Entire layout mirrors for Hebrew | ✓ |
| Labels only | Translate labels but keep LTR layout | |

**User's choice:** Full RTL layout flip

---

## Languages

| Option | Description | Selected |
|--------|-------------|----------|
| English + Hebrew | Two languages at launch | ✓ |
| English + Hebrew + more | Extensible framework with additional languages | |

**User's choice:** English + Hebrew only for now

---

## Frontend Stack

| Option | Description | Selected |
|--------|-------------|----------|
| HTMX + Alpine.js | ~30KB, server-driven + client-side reactivity | ✓ |
| Vanilla JS only | Zero dependencies, more manual DOM work | |
| React/Vue/Svelte | Full framework, needs build toolchain | |
| You decide | Claude picks | |

**User's choice:** HTMX + Alpine.js

---

## File Upload Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Drag-and-drop zone | Drop file onto panel | ✓ |
| File picker button | Standard browse button | ✓ |
| Paste text area | Paste log content directly | ✓ |
| File path input | Type path, server reads directly | ✓ |

**User's choice:** All four input methods

---

## Security Model

**User's choice:** Auto-generated encryption key per session (no manual LOGTOK_KEY entry). Server binds to 127.0.0.1 only.
**Notes:** User explicitly rejected manual key entry — "it will be auto generate"

---

## Server Lifecycle

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-open browser | Start server and open browser automatically | ✓ |
| Print URL only | Start server, user opens manually | |

**User's choice:** Auto-open browser

| Option | Description | Selected |
|--------|-------------|----------|
| Ctrl+C only | Server runs until manually stopped | |
| Auto-stop | Server stops when browser tab closes | ✓ |

**User's choice:** Auto-stop with localStorage persistence for preferences

---

## Persistence (localStorage)

| Option | Description | Selected |
|--------|-------------|----------|
| Theme preference | Persist dark/light choice | |
| Language preference | Persist English/Hebrew choice | ✓ |
| Last active tab | Reopen to same tab | ✓ |
| Recent files | Remember file paths | ✓ |

**User's choice:** Language, last tab, and recent files. Theme follows OS preference each time (not persisted).

---

## Config Editor

| Option | Description | Selected |
|--------|-------------|----------|
| Form-based | Visual form with toggles and text fields | |
| Raw TOML editor | Syntax-highlighted TOML editor | |
| Both modes | Form default with raw TOML toggle | ✓ |

**User's choice:** Both modes — form view default, raw TOML toggle for power users

---

## Claude's Discretion

- Exact CSS values for light theme adaptation
- Responsive breakpoints
- WebSocket heartbeat interval
- Default port selection strategy
- Internal HTML structure

## Deferred Ideas

None — discussion stayed within phase scope
