# Phase 4: Colored CLI Help - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Add colored, styled `--help` output to all logtok commands using clap's built-in `Styles` API, with cross-platform terminal support and NO_COLOR compliance. Enrich help text with long descriptions and usage examples for all subcommands.

</domain>

<decisions>
## Implementation Decisions

### Color Palette
- **D-01:** Use "warm professional" color scheme — yellow bold headers, green flags/commands, cyan usage line. Matches cargo/rustup aesthetic.
- **D-02:** Stick to basic 8 ANSI colors only — no 256-color or truecolor. Maximizes terminal compatibility (Windows cmd.exe, PowerShell, macOS Terminal, Linux terminals).
- **D-03:** Pair colors with bold/underline for light terminal readability — never rely on color alone.

### Help Content
- **D-04:** Add `long_about` to ResetStore (currently has no description beyond the one-liner).
- **D-05:** Expand all subcommand descriptions with usage hints and inline examples via `after_long_help`.
- **D-06:** Keep the root `logtok --help` examples already present in the doc comment.

### Technical Approach
- **D-07:** Use clap's built-in `Styles` API — zero new crate dependencies. Define `const STYLES` with `clap::builder::styling::Styles`.
- **D-08:** Add `color` and `wrap_help` features to clap in Cargo.toml (currently only `derive`).
- **D-09:** Apply styles via `#[command(styles = STYLES)]` on the `Cli` struct.

### Claude's Discretion
- Exact wording of long descriptions and examples — Claude should write clear, concise help text matching existing tone
- Whether to create a separate `help_styles.rs` module or keep styles inline in `cli.rs` — use judgment based on code size

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Context
- `.planning/PROJECT.md` — Core value, constraints, current milestone goals
- `.planning/REQUIREMENTS.md` — HELP-01, HELP-02, HELP-03 requirements

### Research
- `.planning/research/STACK.md` — Confirms clap Styles API, zero new deps
- `.planning/research/ARCHITECTURE.md` — Integration approach with existing cli.rs
- `.planning/research/PITFALLS.md` — Windows ANSI, light terminal visibility, pipe leakage warnings

### Source Code
- `src/cli.rs` — Current CLI definition (77 lines, 3 subcommands, Parser derive)
- `src/main.rs` — Command dispatch (match on Commands enum)
- `Cargo.toml` — clap currently has `features = ["derive"]` only

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `cli.rs` already has well-structured doc comments on `Cli` and all subcommands — these become the `about` text
- Root help already shows 4 usage examples via doc comment on `Cli` struct

### Established Patterns
- Uses `clap::Parser` derive macro — styles apply via `#[command(styles = ...)]` attribute
- Commands enum with 3 variants: `Tokenize`, `Detokenize`, `ResetStore`
- Global `--quiet` and `--config` flags

### Integration Points
- `Cargo.toml` line 10: `clap = { version = "4.6.0", features = ["derive"] }` — add `"color"` and `"wrap_help"`
- `cli.rs` line 16: `#[command(name = "logtok", version, about, long_about)]` — add `styles = STYLES`
- No changes needed to `main.rs` — it just calls `Cli::parse()`

</code_context>

<specifics>
## Specific Ideas

- Color scheme should match cargo/rustup (warm professional) — user explicitly chose this over cool/minimal and bold/contrast options
- Target audience is developers and DevOps — help text should be practical, show real-world commands

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-colored-cli-help*
*Context gathered: 2026-04-28*
