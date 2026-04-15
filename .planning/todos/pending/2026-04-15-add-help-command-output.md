---
created: 2026-04-15T14:26:55.025Z
title: Add --help command output
area: general
files:
  - src/cli.rs
---

## Problem

The CLI should have polished `--help` output that describes all available flags and subcommands clearly. clap provides basic help generation, but it may need custom descriptions, examples section, and proper formatting for a good user experience.

## Solution

Enhance clap derive annotations with detailed `about`, `long_about`, and `help` attributes. Add usage examples via clap's `after_help` or `after_long_help`. Verify `logtok --help` and `logtok -h` produce clear, useful output.
