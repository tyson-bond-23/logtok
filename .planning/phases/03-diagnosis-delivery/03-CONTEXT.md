# Phase 3: Diagnosis & Delivery - Context

**Gathered:** 2026-04-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Complete the tokenize-diagnose-de-tokenize loop across three environments:

1. **Private environment:** User runs `loktok` to tokenize raw logs (already built in Phase 1+2)
2. **Public environment:** User pastes tokenized logs into Claude Code, which understands the token format via a CLAUDE.md instruction block and diagnoses errors using tokens as-is (no de-tokenization, no API call)
3. **Private environment:** User takes Claude's response (containing token placeholders) and runs `loktok detokenize` to replace tokens with real values for clear, readable terminal output

Additionally: polish CLI UX (--help, progress bar), create README.md, and set up cross-platform builds.

NOT in scope: Claude API client (removed — Claude Code is the AI, not an API endpoint), web UI, streaming connectors, NLP detection.

</domain>

<decisions>
## Implementation Decisions

### System Architecture (3-Part Flow)
- **D-01:** No Claude API client — removed from scope. Claude Code is the diagnosis engine, not an API endpoint called by loktok.
- **D-02:** The tool's job is (a) tokenize logs and (b) de-tokenize Claude's response. The AI reasoning happens externally in Claude Code.
- **D-03:** A CLAUDE.md instruction block (~200 words) teaches Claude Code the token format. No full skill/plugin — zero performance overhead.

### Claude Code Integration
- **D-04:** CLAUDE.md block explains: token format `[CATEGORY_NNN]`, all 19 categories and what they represent, how to reason about token relationships, instruction to keep tokens in responses for de-tokenization.
- **D-05:** No skill triggers, no tool calls, no custom MCP tools. Just static context in CLAUDE.md.

### De-tokenization
- **D-06:** New subcommand: `loktok detokenize <file>` or pipe via stdin: `echo "..." | loktok detokenize`
- **D-07:** De-tokenization reads the encrypted token store to resolve tokens back to real values.
- **D-08:** Output goes to terminal — clear, formatted, readable. No file output by default.
- **D-09:** Inline replacement — tokens replaced directly in the text. No legend table, no side-by-side.

### Output Formats
- **D-10:** Default tokenize output: stdout. `-o` flag writes to file. Consistent with Phase 1/2.
- **D-11:** Bullet summary on stdout by default for de-tokenized diagnosis.
- **D-12:** `--detailed` flag outputs a full markdown .md file.

### CLI Polish (Folded Todos)
- **D-13:** `--help` text polished with descriptions, usage examples, and section headers via clap attributes.
- **D-14:** Progress bar using `indicatif` — shows bytes processed, throughput (MB/s), and ETA. Hidden when `--quiet` is set.
- **D-15:** README.md with full documentation: overview, install, usage examples for all 3 parts (tokenize, skill setup, de-tokenize), config reference, security model.

### Cross-Platform Delivery
- **D-16:** GitHub Actions CI matrix builds for Linux x64, macOS (Intel + Apple Silicon), Windows x64, and Linux ARM64.
- **D-17:** Release binaries attached to GitHub releases with checksums.

### Claude's Discretion
- Terminal formatting for de-tokenized output (colors, sections, spacing)
- Error handling for missing/corrupt token store during de-tokenization
- --help text organization and example selection
- README structure and writing style
- CI workflow specifics (triggers, caching, artifact naming)
- CLAUDE.md instruction block wording and structure

### Folded Todos
- **Add --help command output** — Polish CLI help text with clap derive annotations, examples section
- **Add progress bar during processing** — indicatif bar with bytes/throughput/ETA, suppressed by --quiet
- **Add README.md documentation** — Full project docs covering the 3-part workflow

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Technology Stack
- `CLAUDE.md` — Full technology stack with version pins, architecture notes. reqwest and anthropic-sdk-rust are listed but NOT needed (API removed from scope). Key deps for this phase: `indicatif 0.18.4` (already in Cargo.toml), `clap 4.6.0`, `arboard` or similar for clipboard (if added).

### Existing Implementation
- `src/tokenizer.rs` — `TokenMapData` with `token_to_value: HashMap<String, String>` reverse map. This is the core data structure for de-tokenization.
- `src/store.rs` — `EncryptedStore` with load/save. De-tokenize command needs to load the store to access the reverse map.
- `src/cli.rs` — Current clap CLI args. Must be extended with `detokenize` subcommand, `--detailed`, progress bar integration.
- `src/processor.rs` — Block-based processing pipeline. Progress bar hooks into this.
- `src/config.rs` — `LoktokConfig` with TOML parsing. Already has walk-up discovery.

### Requirements
- `.planning/REQUIREMENTS.md` — DIA-01 (modified: skill instead of API), DIA-02, DIA-03, DIA-04, DIA-05, INF-02 are target requirements

### Prior Phase Context
- `.planning/phases/02-detection-token-store/02-CONTEXT.md` — Token store design decisions (D-10 through D-15) that de-tokenization depends on

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `TokenMapData.token_to_value` — Reverse map already populated on every insert. Direct lookup for de-tokenization.
- `EncryptedStore` — Load/save with AES-256-GCM. De-tokenize command loads store, extracts reverse map.
- `indicatif` — Already a Cargo.toml dependency. Ready to use for progress bars.
- `clap` derive macros — Existing CLI structure. Add subcommands via enum variant.

### Established Patterns
- Block-based processing in `processor.rs` — Progress bar wraps the block iterator.
- Config walk-up discovery — Already works for `.loktok.toml`.
- Error types in `error.rs` with `thiserror` — Extend for de-tokenization errors.

### Integration Points
- `src/cli.rs` — Add `detokenize` subcommand alongside existing tokenize flow.
- `src/main.rs` — Route subcommand to new de-tokenize handler.
- `.claude/` directory — CLAUDE.md instruction block for token-aware diagnosis.

</code_context>

<specifics>
## Specific Ideas

- The 3-part flow (private tokenize → public Claude Code diagnosis → private de-tokenize) is the core mental model. Everything serves this flow.
- Claude Code should reason about tokens structurally — "if [IP_001] appears in both the error and the connection string, they're the same host." The CLAUDE.md block should teach this pattern.
- No API key management complexity — the tool never calls Claude. This dramatically simplifies the architecture.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

### Reviewed Todos (not folded)
All 3 matched todos were folded into scope.

</deferred>

---

*Phase: 03-diagnosis-delivery*
*Context gathered: 2026-04-16*
