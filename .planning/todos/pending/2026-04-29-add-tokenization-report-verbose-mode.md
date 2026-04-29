---
created: 2026-04-29T07:11:21.788Z
title: Add tokenization report verbose mode
area: general
files:
  - src/cli.rs
  - src/main.rs
---

## Problem

After tokenizing a log file, users have no visibility into what data was tokenized. They want a report or verbose output showing which sensitive data categories were detected, how many tokens were created per category, and what patterns matched.

## Solution

- Add a `--verbose` or `--report` flag to the `tokenize` subcommand
- After tokenization, output a summary report showing:
  - Number of tokens created per category (IP, HOST, URL, KEY, etc.)
  - Total sensitive values detected
  - Categories with zero hits (optional)
- Could also support `--report <path>` to write a detailed report file
- Integrate with existing token store to pull category counts
