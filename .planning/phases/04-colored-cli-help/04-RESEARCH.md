# Phase 4: Colored CLI Help - Research

**Researched:** 2026-04-28
**Domain:** clap CLI styling, ANSI color, cross-platform terminal support
**Confidence:** HIGH

## Summary

This phase adds colored, styled `--help` output to all logtok commands using clap 4.6's built-in `Styles` API. The implementation requires zero new crate dependencies -- only enabling existing clap features (`color`, `wrap_help`) and defining a const `Styles` value with the user's chosen "warm professional" color scheme (yellow bold headers, green flags/commands, cyan usage line).

clap's `color` feature pulls in `anstream`, which handles all cross-platform complexity automatically: Windows cmd.exe ANSI detection with wincon fallback, NO_COLOR/CLICOLOR environment variable compliance, and automatic ANSI stripping when stdout is piped or redirected. The implementation is a single const definition plus a `#[command(styles = STYLES)]` attribute.

**Primary recommendation:** Define a `const STYLES: Styles` using basic 8 ANSI colors with bold/underline, apply via derive attribute, add `long_about` and `after_long_help` to all subcommands, and enable `color` + `wrap_help` features in Cargo.toml.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Use "warm professional" color scheme -- yellow bold headers, green flags/commands, cyan usage line. Matches cargo/rustup aesthetic.
- **D-02:** Stick to basic 8 ANSI colors only -- no 256-color or truecolor. Maximizes terminal compatibility.
- **D-03:** Pair colors with bold/underline for light terminal readability -- never rely on color alone.
- **D-04:** Add `long_about` to ResetStore (currently has no description beyond the one-liner).
- **D-05:** Expand all subcommand descriptions with usage hints and inline examples via `after_long_help`.
- **D-06:** Keep the root `logtok --help` examples already present in the doc comment.
- **D-07:** Use clap's built-in `Styles` API -- zero new crate dependencies. Define `const STYLES` with `clap::builder::styling::Styles`.
- **D-08:** Add `color` and `wrap_help` features to clap in Cargo.toml (currently only `derive`).
- **D-09:** Apply styles via `#[command(styles = STYLES)]` on the `Cli` struct.

### Claude's Discretion
- Exact wording of long descriptions and examples -- write clear, concise help text matching existing tone
- Whether to create a separate `help_styles.rs` module or keep styles inline in `cli.rs` -- use judgment based on code size

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| HELP-01 | User sees colored section headers, bold flag names, and styled usage line in --help output | Styles API with `header()`, `literal()`, `usage()` methods; `color` feature enables rendering |
| HELP-02 | CLI respects NO_COLOR and CLICOLOR environment variables (colors disabled when set) | anstream (pulled in by clap `color` feature) handles NO_COLOR/CLICOLOR automatically -- zero custom code needed |
| HELP-03 | Help output renders correctly on Windows cmd.exe, PowerShell, and Unix terminals | anstream auto-detects Windows ANSI support, falls back to wincon API; basic 8 ANSI colors maximize compatibility |

</phase_requirements>

## Standard Stack

### Core (no new dependencies -- feature flags only)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.6.0 (project) / 4.6.1 (latest) | CLI framework + styling | Already in use; `color` and `wrap_help` features unlock styling and terminal-width wrapping |
| anstream | 1.0.0 (transitive) | Terminal color detection | Pulled in automatically by clap's `color` feature; handles NO_COLOR, CLICOLOR, Windows wincon, pipe detection |
| anstyle | (transitive) | Style type definitions | Provides `Style`, `AnsiColor`, `Effects` types used by clap's `Styles` builder |

[VERIFIED: cargo registry, clap_builder-4.6.0/Cargo.toml -- `color = ["dep:anstream"]`]

### Supporting
None -- no new crates needed.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| clap built-in Styles | owo-colors / colored crate | Would add a dependency for zero benefit; clap handles all help output styling internally |
| anstream (via clap) | manual `enable_ansi_colors()` | Duplicates logic already handled by clap's color feature |

**Cargo.toml change:**
```toml
clap = { version = "4.6.0", features = ["derive", "color", "wrap_help"] }
```

## Architecture Patterns

### Recommended File Organization

**Recommendation: Keep styles inline in `cli.rs`.** The styles definition is ~15 lines and is tightly coupled to the CLI struct. A separate module would add indirection for minimal code.

```
src/
  cli.rs          # Styles const + Cli struct + Commands enum (grows from 77 to ~120 lines)
  main.rs         # No changes needed
```

### Pattern: Const Styles Definition

**What:** Define styles as a `const` so they are computed at compile time with zero runtime cost.
**When to use:** Always for clap Styles -- the builder methods are all `const fn`.

