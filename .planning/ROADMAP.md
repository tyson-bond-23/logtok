# Roadmap: Logs Tokeniser

## Overview

Deliver a CLI tool that tokenizes sensitive data out of logs, sends sanitized logs to Claude for diagnosis, and de-tokenizes the results -- all without secrets ever leaving the local machine. Phase 1 builds the core tokenization engine with block-based processing and both log formats. Phase 2 adds all detection categories, encrypted persistent token storage, and user configuration. Phase 3 completes the loop with Claude API diagnosis, output formatting, and cross-platform builds.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Core Tokenization Engine** - Block-based pipeline that reads logs, produces deterministic category-prefixed tokens, handles JSON and plain text
- [ ] **Phase 2: Detection & Token Store** - Pattern-based detection for all sensitive data categories with encrypted persistent storage and user configuration
- [ ] **Phase 3: Diagnosis & Delivery** - Claude API integration, de-tokenization of results, output formatting, and cross-platform binary distribution

## Phase Details

### Phase 1: Core Tokenization Engine
**Goal**: User can feed a log file into the tool and get back a tokenized version with deterministic, category-prefixed placeholders -- processing large files without excessive memory usage
**Depends on**: Nothing (first phase)
**Requirements**: INF-01, INF-03, TOK-01, TOK-02, TOK-05, TOK-06
**Success Criteria** (what must be TRUE):
  1. User can run a single binary that accepts a log file path and outputs tokenized content
  2. The same sensitive value in different locations always produces the same token
  3. Tokens are visually distinguishable with category prefixes (e.g., `[IP_001]`, `[KEY_001]`)
  4. User can tokenize both structured JSON logs and unstructured plain text logs
  5. A multi-GB log file processes with bounded memory usage (block-based, not full file load)
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md — Scaffold Rust project, CLI argument parsing, error types
- [x] 01-02-PLAN.md — Detection engine, deterministic token map, JSON and plain text tokenization
- [x] 01-03-PLAN.md — Block processing pipeline, compaction, progress bar, end-to-end wiring

### Phase 2: Detection & Token Store
**Goal**: User can detect all categories of sensitive data (credentials, PII, infrastructure) with configurable rules, preview what would be tokenized, and have mappings persist encrypted across sessions
**Depends on**: Phase 1
**Requirements**: DET-01, DET-02, DET-03, DET-04, DET-05, TOK-03, TOK-04
**Success Criteria** (what must be TRUE):
  1. User can tokenize logs containing API keys, passwords, connection strings, emails, IPs, hostnames, file paths, and internal URLs -- all detected automatically
  2. User can run a dry-run that shows what would be tokenized without modifying anything
  3. User can enable/disable detection categories and add custom regex patterns via configuration
  4. Token mappings are encrypted at rest (AES-256-GCM) and reusable across separate CLI invocations
**Plans**: 3 plans

Plans:
- [ ] 02-01-PLAN.md — Expand detection to 19 categories, config-driven architecture, serializable TokenMap
- [ ] 02-02-PLAN.md — TOML config module and AES-256-GCM encrypted token store module
- [ ] 02-03-PLAN.md — CLI integration (--dry-run, --reset-store, --config), processor wiring, end-to-end tests

### Phase 3: Diagnosis & Delivery
**Goal**: User can send tokenized logs to Claude for diagnosis, receive de-tokenized results in readable formats, and run the tool on any platform
**Depends on**: Phase 2
**Requirements**: DIA-01, DIA-02, DIA-03, DIA-04, DIA-05, INF-02
**Success Criteria** (what must be TRUE):
  1. User can send tokenized logs to Claude API and receive a diagnosis without any sensitive data leaving the machine
  2. Claude's tokenized response is automatically de-tokenized back to real values in the output
  3. User can choose between a bullet-point summary (root cause, affected components, fix) or a detailed markdown report
  4. User can copy tokenized output to clipboard for manual use with any LLM
  5. The tool runs as a single binary on Windows, macOS, Linux, and ARM without runtime dependencies
**Plans**: TBD

Plans:
- [ ] 03-01: TBD
- [ ] 03-02: TBD
- [ ] 03-03: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Core Tokenization Engine | 3/3 | Complete | - |
| 2. Detection & Token Store | 0/3 | Planning complete | - |
| 3. Diagnosis & Delivery | 0/3 | Not started | - |
