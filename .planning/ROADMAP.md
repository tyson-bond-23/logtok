# Roadmap: Logs Tokeniser

## Milestones

- **v1.0 MVP** — Phases 1-3 (shipped 2026-04-28)
- **v2.0 Developer Experience** — Phases 4-7 (in progress)

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

<details>
<summary>v1.0 MVP (Phases 1-3) — SHIPPED 2026-04-28</summary>

- [x] Phase 1: Core Tokenization Engine (3/3 plans) — complete
- [x] Phase 2: Detection & Token Store (3/3 plans) — complete
- [x] Phase 3: Diagnosis & Delivery (3/3 plans) — complete

</details>

### v2.0 Developer Experience (In Progress)

- [x] **Phase 4: Colored CLI Help** - Styled, color-coded --help output with cross-platform terminal support (complete 2026-04-29)
- [x] **Phase 5: HTML Documentation** - Auto-generated single-file HTML docs from clap command tree (complete 2026-04-29)
- [ ] **Phase 6: Local UI Interface** - Interactive browser dashboard with tokenize/detokenize panels, store browser, config editor, dark/light themes, and English/Hebrew RTL support
- [ ] **Phase 7: Tokenization Report** - Verbose mode showing which data was tokenized per category

## Phase Details

### Phase 4: Colored CLI Help
**Goal**: Users see a polished, readable CLI help experience with colored headers, bold flags, and styled usage — across all platforms
**Depends on**: Phase 3 (v1.0 complete)
**Requirements**: HELP-01, HELP-02, HELP-03
**Success Criteria** (what must be TRUE):
  1. Running `logtok --help` displays colored section headers, bold flag names, and a styled usage line
  2. Setting `NO_COLOR=1` or unsetting `CLICOLOR` causes help output to render without any ANSI color codes
  3. Help output renders correctly (no garbled escape sequences) on Windows cmd.exe, PowerShell, and Unix terminals
**Plans**: 1 plan

Plans:
- [x] 04-01-PLAN.md -- Add colored styles, help content, and integration tests

### Phase 5: HTML Documentation
**Goal**: Users can generate a professional, self-contained HTML documentation page directly from the CLI — always in sync with actual commands
**Depends on**: Phase 4
**Requirements**: DOCS-01, DOCS-02, DOCS-03, DOCS-04, DOCS-05, DOCS-06, DOCS-07
**Success Criteria** (what must be TRUE):
  1. Running `logtok docs` produces a single HTML file with embedded CSS and JS — no external dependencies
  2. The HTML page contains an install/getting-started guide and full command reference for all subcommands with flags, arguments, and descriptions
  3. All code examples have copy-to-clipboard buttons that work in Chrome, Firefox, Safari, and Edge (including file:// URLs)
  4. Adding or changing a CLI subcommand in code automatically updates the generated HTML without manual edits (docs derived from clap Command tree)
  5. The HTML page has a clean, professional design appropriate for developer and DevOps audiences
**Plans**: 2 plans

Plans:
- [x] 05-01-PLAN.md — Create Askama HTML template with dark theme, sidebar, content sections, and copy-to-clipboard JS
- [x] 05-02-PLAN.md — Build docs.rs module with clap introspection, CLI integration, and integration tests

### Phase 6: Local UI Interface
**Goal**: Users can access and use logtok through a polished, interactive browser dashboard — tokenize/detokenize panels, token store browser, config editor, and docs reference — with dark/light themes and English/Hebrew RTL support
**Depends on**: Phase 5
**Requirements**: UI-01, UI-02, UI-03, UI-04, UI-05, UI-06, UI-07, UI-08, UI-09, UI-10, UI-11, UI-12, UI-13, UI-14
**Success Criteria** (what must be TRUE):
  1. Running `logtok ui` starts a local HTTP server on 127.0.0.1 and auto-opens the browser to an interactive dashboard
  2. Dashboard has five tabs (Tokenize, Detokenize, Token Store, Config, Docs) with top tab bar navigation
  3. Dark theme uses modern dev tool aesthetic (#0f0f10 bg, #6366f1 primary), light theme uses warm white (#fafaf5 range), theme follows OS preference
  4. English and Hebrew language support with full RTL layout flip when Hebrew is selected
  5. Tokenize panel accepts input via drag-and-drop, file picker, paste text, and file path input
  6. Config editor provides form-based view with category toggles and raw TOML toggle for power users
  7. Server auto-stops when browser tab is closed via WebSocket heartbeat detection
  8. All assets embedded in binary via rust-embed — single binary distribution preserved
**Plans**: 5 plans

Plans:
- [x] 06-01-PLAN.md — Server foundation: deps, ui module tree, axum server, WebSocket heartbeat, rust-embed assets, CLI integration
- [ ] 06-02-PLAN.md — Dashboard shell: HTMX + Alpine.js, CSS themes (dark/light/RTL), base template, tab navigation, i18n
- [ ] 06-03-PLAN.md — Tokenize + Detokenize panels: API handlers with core engine integration, HTMX templates with 4 input methods
- [ ] 06-04-PLAN.md — Store browser + Docs + Config: API handlers for store/docs/config, HTMX templates with form/TOML editor
- [ ] 06-05-PLAN.md — Integration tests + human verification checkpoint

### Phase 7: Tokenization Report
**Goal**: Users can get a detailed report on which data was tokenized — verbose mode showing categories, counts, and patterns matched
**Depends on**: Phase 3 (core tokenization)
**Requirements**: TBD
**Success Criteria** (what must be TRUE):
  1. TBD — define during discuss-phase
**Plans**: TBD

Plans:
- [ ] 07-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 4 -> 5 -> 6 -> 7

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Core Tokenization Engine | v1.0 | 3/3 | Complete | 2026-04-14 |
| 2. Detection & Token Store | v1.0 | 3/3 | Complete | 2026-04-15 |
| 3. Diagnosis & Delivery | v1.0 | 3/3 | Complete | 2026-04-16 |
| 4. Colored CLI Help | v2.0 | 1/1 | Complete | 2026-04-29 |
| 5. HTML Documentation | v2.0 | 2/2 | Complete | 2026-04-29 |
| 6. Local UI Interface | v2.0 | 0/5 | Planning complete | - |
| 7. Tokenization Report | v2.0 | 0/? | Not started | - |
