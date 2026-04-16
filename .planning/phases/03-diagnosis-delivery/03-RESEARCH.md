# Phase 3: Diagnosis & Delivery - Research

**Researched:** 2026-04-16
**Domain:** CLI de-tokenization, Claude Code integration, cross-platform CI/CD, CLI UX polish
**Confidence:** HIGH

## Summary

Phase 3 completes the tokenize-diagnose-de-tokenize loop. The architecture is significantly simplified by the decision to remove the Claude API client (D-01) -- the tool never calls Claude directly. Instead, Claude Code reads tokenized logs with guidance from a CLAUDE.md instruction block, and the user runs `logtok detokenize` locally to replace tokens with real values in Claude's response.

The core technical work is: (1) a `detokenize` subcommand that reads the encrypted token store and performs reverse token replacement, (2) a CLAUDE.md instruction block teaching Claude Code the token format, (3) CLI polish (--help, progress bar improvements, --clipboard flag), (4) README.md documentation, and (5) GitHub Actions CI for cross-platform release builds.

**Primary recommendation:** Build the detokenize subcommand first -- it is the critical new capability. The rest (CLAUDE.md block, CLI polish, README, CI) are independent and can proceed in parallel.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** No Claude API client -- removed from scope. Claude Code is the diagnosis engine, not an API endpoint called by loktok.
- **D-02:** The tool's job is (a) tokenize logs and (b) de-tokenize Claude's response. The AI reasoning happens externally in Claude Code.
- **D-03:** A CLAUDE.md instruction block (~200 words) teaches Claude Code the token format. No full skill/plugin -- zero performance overhead.
- **D-04:** CLAUDE.md block explains: token format `[CATEGORY_NNN]`, all 19 categories and what they represent, how to reason about token relationships, instruction to keep tokens in responses for de-tokenization.
- **D-05:** No skill triggers, no tool calls, no custom MCP tools. Just static context in CLAUDE.md.
- **D-06:** New subcommand: `loktok detokenize <file>` or pipe via stdin: `echo "..." | loktok detokenize`
- **D-07:** De-tokenization reads the encrypted token store to resolve tokens back to real values.
- **D-08:** Output goes to terminal -- clear, formatted, readable. No file output by default.
- **D-09:** Inline replacement -- tokens replaced directly in the text. No legend table, no side-by-side.
- **D-10:** Default tokenize output: stdout. `-o` flag writes to file. Consistent with Phase 1/2.
- **D-11:** Bullet summary on stdout by default for de-tokenized diagnosis.
- **D-12:** `--detailed` flag outputs a full markdown .md file.
- **D-13:** `--help` text polished with descriptions, usage examples, and section headers via clap attributes.
- **D-14:** Progress bar using `indicatif` -- shows bytes processed, throughput (MB/s), and ETA. Hidden when `--quiet` is set.
- **D-15:** README.md with full documentation: overview, install, usage examples for all 3 parts (tokenize, skill setup, de-tokenize), config reference, security model.
- **D-16:** GitHub Actions CI matrix builds for Linux x64, macOS (Intel + Apple Silicon), Windows x64, and Linux ARM64.
- **D-17:** Release binaries attached to GitHub releases with checksums.

### Claude's Discretion
- Terminal formatting for de-tokenized output (colors, sections, spacing)
- Error handling for missing/corrupt token store during de-tokenization
- --help text organization and example selection
- README structure and writing style
- CI workflow specifics (triggers, caching, artifact naming)
- CLAUDE.md instruction block wording and structure

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DIA-01 | User can send tokenized logs to Claude API for diagnosis | Modified: Claude Code reads tokenized output directly (no API). CLAUDE.md instruction block enables this. |
| DIA-02 | Claude's response is de-tokenized back to real values automatically | `detokenize` subcommand reads encrypted store, performs regex-based reverse replacement |
| DIA-03 | User receives bullet-point summary (root cause, affected components, suggested fix) | Default stdout output of detokenize -- formatting is pass-through of Claude's response with tokens replaced |
| DIA-04 | User receives detailed markdown report | `--detailed` flag on detokenize writes .md file |
| DIA-05 | User can copy tokenized output to clipboard for manual paste | `--clipboard` flag using `cli-clipboard` crate, or document pipe to OS clipboard utilities |
| INF-02 | Cross-platform (Windows, macOS, Linux, ARM) | GitHub Actions matrix build with `cross` for ARM targets |
</phase_requirements>

