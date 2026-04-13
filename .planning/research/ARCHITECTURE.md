# Architecture Research

**Domain:** Log tokenization and privacy-preserving log diagnosis CLI
**Researched:** 2026-04-13
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI Interface                            │
│  (commands: tokenize, diagnose, detokenize, configure)           │
├─────────────────────────────────────────────────────────────────┤
│                      Orchestration Layer                         │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐     │
│  │  Tokenize     │  │  Diagnose    │  │  De-tokenize       │     │
│  │  Command      │  │  Command     │  │  Command           │     │
│  └──────┬───────┘  └──────┬───────┘  └────────┬───────────┘     │
├─────────┴────────────────┴──────────────────┴───────────────────┤
│                      Processing Pipeline                         │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌──────────┐  │
│  │  Source     │  │  Block     │  │  Detection  │  │  Token   │  │
│  │  Reader     │→ │  Splitter  │→ │  Engine     │→ │  Replace │  │
│  └────────────┘  └────────────┘  └────────────┘  └──────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                      Detection Engine                            │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌──────────┐  │
│  │ Credentials│  │ Infra      │  │ Business   │  │ PII      │  │
│  │ Detector   │  │ Detector   │  │ Detector   │  │ Detector │  │
│  └────────────┘  └────────────┘  └────────────┘  └──────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                      Storage & Integration                       │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐     │
│  │  Token Vault  │  │  Claude API  │  │  Output Formatter  │     │
│  │  (encrypted)  │  │  Client      │  │  (md / summary)    │     │
│  └──────────────┘  └──────────────┘  └────────────────────┘     │
└─────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Implementation Approach |
|-----------|----------------|------------------------|
| CLI Interface | Parse commands, flags, validate input, route to orchestrator | cobra (Go) or clap (Rust) — standard CLI framework |
| Source Reader | Read log data from file (v1), connectors (v2) | Interface-based; v1 implements file reader with io.Reader/BufReader |
| Block Splitter | Chunk large files into processable blocks respecting log boundaries | Fixed-size blocks with boundary detection (newline or multi-line entry) |
| Detection Engine | Identify sensitive data in text using detector chain | Registry of detectors, each returning spans of matched sensitive data |
| Credential Detector | Find API keys, tokens, passwords, connection strings | Regex patterns (high-entropy strings, known key formats like AWS/GitHub) |
| Infrastructure Detector | Find IPs, hostnames, file paths, OS details | Regex patterns (IPv4/6, RFC 1918 ranges, FQDN, Unix/Windows paths) |
| Business Logic Detector | Find internal function names, class names, internal URLs | Configurable patterns + heuristics (camelCase/snake_case in error context) |
| PII Detector | Find emails, user IDs, names | Regex for structured PII (emails, phone numbers); configurable rules for IDs |
| Token Replacer | Replace detected spans with consistent tokens | Deterministic token generation: same input always maps to same token |
| Token Vault | Persist encrypted bidirectional token-to-original mappings | AES-256-GCM encrypted local file (SQLite or flat JSON, encrypted at rest) |
| Claude API Client | Send tokenized logs, receive diagnosis | HTTP client with retry, rate limiting, streaming response support |
| De-tokenizer | Replace tokens in Claude's response with originals | Reverse lookup from Token Vault; handles partial matches in prose |
| Output Formatter | Format final output as bullet summary or markdown report | Template-based rendering to stdout or file |

## Recommended Project Structure

