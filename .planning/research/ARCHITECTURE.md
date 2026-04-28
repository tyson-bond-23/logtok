# Architecture: v2.0 Developer Experience

**Domain:** Colored CLI help and auto-generated HTML documentation for Rust CLI tool
**Researched:** 2026-04-28
**Confidence:** HIGH

## Scope

This document covers ONLY the architecture for the two new v2.0 features: colored CLI help and the `logtok docs` HTML generation command. The existing processing pipeline, detection engine, token vault, and encryption architecture are unchanged.

## Integration with Existing Architecture

### Current CLI Setup (cli.rs)

The existing `Cli` struct uses clap derive macros:

```rust
#[derive(Parser, Debug)]
#[command(name = "logtok", version, about, long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    // ...
}
```

The derive approach auto-generates the `CommandFactory` trait, providing `Cli::command() -> Command`. This `Command` struct is the key to both features -- it holds all subcommands, arguments, help text, and metadata.

### What Changes vs What Stays

| Component | Change Type | Details |
|-----------|-------------|---------|
| `cli.rs` | MODIFY | Add `Styles` const to `#[command()]`, add `Docs` subcommand |
| `main.rs` | MODIFY | Add match arm for `Commands::Docs` |
| `src/docs.rs` | NEW | HTML generation module -- walks `Command` tree, renders via askama |
| Cargo.toml | MODIFY | Add `askama = "0.15"` |

Key principle: **No existing modules need structural changes.** Colored help is a configuration change on the existing `Cli` struct. HTML generation is a new leaf module with no dependencies on the processing pipeline.

## Feature 1: Colored CLI Help

### How clap Styles Work

clap 4.x has built-in ANSI color support via `clap::builder::Styles`. Attach a `Styles` instance to the root `Command` and it cascades to all subcommands automatically.

**Styleable elements** (9 semantic regions):
- `header` -- section headings ("Usage:", "Arguments:", "Options:")
- `literal` -- command names, flags (`--output`, `tokenize`)
- `placeholder` -- value placeholders (`<FILE>`, `[OPTIONS]`)
- `usage` -- the usage line
- `valid` -- valid value hints
- `invalid` -- error highlights
- `error` -- error headings
- `context` -- defaults and env var notes (`[default: 65536]`)
- `context_value` -- values within context

**Implementation:**

```rust
// In cli.rs
use clap::builder::styling::{AnsiColor, Styles};

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().bold())
    .usage(AnsiColor::Yellow.on_default().bold())
    .literal(AnsiColor::Green.on_default().bold())
    .placeholder(AnsiColor::Cyan.on_default())
    .valid(AnsiColor::Green.on_default())
    .invalid(AnsiColor::Red.on_default());

#[derive(Parser, Debug)]
#[command(name = "logtok", version, about, long_about, styles = STYLES)]
pub struct Cli { ... }
```

This is a `const` definition (evaluated at compile time) plus a single attribute change. ~15 lines total.

**NO_COLOR compliance:** clap respects `NO_COLOR` and `CLICOLOR` environment variables automatically via `ColorChoice::Auto` (the default). No manual handling needed.

**No additional dependencies:** clap's styling uses `anstyle` internally (already a transitive dependency). No need for `colored`, `owo-colors`, or similar crates.

## Feature 2: Auto-Generated HTML Documentation

### Runtime Subcommand Approach

| Approach | Pros | Cons | Verdict |
|----------|------|------|---------|
| Build-time (`build.rs`) | Docs baked into binary | Can't include runtime info, adds build complexity, requires restructuring for build-script access | NOT recommended |
| Runtime subcommand (`logtok docs`) | Always up-to-date, user controls output path, simple | Tiny one-shot runtime cost | **RECOMMENDED** |

### Data Flow: clap -> askama -> HTML

```
User runs: logtok docs --output docs.html
   |
   v
main.rs: parse CLI, match Commands::Docs { output }
   |
   v
docs::generate_docs(Cli::command(), &output)
   |
   v
docs::extract_commands(&command)
   |  Walks: command.get_subcommands()
   |  For each: get_name(), get_about(), get_long_about(),
   |            get_arguments(), get_visible_aliases()
   |  For each arg: get_id(), get_help(), get_long_help(),
   |                get_default_values(), is_required_set(),
   |                get_short(), get_long()
   |
   v
Build DocsTemplate struct (askama context)
   |  version: env!("CARGO_PKG_VERSION")
   |  commands: Vec<CommandInfo>
   |  categories: 19-category reference table
   |
   v
template.render() -> String (HTML)
   |  askama compiles template at build time
   |  Template includes inline CSS and JS
   |
   v
std::fs::write(output, html)
```