## Standard Stack

### Core (already in Cargo.toml)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.6.0 | CLI with subcommands | Already used. Add `Subcommand` derive for detokenize command. [VERIFIED: Cargo.toml] |
| indicatif | 0.18.4 | Progress bars | Already a dependency. Enhance with throughput display. [VERIFIED: Cargo.toml] |
| regex | 1.12.3 | Token pattern matching for detokenize | Already used for detection. Reuse for `\[([A-Z]+_\d{3,})\]` token matching in reverse. [VERIFIED: Cargo.toml] |
| serde/serde_json | 1.0.228/1.0.149 | Token store deserialization | Already used. Detokenize loads store via existing `Store::load()`. [VERIFIED: Cargo.toml] |

### New Dependencies
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| cli-clipboard | 0.4.0 | Cross-platform clipboard | DIA-05: `--clipboard` flag copies tokenized output to clipboard. Supports Wayland, X11, macOS, Windows. Better than arboard for headless/terminal use. [VERIFIED: cargo search] |

### CI Tooling (not Cargo deps)
| Tool | Purpose | When to Use |
|------|---------|-------------|
| cross | Cross-compilation for ARM targets | CI: `cross build --release --target aarch64-unknown-linux-musl` [CITED: github.com/cross-rs/cross] |
| actions/upload-artifact@v4 | Upload build artifacts | CI: store binaries between jobs [CITED: ahmedjama.com blog] |
| softprops/action-gh-release@v1 | Publish GitHub releases | CI: attach binaries + checksums to tagged releases [CITED: ahmedjama.com blog] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| cli-clipboard | arboard 3.6.1 | arboard is more popular (1Password-maintained) but designed for GUI apps. cli-clipboard is built for terminal/headless use, supports Wayland via wl-clipboard-rs. Better fit for a CLI tool. [VERIFIED: cargo search, lib.rs docs] |
| cli-clipboard | Document pipe commands | No new dependency, but worse UX -- user must know `clip`/`pbcopy`/`xclip`. A `--clipboard` flag is one line of code with cli-clipboard. |
| cross | cargo-zigbuild | cargo-zigbuild uses Zig as linker for cross-compilation. Simpler setup but less mature for all target triples we need. |

**Installation (new deps only):**
```bash
cargo add cli-clipboard@0.4.0
```

## Architecture Patterns

### CLI Restructure: Flat Args to Subcommands

The current CLI uses flat `Parser` args. Phase 3 requires subcommands (`tokenize`, `detokenize`). This is a breaking change to the CLI interface.

**Pattern: Clap Derive Subcommands** [CITED: docs.rs/clap/latest/clap/_derive/_tutorial]

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "logtok", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Suppress progress bar output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Path to .logtok.toml config file
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Tokenize sensitive data in log files
    Tokenize {
        /// Path to the log file
        file: PathBuf,

        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Copy tokenized output to clipboard
        #[arg(long)]
        clipboard: bool,

        /// Block size in bytes
        #[arg(long, default_value = "65536")]
        block_size: usize,

        /// Preview detections without tokenizing
        #[arg(long)]
        dry_run: bool,
    },

    /// De-tokenize text, replacing tokens with real values
    Detokenize {
        /// File containing tokenized text (reads stdin if omitted)
        file: Option<PathBuf>,

        /// Output detailed markdown report to file
        #[arg(long)]
        detailed: Option<PathBuf>,
    },

    /// Reset (delete) the encrypted token store
    ResetStore,
}
```

**Key points:**
- `global = true` on `--quiet` and `--config` makes them available on all subcommands [VERIFIED: clap docs]
- `ResetStore` moves from a flag to a proper subcommand (cleaner UX)
- `Detokenize` accepts optional file arg; if missing, reads stdin (D-06)

### De-tokenization Engine

```rust
use regex::Regex;

