# Technology Stack: v2.0 Developer Experience

**Project:** logtok
**Researched:** 2026-04-28
**Scope:** Additions for colored CLI help + auto-generated HTML documentation

## Key Insight: Minimal Additions Required

The existing stack covers most needs. clap 4.6 ships with the `color` feature enabled by default and a `Styles` builder API for full help-text customization. HTML generation needs one new crate (askama). No other dependencies required.

## New Dependencies

### Colored CLI Help: Zero New Crates

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| clap (existing) | 4.6.0 | Styled help output | `color` is already a default feature. `clap::builder::Styles` + `clap::builder::styling::AnsiColor` provide full control over help text colors. No additional crate needed. | HIGH |

**How it works:** clap 4.6 exposes `Styles::styled()` with chainable methods for 9 semantic regions: `header()`, `error()`, `usage()`, `literal()`, `placeholder()`, `valid()`, `invalid()`, `context()`, `context_value()`. Each accepts an `anstyle::Style` built from `AnsiColor` + `Effects` (BOLD, UNDERLINE, etc.). The `color` feature is one of 6 default features in clap 4.6 (along with error-context, help, std, suggestions, usage).

**Integration with existing derive-based CLI:**
```rust
use clap::builder::styling::{AnsiColor, Effects, Styles};

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().bold())
    .usage(AnsiColor::Green.on_default().bold())
    .literal(AnsiColor::Cyan.on_default().bold())
    .placeholder(AnsiColor::Yellow.on_default());

#[derive(Parser)]
#[command(styles = STYLES)]
pub struct Cli { /* existing fields unchanged */ }
```

This is a one-line attribute addition to the existing `Cli` struct plus a `const` definition. No structural changes.

### HTML Documentation: One New Crate

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| askama | 0.15.6 | Compile-time HTML templating | Jinja-like syntax, compiles templates to Rust at build time (zero runtime overhead). Supports inline `source` templates -- entire HTML template lives in Rust source, no external `templates/` directory needed. Type-safe: template context is a struct, missing variables are compile errors. Actively maintained (last release 2026-03-24). | HIGH |

**Inline template example (no external files):**
```rust
use askama::Template;

#[derive(Template)]
#[template(ext = "html", source = r#"
<!DOCTYPE html>
<html>
<head><style>{{ css }}</style></head>
<body>
{% for cmd in commands %}
  <section>
    <h2>{{ cmd.name }}</h2>
    <p>{{ cmd.description }}</p>
    {% for arg in cmd.args %}
      <code>{{ arg.flag }}</code> {{ arg.help }}
    {% endfor %}
  </section>
{% endfor %}
</body>
</html>
"#)]
struct DocsTemplate<'a> {
    css: &'a str,
    commands: Vec<CommandInfo>,
}
```

## Alternatives Considered

### Terminal Coloring Crates (NOT needed)

| Crate | Why Skip |
|-------|----------|
| owo-colors | Zero-allocation terminal coloring. Excellent crate, but only useful for custom terminal output outside clap. logtok's colored output is entirely clap help text -- clap handles it internally. |
| colored | Older, allocating alternative to owo-colors. Not needed when clap handles styling. |
| ansi_term | Unmaintained since 2021. Do not use under any circumstances. |
| color-print | Compile-time ANSI from markup tags. Redundant with clap's `Styles` API for help text. |
| syntect | Syntax highlighting library. Overkill -- we are coloring semantic sections (headers, flags, placeholders), not parsing code syntax. |

### HTML Templating Alternatives

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| HTML templating | askama 0.15 | tera | Tera is runtime-interpreted: templates can fail at runtime with missing variables. Askama catches all template errors at compile time. For a CLI tool that generates docs once, compile-time safety matters more than Tera's runtime flexibility. |
| HTML templating | askama 0.15 | maud | Maud uses Rust macro syntax for HTML (`html! { div { p { "text" } } }`). Unreadable for a full docs page with embedded CSS, navigation, and copy-button JS. Jinja-like syntax is natural for HTML. |
| HTML templating | askama 0.15 | handlebars-rust | Runtime-interpreted like Tera. No compile-time guarantees. Larger dependency. |
| HTML templating | askama 0.15 | format!() strings | Unmaintainable at docs-page scale. No auto-escaping, no conditionals, no loops. |
| HTML templating | askama 0.15 | include_str! + manual replace | No type safety, no escaping, error-prone for complex templates with loops and conditionals. |
| CSS approach | Inline in template | grass (Sass) | CSS preprocessing is overkill for a single embedded stylesheet. Write plain CSS directly in the template. |
| HTML minification | Skip | minify-html | Unnecessary for a locally-generated docs page. File size is irrelevant. |
| Markdown-to-HTML | Skip | pulldown-cmark | We generate HTML directly from clap's command structure, not from markdown source. |

## What Changes in Cargo.toml

```toml
# BEFORE (existing)
clap = { version = "4.6.0", features = ["derive"] }

# AFTER (unchanged -- "color" is already a default feature)
clap = { version = "4.6.0", features = ["derive"] }

# NEW ADDITION (one line)
askama = "0.15"
```

Total new dependency lines: **1**

## Integration Architecture

### clap Styles Integration (cli.rs)

Modify existing `Cli` struct -- add `styles = STYLES` attribute. Define a `const STYLES: Styles` with the color scheme. No new files needed, changes are ~15 lines in `cli.rs`.

### `logtok docs` Command Flow

```
Cli struct (clap)
  |
  v
Command::get_subcommands() introspection
  |
  v
Extract: command names, descriptions, args, flags, defaults
  |
  v
Build askama template context struct
  |
  v
Render HTML with embedded CSS + JS (copy buttons)
  |
  v
Write single .html file to disk
```

Key: clap's `Command` API provides `get_subcommands()`, `get_arguments()`, `get_about()`, `get_long_about()` for programmatic introspection. The `logtok docs` command will use these to extract all command metadata at runtime, then pass it to askama for rendering.

### New Subcommand

```rust
/// Generate HTML documentation
Docs {
    /// Output file path (default: logtok-docs.html)
    #[arg(short, long, default_value = "logtok-docs.html")]
    output: PathBuf,
},
```

## Sources

- [clap 4.6.1 Styles API](https://docs.rs/clap/latest/clap/builder/struct.Styles.html) -- Verified struct API and all 9 styling methods
- [clap builder::styling module](https://docs.rs/clap/latest/clap/builder/styling/) -- AnsiColor, Effects, Style types
- [clap 4.6.0 feature flags](https://docs.rs/crate/clap/latest/features) -- Confirmed `color` is a default feature
- [clap Styles discussion](https://github.com/clap-rs/clap/issues/4132) -- Polishing --help output
- [askama documentation](https://askama.rs/en/stable/) -- Compile-time templates, inline source support
- [askama creating templates](https://askama.rs/en/latest/creating_templates.html) -- Inline source parameter docs
- [askama GitHub](https://github.com/askama-rs/askama) -- Version 0.15.6 (2026-03-24)
- [Rain's Rust CLI recommendations on colors](https://rust-cli-recommendations.sunshowers.io/managing-colors-in-rust.html) -- Ecosystem context for terminal coloring
- [owo-colors crate](https://crates.io/crates/owo-colors) -- Reviewed and deemed unnecessary