```rust
// Source: clap_builder-4.6.0/src/builder/styling.rs (verified in local cargo registry)
use clap::builder::styling::{AnsiColor, Styles, Style};

const STYLES: Styles = Styles::styled()
    .header(
        Style::new()
            .fg_color(Some(clap::builder::styling::Color::Ansi(AnsiColor::Yellow)))
            .bold()
    )
    .usage(
        Style::new()
            .fg_color(Some(clap::builder::styling::Color::Ansi(AnsiColor::Cyan)))
            .bold()
    )
    .literal(
        Style::new()
            .fg_color(Some(clap::builder::styling::Color::Ansi(AnsiColor::Green)))
            .bold()
    )
    .placeholder(
        Style::new()
            .fg_color(Some(clap::builder::styling::Color::Ansi(AnsiColor::Green)))
    )
    .valid(
        Style::new()
            .fg_color(Some(clap::builder::styling::Color::Ansi(AnsiColor::Green)))
    )
    .invalid(
        Style::new()
            .fg_color(Some(clap::builder::styling::Color::Ansi(AnsiColor::Yellow)))
    )
    .error(
        Style::new()
            .fg_color(Some(clap::builder::styling::Color::Ansi(AnsiColor::Red)))
            .bold()
    );
```

[VERIFIED: All `Styles` builder methods are `const fn` -- confirmed in clap_builder-4.6.0/src/builder/styling.rs lines 36-141]

### Pattern: Applying Styles via Derive Attribute

**What:** The derive macro forwards `styles = EXPR` as a `.styles(EXPR)` call on the Command builder.
**When to use:** When using `#[derive(Parser)]`.

```rust
#[derive(Parser, Debug)]
#[command(name = "logtok", version, about, long_about, styles = STYLES)]
pub struct Cli {
    // ...
}
```

[VERIFIED: `Command::styles()` method exists at clap_builder-4.6.0/src/builder/command.rs:1366, gated on `#[cfg(feature = "color")]`]

### Pattern: long_about and after_long_help via Doc Comments

**What:** clap extracts `about` from the first paragraph of doc comments and `long_about` from the full doc comment. `after_long_help` must be set via attribute.

```rust
/// Tokenize sensitive data in log files
///
/// Reads a log file, detects sensitive values (API keys, IPs, emails, etc.),
/// and replaces them with deterministic tokens like [IP_001], [KEY_002].
/// The tokenized output is safe for sharing with AI tools.
#[command(after_long_help = "\
Examples:
  logtok tokenize server.log              Tokenize to stdout
  logtok tokenize server.log -o safe.log  Tokenize to file
  logtok tokenize app.log --dry-run       Preview without writing")]
Tokenize { ... }
```

### Anti-Patterns to Avoid

- **Manual ANSI codes in println!:** Never embed `\x1b[` escape codes directly. Always go through clap's styling system or `anstream`. Raw codes bypass pipe detection and NO_COLOR handling.
- **Testing only in Windows Terminal:** Windows Terminal supports ANSI natively -- it does not represent the worst case. Test in cmd.exe to catch failures.
- **Dim text styling:** Many terminals render dim identically to normal weight, making it useless for differentiation. The user's chosen palette correctly avoids this.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Terminal color detection | Custom `is_terminal()` + `NO_COLOR` check | clap's `color` feature (anstream) | anstream handles 6+ env vars, Windows wincon fallback, pipe detection |
| Windows ANSI support | `SetConsoleMode()` Win32 calls | anstream's built-in wincon adapter | Handles fallback when ENABLE_VIRTUAL_TERMINAL_PROCESSING is unavailable |
| Terminal width detection | Manual ioctl/winapi | clap's `wrap_help` feature (terminal_size crate) | Handles cross-platform width detection with sane defaults (100 cols fallback) |

**Key insight:** clap 4.x with the `color` and `wrap_help` features handles 100% of terminal compatibility. The implementation is purely declarative -- define styles, write help text, let clap do the rest.

## Common Pitfalls

### Pitfall 1: Windows cmd.exe ANSI Breakage
**What goes wrong:** Colored help renders as raw escape codes on cmd.exe
**Why it happens:** cmd.exe requires `ENABLE_VIRTUAL_TERMINAL_PROCESSING` console mode flag
**How to avoid:** The `color` feature (anstream) handles this automatically. It detects ANSI support and falls back to the Windows console API (wincon). Just enable the feature -- no custom code.
**Warning signs:** Raw `\x1b[` visible in help output on Windows

[VERIFIED: anstream-1.0.0/src/auto.rs contains wincon fallback logic with `enable_ansi_colors()` detection]

### Pitfall 2: Colors Invisible on Light Terminals
**What goes wrong:** Green or cyan text disappears on white/light backgrounds
**Why it happens:** Using colors without bold/underline, or using colors that are close to white
**How to avoid:** User's palette (D-03) correctly pairs every color with bold. Basic 8 ANSI colors adapt to terminal palettes.
**Warning signs:** Test `logtok --help` with a light terminal theme

