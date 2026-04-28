# Research Summary: logtok v2.0 Developer Experience

**Synthesized:** 2026-04-28
**Sources:** STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md

## Executive Summary

v2.0 is a focused DX milestone: colored `--help` output and a `logtok docs` subcommand generating self-contained HTML documentation. The existing stack (clap 4.6) covers colored help with zero new dependencies. HTML generation requires one new crate — askama 0.15. No existing modules are structurally changed; HTML generation is a new leaf module (`src/docs.rs`, ~350 lines). All dynamic command content derives from clap's introspection API to guarantee docs never drift from the actual binary.

## Stack Additions

| Crate | Version | Purpose | New? |
|-------|---------|---------|------|
| clap `Styles` API | 4.6 (existing) | Colored help output | No — already in Cargo.toml |
| clap `wrap_help` feature | 4.6 | Word-wrapped help on narrow terminals | Feature flag only |
| askama | 0.15 | Compile-time HTML templating with inline source | Yes — one new dependency |

**Do NOT add:** owo-colors, colored, ansi_term, syntect, tera, maud, handlebars, pulldown-cmark, grass, minify-html.

## Feature Table Stakes

- Colored section headers, bold flags, colored usage line in `--help`
- NO_COLOR / CLICOLOR compliance
- Full command reference in HTML docs with copy-to-clipboard buttons
- Responsive HTML with dark/light theme support
- Single-file HTML output with no external assets

## Architecture

**Modified:** `cli.rs` (add `const STYLES` + `Commands::Docs` variant), `main.rs` (add match arm)
**New:** `src/docs.rs` (~350 LOC) — clap Command tree introspection → askama template → HTML file

**Data flow:** `clap::Command` → `extract_commands()` → `CommandDoc` structs → `DocsTemplate` (askama) → single HTML file

## Top Pitfalls

1. **Windows cmd.exe ANSI** — rely on clap's `anstream`, don't roll custom handling
2. **Light terminal visibility** — 8 basic ANSI colors only, pair with bold/underline
3. **Doc-CLI drift** — use clap introspection exclusively, CI test validates output
4. **Copy buttons on file:// URLs** — `document.execCommand` fallback in try/catch
5. **Binary bloat** — strict <50KB budget, no CSS frameworks, system font stack

## Recommended Build Order

**Phase 1: Colored CLI Help** — Zero new deps, ~15 lines, immediate visual improvement. Validates clap Styles API.
**Phase 2: HTML Documentation** — askama 0.15, ~350 new lines. Builds on stable cli.rs from Phase 1.

## Confidence: HIGH

All API claims verified against current docs.rs. Both phases use well-documented, stable APIs with concrete implementation patterns identified.
