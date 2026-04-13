# Phase 1: Core Tokenization Engine - Research

**Researched:** 2026-04-13
**Domain:** Rust CLI, regex-based text processing, block-based file I/O, log compaction
**Confidence:** HIGH

## Summary

Phase 1 builds the foundational `logtok` CLI binary: a Rust tool that reads log files (JSON and plain text), detects sensitive data via regex patterns, replaces matches with deterministic category-prefixed tokens (`[IP_001]`, `[KEY_001]`), compacts duplicate lines, and outputs the result -- all with bounded memory via block-based processing.

The Rust ecosystem for this is mature and well-understood. The `regex` crate guarantees linear-time matching with SIMD acceleration, `clap` 4.x provides derive-macro CLI parsing, `serde_json` handles structured JSON logs, and `memmap2` or `BufReader` handles large file access. The key architectural challenge is the block processing pipeline: reading configurable-size chunks at newline boundaries, applying regex replacement with a shared token map, compacting duplicates, and streaming output.

**Primary recommendation:** Build a line-oriented block processor using `BufReader` with configurable buffer size (defaulting to 64KB of accumulated lines), apply a `RegexSet` for fast detection followed by individual `Regex` patterns for capture/replacement, maintain an in-memory `HashMap<String, String>` for deterministic token mapping, and compact consecutive duplicate lines with counts.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Square bracket delimiters -- `[CATEGORY_NNN]`
- **D-02:** Category-only prefix, no `TOK_` prefix -- `[IP_001]`, `[KEY_001]`, `[EMAIL_001]` not `[TOK_IP_001]`
- **D-03:** Per-category counters -- each category starts at 001 independently
- **D-04:** 3-digit zero-padded counters -- `[IP_001]` through `[IP_999]`
- **D-05:** Category names: `IP`, `KEY`, `EMAIL`, `HOST`, `PATH`, `PASS`, `URL` (short, uppercase)
- **D-06:** Structural compaction applied during tokenization -- collapse duplicate lines with count, normalize whitespace, preserve every unique log event and ordering
- **D-07:** Compaction optimized for LLM consumption -- minimize Claude token usage while retaining full diagnostic value
- **D-09:** Line-aware splitting -- read configurable-size chunks but always split at newline boundaries, no value spans two blocks
- **D-10:** Default block size 64KB, configurable via `--block-size` flag
- **D-11:** Tokenized JSON output remains valid parseable JSON -- tokens replace values as strings
- **D-12:** Values only -- JSON keys are not tokenized, only values are scanned and replaced
- **D-13:** Binary name: `logtok`
- **D-14:** Flat CLI (no subcommands) -- `logtok <file>` tokenizes
- **D-15:** Output to stdout by default, `--output` / `-o` flag for file output
- **D-16:** Progress bar on stderr (via indicatif) for large files, `--quiet` flag to suppress
- **D-17:** Configurable block size via `--block-size` flag

### Claude's Discretion
- Error message formatting and exit codes
- Internal logging levels and tracing configuration
- Regex pattern ordering and optimization strategy
- Memory-mapped file vs buffered reader decision for different file sizes

