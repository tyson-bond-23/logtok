# Feature Landscape

**Domain:** Log tokenization and privacy-preserving log diagnosis via LLM
**Researched:** 2026-04-13

## Table Stakes

Features users expect. Missing = product feels incomplete or untrustworthy.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Credential detection** (API keys, tokens, passwords, connection strings) | Core promise of the tool; secrets-patterns-db has 1600+ patterns as a baseline | Medium | Use established regex databases (secrets-patterns-db, Gitleaks patterns) as foundation. AWS keys, JWT, Bearer tokens, MongoDB URIs, etc. |
| **PII detection** (emails, IP addresses, usernames, phone numbers) | Basic privacy compliance expectation; every competing tool does this | Medium | Regex-based for structured patterns (email, IP, phone). Names/free-text PII are harder -- flag for v2 NLP. |
| **Infrastructure detail detection** (hostnames, file paths, internal URLs, port numbers) | Logs are full of internal topology; leaking this is a security risk | Medium | Path patterns vary by OS. Internal domain patterns need user-configurable rules. |
| **Consistent/deterministic tokenization** | Same IP appearing 50 times in a log must map to the same token so Claude can correlate patterns. Without this, diagnosis is useless. | Medium | Requires a token vault/map. Same value -> same token within and across sessions. John Snow Labs and others confirm this is standard practice. |
| **Reversible de-tokenization** | The entire value proposition: get Claude's answer, map tokens back to real values so the engineer can act on it | Low | Straightforward lookup from token map. Must handle tokens appearing in Claude's natural language output, not just structured fields. |
| **Structured JSON log support** | Most modern applications emit JSON logs; not supporting them is a dealbreaker | Low | Parse JSON, walk values, tokenize sensitive fields. Preserve JSON structure. |
| **Unstructured/plain text log support** | Legacy apps, syslogs, custom formats -- users will have these | Medium | Line-by-line regex scanning. Must not break log readability. |
| **Multi-line log entry handling** (stack traces, exception blocks) | Stack traces are the primary thing engineers want Claude to diagnose. Splitting them into separate entries destroys context. | High | Detect start/continuation patterns. Java stack traces, Python tracebacks, .NET exceptions, Go panics. Stateful parsing required. |
| **Large file handling** (block/stream processing) | Production logs are often GBs; loading entire file into memory is not viable | High | Block-based processing with bounded memory. The project already specifies this as a constraint. |
| **Cross-platform CLI binary** | Engineers work on Mac, Linux, Windows; SREs need it on servers and in containers | Medium | Single binary, zero dependencies. Go or Rust both excel here. |
| **Clipboard/paste output** | Minimum viable way to get tokenized content to Claude without API integration | Low | Copy tokenized output to clipboard or stdout for manual paste. |
| **Clear token format** | Tokens must be obviously not real data (e.g., `[TOKEN_IP_001]` not `192.168.2.99`) so Claude and humans can distinguish them | Low | Use category-prefixed tokens: `[TOK_IP_001]`, `[TOK_KEY_001]`, `[TOK_EMAIL_001]`. Aids Claude's understanding of what type each token represents. |

## Differentiators

Features that set this product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **LLM-native diagnosis loop** (tokenize -> send to Claude -> de-tokenize response) | No existing tool does the full round-trip. Caviard.ai does browser-level redaction for chat, but nothing exists for CLI log diagnosis workflows. This IS the product. | High | Requires Claude API integration, prompt engineering for diagnosis, and robust de-tokenization of free-text LLM responses. |
| **Claude Code skill integration** | Engineers already using Claude Code get seamless `/diagnose-logs` experience without leaving their workflow | Medium | MCP tool or Claude Code custom command that wraps the tokenize-diagnose-de-tokenize loop. |
| **Encrypted persistent token store** | Tokens persist across sessions so related log investigations maintain consistent mappings. Running diagnosis on Monday's logs then Tuesday's logs keeps `[TOK_IP_001]` meaning the same IP. | Medium | AES-256 encrypted local file. Key derived from user passphrase or machine identity. |
| **Category-aware tokenization** | Not just "replace sensitive data" but "replace and label by category" -- Claude gets `[TOK_CREDENTIAL_001]` not `[REDACTED]`, preserving semantic context for better diagnosis | Low | Map detected patterns to categories. Improves LLM diagnosis quality significantly because Claude knows what TYPE of thing was redacted. |
| **Custom pattern rules** | Users add org-specific patterns (internal service names, custom ID formats, proprietary field names) via config file | Medium | YAML/TOML config with regex patterns, category labels, and optional format-preservation rules. |
| **Diagnosis output formats** (bullet summary + detailed markdown) | Engineers want quick answers AND detailed reports depending on context | Low | Template-based output formatting. Bullet summary for Slack, markdown for tickets/docs. |
| **Confidence scoring on detections** | Show users which tokenizations are high-confidence (regex match on AWS key pattern) vs low-confidence (heuristic match on possible hostname) so they can review before sending | Medium | Score based on pattern specificity. Let users set a threshold -- auto-tokenize above it, flag below it. |
| **Dry-run / preview mode** | Show what WOULD be tokenized without actually sending anything. Critical for building trust. | Low | Display highlighted diff of original vs tokenized. User confirms before proceeding. |
| **Token map export/import** | Share token maps between team members working on the same incident (encrypted, of course) | Low | Export encrypted token map file. Import on another machine. Enables collaborative debugging. |

## Anti-Features

