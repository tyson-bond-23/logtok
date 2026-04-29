# Phase 5: HTML Documentation - Context

**Gathered:** 2026-04-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a `logtok docs` subcommand that generates a self-contained, single-file HTML documentation page derived from the clap Command tree. The page must stay automatically in sync with CLI changes — no manual edits to docs content.

</domain>

<decisions>
## Implementation Decisions

### Visual Design & Layout
- **D-01:** Use CLI-matched dark theme — dark background (#1a1a2e), yellow bold headers, green code/commands, cyan links/accents, light gray body text. Mirrors the terminal --help aesthetic for cohesion.
- **D-02:** Single scroll page with collapsible sidebar navigation. Sidebar visible by default on desktop (>768px), hidden behind hamburger toggle on mobile. Hamburger icon toggles sidebar open/closed.
- **D-03:** Code blocks use slightly lighter dark background with monospace font. System font stack for body text (no web fonts — per REQUIREMENTS.md out-of-scope).

### Content Structure
- **D-04:** Section ordering: Getting Started (install + quick start) → Overview (what logtok does, workflow) → Command Reference (tokenize/detokenize/reset-store with all flags) → Token Categories reference table (all 19 categories).
- **D-05:** Sidebar nav mirrors section headings with nested links for each subcommand under Commands.

### Templating Approach
- **D-06:** Use Askama 0.15 compile-time templates. HTML template files in `templates/` directory, compiled into the binary. Type-safe, zero runtime cost, catches template errors at build time.
- **D-07:** One new crate dependency: `askama` (already noted in CLAUDE.md stack decisions).
- **D-08:** Extract command metadata from clap's `Command::build()` at runtime — iterate subcommands, args, flags, about text, long_about, after_long_help to populate template data structs.

### Copy-to-Clipboard UX
- **D-09:** Small clipboard icon button in top-right corner of each code block. Uses CSS `position: absolute` within a `position: relative` code block wrapper.
- **D-10:** On click: icon changes to checkmark (✓) for 2 seconds, then reverts. Uses `navigator.clipboard.writeText()` with `document.execCommand('copy')` fallback for older browsers and file:// URLs.
- **D-11:** All JS for copy functionality embedded inline in the HTML file (no external scripts). Minimal vanilla JS — no frameworks.

### Claude's Discretion
- Exact CSS values (spacing, font sizes, border radius) — as long as it looks professional
- Responsive breakpoint fine-tuning beyond the 768px mobile threshold
- Whether to include a "back to top" button
- Internal HTML structure (divs, sections, semantic HTML choices)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Stack
- `CLAUDE.md` — Technology stack decisions including askama 0.15 recommendation
- `.planning/REQUIREMENTS.md` — DOCS-01 through DOCS-07 requirements + out-of-scope items

### Prior Phase Context
- `.planning/phases/04-colored-cli-help/04-CONTEXT.md` — Color palette decisions (D-01 through D-03) that should be carried forward to HTML theme

### Source Code
- `src/cli.rs` — Current CLI structure with all subcommands, args, and help text (the source of truth for docs content)
- `Cargo.toml` — Current dependencies

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/cli.rs` — Full CLI definition with Parser/Subcommand derive macros, doc comments with examples, after_long_help blocks. This is the source of truth for all documentation content.
- `STYLES` const in cli.rs — Color palette (yellow/green/cyan/red) to replicate in HTML CSS

### Established Patterns
- clap derive macros used throughout — `Command::build()` can introspect the full command tree at runtime
- Single binary, zero runtime dependency philosophy

### Integration Points
- New `Docs` variant added to `Commands` enum in cli.rs
- New `docs.rs` module for HTML generation logic
- `templates/` directory at crate root for Askama HTML templates
- Main.rs match arm to handle `Commands::Docs` → generate and write HTML file

</code_context>

<specifics>
## Specific Ideas

- Dark theme should feel like you're reading docs inside a terminal — cohesive with `logtok --help` output
- Token categories table should show all 19 categories with their description (same as CLAUDE.md reference table)
- Getting Started should show the 3-step workflow: tokenize → send to Claude → detokenize

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 05-html-documentation*
*Context gathered: 2026-04-29*