### Deferred Ideas (OUT OF SCOPE)
- Claude Code skill for interpreting compact+tokenized logs -- Phase 3
- Subcommand structure (`logtok tokenize`, `logtok diagnose`, `logtok detokenize`) -- Phase 3
- Aggressive compaction (stack trace trimming, error grouping) -- future enhancement
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| INF-01 | Single binary, zero runtime dependencies | Rust compiles to static binary; `cargo build --release` produces single executable. No runtime needed. |
| INF-03 | Block-based processing for large files (bounded memory usage) | Line-aware block processing via `BufReader` with configurable chunk accumulation; see Architecture Patterns. |
| TOK-01 | Same sensitive value always produces the same token (deterministic) | `HashMap<String, String>` token map; lookup-or-insert pattern ensures deterministic mapping across entire file. |
| TOK-02 | Tokens are category-prefixed for LLM context | Category names from D-05, format `[CATEGORY_NNN]` from D-01/D-02/D-03/D-04. Note: REQUIREMENTS.md says `[TOK_IP_001]` but CONTEXT.md decision D-02 overrides to `[IP_001]`. |
| TOK-05 | User can tokenize structured JSON logs preserving structure | Recursive `serde_json::Value` traversal, replacing only string values (D-12), re-serializing to valid JSON (D-11). |
| TOK-06 | User can tokenize unstructured/plain text logs | Line-by-line regex replacement on raw text. |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- **Language:** Rust (stable 1.85+)
- **Performance:** Must handle large log files (GBs) without excessive memory -- block/stream processing required
- **Security:** Token mappings never transmitted -- encrypted at rest (Phase 2), decrypted only locally
- **Portability:** Single binary, zero runtime dependencies, cross-platform
- **Privacy:** No sensitive data in any output, intermediate file, or network request
- **Stack pinned in CLAUDE.md:** clap 4.6.0, regex 1.12.3, serde 1.0.228, serde_json 1.0.149, memmap2 0.9.10, indicatif 0.18.4, tracing 0.1.44, anyhow 1.0.102, thiserror 2.0+, rayon 1.11.0

## Standard Stack

### Core (Phase 1 subset)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.6.0 | CLI argument parsing | Derive macros, auto-generated help, validation. De facto Rust CLI standard. [VERIFIED: CLAUDE.md pin] |
| regex | 1.12.3 | Sensitive data pattern matching | O(m*n) guaranteed, SIMD-accelerated, no catastrophic backtracking. [VERIFIED: CLAUDE.md pin] |
| serde | 1.0.228 | Serialization framework | Universal Rust serialization for JSON log parsing and token map. [VERIFIED: CLAUDE.md pin] |
| serde_json | 1.0.149 | JSON parsing/output | Parse structured JSON logs, re-serialize with tokens. [VERIFIED: CLAUDE.md pin] |
| indicatif | 0.18.4 | Progress bars | Visual feedback on stderr during large file processing. [VERIFIED: CLAUDE.md pin] |
| tracing | 0.1.44 | Structured logging | Internal diagnostics. [VERIFIED: CLAUDE.md pin] |
| tracing-subscriber | 0.3+ | Log output formatting | Console output for tracing. [VERIFIED: CLAUDE.md pin] |
| anyhow | 1.0.102 | Application error handling | Ergonomic error chains for the binary. [VERIFIED: CLAUDE.md pin] |
| thiserror | 2.0+ | Library error handling | Structured error types for core modules. [VERIFIED: CLAUDE.md pin] |

### Phase 1 -- Deferred from Stack

| Library | Reason |
|---------|--------|
| memmap2 | Claude's discretion: may use BufReader for Phase 1 simplicity; memmap2 can be added as optimization later |
| rayon | Parallel regex not needed until performance tuning; sequential is sufficient for correctness-first Phase 1 |
| tokio / tokio-stream | Async not needed for Phase 1 file I/O; synchronous BufReader is simpler and sufficient |
| aes-gcm / argon2 | Encrypted token store is Phase 2 |
| reqwest | Claude API integration is Phase 3 |

**Recommendation:** Phase 1 should use synchronous `BufReader` for file I/O. This is simpler, avoids async complexity, and meets the bounded-memory requirement. `memmap2` and `rayon` can be introduced in later optimization if profiling shows need. [ASSUMED -- discretion area per CONTEXT.md]

### Installation (Phase 1)

```bash
cargo init logtok
cd logtok
cargo add clap --features derive
cargo add regex
cargo add serde --features derive
cargo add serde_json
cargo add indicatif
cargo add tracing
cargo add tracing-subscriber
cargo add anyhow
cargo add thiserror
```

## Architecture Patterns

### Recommended Project Structure

