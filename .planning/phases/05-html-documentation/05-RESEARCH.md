# Phase 5: HTML Documentation - Research

**Researched:** 2026-04-29
**Domain:** Compile-time HTML templating from clap Command tree introspection
**Confidence:** HIGH

## Summary

This phase adds a `logtok docs` subcommand that generates a single-file HTML documentation page. The implementation combines two well-understood Rust techniques: (1) clap's `CommandFactory` trait to introspect the CLI tree at runtime, extracting subcommands, args, flags, and help text; and (2) Askama 0.15 compile-time templates to render that metadata into a self-contained HTML file with embedded CSS and JS.

The technical risk is low. Both clap introspection and Askama are mature, well-documented APIs. The main complexity is in the HTML/CSS/JS template itself -- ensuring the dark theme looks professional, the sidebar navigation works on mobile, and copy-to-clipboard functions correctly on `file://` URLs (which require a `document.execCommand('copy')` fallback since `navigator.clipboard` is unavailable outside secure contexts).

**Primary recommendation:** Build a `docs.rs` module that constructs data structs from `Cli::command().build()`, passes them to an Askama template, and writes the rendered HTML to a file. Keep template files in `templates/` at crate root.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Use CLI-matched dark theme -- dark background (#1a1a2e), yellow bold headers, green code/commands, cyan links/accents, light gray body text. Mirrors the terminal --help aesthetic for cohesion.
- **D-02:** Single scroll page with collapsible sidebar navigation. Sidebar visible by default on desktop (>768px), hidden behind hamburger toggle on mobile.
- **D-03:** Code blocks use slightly lighter dark background with monospace font. System font stack for body text (no web fonts).
- **D-04:** Section ordering: Getting Started (install + quick start) -> Overview (what logtok does, workflow) -> Command Reference (tokenize/detokenize/reset-store with all flags) -> Token Categories reference table (all 19 categories).
- **D-05:** Sidebar nav mirrors section headings with nested links for each subcommand under Commands.
- **D-06:** Use Askama 0.15 compile-time templates. HTML template files in `templates/` directory, compiled into the binary.
- **D-07:** One new crate dependency: `askama`.
- **D-08:** Extract command metadata from clap's `Command::build()` at runtime -- iterate subcommands, args, flags, about text, long_about, after_long_help to populate template data structs.
- **D-09:** Small clipboard icon button in top-right corner of each code block. CSS `position: absolute` within a `position: relative` code block wrapper.
- **D-10:** On click: icon changes to checkmark for 2 seconds, then reverts. Uses `navigator.clipboard.writeText()` with `document.execCommand('copy')` fallback for older browsers and file:// URLs.
- **D-11:** All JS for copy functionality embedded inline in the HTML file. Minimal vanilla JS, no frameworks.

### Claude's Discretion
- Exact CSS values (spacing, font sizes, border radius) -- as long as it looks professional
- Responsive breakpoint fine-tuning beyond the 768px mobile threshold
- Whether to include a "back to top" button
- Internal HTML structure (divs, sections, semantic HTML choices)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DOCS-01 | User can run `logtok docs` to generate a self-contained HTML documentation file | New `Docs` variant in Commands enum + `docs.rs` module with Askama rendering |
| DOCS-02 | HTML page includes an install/getting-started guide with step-by-step instructions | Static content in Askama template -- Getting Started section with 3-step workflow |
| DOCS-03 | HTML page includes full command reference for all subcommands with flags, arguments, and descriptions | clap `CommandFactory::command().build()` introspection populates template data structs |
| DOCS-04 | All code examples in HTML have copy-to-clipboard buttons that work across browsers | Inline JS with `navigator.clipboard.writeText()` + `document.execCommand('copy')` fallback |
| DOCS-05 | HTML is a single file with embedded CSS and JS -- no external dependencies | Askama template contains all CSS in `<style>` and JS in `<script>` tags |
| DOCS-06 | HTML page has a clean, professional design targeted at developers and DevOps engineers | Dark theme matching CLI aesthetic, system font stack, collapsible sidebar |
| DOCS-07 | Generated HTML stays in sync with actual CLI commands (derived from clap Command tree) | Runtime introspection of `Cli::command()` -- no hardcoded command data |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| askama | 0.15.6 | Compile-time HTML templating | Jinja-like syntax, type-safe, zero runtime cost, template errors caught at build time [VERIFIED: crates.io search] |

### Supporting (already in project)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| clap | 4.6.0 | CLI definition + Command introspection | `CommandFactory::command()` to build Command tree for docs extraction |
| anyhow | 1.0.102 | Error handling | Wrap file write and template render errors |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Askama | format!() macros | Would work but unmaintainable -- HTML templates as Rust strings are painful to edit and debug |
| Askama | Tera | Tera is runtime-evaluated. Askama compiles templates at build time -- better performance, catches errors earlier |
| Askama | maud | maud uses Rust macro syntax for HTML, not template files. Harder to iterate on design. |

**Installation:**
```bash
cargo add askama@0.15
```

## Architecture Patterns

### Recommended Project Structure
```
src/
  cli.rs          # Add Docs variant to Commands enum
  docs.rs         # NEW: Command introspection + HTML generation logic
  main.rs         # Add match arm for Commands::Docs
  lib.rs          # Add pub mod docs
templates/
  docs.html       # Askama HTML template (single file with embedded CSS/JS)
```

### Pattern 1: Command Metadata Extraction
**What:** Build intermediate data structs from clap's Command tree, pass to Askama template.
**When to use:** Always -- separates introspection logic from template rendering.
**Example:**
```rust
// Source: clap docs.rs Command introspection API
use clap::CommandFactory;
use crate::cli::Cli;

struct CommandInfo {
    name: String,
    about: String,
    long_about: Option<String>,
    after_long_help: Option<String>,
    args: Vec<ArgInfo>,
}

struct ArgInfo {
    name: String,
    short: Option<char>,
    long: Option<String>,
    help: String,
    required: bool,
    default_value: Option<String>,
}

fn extract_commands() -> Vec<CommandInfo> {
    let mut cmd = Cli::command();
    cmd.build(); // Propagate global args, finalize
    
    cmd.get_subcommands()
        .map(|sub| CommandInfo {
            name: sub.get_name().to_string(),
            about: sub.get_about().map(|s| s.to_string()).unwrap_or_default(),
            long_about: sub.get_long_about().map(|s| s.to_string()),
            after_long_help: sub.get_after_long_help().map(|s| s.to_string()),
            args: sub.get_arguments()
                .filter(|a| a.get_id() != "help" && a.get_id() != "version")
                .map(|a| ArgInfo {
                    name: a.get_id().to_string(),
                    short: a.get_short(),
                    long: a.get_long().map(|s| s.to_string()),
                    help: a.get_help().map(|s| s.to_string()).unwrap_or_default(),
                    required: a.is_required_set(),
                    default_value: a.get_default_values()
                        .first()
                        .map(|v| v.to_string_lossy().to_string()),
                })
                .collect(),
        })
        .collect()
}
```
[VERIFIED: clap docs.rs API documentation for Command and Arg getter methods]

### Pattern 2: Askama Template Rendering
**What:** Define a Template struct, render to string, write to file.
**When to use:** For the HTML generation step.
**Example:**
```rust
// Source: Askama 0.15 docs.rs documentation
use askama::Template;

#[derive(Template)]
#[template(path = "docs.html")]
struct DocsTemplate {
    version: String,
    commands: Vec<CommandInfo>,
    // ... other template data
}

fn generate_docs(output_path: &Path) -> Result<()> {
    let commands = extract_commands();
    let template = DocsTemplate {
        version: env!("CARGO_PKG_VERSION").to_string(),
        commands,
    };
    let html = template.render()?;
    std::fs::write(output_path, html)?;
    Ok(())
}
```
[VERIFIED: Askama 0.15.6 docs.rs -- Template derive, render() method]

### Pattern 3: Copy-to-Clipboard JS with file:// Fallback
**What:** Inline JS that handles both secure contexts (HTTPS/localhost) and insecure contexts (file:// URLs).
**When to use:** For all code block copy buttons.
**Example:**
```javascript
// Source: MDN Clipboard API docs + web.dev clipboard article
function copyCode(btn) {
    var code = btn.parentElement.querySelector('code').textContent;
    if (navigator.clipboard && window.isSecureContext) {
        navigator.clipboard.writeText(code).then(function() {
            showCopied(btn);
        });
    } else {
        // Fallback for file:// URLs and older browsers
        var textarea = document.createElement('textarea');
        textarea.value = code;
        textarea.style.position = 'fixed';
        textarea.style.opacity = '0';
        document.body.appendChild(textarea);
        textarea.select();
        document.execCommand('copy');
        document.body.removeChild(textarea);
        showCopied(btn);
    }
}

function showCopied(btn) {
    var original = btn.textContent;
    btn.textContent = '\u2713';
    setTimeout(function() { btn.textContent = original; }, 2000);
}
```
[CITED: developer.mozilla.org/en-US/docs/Web/API/Clipboard/writeText -- secure context requirement]
[CITED: web.dev/async-clipboard/ -- fallback pattern]

### Anti-Patterns to Avoid
- **Hardcoding command data in template:** Never write command names, flags, or descriptions directly in the HTML template. Always derive from clap introspection -- this is what makes DOCS-07 work.
- **External CSS/JS files:** Violates DOCS-05. Everything must be inline in the single HTML file.
- **Using `source = "..."` for large templates:** Askama supports inline source strings, but for a full HTML page with embedded CSS/JS, use `path = "docs.html"` and keep the template as a separate file in `templates/`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTML escaping | Manual string escaping | Askama auto-escaping | Askama escapes HTML by default; use `\|safe` filter only for trusted content like pre-formatted HTML |
| CLI introspection | Manual arg parsing | clap `CommandFactory` + `Command::get_subcommands()` | Derive macro already knows the full tree; just call `.command().build()` |
| Template rendering | format!() string building | Askama templates | Maintainable, type-checked at compile time, proper escaping |

## Common Pitfalls

### Pitfall 1: Forgetting Command::build() Before Introspection
**What goes wrong:** `get_subcommands()` returns incomplete data -- global args not propagated, some metadata missing.
**Why it happens:** clap lazily initializes Command tree. `build()` must be called to finalize.
**How to avoid:** Always call `cmd.build()` before iterating subcommands or args.
**Warning signs:** Missing global flags (--quiet, --config) in subcommand arg lists.
[VERIFIED: clap docs.rs -- Command::build() documentation]

### Pitfall 2: Askama HTML Auto-Escaping Breaking Pre-formatted Content
**What goes wrong:** Template variables containing `<`, `>`, `&` get escaped to `&lt;`, `&gt;`, `&amp;` -- breaking intentional HTML or code examples.
**Why it happens:** Askama auto-escapes all variables in HTML templates by default.
**How to avoid:** Use `{{ value|safe }}` filter for content that should render as raw HTML. Use this sparingly and only for content generated by the tool itself (not user input).
**Warning signs:** HTML entities appearing literally in rendered output.
[VERIFIED: Askama 0.15 docs -- auto-escaping behavior]

### Pitfall 3: navigator.clipboard Undefined on file:// URLs
**What goes wrong:** Copy buttons silently fail when opening the HTML file directly from disk.
**Why it happens:** Clipboard API requires a secure context (HTTPS or localhost). `file://` is not secure context.
**How to avoid:** Always implement `document.execCommand('copy')` fallback with textarea element (see Pattern 3 above).
**Warning signs:** Copy button works when served via HTTP but not when opened as a local file.
[CITED: developer.mozilla.org/en-US/docs/Web/API/Clipboard/writeText]

### Pitfall 4: Global Args Appearing in Every Subcommand
**What goes wrong:** After `build()`, global args (--quiet, --config) appear in every subcommand's `get_arguments()` iterator.
**Why it happens:** `build()` propagates global args to all subcommands.
**How to avoid:** Filter out known global args by ID when building per-subcommand arg lists, or document them once in a "Global Options" section.
**Warning signs:** --quiet and --config listed under every subcommand.
[ASSUMED]

### Pitfall 5: Docs Subcommand Appearing in Its Own Output
**What goes wrong:** The `docs` command itself shows up in the generated command reference.
**Why it happens:** It's a valid subcommand in the Command tree.
**How to avoid:** Filter out the "docs" subcommand when iterating `get_subcommands()`.
**Warning signs:** Circular reference -- "docs" command documented in docs.
[ASSUMED]

## Code Examples

### Extracting clap Command Tree from Derive Struct
```rust
// Source: clap docs.rs -- CommandFactory trait
use clap::CommandFactory;
use crate::cli::Cli;

let mut cmd = Cli::command();
cmd.build();

let version = cmd.get_version().unwrap_or("unknown");
let about = cmd.get_about().map(|s| s.to_string());

for sub in cmd.get_subcommands() {
    println!("Command: {}", sub.get_name());
    for arg in sub.get_arguments() {
        if let Some(long) = arg.get_long() {
            println!("  --{}: {}", long, arg.get_help().unwrap_or_default());
        }
    }
}
```
[VERIFIED: clap docs.rs Command/Arg getter API]

### Askama Template Struct with Nested Data
```rust
// Source: Askama 0.15 docs
use askama::Template;

#[derive(Template)]
#[template(path = "docs.html")]
struct DocsTemplate<'a> {
    version: &'a str,
    commands: &'a [CommandInfo],
    token_categories: &'a [TokenCategory],
}

// Render to string
let tmpl = DocsTemplate { version: "0.1.0", commands: &cmds, token_categories: &cats };
let html = tmpl.render()?; // Returns Result<String, askama::Error>
```
[VERIFIED: Askama 0.15.6 docs.rs]

### Askama Template Syntax for Command Reference
```html
{# templates/docs.html - Jinja-like syntax #}
{% for cmd in commands %}
<section id="cmd-{{ cmd.name }}">
  <h3>logtok {{ cmd.name }}</h3>
  <p>{{ cmd.about }}</p>
  {% if let Some(long) = cmd.long_about %}
  <p>{{ long }}</p>
  {% endif %}
  
  {% if !cmd.args.is_empty() %}
  <table>
    <tr><th>Flag</th><th>Description</th><th>Default</th></tr>
    {% for arg in cmd.args %}
    <tr>
      <td>
        {% if let Some(s) = arg.short %}-{{ s }}, {% endif %}
        {% if let Some(l) = arg.long %}--{{ l }}{% endif %}
      </td>
      <td>{{ arg.help }}</td>
      <td>{{ arg.default_value.as_deref().unwrap_or("--") }}</td>
    </tr>
    {% endfor %}
  </table>
  {% endif %}
</section>
{% endfor %}
```
[VERIFIED: Askama template syntax from docs.rs and askama.rs]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Askama 0.12 with `askama_derive` separate crate | Askama 0.15 unified crate (derive included) | 2024-2025 | Single dependency, `use askama::Template` is all you need |
| `document.execCommand('copy')` only | Clipboard API with execCommand fallback | 2020+ | Must support both paths for file:// compatibility |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Global args propagate to subcommand `get_arguments()` after `build()` | Pitfall 4 | Would need different filtering logic -- easy to verify during implementation |
| A2 | The `docs` subcommand would appear in its own `get_subcommands()` output | Pitfall 5 | Minor -- just remove the filter if it doesn't appear |

## Open Questions

1. **Output file path default**
   - What we know: `logtok docs` needs to write an HTML file somewhere
   - What's unclear: Should it default to `./logtok-docs.html` in CWD, or require `-o` flag?
   - Recommendation: Default to `logtok-docs.html` in CWD with optional `-o` override -- matches `tokenize` pattern

2. **Token categories table data source**
   - What we know: D-04 requires a token categories table with all 19 categories
   - What's unclear: Whether to hardcode the 19 categories in a const or derive from detector module
   - Recommendation: Define as a const array in `docs.rs` -- the categories are stable and documented in CLAUDE.md. Deriving from detector patterns would be fragile and unnecessary.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Compilation | Yes | 1.94.1 | -- |
| askama 0.15.6 | Template rendering | Yes (crates.io) | 0.15.6 | -- |

No missing dependencies.

## Security Domain

This phase generates static HTML documentation. No user input is processed at runtime (all content comes from the compiled clap tree). Askama's auto-escaping prevents any injection in template variables.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | -- |
| V3 Session Management | No | -- |
| V4 Access Control | No | -- |
| V5 Input Validation | No | All template data is from compiled source, not user input |
| V6 Cryptography | No | -- |

No security concerns for this phase.

## Sources

### Primary (HIGH confidence)
- [crates.io] -- askama 0.15.6 version verified via `cargo search`
- [docs.rs/askama/0.15.6](https://docs.rs/askama/0.15.6/askama/) -- Template derive, render(), auto-escaping
- [docs.rs/clap/latest/clap/struct.Command.html](https://docs.rs/clap/latest/clap/struct.Command.html) -- Command introspection API (get_subcommands, get_arguments, get_about, etc.)
- [docs.rs/clap/latest/clap/struct.Arg.html](https://docs.rs/clap/latest/clap/struct.Arg.html) -- Arg getter methods (get_id, get_long, get_short, get_help, etc.)
- [askama.rs](https://askama.rs/) -- Official Askama book and documentation

### Secondary (MEDIUM confidence)
- [developer.mozilla.org/en-US/docs/Web/API/Clipboard/writeText](https://developer.mozilla.org/en-US/docs/Web/API/Clipboard/writeText) -- Clipboard API secure context requirement
- [web.dev/async-clipboard/](https://web.dev/async-clipboard/) -- Clipboard fallback patterns

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- askama 0.15 is verified on crates.io, clap introspection API documented
- Architecture: HIGH -- straightforward pattern: extract data from clap, render with Askama, write file
- Pitfalls: MEDIUM -- clipboard fallback well-documented, global arg propagation assumed but easy to verify

**Research date:** 2026-04-29
**Valid until:** 2026-05-29 (stable libraries, no fast-moving dependencies)