### Intermediate Data Structures

```rust
/// Extracted from a single clap argument
struct ArgInfo {
    name: String,
    short: Option<char>,
    long: Option<String>,
    help: String,
    required: bool,
    default: Option<String>,
    is_positional: bool,
}

/// Extracted from a single clap subcommand
struct CommandInfo {
    name: String,
    about: String,
    long_about: Option<String>,
    args: Vec<ArgInfo>,
}

/// Token category for the reference table
struct CategoryInfo {
    prefix: &'static str,
    description: &'static str,
    example: &'static str,
}
```

The intermediate representation decouples clap's API from HTML rendering. The extraction can be tested independently of the template.

### askama Template Architecture

Use askama's inline `source` attribute to embed the entire HTML template in Rust source code, compiled at build time:

```rust
#[derive(askama::Template)]
#[template(ext = "html", source = r#"
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>logtok v{{ version }} -- Documentation</title>
  <style>{{ css }}</style>
</head>
<body>
  <header>...</header>
  <nav>
    <a href="#install">Install</a>
    <a href="#quickstart">Quick Start</a>
    {% for cmd in commands %}
    <a href="#{{ cmd.name }}">{{ cmd.name }}</a>
    {% endfor %}
    <a href="#categories">Categories</a>
  </nav>
  <main>
    <section id="install">...</section>
    <section id="quickstart">...</section>
    {% for cmd in commands %}
    <article id="{{ cmd.name }}">
      <h2>logtok {{ cmd.name }}</h2>
      <p>{{ cmd.about }}</p>
      <table>
      {% for arg in cmd.args %}
        <tr>
          <td><code>{% if let Some(s) = arg.short %}-{{ s }}, {% endif %}{% if let Some(l) = arg.long %}--{{ l }}{% endif %}</code></td>
          <td>{{ arg.help }}</td>
          {% if let Some(d) = arg.default %}<td>{{ d }}</td>{% endif %}
        </tr>
      {% endfor %}
      </table>
    </article>
    {% endfor %}
    <section id="categories">...</section>
  </main>
  <script>{{ js }}</script>
</body>
</html>
"#)]
struct DocsTemplate {
    version: String,
    css: String,
    js: String,
    commands: Vec<CommandInfo>,
    categories: Vec<CategoryInfo>,
}
```

**Why askama over format!():** The HTML template is 100-200+ lines with loops, conditionals, and HTML escaping requirements. `format!()` becomes unreadable and error-prone at this scale. askama provides:
- Auto-escaping of dynamic content (prevents XSS)
- Jinja-like syntax natural for HTML authors
- Compile-time template validation
- Zero runtime template parsing overhead

**Why inline `source` over external file:** Keeps everything in one `.rs` file. No `templates/` directory to manage. If the template grows past ~200 lines, move to a `templates/docs.html` file and use askama's `path` attribute instead -- this is a readability decision, not architectural.

### Single-File HTML Requirements

The output must be self-contained:
- All CSS in `<style>` tags (no external stylesheets)
- All JS in `<script>` tags (no external scripts)
- No external images (use CSS for any visual elements)
- Works from `file://` URLs, can be committed to repos, shared on Slack

### CSS Architecture

Use CSS custom properties for theming:

```css
:root {
  --bg: #ffffff; --text: #1a1a2e; --accent: #0066cc;
  --code-bg: #f5f5f5; --border: #e0e0e0;
}
@media (prefers-color-scheme: dark) {
  :root {
    --bg: #1a1a2e; --text: #e0e0e0; --accent: #66b3ff;
    --code-bg: #2a2a3e; --border: #444;
  }
}
```

No CSS framework. ~100-150 lines covering layout, typography, code blocks, responsive design.

### JavaScript (Minimal)

Two features requiring JS:
1. **Copy buttons** (~15 lines): `navigator.clipboard.writeText()` on code blocks
2. **Smooth scroll** (~5 lines): `scrollIntoView({ behavior: 'smooth' })` for nav links

