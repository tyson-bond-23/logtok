# Phase 2: Detection & Token Store - Context

**Gathered:** 2026-04-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Expand sensitive data detection from 7 hardcoded regex categories to comprehensive coverage across credentials, PII, infrastructure, and network identifiers. Add per-project TOML configuration for enabling/disabling categories and defining custom patterns. Add dry-run preview mode. Persist token mappings in an AES-256-GCM encrypted local store that survives across CLI invocations.

NOT in scope: LLM-based contextual detection (v2), Claude API integration (Phase 3), de-tokenization (Phase 3), subcommands (Phase 3).

</domain>

<decisions>
## Implementation Decisions

### Detection Expansion
- **D-01:** Comprehensive regex-based detection — all categories active by default, no opt-in required
- **D-02:** New categories beyond Phase 1's 7 (EMAIL, URL, KEY, PASS, IP, HOST, PATH):
  - `CONN` — Connection strings: Kafka brokers, Redis URIs, PostgreSQL/MySQL/MongoDB URIs
  - `PHONE` — Phone numbers, international formats
  - `UUID` — UUIDs and session IDs
  - `ARN` — AWS ARNs and cloud resource identifiers (AWS account IDs, GCP project IDs, Azure resource IDs)
  - `SSN` — US Social Security Numbers
  - `CC` — Credit card numbers (major card patterns, Luhn-validatable)
  - `JWT` — JWT tokens (`eyJ...` three-part base64 structure)
  - `PEM` — Private keys and PEM blocks (`-----BEGIN ... PRIVATE KEY-----`)
  - `MAC` — MAC addresses
  - `DNS` — Full domain names (broader than current HOST which only matches `.internal/.local/.corp` etc.)
  - `OS` — OS type and version strings (`Linux 5.15.0`, `Windows NT 10.0`, `Darwin 23.1.0`)
  - `NAME` — Usernames and names in structured contexts only
- **D-03:** Name/username detection is structured-only — `user=X`, `"name": "X"`, `author: X`, `"username": "jdoe"` patterns. No free-text NLP name detection in v1.
- **D-04:** Existing Phase 1 HOST pattern (`.internal/.local/.corp`) remains as-is; DNS category covers broader domain matching

### Configuration Design
- **D-05:** Per-project config file: `.logtok.toml` in project root
- **D-06:** TOML format — human-editable, language-neutral, Rust-native (`toml` crate)
- **D-07:** Config discovery: `logtok` walks up from CWD to find `.logtok.toml`
- **D-08:** All detection categories enabled by default — zero-config for the common case. Config only needed to disable specific categories or add custom regex patterns.
- **D-09:** No global/user-level config for v1 — per-project only

### Token Store Design
- **D-10:** Store location: `.logtok/store.enc` in the project directory (next to `.logtok.toml`)
- **D-11:** Encryption: AES-256-GCM with key derived via Argon2 from `LOGTOK_KEY` environment variable
- **D-12:** No interactive passphrase prompt — env var only. CLI errors clearly if `LOGTOK_KEY` is not set when store operations are needed.
- **D-13:** Append-only store with optional TTL expiry — new values accumulate, old entries expire after configurable TTL (set in `.logtok.toml`)
- **D-14:** Manual reset via `--reset-store` flag to wipe and start fresh
- **D-15:** Store format: serialized via serde, encrypted as single blob (consistent with CLAUDE.md architecture notes)

### Dry-Run Mode
- **D-16:** Claude's discretion — output format, detail level, and presentation of dry-run preview

### Claude's Discretion
- Dry-run output format and detail level
- Regex pattern specifics and ordering/priority for new categories
- Error handling for malformed config files
- Store migration strategy if format changes
- TTL default value and granularity

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Technology Stack
- `CLAUDE.md` — Full technology stack with version pins (aes-gcm 0.10.3, argon2 0.5.3, rand 0.9+, serde 1.0.228), architecture notes, and encryption pipeline design

### Existing Implementation
- `src/detector.rs` — Current `DetectionPatterns` struct with 7 hardcoded regex patterns. Must be refactored to support configurable/extensible patterns.
- `src/tokenizer.rs` — Current `TokenMap` (in-memory HashMap). Must be extended with serde serialization for encrypted persistence.
- `src/cli.rs` — Current clap CLI args. Must be extended with `--dry-run`, `--reset-store`, and config-related flags.
- `Cargo.toml` — Current dependencies. Must add: `toml`, `aes-gcm`, `argon2`, `rand`.

### Requirements
- `.planning/REQUIREMENTS.md` — DET-01 through DET-05, TOK-03, TOK-04 are the target requirements for this phase

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `DetectionPatterns` (`src/detector.rs`) — Priority-ordered regex Vec with overlap resolution. Architecture supports expansion, but patterns are hardcoded in `new()`. Needs refactoring to load from config + built-in defaults.
- `TokenMap` (`src/tokenizer.rs`) — Deterministic `HashMap<String, String>` with per-category counters. Clean interface for `get_or_insert()`. Needs serde derives and a persistence layer wrapping encryption.
- `DetectionMatch` struct — Already captures category, value, start, end. No changes needed for new categories.
- Block processing pipeline (`src/processor.rs`) — Line-aware block splitting. No changes needed for Phase 2.

### Established Patterns
- Priority-based overlap resolution in detector (earlier pattern wins)
- Right-to-left replacement in tokenizer to preserve byte offsets
- KEY/PASS patterns use capture groups to extract only the secret value — new patterns like CONN should follow this approach

### Integration Points
- `DetectionPatterns::new()` — needs to accept config-driven pattern list instead of hardcoding
- `TokenMap` — needs `save()`/`load()` methods with encryption
- `Cli` struct — needs new flags: `--dry-run`, `--reset-store`, `--config`
- `.logtok/` directory — new project-level directory for store and potentially config overrides

</code_context>

<specifics>
## Specific Ideas

- Tool is designed as a generic, language-agnostic package — engineer drops `.logtok.toml` in any project (Python, Java, Node, etc.) and runs `logtok` from that directory
- MVP priority — ship fast without losing quality. Comprehensive detection via regex, defer LLM-based detection to v2.
- `.logtok/` directory gets a single `.gitignore` entry to prevent accidental commit of encrypted store

</specifics>

<deferred>
## Deferred Ideas

- **LLM-based contextual name/PII detection** — Run local LLM (Ollama/llama.cpp) for NER on log lines. Deferred to v2 due to single-binary constraint, performance impact on GB-scale files, and non-deterministic output. Could be an optional `--deep-scan` mode with external Ollama endpoint.
- Subcommand structure (`logtok tokenize`, `logtok diagnose`) — Phase 3
- Global/user-level config (`~/.config/logtok/`) — v2, if per-project proves insufficient

</deferred>

---

*Phase: 02-detection-token-store*
*Context gathered: 2026-04-14*
