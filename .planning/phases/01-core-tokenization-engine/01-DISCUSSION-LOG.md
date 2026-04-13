# Phase 1: Core Tokenization Engine - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-13
**Phase:** 01-core-tokenization-engine
**Areas discussed:** Token format design, Block boundary handling, JSON structure preservation, CLI interface design

---

## Token Format Design

### Bracket Style

| Option | Description | Selected |
|--------|-------------|----------|
| Square brackets | `[TOK_IP_001]` — visually distinct, unlikely to collide | ✓ |
| Angle brackets | `<TOK_IP_001>` — conflicts with XML/HTML in logs | |
| Double curly braces | `{{TOK_IP_001}}` — template-style, verbose | |

**User's choice:** Square brackets
**Notes:** None

### Category Prefix Naming

| Option | Description | Selected |
|--------|-------------|----------|
| Short category (TOK_ prefix) | `[TOK_IP_001]`, `[TOK_KEY_001]` | |
| Full category name | `[TOK_IPADDR_001]`, `[TOK_APIKEY_001]` | |
| Category only, no TOK prefix | `[IP_001]`, `[KEY_001]` | ✓ |

**User's choice:** Category only, no TOK prefix
**Notes:** Shorter tokens = fewer Claude tokens

### Counter Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Per-category | `[IP_001]`, `[IP_002]`, `[KEY_001]` — counters reset per category | ✓ |
| Global counter | `[IP_001]`, `[IP_002]`, `[KEY_003]` — single incrementing counter | |

**User's choice:** Per-category
**Notes:** None

### Counter Padding

| Option | Description | Selected |
|--------|-------------|----------|
| 3-digit padding | `[IP_001]` — consistent width | ✓ |
| No padding | `[IP_1]` — shorter but uneven | |
| You decide | Claude picks | |

**User's choice:** 3-digit padding
**Notes:** None

### Compaction Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Structural compaction | Collapse duplicates with count, normalize whitespace, preserve every unique event | ✓ |
| Aggressive compaction | Structural + stack trace trimming + error grouping | |
| You decide | Claude picks | |

**User's choice:** Structural compaction
**Notes:** User wants logs optimized for Claude readability and token efficiency. Envisions a companion Claude Code skill that understands the compact format (deferred to Phase 3).

---

## Block Boundary Handling

### Split Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Line-aware splitting | Read chunks, always split at newline boundaries | ✓ |
| Overlap strategy | Fixed blocks with ~1KB overlap, dedup matches | |
| Record-aware chunking | Parse log format first, chunk by complete records | |

**User's choice:** Line-aware splitting
**Notes:** None

### Block Size

| Option | Description | Selected |
|--------|-------------|----------|
| 64 KB | Good balance of memory efficiency and throughput | |
| 1 MB | Better throughput on modern systems | |
| Configurable with sensible default | User sets via flag, default 64KB | ✓ |

**User's choice:** Configurable with sensible default (64KB)
**Notes:** None

---

## JSON Structure Preservation

### Output Validity

| Option | Description | Selected |
|--------|-------------|----------|
| Valid JSON preserved | Tokens replace values as strings, output is valid JSON | ✓ |
| Best-effort | Regex replacement, usually valid but edge cases may break | |
| Convert to compact format | Extract JSON to compact log form, lose structure | |

**User's choice:** Valid JSON preserved
**Notes:** None

### Key vs Value Tokenization

| Option | Description | Selected |
|--------|-------------|----------|
| Values only | Only scan and replace values, keys untouched | ✓ |
| Both keys and values | Scan everything, may break JSON consumers | |
| You decide | Claude picks | |

**User's choice:** Values only
**Notes:** None

---

## CLI Interface Design

### Binary Name

| Option | Description | Selected |
|--------|-------------|----------|
| logtok | Short, memorable | ✓ |
| logtoken | Slightly more descriptive | |
| ltok | Ultra-short | |
| logs-tokeniser | Matches project name | |

**User's choice:** logtok
**Notes:** None

### Output Destination

**User's choice:** Stdout by default, `--output` / `-o` flag for file output
**Notes:** User specified this directly without selecting from options.

### Progress Reporting

| Option | Description | Selected |
|--------|-------------|----------|
| Progress bar on stderr | indicatif progress bar, `--quiet` to suppress | ✓ |
| Summary only | Just end summary line | |
| You decide | Claude picks | |

**User's choice:** Progress bar on stderr
**Notes:** None

### CLI Shape

| Option | Description | Selected |
|--------|-------------|----------|
| Flat CLI for Phase 1 | `logtok <file>`, flags control behavior | ✓ |
| Subcommands from the start | `logtok tokenize <file>`, extensible structure | |

**User's choice:** Flat CLI for Phase 1
**Notes:** Subcommands deferred to later phases

---

## Claude's Discretion

- Error message formatting and exit codes
- Internal logging levels and tracing configuration
- Regex pattern ordering and optimization strategy
- Memory-mapped file vs buffered reader decision for different file sizes

## Deferred Ideas

- Claude Code skill for interpreting compact+tokenized logs (Phase 3)
- Subcommand structure (Phase 3)
- Aggressive compaction with stack trace trimming (future enhancement)