Total: ~20-30 lines. No build tools, no bundler, no framework.

## New Module Structure

```
src/
  cli.rs          # MODIFIED: add const STYLES, add Docs subcommand
  main.rs         # MODIFIED: add Docs match arm
  docs.rs         # NEW: ~300-400 lines
    - generate_docs()     Public entry point
    - extract_commands()  Walk clap Command tree -> Vec<CommandInfo>
    - DocsTemplate        askama struct with inline HTML template
    - CSS_CONTENT         const &str with embedded CSS
    - JS_CONTENT          const &str with embedded JS
  ... (all existing modules unchanged)
```

## Build Order

**Build colored CLI help FIRST, then HTML docs.** Rationale:

1. Colored help is simpler (~15 lines) and validates clap's `Styles` API
2. Colored help modifies `cli.rs` -- do this first so HTML docs builds on stable CLI
3. HTML docs requires understanding the `Command` introspection API -- colored help builds familiarity with clap's builder types
4. The `Docs` subcommand addition happens after colored help is stable

### Phase Sequence

```
Phase 1: Colored CLI Help
  1. Add const STYLES to cli.rs
  2. Apply styles = STYLES attribute
  3. Add styled examples via after_long_help
  4. Test: logtok --help shows colored output
  5. Test: NO_COLOR=1 logtok --help shows plain output
  6. Test: piped output has no ANSI codes

Phase 2: HTML Documentation Generation
  1. cargo add askama (one new dependency)
  2. Add Commands::Docs to cli.rs
  3. Create docs.rs with extract_commands()
  4. Create askama DocsTemplate with inline HTML/CSS/JS
  5. Wire up in main.rs
  6. Test: logtok docs produces valid HTML
  7. Test: HTML renders correctly in browser (both themes)
  8. Test: Copy buttons work
  9. Test: Command reference matches --help content
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: External CSS/JS Files
**What:** Referencing external stylesheets or scripts in the generated HTML.
**Why bad:** Breaks the single-file requirement. File won't render when opened locally or shared.
**Instead:** Embed all CSS in `<style>` and all JS in `<script>` tags.

### Anti-Pattern 2: Build-Time Generation with build.rs
**What:** Generating HTML in a build script and embedding it in the binary.
**Why bad:** Requires the CLI struct to be accessible from build.rs (needs crate restructuring). Generated HTML becomes stale if help text changes without rebuild.
**Instead:** Generate at runtime via `logtok docs`. Milliseconds of cost, zero build complexity.

### Anti-Pattern 3: Parsing --help Output as Text
**What:** Running `logtok --help` and parsing the text output to extract command info.
**Why bad:** Fragile, breaks when help formatting changes, loses structured data (types, defaults, required flags).
**Instead:** Use `Cli::command()` introspection API directly.

### Anti-Pattern 4: Duplicating Help Text
**What:** Writing command descriptions in both cli.rs doc comments AND in the HTML template.
**Why bad:** Two sources of truth. They will drift apart.
**Instead:** Single source of truth in cli.rs doc comments. HTML generation reads from `Command::get_about()` / `get_long_about()`.

### Anti-Pattern 5: Hardcoding ANSI Escape Codes in Help Strings
**What:** Manually embedding `\x1b[32m` escape codes in clap help text.
**Why bad:** Breaks on terminals that don't support those codes. Breaks when piped. Breaks NO_COLOR compliance.
**Instead:** Use clap's `Styles` API which handles terminal detection automatically.

## Sources

- [clap Styles API](https://docs.rs/clap/latest/clap/builder/struct.Styles.html) -- Authoritative, current documentation
- [clap Command introspection](https://docs.rs/clap/latest/clap/struct.Command.html) -- get_subcommands(), get_arguments()
- [clap ColorChoice](https://docs.rs/clap/latest/clap/enum.ColorChoice.html) -- NO_COLOR/CLICOLOR handling
- [askama inline templates](https://askama.rs/en/latest/creating_templates.html) -- source attribute documentation
- [CSS prefers-color-scheme](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-color-scheme) -- dark/light theme standard
- [Rust CLI documentation patterns](https://rust-cli.github.io/book/in-depth/docs.html) -- Official CLI book