```
logtok/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point, argument parsing, orchestration
│   ├── cli.rs               # clap derive structs for CLI arguments
│   ├── detector.rs          # Regex patterns, sensitive data detection
│   ├── tokenizer.rs         # Token map, deterministic replacement logic
│   ├── compactor.rs         # Duplicate line collapsing with counts
│   ├── processor.rs         # Block processing pipeline (orchestrates detect -> tokenize -> compact)
│   ├── json_processor.rs    # JSON-aware tokenization (recursive Value traversal)
│   └── error.rs             # thiserror error types
├── tests/
│   ├── integration/
│   │   ├── cli_tests.rs     # End-to-end binary tests
│   │   └── fixtures/        # Sample log files for testing
│   └── snapshots/           # insta snapshot files (if used)
```

### Pattern 1: Deterministic Token Map

**What:** A `HashMap<String, String>` that maps original sensitive values to tokens, ensuring the same value always gets the same token regardless of where it appears in the file.

**When to use:** Every tokenization operation.

**Architecture:**

```rust
// Source: Design from CONTEXT.md decisions D-01 through D-05
use std::collections::HashMap;

pub struct TokenMap {
    /// Maps original sensitive value -> token string
    value_to_token: HashMap<String, String>,
    /// Per-category counters for generating sequential IDs
    category_counters: HashMap<String, u16>,
}

impl TokenMap {
    pub fn new() -> Self {
        Self {
            value_to_token: HashMap::new(),
            category_counters: HashMap::new(),
        }
    }

    /// Returns existing token or creates a new one
    pub fn get_or_insert(&mut self, value: &str, category: &str) -> String {
        if let Some(token) = self.value_to_token.get(value) {
            return token.clone();
        }
        let counter = self.category_counters.entry(category.to_string()).or_insert(0);
        *counter += 1;
        let token = format!("[{}_{:03}]", category, counter);
        self.value_to_token.insert(value.to_string(), token.clone());
        token
    }
}
```

**Key detail:** The token map must be shared across all blocks. It lives outside the block processing loop and is passed mutably to each block's processing. This guarantees determinism (TOK-01).

### Pattern 2: Line-Aware Block Processing

**What:** Read the file in configurable-size chunks that always break at newline boundaries, ensuring no sensitive value is split across blocks.

**When to use:** All file processing (decision D-09).

**Architecture:**

```rust
// Source: Rust std BufRead docs + CONTEXT.md D-09/D-10
use std::io::{BufRead, BufReader};
use std::fs::File;

fn process_file(path: &str, block_size: usize) -> anyhow::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut token_map = TokenMap::new();
    let mut block: Vec<String> = Vec::new();
    let mut block_bytes: usize = 0;

    for line in reader.lines() {
        let line = line?;
        block_bytes += line.len() + 1; // +1 for newline
        block.push(line);

        if block_bytes >= block_size {
            process_block(&block, &mut token_map)?;
            block.clear();
            block_bytes = 0;
        }
    }

    // Process remaining lines
    if !block.is_empty() {
        process_block(&block, &mut token_map)?;
    }

    Ok(())
}
```

**Why BufReader over memmap2 for Phase 1:** BufReader is simpler, handles all platforms identically, and naturally provides line-oriented iteration. memmap2 requires manual newline boundary detection on raw bytes. BufReader's `lines()` iterator handles both `\n` and `\r\n`. [ASSUMED -- discretion area]

### Pattern 3: JSON-Aware Tokenization

**What:** Parse JSON log lines into `serde_json::Value`, recursively walk the tree replacing only string values, then re-serialize to valid JSON.

**When to use:** When input is detected as JSON (D-11, D-12).

**Architecture:**

