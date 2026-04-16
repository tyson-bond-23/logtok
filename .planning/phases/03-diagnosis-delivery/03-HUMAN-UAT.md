---
status: partial
phase: 03-diagnosis-delivery
source: [03-VERIFICATION.md]
started: 2026-04-16T00:00:00Z
updated: 2026-04-16T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Claude Code Token-Aware Diagnosis
Open a Claude Code session with the project's CLAUDE.md. Run `logtok tokenize`, paste output to Claude Code, confirm it preserves tokens in its response and reasons about them correctly.
expected: Claude uses `[CATEGORY_NNN]` tokens throughout without inventing replacement values
result: [pending]

### 2. Full Tokenize-Diagnose-Detokenize Loop
Complete the entire 3-part workflow (tokenize log, paste to Claude Code, detokenize response), confirm the final output is a readable diagnosis with real values restored.
expected: `logtok detokenize response.txt` produces actionable diagnosis text with tokens replaced
result: [pending]

### 3. Clipboard Integration on Windows
Run `logtok tokenize file.log --clipboard` in a graphical Windows session and paste from clipboard.
expected: Clipboard contains tokenized output (tokens like `[IP_001]`), not original sensitive values
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
