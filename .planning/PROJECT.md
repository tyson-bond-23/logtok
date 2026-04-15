# Logs Tokeniser

## What This Is

A high-performance, cross-platform CLI tool that tokenizes sensitive data out of application logs — credentials, infrastructure details, business logic, and PII — so they can be safely analyzed by Claude Code for error diagnosis. Claude's results are then de-tokenized back into meaningful, readable output without ever exposing the original secrets.

## Core Value

Engineers can diagnose production log errors through Claude Code without revealing any sensitive information — secrets, internal architecture, business logic, or PII never leave the local environment.

## Requirements

### Validated

- [x] Tokenize log files with block-based processing for performance — Validated in Phase 1: Core Tokenization Engine
- [x] Detect and replace credentials (API keys, tokens, passwords, connection strings) — Validated in Phase 1: Core Tokenization Engine (basic patterns; full coverage in Phase 2)
- [x] Detect and replace infrastructure details (IPs, hostnames, paths, OS info) — Validated in Phase 1: Core Tokenization Engine (basic patterns; full coverage in Phase 2)
- [x] Detect and replace credentials (API keys, tokens, passwords, connection strings) — Validated in Phase 2: Detection & Token Store (19 categories, Luhn-validated CC, JWT, PEM, CONN)
- [x] Detect and replace infrastructure details (IPs, hostnames, paths, OS info) — Validated in Phase 2: Detection & Token Store (DNS, ARN, MAC, OS, UUID added)
- [x] Detect and replace PII (emails, user IDs, names) — Validated in Phase 2: Detection & Token Store (SSN, PHONE, NAME structured matching)
- [x] Store token mappings in encrypted local store, reusable across sessions — Validated in Phase 2: Detection & Token Store (AES-256-GCM + Argon2id)
- [x] Detect and replace business logic references (function names, class names, internal URLs) — Validated in Phase 2: Detection & Token Store (config-driven custom patterns)

### Active

- [ ] Send tokenized logs to Claude via API for diagnosis
- [ ] Claude Code skill integration for interactive diagnosis
- [ ] Clipboard/paste output as fallback method
- [ ] De-tokenize Claude's diagnosis results
- [ ] Output as bullet-point summary (root cause, affected components, fix)
- [ ] Output as detailed markdown report
- [ ] Cross-platform binary (local PC, servers, Docker, K8s pods)

### Out of Scope

- Elasticsearch connector — v2, after core tokenize+diagnose loop proven
- CloudWatch connector — v2
- Datadog/Splunk connectors — v2
- Kafka/RabbitMQ streaming pipeline — v2, architectural foundation laid in v1 block processing
- Real-time log watching/tailing — v2 streaming feature
- Web UI or dashboard — CLI-first tool
- Log storage or persistence — tokenizer only, not a log management system

## Context

- Target users: engineers and SREs who need to diagnose production errors but can't share raw logs externally due to security policies
- Logs come from diverse sources — the tool must handle varied log formats (structured JSON, unstructured text, stack traces, multi-line entries)
- Block-based processing architecture chosen to handle large log volumes efficiently and lay groundwork for future streaming (Kafka/RabbitMQ)
- The encrypted token store enables session persistence — re-running diagnosis on related logs maintains consistent token mappings
- Claude Code integration is the primary diagnosis engine, but the tokenized output format should be generic enough for other LLMs
- Must run anywhere: developer laptops (Windows/Mac/Linux), CI/CD pipelines, Docker containers, Kubernetes pods

## Constraints

- **Performance**: Must handle large log files (GBs) without excessive memory usage — block/stream processing required
- **Security**: Token mappings never transmitted — encrypted at rest, decrypted only locally
- **Portability**: Single binary, zero runtime dependencies, cross-platform (Windows, macOS, Linux, ARM)
- **Privacy**: No sensitive data should appear in any output, intermediate file, or network request

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Block-based processing architecture | Handle large files efficiently, future-proof for streaming | Implemented in Phase 1 |
| Encrypted local token store | Security requirement — mappings persist across sessions but never leave machine | Implemented in Phase 2 (AES-256-GCM + Argon2id) |
| Language choice: Rust | Performance + cross-platform + single binary + memory safety | Decided, implemented in Phase 1 |
| v1 = file-based only | Prove core tokenize→diagnose→de-tokenize loop before adding connectors | — Pending |

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
*Last updated: 2026-04-15 after Phase 2 completion*
