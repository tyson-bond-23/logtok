# Logs Tokeniser

## What This Is

A high-performance, cross-platform CLI tool that tokenizes sensitive data out of application logs — credentials, infrastructure details, business logic, and PII — so they can be safely analyzed by Claude Code for error diagnosis. Claude's results are then de-tokenized back into meaningful, readable output without ever exposing the original secrets.

## Core Value

Engineers can diagnose production log errors through Claude Code without revealing any sensitive information — secrets, internal architecture, business logic, or PII never leave the local environment.

## Current Milestone: v2.0 Developer Experience

**Goal:** Make logtok documentation visually polished and easily accessible — colored CLI help and a professional auto-generated HTML docs page for developers and DevOps engineers.

**Target features:**
- Colored, visually structured CLI help output (syntax-highlighted commands, bold headers, colored sections)
- Auto-generated single-file HTML documentation page (install guide, quick start flow, full command reference with copy buttons)
- `logtok docs` command that generates HTML from the CLI's actual commands
- Professional, clean design targeted at developers and DevOps

## Current State

**Shipped:** v1.0 MVP (2026-04-28)
**Phase 5 complete:** `logtok docs` generates self-contained HTML documentation from clap command tree
**Tech stack:** Rust 1.94, clap 4.6, askama 0.15, regex, serde_json, aes-gcm, argon2, indicatif

**What's working:**
- Tokenize logs (JSON and plain text) with 19-category detection
- Encrypted token store (AES-256-GCM + Argon2id) persists across sessions
- Detokenize Claude Code's diagnosis back to real values
- Configurable detection (disable categories, custom patterns via .logtok.toml)
- Clipboard integration, dry-run preview, progress bar
- CI/CD with cross-platform release workflows

## Requirements

### Validated

- ✓ Tokenize log files with block-based processing for performance — v1.0
- ✓ Detect and replace credentials (API keys, tokens, passwords, connection strings) — v1.0
- ✓ Detect and replace infrastructure details (IPs, hostnames, paths, OS info) — v1.0
- ✓ Detect and replace PII (emails, user IDs, names) — v1.0
- ✓ Store token mappings in encrypted local store, reusable across sessions — v1.0
- ✓ Configure detection rules (enable/disable categories, custom patterns) — v1.0
- ✓ Preview what would be tokenized (dry-run mode) — v1.0
- ✓ De-tokenize Claude's diagnosis back to real values — v1.0
- ✓ Copy tokenized output to clipboard — v1.0
- ✓ Cross-platform binary distribution (CI/CD pipelines) — v1.0
- ✓ Block-based processing for large files (bounded memory) — v1.0

### Active

- [x] Colored, visually structured CLI help output — Validated in Phase 4
- [x] Auto-generated single-file HTML documentation page with install guide, quick start, and command reference — Validated in Phase 5
- [x] `logtok docs` command that generates HTML from CLI's actual commands — Validated in Phase 5
- [x] Professional design targeted at developers and DevOps — Validated in Phase 5

### Out of Scope

- Elasticsearch connector — v2, after core loop proven
- CloudWatch connector — v2
- Datadog/Splunk connectors — v2
- Kafka/RabbitMQ streaming pipeline — v2, architectural foundation laid in v1 block processing
- Real-time log watching/tailing — v2 streaming feature
- Web UI or dashboard — CLI-first tool
- Log storage or persistence — tokenizer only, not a log management system
- NLP/ML-based name detection — requires Python runtime, kills single-binary goal

## Context

- Target users: engineers and SREs who need to diagnose production errors but can't share raw logs externally
- Logs come from diverse sources — handles JSON and plain text formats
- Block-based processing architecture handles large files and lays groundwork for future streaming
- Claude Code integration via static CLAUDE.md instruction block — no plugins or API keys needed
- Tokenized output format is generic enough for any LLM, not just Claude

## Constraints

- **Performance**: Must handle large log files (GBs) without excessive memory usage
- **Security**: Token mappings never transmitted — encrypted at rest, decrypted only locally
- **Portability**: Single binary, zero runtime dependencies, cross-platform
- **Privacy**: No sensitive data should appear in any output, intermediate file, or network request

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Askama 0.15 for HTML docs | Compile-time templating, no runtime overhead, embedded in binary | ✓ Good |
| Language: Rust | Performance + cross-platform + single binary + memory safety | ✓ Good |
| Block-based processing | Handle large files efficiently, future-proof for streaming | ✓ Good |
| Encrypted local token store | Security — mappings persist across sessions but never leave machine | ✓ Good |
| Claude Code via CLAUDE.md | No plugins/API keys needed, works with any LLM | ✓ Good |
| Token format `[CATEGORY_NNN]` | Shorter than `[TOK_CATEGORY_NNN]`, better LLM context efficiency | ✓ Good |
| v1 = file-based only | Prove core tokenize→diagnose→de-tokenize loop before adding connectors | ✓ Good |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-29 after Phase 05 completion*
