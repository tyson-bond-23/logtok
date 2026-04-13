# Phase 1: Core Tokenization Engine - Context

**Gathered:** 2026-04-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Build the core tokenization pipeline: a Rust CLI (`logtok`) that reads log files (JSON and plain text), detects sensitive data via regex, replaces matches with deterministic category-prefixed tokens, and outputs a compact tokenized version — processing large files with bounded memory via block-based streaming.

Compaction is IN scope: the tool collapses duplicate log lines with counts to minimize Claude token usage while preserving every unique event.

NOT in scope: encrypted token store (Phase 2), detection configuration (Phase 2), Claude API integration (Phase 3), Claude Code skill (Phase 3).

</domain>

<decisions>
## Implementation Decisions

### Token Format
- **D-01:** Square bracket delimiters — `[CATEGORY_NNN]`
- **D-02:** Category-only prefix, no `TOK_` prefix — `[IP_001]`, `[KEY_001]`, `[EMAIL_001]` not `[TOK_IP_001]`
- **D-03:** Per-category counters — each category starts at 001 independently
- **D-04:** 3-digit zero-padded counters — `[IP_001]` through `[IP_999]`
- **D-05:** Category names: `IP`, `KEY`, `EMAIL`, `HOST`, `PATH`, `PASS`, `URL` (short, uppercase)

### Log Compaction
- **D-06:** Structural compaction applied during tokenization — collapse duplicate lines with count, normalize whitespace, preserve every unique log event and ordering
- **D-07:** Compaction optimized for LLM consumption — minimize Claude token usage while retaining full diagnostic value
- **D-08:** Companion Claude Code skill to interpret compact+tokenized format — deferred to Phase 3

### Block Processing
- **D-09:** Line-aware splitting — read configurable-size chunks but always split at newline boundaries, no value spans two blocks
- **D-10:** Default block size 64KB, configurable via `--block-size` flag

### JSON Handling
- **D-11:** Tokenized JSON output remains valid parseable JSON — tokens replace values as strings
- **D-12:** Values only — JSON keys are not tokenized, only values are scanned and replaced

### CLI Interface
- **D-13:** Binary name: `logtok`
- **D-14:** Flat CLI (no subcommands) — `logtok <file>` tokenizes. Subcommands added in later phases.
- **D-15:** Output to stdout by default, `--output` / `-o` flag for file output
- **D-16:** Progress bar on stderr (via indicatif) for large files, `--quiet` flag to suppress
- **D-17:** Configurable block size via `--block-size` flag

### Claude's Discretion
- Error message formatting and exit codes
- Internal logging levels and tracing configuration
- Regex pattern ordering and optimization strategy
- Memory-mapped file vs buffered reader decision for different file sizes

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Technology Stack
- `CLAUDE.md` — Full technology stack with version pins, architecture notes, and alternatives considered

No external specs — requirements fully captured in decisions above and REQUIREMENTS.md.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield project, no existing code

### Established Patterns
- None yet — this phase establishes the foundational patterns (project structure, error handling, module organization)

### Integration Points
- Phase 2 will add encrypted token store — the in-memory token map from Phase 1 needs a clean interface for persistence
- Phase 3 will add Claude API + de-tokenization — tokenized output format must be stable

</code_context>

<specifics>
## Specific Ideas

- Token format designed for minimal Claude token usage — short category names, no redundant prefix
- Structural compaction (duplicate collapsing with counts) specifically to reduce LLM input tokens and processing time
- A future Claude Code skill will be purpose-built to understand the compact+tokenized format (Phase 3)

</specifics>

<deferred>
## Deferred Ideas

- Claude Code skill for interpreting compact+tokenized logs — Phase 3 (Diagnosis & Delivery)
- Subcommand structure (`logtok tokenize`, `logtok diagnose`, `logtok detokenize`) — Phase 3
- Aggressive compaction (stack trace trimming, error grouping) — future enhancement if structural compaction proves insufficient

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-core-tokenization-engine*
*Context gathered: 2026-04-13*