```
cmd/
├── root.go                 # CLI root command setup
├── tokenize.go             # tokenize subcommand
├── diagnose.go             # diagnose subcommand (tokenize + send + detokenize)
└── config.go               # configuration management

internal/
├── pipeline/               # Processing pipeline orchestration
│   ├── pipeline.go         # Block-based pipeline coordinator
│   └── block.go            # Block splitting and boundary detection
├── source/                 # Log source abstraction
│   ├── source.go           # Source interface definition
│   ├── file.go             # File source (v1)
│   └── stdin.go            # Stdin/pipe source
├── detect/                 # Detection engine
│   ├── engine.go           # Detector registry and chain runner
│   ├── detector.go         # Detector interface
│   ├── credentials.go      # Credential patterns
│   ├── infrastructure.go   # Infrastructure patterns
│   ├── business.go         # Business logic patterns
│   ├── pii.go              # PII patterns
│   └── patterns/           # Compiled regex pattern sets
│       └── patterns.go
├── token/                  # Tokenization core
│   ├── replacer.go         # Token generation and text replacement
│   ├── vault.go            # Encrypted token store interface
│   ├── vault_file.go       # File-based vault implementation
│   └── crypto.go           # AES-256-GCM encryption utilities
├── claude/                 # Claude API integration
│   ├── client.go           # API client with retry/rate limiting
│   └── prompt.go           # Prompt templates for log diagnosis
├── detokenize/             # De-tokenization
│   └── detokenize.go       # Reverse token replacement in responses
├── output/                 # Output formatting
│   ├── summary.go          # Bullet-point summary format
│   └── report.go           # Detailed markdown report format
└── config/                 # Configuration
    └── config.go           # Config file loading, defaults

testdata/                   # Test fixtures
├── logs/                   # Sample log files for testing
└── expected/               # Expected tokenized outputs
```

### Structure Rationale

