# Requirements: Logs Tokeniser

**Defined:** 2026-04-28
**Core Value:** Engineers can diagnose production log errors through Claude Code without revealing any sensitive information

## v2.0 Requirements

Requirements for developer experience milestone. Each maps to roadmap phases.

### CLI Help Styling

- [x] **HELP-01**: User sees colored section headers, bold flag names, and styled usage line in --help output
- [x] **HELP-02**: CLI respects NO_COLOR and CLICOLOR environment variables (colors disabled when set)
- [x] **HELP-03**: Help output renders correctly on Windows cmd.exe, PowerShell, and Unix terminals

### HTML Documentation

- [x] **DOCS-01**: User can run `logtok docs` to generate a self-contained HTML documentation file
- [x] **DOCS-02**: HTML page includes an install/getting-started guide with step-by-step instructions
- [x] **DOCS-03**: HTML page includes full command reference for all subcommands with flags, arguments, and descriptions
- [x] **DOCS-04**: All code examples in HTML have copy-to-clipboard buttons that work across browsers
- [x] **DOCS-05**: HTML is a single file with embedded CSS and JS -- no external dependencies
- [x] **DOCS-06**: HTML page has a clean, professional design targeted at developers and DevOps engineers
- [x] **DOCS-07**: Generated HTML stays in sync with actual CLI commands (derived from clap Command tree, not hardcoded)

### Local UI Interface

- [x] **UI-01**: Running `logtok ui` starts a local HTTP server on 127.0.0.1 and auto-opens the default browser
- [ ] **UI-02**: Dashboard has five tabs: Tokenize, Detokenize, Token Store, Config, Docs -- with top tab bar navigation
- [x] **UI-03**: Dark theme with modern dev tool aesthetic (#0f0f10 background, #6366f1 primary, #1c1c1e surface)
- [x] **UI-04**: Light theme with warm white palette (#fafaf5 range) -- theme follows OS preference via prefers-color-scheme
- [ ] **UI-05**: English and Hebrew language support with full RTL layout flip when Hebrew is selected
- [x] **UI-06**: Tokenize panel accepts input via drag-and-drop, file picker, paste text area, and file path input
- [ ] **UI-07**: Detokenize panel accepts tokenized text and replaces tokens with real values from the session store
- [ ] **UI-08**: Token Store browser displays current token mappings from the session store
- [ ] **UI-09**: Config editor with form-based view (toggles per category, text fields) and raw TOML editor toggle
- [ ] **UI-10**: Docs tab displays command reference and token categories derived from clap Command tree
- [x] **UI-11**: Server auto-stops when browser tab is closed via WebSocket heartbeat detection
- [x] **UI-12**: All frontend assets (HTML, CSS, JS) are embedded in the binary via rust-embed -- single binary distribution
- [x] **UI-13**: Encryption key is auto-generated per session -- no manual key entry required
- [x] **UI-14**: Language preference, last active tab, and recent file paths persist via localStorage across sessions

## v2.1+ Requirements

Deferred to future release. Tracked but not in current roadmap.

### CLI Help Enhancements

- **HELP-04**: Inline usage examples shown in after_long_help for each subcommand
- **HELP-05**: Automatic word wrapping on narrow terminals (clap wrap_help feature)

### HTML Documentation Enhancements

- **DOCS-08**: Dark/light theme toggle matching OS preference
- **DOCS-09**: 19-category reference table showing what logtok detects
- **DOCS-10**: Man page generation from clap metadata
- **DOCS-11**: Shell completion scripts generation

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Multi-page documentation site | Overkill -- single-file HTML is portable and sufficient |
| Hosted docs (GitHub Pages) | Adds deployment complexity, defer to later |
| Syntax highlighting JS (Prism.js) | Binary bloat risk, not needed for command reference |
| Markdown output format | HTML is the target; markdown can be added later |
| CSS framework (Tailwind, Bootstrap) | Binary bloat, violates <50KB budget |
| Web fonts | Binary bloat, system font stack is professional enough |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| HELP-01 | Phase 4 | Complete |
| HELP-02 | Phase 4 | Complete |
| HELP-03 | Phase 4 | Complete |
| DOCS-01 | Phase 5 | Complete |
| DOCS-02 | Phase 5 | Complete |
| DOCS-03 | Phase 5 | Complete |
| DOCS-04 | Phase 5 | Complete |
| DOCS-05 | Phase 5 | Complete |
| DOCS-06 | Phase 5 | Complete |
| DOCS-07 | Phase 5 | Complete |
| UI-01 | Phase 6 | Not started |
| UI-02 | Phase 6 | Not started |
| UI-03 | Phase 6 | Not started |
| UI-04 | Phase 6 | Not started |
| UI-05 | Phase 6 | Not started |
| UI-06 | Phase 6 | Not started |
| UI-07 | Phase 6 | Not started |
| UI-08 | Phase 6 | Not started |
| UI-09 | Phase 6 | Not started |
| UI-10 | Phase 6 | Not started |
| UI-11 | Phase 6 | Not started |
| UI-12 | Phase 6 | Not started |
| UI-13 | Phase 6 | Not started |
| UI-14 | Phase 6 | Not started |

**Coverage:**
- v2.0 requirements: 24 total
- Mapped to phases: 24
- Unmapped: 0

---
*Requirements defined: 2026-04-28*
*Last updated: 2026-04-29 after Phase 06 planning*