```rust
// Source: serde_json::Value enum + CONTEXT.md D-11/D-12
use serde_json::Value;

fn tokenize_json_value(value: &mut Value, token_map: &mut TokenMap) {
    match value {
        Value::String(s) => {
            // Apply regex detection to the string value
            let tokenized = apply_regex_replacements(s, token_map);
            *s = tokenized;
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                tokenize_json_value(item, token_map);
            }
        }
        Value::Object(map) => {
            // D-12: Only values, not keys
            for (_key, val) in map.iter_mut() {
                tokenize_json_value(val, token_map);
            }
        }
        // Null, Bool, Number -- leave unchanged
        _ => {}
    }
}
```

**JSON detection heuristic:** Try parsing the first non-empty line as JSON. If it parses as an object or array, treat the file as JSON Lines (one JSON object per line). Otherwise, treat as plain text. [ASSUMED]

### Pattern 4: Log Compaction

**What:** Collapse consecutive identical lines (after tokenization) into a single line with a count prefix, preserving ordering and every unique event.

**When to use:** Always applied during output (D-06, D-07).

**Architecture:**

```rust
// Source: CONTEXT.md D-06/D-07
struct Compactor {
    last_line: Option<String>,
    count: usize,
}

impl Compactor {
    fn new() -> Self {
        Self { last_line: None, count: 0 }
    }

    /// Feed a tokenized line; returns a completed line if the streak breaks
    fn feed(&mut self, line: String) -> Option<String> {
        match &self.last_line {
            Some(last) if *last == line => {
                self.count += 1;
                None
            }
            _ => {
                let output = self.flush();
                self.last_line = Some(line);
                self.count = 1;
                output
            }
        }
    }

    /// Flush the current streak
    fn flush(&mut self) -> Option<String> {
        self.last_line.take().map(|line| {
            if self.count > 1 {
                format!("[x{}] {}", self.count, line)
            } else {
                line
            }
        })
    }
}
```

**Compaction format:** `[x5] [IP_001] - GET /api/health 200` for 5 identical consecutive lines. This is concise for LLM consumption. [ASSUMED -- format not specified in decisions]

### Pattern 5: CLI Structure with clap Derive

**What:** Flat CLI using clap's derive macros. No subcommands per D-14.

```rust
// Source: clap docs + CONTEXT.md D-13 through D-17
use clap::Parser;

/// Tokenize sensitive data out of log files for safe AI analysis
#[derive(Parser, Debug)]
#[command(name = "logtok", version, about)]
struct Cli {
    /// Path to the log file to tokenize
    file: std::path::PathBuf,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,

    /// Block size in bytes for processing (default: 65536)
    #[arg(long, default_value = "65536")]
    block_size: usize,

    /// Suppress progress bar
    #[arg(short, long)]
    quiet: bool,
}
```

### Anti-Patterns to Avoid

- **Loading entire file into memory:** Violates INF-03. Always use line-by-line BufReader accumulating into blocks.
- **Compiling regex per line:** Regex compilation is expensive (microseconds to milliseconds). Compile all patterns once at startup, reuse across all lines. Use `lazy_static` or `OnceLock` if needed.
- **Non-deterministic token assignment:** The token map MUST persist across blocks. Never reset counters or create new maps per block.
- **Tokenizing JSON keys:** Decision D-12 explicitly says values only. Keys are structural and needed for diagnosis.
- **Progress bar on stdout:** Decision D-16 says stderr. `indicatif` defaults to stderr which is correct. Never mix progress output with tokenized data on stdout.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CLI argument parsing | Custom arg parser | clap 4.x derive macros | Handles help, version, validation, completions automatically |
| Regex engine | Custom pattern matcher | regex crate | SIMD-accelerated, guaranteed linear time, battle-tested |
| JSON parsing | Custom JSON parser | serde_json | Handles edge cases (escaping, unicode, nesting) correctly |
| Progress bars | Custom terminal output | indicatif | Handles terminal width, draw rate, ETA calculation, stderr targeting |
| Error handling | Manual Result chains | anyhow (binary) + thiserror (library) | Context chains, downcasting, Display impl automation |