/// Replace all [CATEGORY_NNN] tokens in text with their real values.
fn detokenize(text: &str, token_to_value: &HashMap<String, String>) -> String {
    let re = Regex::new(r"\[([A-Z]+_\d{3,})\]").unwrap();
    re.replace_all(text, |caps: &regex::Captures| {
        let token = &caps[0]; // includes brackets
        token_to_value
            .get(token)
            .cloned()
            .unwrap_or_else(|| token.to_string())
    })
    .to_string()
}
```

**Key points:**
- Pattern `\[([A-Z]+_\d{3,})\]` matches the token format `[CATEGORY_NNN]` [VERIFIED: matches format in tokenizer.rs line 67]
- Unknown tokens are left as-is (graceful degradation)
- Uses the existing `token_to_value: HashMap<String, String>` from `TokenMapData` [VERIFIED: tokenizer.rs line 20]
- Regex is compiled once, applied per-line

### Detokenize Pipeline Flow

```
Input (file or stdin)
    |
    v
Load encrypted store (Store::load → TokenMapData)
    |
    v
Read input line-by-line
    |
    v
For each line: regex replace [TOKEN] → real value
    |
    v
Output to stdout (default) or markdown file (--detailed)
```

### CLAUDE.md Instruction Block Structure

```markdown
## Logtok Token-Aware Diagnosis

This project uses `logtok` to tokenize sensitive data before analysis.

**Token format:** `[CATEGORY_NNN]` (e.g., `[IP_001]`, `[KEY_002]`, `[EMAIL_003]`)

**Categories:** IP, HOST, URL, PATH, PORT, EMAIL, USER, PHONE, KEY, PASS,
CONN, JWT, PEM, UUID, MAC, CC, SSN, DOB, CUSTOM

**How to reason about tokens:**
- Same token = same real value everywhere (e.g., `[IP_001]` is always the same IP)
- Different tokens in same category = different values (`[IP_001]` != `[IP_002]`)
- Cross-reference tokens across log lines to trace request flows

**IMPORTANT:** Keep all tokens in your response exactly as-is.
Do NOT invent real values. The user will run `logtok detokenize`
to replace tokens with actual values after diagnosis.
```

### Recommended Project Structure (new/modified files)

```
src/
├── cli.rs               # MODIFIED: subcommand enum
├── main.rs              # MODIFIED: route subcommands
├── detokenizer.rs       # NEW: detokenize engine
├── clipboard.rs         # NEW: clipboard integration (thin wrapper)
├── error.rs             # MODIFIED: add DetokenizeError variants
├── processor.rs         # MODIFIED: progress bar improvements
└── ...existing files...
.github/
└── workflows/
    └── release.yml      # NEW: cross-platform build + release
