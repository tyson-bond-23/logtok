# Requirements: Logs Tokeniser

**Defined:** 2026-04-13
**Core Value:** Engineers can diagnose production log errors through Claude Code without revealing any sensitive information

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Detection

- [ ] **DET-01**: User can detect and tokenize credentials (API keys, tokens, passwords, connection strings)
- [ ] **DET-02**: User can detect and tokenize PII (emails, IP addresses, usernames, phone numbers)
- [ ] **DET-03**: User can detect and tokenize infrastructure details (hostnames, file paths, internal URLs, ports)
- [ ] **DET-04**: User can preview what would be tokenized before sending (dry-run mode)
- [ ] **DET-05**: User can configure detection rules (enable/disable categories, add custom patterns)

### Tokenization

- [ ] **TOK-01**: Same sensitive value always produces the same token (deterministic)
- [ ] **TOK-02**: Tokens are category-prefixed for LLM context (`[TOK_IP_001]`, `[TOK_KEY_001]`, `[TOK_EMAIL_001]`)
- [ ] **TOK-03**: Token mappings stored in encrypted local store (AES-256-GCM)
- [ ] **TOK-04**: Token store persists across sessions and can be reused
- [ ] **TOK-05**: User can tokenize structured JSON logs preserving structure
- [ ] **TOK-06**: User can tokenize unstructured/plain text logs

### Diagnosis

- [ ] **DIA-01**: User can send tokenized logs to Claude API for diagnosis
- [ ] **DIA-02**: Claude's response is de-tokenized back to real values automatically
- [ ] **DIA-03**: User receives bullet-point summary (root cause, affected components, suggested fix)
- [ ] **DIA-04**: User receives detailed markdown report
- [ ] **DIA-05**: User can copy tokenized output to clipboard for manual paste

### Infrastructure

- [ ] **INF-01**: Single binary, zero runtime dependencies
- [ ] **INF-02**: Cross-platform (Windows, macOS, Linux, ARM)
- [ ] **INF-03**: Block-based processing for large files (bounded memory usage)

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Streaming

- **STR-01**: User can ingest logs from Kafka for streaming tokenization
- **STR-02**: User can ingest logs from RabbitMQ for streaming tokenization
- **STR-03**: Continuous log stream processing in real-time

### Connectors

- **CON-01**: User can pull logs directly from Elasticsearch/OpenSearch
- **CON-02**: User can pull logs from AWS CloudWatch
- **CON-03**: User can pull logs from Datadog
- **CON-04**: User can pull logs from Splunk

### Advanced Detection

- **ADV-01**: Multi-line log entry handling (stack traces grouped as single entries)
- **ADV-02**: NLP/ML-based PII detection for free-text names
- **ADV-03**: Confidence scoring on detections with configurable thresholds

### Collaboration

- **COL-01**: Token map export/import for team sharing (encrypted)
- **COL-02**: Claude Code skill integration (`/diagnose-logs` command)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Log storage/persistence | Tokenizer only, not a log management system |
| Web UI or dashboard | CLI-first tool; web adds security surface area |
| Real-time log tailing | v2 streaming feature |
| Format-preserving tokenization | Tokens must be obviously NOT real data for LLM context |
| Automatic secret validation | Security nightmare — never attempt to use detected credentials |
| NLP/ML-based name detection | Requires Python runtime, kills single-binary goal |
| Mobile app | CLI tool, not a mobile product |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| DET-01 | — | Pending |
| DET-02 | — | Pending |
| DET-03 | — | Pending |
| DET-04 | — | Pending |
| DET-05 | — | Pending |
| TOK-01 | — | Pending |
| TOK-02 | — | Pending |
| TOK-03 | — | Pending |
| TOK-04 | — | Pending |
| TOK-05 | — | Pending |
| TOK-06 | — | Pending |
| DIA-01 | — | Pending |
| DIA-02 | — | Pending |
| DIA-03 | — | Pending |
| DIA-04 | — | Pending |
| DIA-05 | — | Pending |
| INF-01 | — | Pending |
| INF-02 | — | Pending |
| INF-03 | — | Pending |

**Coverage:**
- v1 requirements: 19 total
- Mapped to phases: 0
- Unmapped: 19

---
*Requirements defined: 2026-04-13*
*Last updated: 2026-04-13 after initial definition*
