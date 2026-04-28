# Feature Landscape: v2.0 Developer Documentation

**Domain:** CLI help styling and auto-generated HTML documentation for a Rust CLI tool
**Researched:** 2026-04-28
**Milestone focus:** Colored CLI help output + auto-generated HTML docs page

## Table Stakes

Features developers and DevOps engineers expect from a polished CLI tool's help and documentation. Missing any of these makes the tool feel unfinished or amateur.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Colored section headers in --help** | Every modern Rust CLI (cargo, ripgrep, bat, starship) uses colored help. Plain white text signals "hobby project." | Low | clap 4.6 has built-in `Styles` API with `Command::styles()`. Zero new dependencies needed. Use `Styles::styled()` as base, customize header/usage/literal colors. |
| **Bold command names and flags** | Users scan for flags visually. Bold makes `--output`, `--dry-run` pop out from descriptions. Standard in cargo, ripgrep. | Low | Part of clap's `Styles` -- `literal()` method controls flag/command styling. Already supported via ANSI bold. |
| **Colored usage line** | The `Usage: logtok tokenize <FILE>` line is the first thing users look at. Coloring it separately improves scannability. | Low | `Styles::styled().usage(AnsiColor::Green.on_default().bold())` -- single line of code. |
| **Respect NO_COLOR / --color flag** | The NO_COLOR standard (no-color.org) is expected by CLI-savvy users. Tools that force color on piped output break scripts. | Low | clap's `ColorChoice::Auto` already handles this by default. Verify it respects `NO_COLOR` env var. |
| **Command reference in HTML docs** | Any docs page for a CLI tool must list all commands, flags, and their descriptions. Without this, the page is useless. | Medium | Extract from clap's `Command` tree at runtime using introspection API. |
| **Copy-to-clipboard buttons on code blocks** | Every developer docs site (GitHub, MDN, Tailwind, Vercel) has these. Developers expect to click-copy install commands and examples. | Low | ~15 lines of JavaScript using `navigator.clipboard.writeText()`. No library needed. |
| **Install instructions in HTML docs** | First thing a new user looks for. Must cover the primary install methods (cargo install, binary download). | Low | Static content. Template with platform-specific commands. |
| **Quick start / getting started flow** | Users want to go from install to first successful tokenization in under 2 minutes. A linear walkthrough is expected. | Low | 3-4 step guide: install, tokenize a file, view output, detokenize. Static content with copy-able commands. |
| **Responsive/readable HTML layout** | Developers read docs on laptops, tablets, and phones. A docs page that requires horizontal scrolling is unusable. | Low | Standard CSS max-width container, responsive typography. No framework needed -- ~50 lines of CSS. |

## Differentiators

Features that elevate logtok's documentation beyond the baseline. Not expected, but signal quality and care.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **`logtok docs` subcommand** | Self-documenting CLI -- generate docs from the tool itself. Ensures docs never drift from actual CLI behavior. Very few CLI tools do this. | Medium | New clap subcommand. Walks `Command` tree, generates HTML with askama. Outputs single .html file. |
| **Single-file HTML output** (no external assets) | One file, zero dependencies, works offline. Drop it in a repo, open in browser, done. No static site generator, no build step. | Medium | Embed CSS and JS inline in the HTML via askama template. |
| **Styled examples section in --help** | clap's default help doesn't render examples prominently. Adding a visually distinct examples block (like cargo does) helps new users. | Low | Use clap's `after_help` or `after_long_help` with ANSI-styled text. |
| **Dark/light theme support in HTML** | Developer preference. Many developers use dark mode and docs pages that blind them get closed. | Low | CSS `prefers-color-scheme` media query. ~20 lines of additional CSS. Zero JavaScript needed. |
| **Anchor links for each command** | Deep-linking to `docs.html#tokenize` or `docs.html#detokenize` lets users bookmark and share specific command references. | Low | Standard HTML id attributes on section headers. |
| **Category reference table in HTML docs** | logtok's 19 token categories are unique to this tool. A clear reference table showing each category, what it detects, and example patterns is a differentiator. | Low | Static content generated from the detector module's category definitions. |
| **Version-stamped docs** | Generated HTML shows which logtok version it was generated from. Prevents confusion when docs are shared across teams. | Low | Embed `env!("CARGO_PKG_VERSION")` in generated output. |