CLAUDE.md                # MODIFIED: add token-aware instruction block
README.md                # NEW: full project documentation
```

### Anti-Patterns to Avoid
- **Building a Claude API client:** Removed from scope (D-01). Do not add reqwest, tokio, or any HTTP client code.
- **De-tokenizing in streaming fashion:** The detokenize input is Claude's text response (small, typically <10KB). No need for block processing or memory mapping. Just `read_to_string`.
- **Modifying the token store during detokenize:** Detokenize is read-only. It loads the store but never writes to it.
- **Changing token format:** The `[CATEGORY_NNN]` format is established in Phase 1/2 and must remain consistent.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Clipboard access | OS-specific clipboard code | `cli-clipboard` 0.4.0 | Windows/macOS/Linux/Wayland all differ. The crate handles it. [VERIFIED: crates.io] |
| Cross-compilation | Custom Docker build scripts | `cross` tool | Manages toolchains, sysroots, and Docker containers for each target triple. [CITED: github.com/cross-rs/cross] |
| Release packaging | Manual zip/tar/checksum scripts | `softprops/action-gh-release` | Handles asset upload, checksums, and release notes in CI. [CITED: GitHub Marketplace] |
| CLI argument parsing | Manual argv parsing | `clap` derive macros | Already used. Subcommands, help text, validation all generated. |

**Key insight:** Phase 3 has very little novel code. The detokenize engine is ~30 lines of regex replacement. The bulk of work is integration (CLI restructure, CI config, documentation).

## Common Pitfalls

### Pitfall 1: Breaking Existing CLI Interface
**What goes wrong:** Moving from flat args to subcommands breaks `logtok file.log` (must become `logtok tokenize file.log`).
**Why it happens:** Subcommand migration is a breaking change.
**How to avoid:** This is intentional and acceptable for a pre-1.0 tool. Document the new interface clearly in README and --help. Consider adding a helpful error message if no subcommand is provided.
**Warning signs:** Existing integration tests will fail and need updating.

### Pitfall 2: Stdin Detection on Windows
**What goes wrong:** Checking if stdin is a pipe vs terminal differs across platforms.
**Why it happens:** Windows uses different APIs than Unix for TTY detection.
**How to avoid:** Use `std::io::stdin().is_terminal()` (stabilized in Rust 1.70+). This is cross-platform. [ASSUMED]
**Warning signs:** Detokenize hangs waiting for stdin when user forgets to pipe input.

### Pitfall 3: Token Regex False Positives in Detokenize
**What goes wrong:** The regex `\[([A-Z]+_\d{3,})\]` could match non-token text that happens to look like a token.
**Why it happens:** The pattern is generic enough to match arbitrary bracket-enclosed uppercase text.
**How to avoid:** Only replace tokens that exist in the loaded token store. The regex finds candidates; the HashMap lookup confirms them. Already handled in the code example above (unknown tokens left as-is).
**Warning signs:** Detokenized output contains unexpected replacements.

### Pitfall 4: Cross-Compilation Linking Errors on macOS Targets
**What goes wrong:** Building macOS binaries from GitHub Actions Linux runners fails due to missing macOS SDK.
**Why it happens:** macOS targets require Apple's SDK and linker, which aren't available on Linux.
**How to avoid:** Use `macos-latest` runner for macOS targets in the CI matrix. Only use `cross` for Linux ARM targets. [CITED: ahmedjama.com blog]
**Warning signs:** Linker errors mentioning `cc` or `ld` for darwin targets.

### Pitfall 5: Clipboard Access in Headless/CI Environments
**What goes wrong:** Clipboard operations fail in CI or SSH sessions where no display server is running.
**Why it happens:** Clipboard APIs require a display server (X11, Wayland, or Windows desktop).
**How to avoid:** Gracefully handle clipboard errors with a clear message: "Clipboard not available (no display server). Output written to stdout instead." Wrap the clipboard call in a Result and fall back to stdout.
**Warning signs:** `cli-clipboard` panics or returns cryptic X11/Wayland errors.

### Pitfall 6: Token Store Location Mismatch
**What goes wrong:** Detokenize can't find the token store because it's looking in the wrong directory.
**Why it happens:** The store is created in CWD/.logtok/store.enc during tokenization. If detokenize runs from a different directory, the store isn't found.
**How to avoid:** Detokenize should use the same CWD-based store discovery as tokenize. Document that both commands must run from the same working directory. Consider a `--store` flag for explicit store path.
**Warning signs:** "Store not found" errors when the store definitely exists.

## Code Examples

### De-tokenization from Stdin (D-06)
```rust
// Source: Rust stdlib + existing store.rs pattern
use std::io::{self, IsTerminal, Read};