- **cmd/**: Thin CLI layer. Each command file wires up dependencies and calls into `internal/`. Keeps CLI concerns separate from logic.
- **internal/pipeline/**: Owns the block-based processing flow. This is the central coordinator that pulls from sources, runs detection, and outputs tokenized text.
- **internal/source/**: Interface abstraction means adding Elasticsearch/CloudWatch/Kafka connectors in v2 is just adding new implementations, zero changes to pipeline.
- **internal/detect/**: Each detector category is independent. The engine runs them as a chain. New detector types are added by implementing the interface and registering.
- **internal/token/**: The vault is the security boundary. Crypto operations are isolated here. Vault interface allows swapping file-based for other backends later.
- **internal/claude/**: Isolated API integration. If the user wants to use a different LLM, this is the only package that changes.

## Architectural Patterns

### Pattern 1: Source-Transform-Sink Pipeline

**What:** The dominant pattern in log processing tools (Vector, Fluent Bit, Logstash). Data flows through a DAG: Sources emit events, Transforms process them, Sinks consume results. For this tool, the pipeline is linear (not a DAG) since there is one source, one transform chain, and one sink per invocation.

**When to use:** Any data processing that reads from input, transforms, and writes output. This is the architecture.

**Trade-offs:** Simple and composable. The linear variant avoids DAG complexity while preserving the ability to add fan-out later.

**Example:**
```go
// Source interface - v1 is files, v2 adds connectors
type Source interface {
    // Read returns the next block of log data.
    // Returns io.EOF when exhausted.
    Read(ctx context.Context) (Block, error)
    Close() error
}

// Detector interface - each sensitive data category
type Detector interface {
    Name() string
    Detect(text []byte) []Span // Span = {Start, End, Category, Original}
}

// Sink interface - where tokenized output goes
type Sink interface {
    Write(ctx context.Context, block TokenizedBlock) error
    Close() error
}
```

### Pattern 2: Block-Based Streaming with Boundary Detection

**What:** Process large files in fixed-size blocks (e.g., 1MB) rather than loading entirely into memory. Each block must respect log entry boundaries -- a block boundary must not split a multi-line log entry (stack trace, JSON object). The block splitter reads ahead to find the nearest safe boundary.

**When to use:** Always, for files above a threshold (e.g., 10MB). Below that, single-block processing is fine.

**Trade-offs:** Enables processing of multi-GB files with constant memory. Adds complexity around boundary detection -- must handle multi-line entries (stack traces, JSON logs) correctly. Block size is tunable for throughput vs. memory.

**Example:**
```go
type BlockSplitter struct {
    reader    io.Reader
    blockSize int
    overlap   int // bytes of overlap for boundary safety
}

func (bs *BlockSplitter) Next() (Block, error) {
    buf := make([]byte, bs.blockSize+bs.overlap)
    n, err := io.ReadFull(bs.reader, buf)
    if n == 0 {
        return Block{}, io.EOF
    }
    // Find last complete log entry boundary
    boundary := findLastEntryBoundary(buf[:n])
    // Seek reader back to boundary for next block
    return Block{Data: buf[:boundary]}, err
}
```

### Pattern 3: Detector Chain with Span Merging

**What:** Run multiple detectors in sequence, each producing a list of byte spans that contain sensitive data. Merge overlapping spans before replacement to avoid double-tokenization. Replace spans from right-to-left to preserve byte offsets.

**When to use:** Always. Multiple detector categories will produce overlapping matches (e.g., an email address contains a hostname).

**Trade-offs:** Right-to-left replacement is simple and correct. Span merging prevents garbled output. Detector ordering does not matter because merging normalizes overlaps.

**Example:**
```go
func DetectAll(text []byte, detectors []Detector) []Span {
    var allSpans []Span
    for _, d := range detectors {
        allSpans = append(allSpans, d.Detect(text)...)
    }
    // Sort by start position, merge overlapping
    return mergeSpans(allSpans)
}

func ReplaceSpans(text []byte, spans []Span, vault *Vault) []byte {
    // Process right-to-left to preserve offsets
    sort.Slice(spans, func(i, j int) bool {
        return spans[i].Start > spans[j].Start
    })
    result := make([]byte, len(text))
    copy(result, text)
    for _, span := range spans {
        token := vault.GetOrCreate(span.Original, span.Category)
        result = replaceRange(result, span.Start, span.End, token)
    }
    return result
}
```

### Pattern 4: Deterministic Token Generation

**What:** The same sensitive value always maps to the same token within a session (and across sessions if the vault persists). Tokens are human-readable placeholders that preserve category context: `[CRED_001]`, `[IP_003]`, `[EMAIL_012]`. This lets Claude reason about relationships ("the same IP appears in both error messages") without seeing the actual value.

**When to use:** Always. Non-deterministic tokens would destroy the ability to correlate across log entries.

**Trade-offs:** Deterministic mapping means the vault must be consulted on every replacement (lookup or insert). This is fast since the vault is in-memory with periodic flush to encrypted disk. Token format with category prefix aids Claude's diagnosis.

### Pattern 5: Encrypted Vault with Session Persistence

**What:** Token-to-original mappings stored in an AES-256-GCM encrypted file. Key derived from a user-provided passphrase via Argon2id (or system keyring). Vault loaded into memory at start, flushed to disk on changes. Separate vault files per project/context.

**When to use:** Always. The vault is the security-critical component.

**Trade-offs:** File-based vault is simple and portable (works in Docker, K8s, CI). SQLite would add a dependency for marginal benefit at v1 scale. Argon2id is the current best practice for key derivation. System keyring integration is a nice-to-have for developer ergonomics but not required for v1.

## Data Flow

### Primary Flow: Tokenize-Diagnose-Detokenize

```
[Log File on Disk]
        │
        ▼
  ┌─────────────┐
  │ Source Reader│ ── reads file in buffered chunks
  └──────┬──────┘
         │ raw bytes
         ▼
  ┌─────────────┐
  │Block Splitter│ ── splits at log entry boundaries
  └──────┬──────┘
         │ Block (complete log entries)
         ▼
  ┌─────────────┐
  │  Detection  │ ── runs all detectors, produces spans
  │  Engine     │
  └──────┬──────┘
         │ []Span (start, end, category)
         ▼
  ┌─────────────┐       ┌──────────────┐
  │   Token     │ ◄───► │  Token Vault │ (lookup or create token)
  │  Replacer   │       │  (in-memory) │
  └──────┬──────┘       └──────┬───────┘
         │ tokenized text       │ periodic flush
         ▼                      ▼
  ┌─────────────┐       ┌──────────────┐
  │  Tokenized  │       │  Encrypted   │
  │  Output     │       │  Vault File  │
  └──────┬──────┘       └──────────────┘
         │
         ▼ (if diagnose mode)
  ┌─────────────┐
  │ Claude API  │ ── sends tokenized logs + diagnosis prompt
  │ Client      │
  └──────┬──────┘
         │ Claude's response (contains tokens)
         ▼
  ┌──────────────┐      ┌──────────────┐
  │ De-tokenizer │ ◄──  │  Token Vault │ (reverse lookup)
  └──────┬───────┘      └──────────────┘
         │ readable diagnosis
         ▼
  ┌──────────────┐
  │   Output     │ ── bullet summary or markdown report
  │  Formatter   │
  └──────────────┘
```

### Key Data Flows

1. **Tokenize flow:** File -> Block Splitter -> Detection -> Replacement -> Tokenized output. Vault is consulted for every replacement to ensure deterministic tokens. Output can be written to file, piped, or copied to clipboard.

2. **Diagnose flow:** Runs tokenize flow first, then sends tokenized output to Claude API, receives diagnosis, de-tokenizes the response, formats output. This is the primary user workflow.

3. **Vault flow:** In-memory hashmap (original -> token, token -> original) loaded from encrypted file at startup. New mappings added during tokenization. Flushed to encrypted file at end of run (or periodically for long runs). Never transmitted over network.

4. **De-tokenize flow:** Scans Claude's response for token patterns (regex: `\[CATEGORY_\d+\]`), looks up each in vault, replaces with original. Must handle tokens appearing in prose, code blocks, bullet points.

## Scaling Considerations

| Concern | Small files (<100MB) | Large files (1-10GB) | Massive files (>10GB) |
|---------|---------------------|---------------------|----------------------|
| Memory | Load entire file | Block-based, 1-4MB blocks | Block-based, parallel block processing |
| Detection | Single-pass regex | Per-block regex, merge cross-boundary matches | Same + worker pool for parallel detection |
| Vault | In-memory map | In-memory map (millions of entries is fine) | In-memory map, consider LRU if 100M+ unique tokens |
| Claude API | Single request | Chunked requests (API has input limits) | Summarize per-block, send aggregated tokenized context |

### Scaling Priorities

1. **First bottleneck -- Detection regex performance:** Regex compilation should happen once at startup, not per block. Use compiled regex sets. Pre-filter lines that cannot contain sensitive data (e.g., blank lines, known-safe prefixes). Google RE2-style engines (linear time) over backtracking PCRE for safety.

2. **Second bottleneck -- Claude API context window:** Large log files will exceed Claude's context window. Strategy: send most relevant blocks (error context, stack traces) rather than entire files. The block splitter should tag blocks with relevance signals (contains ERROR, FATAL, exception keywords).

3. **Third bottleneck -- Vault size for extremely large log sets:** Unlikely to be a real problem. A million unique tokens is ~100MB of memory. Only becomes an issue at truly extreme scale.

## Anti-Patterns

### Anti-Pattern 1: Loading Entire File into Memory

**What people do:** `ioutil.ReadAll(file)` or equivalent, process the whole thing at once.
**Why it's wrong:** Fails on multi-GB log files. OOM kills the process. Violates the core performance constraint.
**Do this instead:** Block-based processing from day one. Even if v1 only handles small files, the block-based architecture must be the foundation, not bolted on later.

### Anti-Pattern 2: Sequential Single-Pass Detection

**What people do:** Run one massive regex with alternation for all patterns: `(email|ip|key|...)`.
**Why it's wrong:** One giant regex is unmaintainable, hard to debug, and a nightmare to extend. Adding a new pattern means modifying a monolithic expression.
**Do this instead:** Separate detectors per category, each with their own pattern set. Run them independently, merge spans afterward. Easier to test, debug, and extend.

### Anti-Pattern 3: Non-Deterministic Token Assignment

**What people do:** Generate random tokens for each match without checking if the value was seen before.
**Why it's wrong:** The same IP appearing in 50 log lines gets 50 different tokens. Claude cannot correlate them. The diagnosis becomes useless ("IP_001 failed to connect to IP_047" when they are the same IP).
**Do this instead:** Vault-backed deterministic assignment. Same original value always yields the same token.

### Anti-Pattern 4: Tight Coupling to Claude API

**What people do:** Hard-code Claude API calls throughout the codebase.
**Why it's wrong:** Makes testing impossible without API calls. Prevents supporting other LLMs. Makes the tokenization step dependent on the diagnosis step.
**Do this instead:** Tokenization is a standalone capability. The Claude client is behind an interface. The `tokenize` command works independently of the `diagnose` command.

### Anti-Pattern 5: Storing Vault Unencrypted

**What people do:** Write token mappings as plaintext JSON for convenience during development, plan to "add encryption later."
**Why it's wrong:** The vault IS the sensitive data. If someone gets the vault, they can reverse every token. "Later" often means "after the first security incident."
**Do this instead:** Encrypt from day one. AES-256-GCM is straightforward to implement. The development cost is minimal compared to the security risk.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| Claude API | HTTPS REST client with exponential backoff | Token-based auth; respect rate limits; stream responses for large outputs |
| File System | io.Reader/io.Writer abstractions | Must handle symlinks, permissions, large files; cross-platform path handling |
| System Keyring (optional) | OS-specific keyring APIs | macOS Keychain, Windows Credential Manager, Linux Secret Service; nice-to-have for vault key storage |
| Clipboard | OS-specific clipboard APIs | Fallback output method; `pbcopy`/`xclip`/`clip.exe` |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| CLI -> Pipeline | Direct function calls | CLI constructs pipeline with configured components, calls Run() |
| Pipeline -> Detectors | Interface method calls | Pipeline owns detector registry; detectors are stateless |
| Pipeline -> Vault | Interface method calls | Vault is shared across blocks; thread-safe if parallel processing |
| Diagnose -> Claude Client | Interface method calls | Client interface allows mocking for tests |
| Diagnose -> De-tokenizer | Direct function calls | De-tokenizer takes vault reference and response text |

### Suggested Build Order (Dependencies)

Build order follows the dependency chain from bottom up:

```
Phase 1: Foundation (no external dependencies)
  1. Token Vault (crypto + storage) -- everything depends on this
  2. Detection Engine + Detector interface
  3. Credential Detector + Infrastructure Detector (most common patterns)

Phase 2: Core Pipeline
  4. Block Splitter (depends on: log format understanding)
  5. Token Replacer (depends on: Detection Engine + Vault)
  6. File Source Reader (depends on: Block Splitter)
  7. Pipeline Orchestrator (depends on: Source + Detection + Replacer)
  8. CLI Tokenize Command (depends on: Pipeline)

Phase 3: Diagnosis Loop
  9. Claude API Client (depends on: nothing internal, but needs tokenized output)
  10. De-tokenizer (depends on: Vault)
  11. Output Formatter (depends on: nothing)
  12. CLI Diagnose Command (depends on: Pipeline + Claude Client + De-tokenizer + Formatter)

Phase 4: Completeness
  13. PII Detector + Business Logic Detector
  14. Clipboard output
  15. Configuration management
  16. Claude Code skill integration
```

**Rationale:** The vault and detection engine are the architectural foundation. Build them first so they can be tested in isolation. The pipeline assembles these pieces. Claude integration comes after tokenization works standalone -- this validates the core value proposition before adding network dependencies.

## Sources

- [Vector Components - Source, Transform, Sink pattern](https://vector.dev/components/)
- [Elastic Observability - PII detection with NER and regex in logs](https://www.elastic.co/observability-labs/blog/pii-ner-regex-assess-redact-part-1)
- [HashiCorp Vault - Tokenization transform patterns](https://developer.hashicorp.com/vault/docs/secrets/transform/tokenization)
- [OneUptime - Data masking pipeline for PII redaction](https://oneuptime.com/blog/post/2026-02-06-data-masking-pipeline-pii-redaction/view)
- [HashiCorp go-plugin - Plugin system over RPC for Go](https://github.com/hashicorp/go-plugin)
- [OneUptime - High-throughput data ingestion pipeline in Rust](https://oneuptime.com/blog/post/2026-01-25-high-throughput-data-ingestion-pipeline-rust/view)
- [Hoop.dev - Mask PII in production logs best practices](https://hoop.dev/blog/mask-pii-in-production-logs-data-masking-best-practices)

---
*Architecture research for: Log tokenization and privacy-preserving log diagnosis CLI*
*Researched: 2026-04-13*