## Anti-Features

Features to explicitly NOT build for this milestone. Tempting but wrong.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Full static site (multi-page docs)** | Massive scope creep. mdBook or mkdocs sites require build pipelines, hosting, CI integration, theme maintenance. logtok has 3 commands -- a static site is overkill. | Single-file HTML. One `logtok docs` command, one output file. |
| **Man page generation** | man pages are a Unix tradition but modern developers use `--help` and web docs. Adding clap_mangen is low effort but low value for the target audience. | Stick with `--help` and HTML docs. Add man pages later if requested. |
| **Shell completion generation** | Useful but orthogonal to docs milestone. Different feature, different testing surface. | Defer to a future milestone. clap supports this but it deserves its own scope. |
| **PDF documentation** | No one reads CLI tool docs as PDFs. Enterprise checkbox thinking. | HTML is the universal format. |
| **Custom theme engine for help output** | clap-help crate offers rich terminal rendering but does NOT support subcommands (confirmed limitation). logtok uses subcommands. | Use clap's built-in `Styles` API. Supports subcommands, zero-dependency. |
| **Internationalization (i18n)** | Premature. English-only is fine for a developer tool at this stage. | All docs in English. |
| **Auto-publish docs to GitHub Pages** | CI/CD integration for docs publishing is a separate concern. This milestone is about generating the HTML. | Generate the file. Users host it however they want. |
| **Colored tokenize/detokenize output** | Tempting to colorize tokenized output, but it complicates piping, redirection, and clipboard copy. | Keep tokenize/detokenize output plain text. Color only in `--help` and status messages. |
| **Interactive web playground** | WASM compilation, massive binary size, whole new attack surface. Minimal value for a tokenization tool. | Provide copy-paste examples that users run locally. |
| **Syntax highlighting in HTML code blocks** | Prism.js or highlight.js adds weight and complexity for marginal gain on ~10 code snippets. Plain `<pre><code>` with monospace font and background color is sufficient. | Style code blocks with CSS only. No JS syntax highlighter. |

## Feature Dependencies

```
clap Styles const (already available in clap 4.6)
       |
       v
Colored --help output  (zero new dependencies)

clap Command introspection API
       |
       v
Extract command names, descriptions, args, flags, defaults
       |
       v
askama template (inline source, embedded CSS/JS)
       |
       v
`logtok docs` subcommand --> single-file HTML output
```

No circular dependencies. Colored help and HTML docs are independent features that share only the clap dependency.

## MVP Recommendation for v2.0

### Phase 1 -- Colored CLI Help (smallest useful increment)

1. Define `const STYLES` with color scheme
2. Add `styles = STYLES` attribute to `Cli` derive macro
3. Add styled examples via `after_long_help`
4. Verify NO_COLOR compliance
5. Test on Windows Terminal, macOS Terminal, common Linux terminals

**Rationale:** Immediate visual improvement, zero new dependencies, can ship in hours.

### Phase 2 -- HTML Documentation Generation

1. Add `Docs` subcommand to `Commands` enum
2. Walk clap `Command` tree to extract all commands, flags, descriptions
3. Create askama inline template with embedded CSS (responsive, dark/light theme)
4. Add copy-to-clipboard buttons on all code blocks
5. Include: install guide, quick start flow, full command reference, category reference table
6. Embed version stamp from Cargo.toml

**Rationale:** The `docs` subcommand is the headline feature -- self-documenting CLI that never drifts from actual behavior.

### Defer

- Man pages, shell completions, syntax highlighting JS, hosted docs, Markdown output
