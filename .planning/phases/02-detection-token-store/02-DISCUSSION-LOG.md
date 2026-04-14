# Phase 2: Detection & Token Store - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-14
**Phase:** 02-detection-token-store
**Areas discussed:** Detection expansion, Configuration design, Token store design

---

## Detection Expansion

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal addition | Connection strings only, rely on custom patterns for edge cases | |
| Standard coverage | Connection strings, phone numbers, UUIDs, AWS ARNs | |
| Comprehensive | All above plus SSNs, credit cards, JWTs, PEM blocks, MAC addresses, cloud IDs | ✓ |

**User's choice:** Comprehensive coverage plus: connection strings for Kafka/Redis/Postgres specifically, DNS/domain names, hostnames, OS type/version strings, and structured name/username detection.

**Notes:** User wants broad out-of-the-box detection. Specifically called out Kafka, Redis, and PostgreSQL connection strings. Added DNS and OS type coverage beyond the standard options presented.

### Name Detection Sub-question

| Option | Description | Selected |
|--------|-------------|----------|
| Structured only | Detect names in key=value contexts (user=X, "name": "X") | ✓ |
| Structured + common log patterns | Above plus "User 'john.doe' logged in" style patterns | |
| Local LLM | Context-based detection via Ollama/llama.cpp | Deferred to v2 |

**User's choice:** Structured only for v1. User asked about local LLM possibility — confirmed it's feasible but conflicts with single-binary, performance, and determinism constraints. User agreed to defer LLM detection to v2 as a todo.

---

## Configuration Design

### Config Format & Location

| Option | Description | Selected |
|--------|-------------|----------|
| TOML at ~/.config/logtok/ | XDG-friendly, user-level | |
| TOML at ~/.logtok/ | Simple, cross-platform | |
| JSON | Already have serde_json, less human-friendly | |
| Per-project .logtok.toml | In project root, language-agnostic | ✓ |

**User's choice:** Per-project `.logtok.toml` in the project directory. User described the use case: a generic package downloaded into any project regardless of language, with a config file that is also generic.

### Category Enable/Disable

| Option | Description | Selected |
|--------|-------------|----------|
| All-on by default | Every category active, config only to disable/customize | ✓ |
| Explicit opt-in | Nothing detected until configured | |

**User's choice:** All-on by default.

---

## Token Store Design

### Store Location

| Option | Description | Selected |
|--------|-------------|----------|
| Project directory (.logtok/store.enc) | Next to config, easy to .gitignore | ✓ |
| User home (~/.logtok/stores/) | Separates secrets from project files | |

**User's choice:** Project directory.

### Passphrase Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Prompt + env var | Interactive first use, LOGTOK_KEY for automation | |
| Always prompt | Ask every time | |
| Env var only | LOGTOK_KEY required, no prompt | ✓ |

**User's choice:** Env var only.

### Store Lifecycle

| Option | Description | Selected |
|--------|-------------|----------|
| Append-only, manual reset | Grows until --reset-store | ✓ |
| Auto-expire TTL | Entries expire after configurable period | ✓ |

**User's choice:** Both — append-only with optional TTL expiry.

---

## Claude's Discretion

- Dry-run output format and detail level (user chose not to discuss this area)

## Deferred Ideas

- LLM-based contextual name/PII detection via local Ollama — v2 feature