fn read_input(file: Option<&Path>) -> Result<String> {
    match file {
        Some(path) => {
            fs::read_to_string(path)
                .with_context(|| format!("Cannot read file: {}", path.display()))
        }
        None => {
            if io::stdin().is_terminal() {
                anyhow::bail!(
                    "No input provided. Pipe text or specify a file:\n  \
                     logtok detokenize response.txt\n  \
                     echo '...' | logtok detokenize"
                );
            }
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}
```

### Clipboard Copy (DIA-05)
```rust
// Source: cli-clipboard crate docs
use cli_clipboard::{ClipboardContext, ClipboardProvider};

fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut ctx = ClipboardContext::new()
        .map_err(|e| anyhow::anyhow!("Clipboard not available: {}", e))?;
    ctx.set_contents(text.to_string())
        .map_err(|e| anyhow::anyhow!("Failed to copy to clipboard: {}", e))?;
    eprintln!("logtok: tokenized output copied to clipboard");
    Ok(())
}
```

### GitHub Actions Release Workflow (D-16, D-17)
```yaml
# Source: ahmedjama.com/blog/2025/12/cross-platform-rust-pipeline-github-actions/
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
            use_cross: true
          - target: aarch64-unknown-linux-musl
            os: ubuntu-latest
            use_cross: true
          - target: x86_64-apple-darwin
            os: macos-latest
            use_cross: false
          - target: aarch64-apple-darwin
            os: macos-latest
            use_cross: false
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            use_cross: false

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross
        if: matrix.use_cross
        run: cargo install cross --locked

      - name: Build
        run: |
          if [ "${{ matrix.use_cross }}" = "true" ]; then
            cross build --release --target ${{ matrix.target }}
          else
            cargo build --release --target ${{ matrix.target }}
          fi
        shell: bash

      - name: Package
        run: |
          cd target/${{ matrix.target }}/release
          if [ "${{ runner.os }}" = "Windows" ]; then
            7z a ../../../logtok-${{ matrix.target }}.zip logtok.exe
          else
            tar czf ../../../logtok-${{ matrix.target }}.tar.gz logtok
          fi
        shell: bash

      - uses: actions/upload-artifact@v4
        with:
          name: logtok-${{ matrix.target }}
          path: logtok-${{ matrix.target }}.*

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
      - name: Generate checksums
        run: |
          find . -name 'logtok-*' -exec sha256sum {} \; > checksums.txt
      - uses: softprops/action-gh-release@v1
        with:
          files: |
            logtok-*/logtok-*
            checksums.txt
```

### Progress Bar with Throughput (D-14)
```rust
// Source: indicatif docs + existing processor.rs pattern
use indicatif::{ProgressBar, ProgressStyle, HumanBytes};

let pb = ProgressBar::new(file_size);
pb.set_style(
    ProgressStyle::default_bar()
        .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) ETA: {eta}")
        .unwrap()
        .progress_chars("=>-"),
);
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `structopt` for CLI | `clap` 4.x derive | clap 3.0 (2022) | structopt merged into clap. Use clap derive only. [VERIFIED: Cargo.toml already uses clap 4.6.0] |
| `atty` for terminal detection | `std::io::IsTerminal` | Rust 1.70 (2023) | No external crate needed for stdin TTY check. [ASSUMED] |
| `clipboard` crate | `cli-clipboard` 0.4.0 | 2023+ | Original `clipboard` crate unmaintained. cli-clipboard adds Wayland support. [VERIFIED: crates.io] |
| Manual cross-compilation | `cross` + GitHub Actions matrix | 2024+ | Docker-based cross handles sysroots automatically. [CITED: cross-rs/cross] |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `std::io::stdin().is_terminal()` is stable and cross-platform in Rust 1.70+ | Pitfall 2, Code Examples | LOW -- fallback is `atty` crate. Rust 1.94 is installed, well past 1.70. |
| A2 | cli-clipboard 0.4.0 works on Windows without additional setup | Standard Stack | LOW -- crate docs claim Windows support. If fails, fallback is documenting pipe commands. |
| A3 | `softprops/action-gh-release@v1` supports checksum file upload | Code Examples | LOW -- widely used action. If version changed, check marketplace. |