**Key insight:** Every "simple" custom implementation in this domain has edge cases that consume days. Regex has catastrophic backtracking if hand-rolled, JSON has Unicode escaping nightmares, progress bars have terminal compatibility issues.

## Common Pitfalls

### Pitfall 1: Regex Compilation in Hot Loop

**What goes wrong:** Compiling regex patterns on every line makes processing 100-1000x slower.
**Why it happens:** Natural to write `Regex::new(pattern)` inside a function called per-line.
**How to avoid:** Compile all regex patterns once at startup into a struct (e.g., `DetectionPatterns`). Pass as reference to all processing functions.
**Warning signs:** Processing takes seconds per MB instead of milliseconds.

### Pitfall 2: Token Map Not Shared Across Blocks

**What goes wrong:** Same IP address gets different tokens in different blocks, violating TOK-01.
**Why it happens:** Block processing creates independent token maps per chunk.
**How to avoid:** Token map is created once before the processing loop and passed as `&mut` to each block.
**Warning signs:** Running diff on output shows same original value with different tokens.

### Pitfall 3: JSON String Escaping on Re-Serialization

**What goes wrong:** Tokens inserted into JSON strings get double-escaped: `"[IP_001]"` becomes `"[IP_001]"` which is fine, but if the original value had backslashes or quotes, replacement can corrupt the JSON.
**Why it happens:** Replacing within serialized JSON strings rather than deserialized values.
**How to avoid:** Always parse JSON into `serde_json::Value`, replace in the deserialized `String` variant, then re-serialize. Let serde handle escaping.
**Warning signs:** Output JSON fails to parse, or contains unexpected escape sequences.

### Pitfall 4: Regex Patterns Matching Tokens

**What goes wrong:** A token like `[IP_001]` gets matched by a subsequent regex pattern and re-tokenized.
**Why it happens:** If patterns are applied sequentially and a later pattern matches the token format.
**How to avoid:** Apply all regex patterns to the original text in a single pass. Use `regex::RegexSet` for detection, then apply individual replacements to the original matches. Alternatively, replace left-to-right tracking positions to avoid re-matching already-replaced regions.
**Warning signs:** Tokens contain nested tokens or garbled output.

### Pitfall 5: Line Ending Inconsistencies

**What goes wrong:** Windows `\r\n` vs Unix `\n` causes mismatches in compaction or unexpected whitespace.
**Why it happens:** Mixed-origin log files, or Windows-generated logs processed on Linux.
**How to avoid:** `BufReader::lines()` strips both `\n` and `\r\n` automatically. Use it rather than manual splitting.
**Warning signs:** Compaction fails to collapse identical lines, trailing `\r` in output.

### Pitfall 6: Overlapping Regex Matches

**What goes wrong:** An email like `admin@192.168.1.1` matches both the EMAIL and IP patterns, producing garbled output.
**Why it happens:** Multiple regex patterns fire on overlapping regions of the same text.
**How to avoid:** Process matches by priority order. Apply patterns from most specific to least specific (e.g., EMAIL before IP, URL before HOST). Alternatively, find all matches with positions, resolve overlaps by priority, then apply non-overlapping replacements.
**Warning signs:** Partial tokens in output, broken email/URL formatting.

## Code Examples

### Complete Regex Detection Setup

```rust
// Source: regex crate docs + CONTEXT.md D-05 categories
use regex::Regex;

pub struct DetectionPatterns {
    patterns: Vec<(String, Regex)>, // (category, compiled_regex)
}

impl DetectionPatterns {
    pub fn new() -> Self {
        // Order matters: more specific patterns first
        let patterns = vec![
            ("EMAIL", r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"),
            ("URL", r"https?://[^\s\"\]>]+"),
            ("IP", r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b"),
            ("KEY", r"(?i)(?:api[_-]?key|token|secret|password|bearer)\s*[=:]\s*['\"]?([a-zA-Z0-9_\-/.+=]{16,})['\"]?"),
            ("PASS", r#"(?i)(?:password|passwd|pwd)\s*[=:]\s*['"]?(\S+)['"]?"#),
            ("HOST", r"\b[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.(?:internal|local|corp|intra|private|lan)\b"),
            ("PATH", r"(?:/[a-zA-Z0-9._-]+){3,}"),
        ];

        Self {
            patterns: patterns
                .into_iter()
                .map(|(cat, pat)| (cat.to_string(), Regex::new(pat).unwrap()))
                .collect(),
        }
    }
}
```

