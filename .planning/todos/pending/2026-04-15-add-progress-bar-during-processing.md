---
created: 2026-04-15T14:26:55.025Z
title: Add progress bar during processing
area: general
files:
  - src/processor.rs
  - src/main.rs
---

## Problem

When processing large log files, the user sees no visual feedback until the job completes. For GB-scale files this could take significant time with no indication of progress.

## Solution

Use the `indicatif` crate (already in the tech stack) to add a progress bar showing: bytes processed / total bytes, throughput (MB/s), ETA, and elapsed time. Display during block processing in processor.rs. Suppress when `--quiet` flag is set.