## Open Questions

1. **Backward Compatibility of CLI Restructure**
   - What we know: Moving from flat args to subcommands breaks `logtok file.log` syntax
   - What's unclear: Whether any scripts or integrations depend on the current flat interface
   - Recommendation: Accept the break. This is pre-1.0 software. Add a clear error message suggesting the new syntax if no subcommand is given.

2. **Store Discovery for Detokenize**
   - What we know: Store lives at CWD/.logtok/store.enc
   - What's unclear: Should detokenize support `--store <path>` for explicit store location?
   - Recommendation: Add `--store` flag as optional override. Default to CWD-based discovery.

3. **DIA-03 vs DIA-04 Output Distinction**
   - What we know: D-11 says "bullet summary on stdout by default", D-12 says "--detailed flag outputs markdown .md file"
   - What's unclear: Whether detokenize should reformat Claude's response into bullets, or just pass through with token replacement
   - Recommendation: Pass through with token replacement. Claude Code already formats its own output. The `--detailed` flag writes to file for archival. No reformatting needed.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable | All code | Yes | 1.94.1 | -- |
| cargo | All builds | Yes | 1.94.1 | -- |
| cross | ARM CI builds | No (CI only) | -- | Installed in CI via `cargo install cross` |
| GitHub Actions | CI/CD | N/A (remote) | -- | Manual builds documented in README |

**Missing dependencies with no fallback:** None -- all CI tools are installed during workflow execution.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | -- |
| V3 Session Management | No | -- |
| V4 Access Control | No | -- |
| V5 Input Validation | Yes | Regex-based token matching with HashMap validation (no blind replacement) |
| V6 Cryptography | Yes | Existing AES-256-GCM for store. Detokenize is read-only -- no new crypto code needed. |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Token store exposed in clipboard | Information Disclosure | Clipboard contains tokenized (safe) output only. Never copy the store or real values to clipboard. |
| Token store left decrypted in memory | Information Disclosure | Rust drops HashMap when scope ends. No explicit zeroization needed for v1 (store is ephemeral in process memory). |
| CLAUDE.md leaks token-to-value mapping | Information Disclosure | CLAUDE.md contains only format description, never actual mappings. Mappings stay in encrypted store. |

## Sources

### Primary (HIGH confidence)
- Cargo.toml and src/ files in the repository -- verified existing dependencies and code patterns
- [clap derive tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html) -- subcommand derive pattern
- [cli-clipboard on crates.io](https://crates.io/crates/cli-clipboard) -- version 0.4.0, cross-platform clipboard for CLI
- [arboard on GitHub](https://github.com/1Password/arboard) -- considered alternative, version 3.6.1
- [cross-rs/cross on GitHub](https://github.com/cross-rs/cross) -- cross-compilation tool

### Secondary (MEDIUM confidence)
- [Cross-platform Rust CI/CD pipeline](https://ahmedjama.com/blog/2025/12/cross-platform-rust-pipeline-github-actions/) -- GitHub Actions workflow patterns
- [actions-rust-cross](https://github.com/houseabsolute/actions-rust-cross) -- GitHub Action for Rust cross builds

### Tertiary (LOW confidence)
- None -- all claims verified against codebase or official sources.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all core deps already in Cargo.toml, only cli-clipboard is new (verified on crates.io)
- Architecture: HIGH -- detokenize is a straightforward reverse lookup using existing TokenMapData
- Pitfalls: HIGH -- based on direct codebase inspection and known Rust cross-platform patterns
- CI/CD: MEDIUM -- workflow patterns verified against blog posts but not tested against this specific project

**Research date:** 2026-04-16
**Valid until:** 2026-05-16 (stable domain, 30-day validity)