Features to explicitly NOT build. These are tempting but wrong for v1.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **NLP/ML-based PII detection** | Requires Python runtime (kills single-binary goal), adds latency, model dependency, and false positive complexity. Microsoft Presidio is great but it's a Python framework -- wrong fit for a Go/Rust CLI. | Use regex + pattern databases for v1. Flag NLP as v2 exploration if regex detection proves insufficient. |
| **Log storage/persistence** | Scope creep into log management territory (Elastic, Loki, Datadog). Tool is a tokenizer, not a log store. | Tokenize and pass through. Users bring their own log source and destination. |
| **Real-time log tailing/streaming** | Complex stateful processing, partial-line buffering, reconnection logic. v1 needs to nail file-based first. | Block-based file processing in v1 lays architectural foundation. Streaming is explicitly v2 per PROJECT.md. |
| **Web UI or dashboard** | CLI-first tool. Web UI adds massive surface area, hosting complexity, and security concerns (now the token map is in a browser). | CLI with clean stdout/stderr. Structured output (JSON) enables others to build UIs if needed. |
| **Log source connectors** (Elastic, CloudWatch, Datadog, Splunk) | Each connector is its own project. Premature before the core loop is proven. | Accept file input and stdin. Users pipe/download logs themselves. Connectors are explicitly v2 per PROJECT.md. |
| **Format-preserving tokenization** | FPT makes tokens look like real data (fake IP for real IP). This is WRONG for LLM diagnosis -- Claude needs to know it's a token, not confuse it with real data. | Use obvious token labels like `[TOK_IP_001]`. The whole point is Claude treats them as opaque references. |
| **Automatic secret validation** (calling APIs to verify if detected key is live) | Security nightmare -- tool would be making auth requests with potentially stolen credentials. Also requires network access. | Detect patterns only. Never attempt to use/validate detected secrets. |
| **Multi-language NER for name detection** | Enormous complexity for detecting names in free text across languages. Regex can't do this well and NLP is out of scope (see above). | Detect structured PII (emails, IDs, phones) via regex. Offer custom patterns for org-specific name fields. |

## Feature Dependencies

```
Credential detection ─┐
PII detection ─────────┤
Infra detail detection ┼──> Core tokenization engine ──> Token map store
Custom patterns ───────┘           │
                                   │
           ┌───────────────────────┘
           │
           v
   Consistent tokenization ──> Encrypted persistent store
           │
           v
   ┌───────┴────────┐
   │                 │
   v                 v
Clipboard output   Claude API integration
                     │
                     v
              De-tokenize response
                     │
                     v
              Output formatting (bullet / markdown)
                     │
                     v
              Claude Code skill integration
```

Key dependency chains:
- Detection patterns must exist before tokenization engine works
- Token map must exist before de-tokenization works
- Claude API integration requires tokenization to be solid first
- Claude Code skill requires the Claude API integration to work
- Dry-run/preview requires the detection engine but NOT the API integration
- Encrypted store requires token map but can be added after basic in-memory map works

## MVP Recommendation

**Phase 1 -- Core tokenization (no LLM yet):**
1. Credential detection (highest security value)
2. PII detection (emails, IPs)
3. Infrastructure detail detection
4. Consistent deterministic tokenization with in-memory map
5. Structured JSON + unstructured text support
6. Category-aware token labels
7. Dry-run preview mode
8. Cross-platform binary

**Phase 2 -- Diagnosis loop:**
1. Claude API integration (tokenize -> send -> receive)
2. De-tokenize Claude's response
3. Output formatting (bullet + markdown)
4. Clipboard/paste fallback
5. Encrypted persistent token store

**Phase 3 -- Developer experience:**
1. Claude Code skill integration
2. Multi-line/stack trace handling (upgrade from basic to robust)
3. Custom pattern rules via config
4. Confidence scoring on detections
5. Token map export/import

**Defer to v2:** NLP-based detection, streaming, connectors, log tailing

**Rationale:** Build trust by proving tokenization is thorough and correct (Phase 1) before connecting to external services (Phase 2). Engineers need to verify what gets tokenized before they'll trust it with their production logs. The dry-run preview in Phase 1 is critical for this trust-building.

## Sources

- [Secrets Patterns DB - 1600+ regex patterns for secret detection](https://github.com/mazen160/secrets-patterns-db)
- [Microsoft Presidio - PII detection framework](https://microsoft.github.io/presidio/)
- [Caviard.ai - Browser-based PII redaction for LLMs](https://www.caviard.ai)
- [DZone - Reversible Data Anonymization for LLMs](https://dzone.com/articles/llm-pii-anonymization-guide)
- [John Snow Labs - Consistent Tokenization and Obfuscation](https://www.johnsnowlabs.com/consistent-linking-tokenization-and-obfuscation-for-regulatory-grade-de-identification/)
- [Grafana Alloy - Log secret redaction](https://grafana.com/blog/2025/03/20/how-to-redact-secrets-from-logs-with-grafana-alloy-and-loki/)
- [OpenObserve - 144 prebuilt redaction rules](https://openobserve.ai/blog/redact-sensitive-data-in-logs/)
- [Datadog - Multi-line logging best practices](https://www.datadoghq.com/blog/multiline-logging-guide/)
- [BetterStack - Logging practices for safeguarding sensitive data](https://betterstack.com/community/guides/logging/sensitive-data/)
- [Skyflow - Keeping sensitive data out of logs](https://www.skyflow.com/post/how-to-keep-sensitive-data-out-of-your-logs-nine-best-practices)
- [arxiv - Protecting Privacy in Software Logs](https://arxiv.org/html/2409.11313v2)