**Note on patterns:** These are starter patterns for Phase 1. Phase 2 adds configurable detection rules (DET-05). Phase 1 patterns should cover the basics: IPs, emails, URLs, API keys, passwords, internal hostnames, and file paths. [ASSUMED -- exact patterns are Claude's discretion area]

### Progress Bar Integration

```rust
// Source: indicatif docs + CONTEXT.md D-16
use indicatif::{ProgressBar, ProgressStyle};

fn create_progress(file_size: u64, quiet: bool) -> Option<ProgressBar> {
    if quiet {
        return None;
    }
    let pb = ProgressBar::new(file_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("=>-"),
    );
    // indicatif draws to stderr by default -- correct per D-16
    Some(pb)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `lazy_static!` for regex | `std::sync::OnceLock` (stable) | Rust 1.70 (2023) | No macro dependency needed for lazy static initialization |
| `structopt` for CLI | `clap` 4.x derive | clap 3.0 (2022) | structopt merged into clap; use clap directly |
| Manual `impl Error` | `thiserror` 2.0 derive | 2024 | Automatic Display + Error implementations |
| `failure` crate | `anyhow` + `thiserror` | 2020 | failure is abandoned; anyhow/thiserror is the standard |

**Deprecated/outdated:**
- `structopt`: Merged into clap 3+. Use `clap` with derive feature. [VERIFIED: CLAUDE.md]
- `failure`: Abandoned. Use `anyhow` (application) + `thiserror` (library). [VERIFIED: CLAUDE.md]
- `lazy_static`: Still works but `OnceLock` from std is preferred for simple cases. [ASSUMED]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | BufReader is better than memmap2 for Phase 1 simplicity | Architecture Patterns | Low -- memmap2 could be swapped in later; BufReader is correct and sufficient |
| A2 | JSON detection via first-line parsing heuristic | Pattern 3 | Medium -- could misdetect; may need a `--format` flag |
| A3 | Compaction format `[x5] line` is good for LLM consumption | Pattern 4 | Low -- format can be adjusted; concept is locked by D-06 |
| A4 | Starter regex patterns are sufficient for Phase 1 | Code Examples | Low -- Phase 2 adds configurable patterns; Phase 1 just needs basics |
| A5 | `OnceLock` preferred over `lazy_static` for simple statics | State of the Art | Very low -- both work; `OnceLock` avoids dependency |
| A6 | Sequential (non-parallel) processing is sufficient for Phase 1 | Standard Stack | Low -- rayon can be added later if profiling shows need |

## Open Questions

1. **JSON detection heuristic vs explicit flag**
   - What we know: D-11 says JSON output must remain valid JSON; D-09 says line-aware splitting
   - What's unclear: Should the tool auto-detect JSON vs plain text, or require `--format json|text`?
   - Recommendation: Auto-detect by trying to parse the first non-empty line as JSON, with an optional `--format` override flag. This covers most cases and allows explicit control when heuristic fails.

2. **Compaction output format**
   - What we know: D-06 says collapse duplicates with counts, D-07 says optimize for LLM
   - What's unclear: Exact format string for compacted lines (e.g., `[x5] line` vs `(5x) line` vs `line [repeated 5 times]`)
   - Recommendation: Use `[x5] line` -- concise, grep-friendly, and minimal token overhead for Claude.

3. **Multi-line JSON objects**
   - What we know: D-09 says line-aware splitting at newline boundaries
   - What's unclear: How to handle pretty-printed JSON spanning multiple lines
   - Recommendation: Phase 1 supports JSON Lines (one object per line) only. Pretty-printed multi-line JSON is a future enhancement. Document this limitation.

4. **Token counter overflow**
   - What we know: D-04 specifies 3-digit zero-padded counters (001-999)
   - What's unclear: What happens if a category exceeds 999 unique values?
   - Recommendation: Overflow to 4+ digits gracefully (e.g., `[IP_1000]`). Log a warning via tracing. Do not error.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain (rustc + cargo) | Everything | NO | -- | Must install via rustup |
| rustup | Toolchain management | NO | -- | Download from https://rustup.rs |

**Missing dependencies with no fallback:**
- **Rust toolchain not installed.** Must install before any development. Run: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` (Unix) or download from https://rustup.rs (Windows). The planner MUST include a Wave 0 / setup task for toolchain installation.

**Missing dependencies with fallback:**
- None. All other dependencies are Rust crates installed via `cargo add`.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | N/A -- Phase 1 has no auth |
| V3 Session Management | No | N/A -- CLI tool, no sessions |
| V4 Access Control | No | N/A -- local file processing only |
| V5 Input Validation | Yes | regex crate (linear time, no ReDoS); validate file paths; validate block-size argument range |
| V6 Cryptography | No | Deferred to Phase 2 (encrypted token store) |

### Known Threat Patterns for Rust CLI

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Regex denial of service (ReDoS) | Denial of Service | Rust `regex` crate guarantees O(m*n) -- immune to catastrophic backtracking [VERIFIED: regex crate docs] |
| Path traversal in file argument | Tampering | Validate file path exists and is a regular file before processing |
| Token leakage to stdout | Information Disclosure | Token map (original values) NEVER written to stdout; only tokenized output goes to stdout |
| Large file memory exhaustion | Denial of Service | Block-based processing with bounded memory (D-09, D-10) |

## Sources

### Primary (HIGH confidence)
- [CLAUDE.md](../../CLAUDE.md) -- Technology stack, version pins, architecture decisions
- [01-CONTEXT.md](./01-CONTEXT.md) -- User decisions D-01 through D-17
- [REQUIREMENTS.md](../../REQUIREMENTS.md) -- Requirements INF-01, INF-03, TOK-01, TOK-02, TOK-05, TOK-06
- [regex crate docs](https://docs.rs/regex/latest/regex/) -- RegexSet, Replacer trait, performance guarantees
- [serde_json Value docs](https://docs.rs/serde_json/latest/serde_json/value/enum.Value.html) -- Recursive enum traversal pattern
- [Rust std BufRead docs](https://doc.rust-lang.org/std/io/trait.BufRead.html) -- lines() iterator, fill_buf behavior
- [clap docs](https://docs.rs/clap/latest/clap/) -- derive macros, Command configuration
- [indicatif docs](https://docs.rs/indicatif/0.18.4/indicatif/) -- ProgressBar, stderr default draw target

### Secondary (MEDIUM confidence)
- [RegexSet multi-pattern matching](https://arcmutex.com/content/regexset-multi-pattern-matching-performance) -- Performance characteristics and limitations
- [Efficiently Processing JSON with Rust, Serde, Tokio](https://www.ixpantia.com/en/blog/json-with-rust-serde-tokio) -- Streaming JSON patterns

### Tertiary (LOW confidence)
- [async-jsonl for JSONL processing](https://blog.ssdd.dev/building-async-jsonl) -- Alternative approach for JSON Lines (not recommended for Phase 1 due to async complexity)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all versions pinned in CLAUDE.md, well-established Rust ecosystem
- Architecture: HIGH -- patterns are standard Rust idioms for text processing; decisions well-constrained by CONTEXT.md
- Pitfalls: HIGH -- common Rust text processing pitfalls are well-documented in ecosystem

**Research date:** 2026-04-13
**Valid until:** 2026-05-13 (stable ecosystem, locked versions)