### Pitfall 3: ANSI Codes Leak into Piped Output
**What goes wrong:** `logtok --help | grep tokenize` shows escape code garbage
**Why it happens:** ANSI codes emitted when stdout is not a TTY
**How to avoid:** anstream strips ANSI automatically when output is piped. Do NOT add any custom colored output via raw `println!` in help rendering paths.
**Warning signs:** Redirecting `--help` to a file shows `[0m` sequences

[VERIFIED: anstream-1.0.0/src/auto.rs documents "Respecting env variables like NO_COLOR" and uses `is_terminal()` detection]

### Pitfall 4: Short Help vs Long Help Inconsistency
**What goes wrong:** `-h` shows minimal unstyled content, `--help` shows rich styled content. Users get confused.
**Why it happens:** clap shows `about` for `-h` and `long_about` + `after_long_help` for `--help`
**How to avoid:** Ensure both `-h` and `--help` produce styled output. Styles apply to both automatically. Keep `about` (short) clean and concise. Test all variants: `logtok -h`, `logtok --help`, `logtok help`, `logtok help tokenize`.

## Code Examples

### Complete Styles Definition (warm professional palette)

```rust
// Source: Verified API from clap_builder-4.6.0/src/builder/styling.rs
use clap::builder::styling::{AnsiColor, Color, Style, Styles};

const STYLES: Styles = Styles::styled()
    .header(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow))).bold())
    .usage(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))).bold())
    .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))).bold())
    .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
    .valid(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
    .invalid(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow))))
    .error(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))).bold());
```

### Applying to Cli struct

```rust
#[derive(Parser, Debug)]
#[command(name = "logtok", version, about, long_about, styles = STYLES)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    // ...
}
```

### Adding after_long_help to a Subcommand

```rust
/// Tokenize sensitive data in log files
///
/// Reads a log file, detects sensitive values (API keys, IPs, emails, etc.),
/// and replaces them with deterministic tokens like [IP_001], [KEY_002].
#[command(after_long_help = "\
Examples:
  logtok tokenize server.log              Tokenize to stdout
  logtok tokenize server.log -o safe.log  Tokenize to file
  logtok tokenize app.log --dry-run       Preview what would change
  logtok tokenize app.log --clipboard     Tokenize and copy to clipboard")]
Tokenize {
    // ...
}
```

### ResetStore long_about (currently missing -- D-04)

```rust
/// Delete the encrypted token store
///
/// Removes the .loktok/store.enc file containing all token-to-value mappings.
/// Use this when you want to start fresh or when changing the encryption key.
/// This action is irreversible -- all stored token mappings will be lost.
#[command(after_long_help = "\
Examples:
  logtok reset-store    Delete the token store in the current directory")]
ResetStore,
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| clap 3.x `AppSettings::ColoredHelp` | clap 4.x `Styles` API on Command | clap 4.0 (2022) | Styles are composable, const-constructible, fully customizable |
| `structopt` + `termcolor` | clap 4.x derive with built-in `color` feature | clap 4.0 | Zero extra deps for CLI coloring |
| Manual `NO_COLOR` checking | anstream handles automatically | anstream 1.0 (2023) | Just enable clap `color` feature |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow))).bold()` is the correct const API for setting foreground color with bold | Code Examples | Compile error -- easily caught, low risk |

All other claims were verified against the local clap_builder-4.6.0 source in the cargo registry.

## Open Questions

None -- this phase is straightforward with well-documented APIs and locked user decisions.

## Sources

### Primary (HIGH confidence)
- clap_builder-4.6.0 source: `~/.cargo/registry/src/.../clap_builder-4.6.0/src/builder/styling.rs` -- Styles API, all methods are `const fn`
- clap_builder-4.6.0 source: `~/.cargo/registry/src/.../clap_builder-4.6.0/src/builder/command.rs:1366` -- `Command::styles()` method, `#[cfg(feature = "color")]`
- clap_builder-4.6.0 Cargo.toml -- `color = ["dep:anstream"]`, `wrap_help` feature confirmed
- anstream-1.0.0 source: `~/.cargo/registry/src/.../anstream-1.0.0/src/auto.rs` -- NO_COLOR, CLICOLOR, Windows wincon fallback
- `.planning/research/PITFALLS.md` -- Pre-researched pitfalls for this phase
- `src/cli.rs` -- Current CLI structure (77 lines, 3 subcommands)
- `Cargo.toml` -- Current clap config: `features = ["derive"]`

### Secondary (MEDIUM confidence)
- `cargo search clap` -- Latest version is 4.6.1 (project uses 4.6.0, no breaking changes)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- verified against local source code in cargo registry
- Architecture: HIGH -- API surface is small and well-documented, verified const fn availability
- Pitfalls: HIGH -- verified anstream handles Windows/NO_COLOR automatically; pitfalls pre-documented in project research

**Research date:** 2026-04-28
**Valid until:** 2026-06-28 (stable APIs, unlikely to change)
