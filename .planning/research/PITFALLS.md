# Domain Pitfalls: Colored CLI Help & Auto-Generated HTML Documentation

**Domain:** Adding documentation/DX features to an existing Rust CLI tool (logtok)
**Researched:** 2026-04-28
**Confidence:** HIGH
**Milestone:** v2.0 Developer Experience

## Critical Pitfalls

### Pitfall 1: Windows cmd.exe ANSI Escape Code Breakage

**What goes wrong:**
Colored CLI help output renders as raw escape codes (`[0m[1mUsage[0m: logtok...`) instead of styled text on Windows. This happens in cmd.exe, older PowerShell versions, and certain CI/CD runners.

**Why it happens:**
Windows has a fragmented terminal landscape. Windows Terminal supports ANSI natively. But cmd.exe requires the `ENABLE_VIRTUAL_TERMINAL_PROCESSING` console mode flag to be set via Win32 API -- not enabled by default. Piping output (`logtok --help | more`) disables the terminal flag.

**Consequences:**
- First impression for Windows users is a broken-looking tool
- Help text becomes unreadable
- CI/CD logs (GitHub Actions Windows runners) show garbage characters

**Prevention:**
- clap's `color` feature (already enabled by default in logtok's Cargo.toml) handles color detection via `anstream`. It respects `NO_COLOR`, `CLICOLOR_FORCE`, and detects TTY via `IsTerminal`. Do NOT duplicate this logic.
- Test on actual cmd.exe, not just Windows Terminal. Use `cmd.exe /c "logtok --help"` during development.
- Verify piped output strips ANSI codes: `logtok --help | findstr Usage` should not contain escape sequences.

**Detection:** Raw `\x1b[` sequences visible in help output on Windows.

**Phase to address:** Phase 1 (Colored CLI Help).

---

### Pitfall 2: Clap Styles Override Breaking Readability on Light/Dark Terminals

**What goes wrong:**
Custom `Styles` configuration uses colors that look great on the developer's dark terminal but become invisible on light backgrounds, or vice versa.

**Why it happens:**
Developers customize colors for their terminal theme without testing the opposite. A green that looks great on dark backgrounds becomes invisible on white backgrounds.

**Prevention:**
- Use only the 8 basic ANSI colors (not 256-color or RGB). Basic ANSI colors adapt to the terminal's palette -- terminals remap them for their theme.
- Test with both dark and light terminal themes.
- Never rely solely on color to convey meaning. Use bold/underline alongside color.
- Avoid dim text -- many terminals render it identically to normal or make it unreadable.
- Start with clap's `Styles::styled()` default and only modify what you need.

**Phase to address:** Phase 1 (Colored CLI Help).

---

### Pitfall 3: Generated HTML Documentation Drifting Out of Sync with CLI

**What goes wrong:**
The HTML documentation page shows commands, flags, or descriptions that do not match the actual CLI.

**Why it happens:**
Three failure modes:
1. Docs generated once, committed, never regenerated when CLI changes
2. `logtok docs` command exists but no CI test verifies correctness
3. Hand-written sections (install guide, quick start) reference outdated commands

**Prevention:**
- Generate the command reference at runtime from `Cli::command()` introspection. This guarantees the reference matches the binary.
- Do not hardcode command names in the askama template. Use clap's `Command::get_subcommands()`, `Command::get_arguments()`, etc.
- Add a CI test that runs `logtok docs --output /tmp/docs.html` and validates the output contains all current subcommands.
- Keep hand-written content in the template but reference command names via clap introspection, not string literals.

**Phase to address:** Phase 2 (HTML Docs Generation).

---

### Pitfall 4: Binary Size Bloat from Embedded HTML/CSS/JS

**What goes wrong:**
Embedding CSS frameworks, fonts, syntax highlighting JS, and icons inflates the binary by 500KB-2MB+.

**Why it happens:**
Developers start with a minimal template (5KB), then add Tailwind CSS (~300KB), highlight.js (~70KB), web fonts (100-400KB), and a clipboard library. Each seems small but they compound.

**Prevention:**
- **Budget: <50KB total** for the HTML template including inline CSS and JS.
- No CSS frameworks. Write minimal custom CSS (~2-3KB). A docs page does not need Bootstrap or Tailwind.
- No external fonts. Use the system font stack (`font-family: system-ui, -apple-system, sans-serif`).
- No JS syntax highlighting. CSS-only styling for code blocks is sufficient.
- Copy-to-clipboard: Use native `navigator.clipboard.writeText()` (~15 lines of JS). No clipboard.js library.
- askama compiles the inline template into Rust code, so the HTML string is part of the binary's `.rodata` section. Measure with `cargo bloat --release`.

**Phase to address:** Phase 2 (HTML Docs Generation). Size budget must be set before designing the template.

---

## Moderate Pitfalls

### Pitfall 5: Clipboard API Failing on file:// URLs

**What goes wrong:**
Copy-to-clipboard buttons work when served via localhost/HTTPS but fail silently when the HTML file is opened directly as a `file://` URL -- which is exactly how most users will open locally-generated docs.

**Why it happens:**
`navigator.clipboard.writeText()` requires a "secure context" (HTTPS or localhost). `file://` protocol behavior is inconsistent: Chrome allows it, Firefox blocks it, Safari varies.

**Prevention:**
Implement a fallback using the legacy `document.execCommand('copy')`:

```javascript
async function copyText(text, btn) {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    const ta = document.createElement('textarea');
    ta.value = text;
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.appendChild(ta);
    ta.select();
    document.execCommand('copy');
    document.body.removeChild(ta);
  }
  btn.textContent = 'Copied!';
  setTimeout(() => btn.textContent = 'Copy', 1500);
}
```

- Always wrap clipboard calls in try/catch. On failure, show visible feedback.
- Safari requires clipboard write to happen in direct response to a user gesture (click handler). Do not wrap in setTimeout or async chains that break the gesture.
- Test the generated HTML by opening it directly as a file (double-click) in Chrome, Firefox, and Edge.

**Phase to address:** Phase 2 (HTML Docs Generation).

---

### Pitfall 6: askama Template Compile Errors Are Cryptic

**What goes wrong:**
askama validates templates at compile time (a feature), but error messages point to the `#[derive(Template)]` line rather than the specific template line with the error. A typo in the Jinja syntax or a missing struct field produces a proc-macro error that's hard to debug.

**Prevention:**
- Start with a minimal template and add sections incrementally. Compile after each addition.
- Keep the inline `source` template under ~150 lines. If it grows larger, move to an external `templates/docs.html` file where editors provide Jinja syntax highlighting and line numbers.
- Use `cargo check` frequently -- faster than `cargo build` for catching template errors.
- Common mistakes: `{{ variable }}` vs `{% tag %}` confusion, forgetting `{% endfor %}` or `{% endif %}`, using `Option` fields without `{% if let %}` guards.

**Phase to address:** Phase 2 (HTML Docs Generation).

---

### Pitfall 7: clap Introspection Returning Hidden Arguments

**What goes wrong:**
`Command::get_arguments()` returns ALL arguments including hidden ones. clap auto-adds `--help` and `--version` as hidden arguments. The generated HTML docs include these for every subcommand, cluttering the reference.

**Prevention:**
- Filter arguments: skip any where `arg.is_hide_set()` returns true.
- Either exclude `--help`/`--version` entirely (users know about these) or include them once in a "Global Options" section rather than repeating per-subcommand.

**Phase to address:** Phase 2 (HTML Docs Generation).

---

### Pitfall 8: ANSI Escape Codes Leaking into Piped/Redirected Output

**What goes wrong:**
Colored help works in terminals but when piped (`logtok --help | grep tokenize`) or redirected (`logtok --help > help.txt`), ANSI codes appear as garbage.

**Prevention:**
- clap's `anstream` already strips ANSI when stdout is not a TTY. This works out of the box IF you do not bypass anstream.
- If you add custom colored output outside clap's help system, always write through `anstream::stdout()`, never raw `println!` with embedded ANSI.
- Test: `logtok --help > /tmp/help.txt && cat /tmp/help.txt` should contain no escape sequences.

**Phase to address:** Phase 1 (Colored CLI Help).

---

### Pitfall 9: wrap_help Feature Not Enabled

**What goes wrong:**
Long help descriptions wrap at the terminal's character boundary, breaking words mid-syllable and making help text hard to read on narrow terminals (80 columns, laptop screens).

**Prevention:**
- Enable clap's `wrap_help` feature: `clap = { version = "4.6.0", features = ["derive", "wrap_help"] }`. This makes clap detect terminal width and wrap at word boundaries.
- This feature is NOT enabled by default. It adds a dependency on `terminal_size` crate (minimal size impact).
- Test with narrow terminal widths (80 columns, 60 columns).

**Phase to address:** Phase 1 (Colored CLI Help).

---

## Minor Pitfalls

### Pitfall 10: HTML Not Accessible

**What goes wrong:**
Generated HTML docs fail accessibility: no semantic headings, poor contrast, copy buttons without aria labels, code blocks not keyboard-navigable.

**Prevention:**
- Use semantic HTML: `<h1>` for tool name, `<h2>` for sections, `<h3>` for subcommands.
- Copy buttons: add `aria-label="Copy command to clipboard"`, make buttons keyboard-focusable.
- Color contrast: minimum 4.5:1 ratio for body text (WCAG AA).

**Phase to address:** Phase 2 (HTML Docs Generation).

### Pitfall 11: Short Help vs Long Help Rendering Differences

**What goes wrong:**
`-h` shows `about`, `--help` shows `long_about` + `after_help`. Users see styled content sometimes and plain content other times.

**Prevention:**
- Test all variants: `logtok -h`, `logtok --help`, `logtok help`, `logtok help tokenize`.
- Keep the short `about` clean. Put styled content only in `long_about` or `after_help`.

**Phase to address:** Phase 1 (Colored CLI Help).

### Pitfall 12: askama HTML Auto-Escaping Surprises

**What goes wrong:**
askama in HTML mode auto-escapes all `{{ variable }}` output. If clap help text contains intentional HTML-like content (e.g., `<FILE>` placeholder), it renders as `&lt;FILE&gt;` in the docs page.

**Prevention:**
- This is actually correct behavior -- you WANT escaping for safety. Design the template so `<FILE>` displays properly by using it in a `<code>` context where the escaped text is still readable.
- Do NOT use the `|safe` filter on any content from clap metadata.
- If you need literal HTML in static template sections, write it directly in the template markup, not through variables.

**Phase to address:** Phase 2 (HTML Docs Generation).

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Colored CLI Help (Phase 1) | Windows cmd.exe shows raw escape codes | Test on actual cmd.exe, rely on clap's anstream |
| Colored CLI Help (Phase 1) | Custom styles unreadable on light terminals | Stick to basic 8 ANSI colors, test both themes |
| Colored CLI Help (Phase 1) | Piped output contains escape codes | Rely on clap's anstream, do not bypass with raw println |
| Colored CLI Help (Phase 1) | Word wrapping ugly on narrow terminals | Enable `wrap_help` clap feature |
| HTML Docs (Phase 2) | Docs drift from actual CLI | Generate from clap Command tree, CI test for sync |
| HTML Docs (Phase 2) | Binary bloated by embedded assets | 50KB budget, no frameworks, system fonts, minimal JS |
| HTML Docs (Phase 2) | Copy buttons fail on file:// URLs | Fallback to execCommand, try/catch with visible error |
| HTML Docs (Phase 2) | askama errors hard to debug | Build template incrementally, compile-check often |
| HTML Docs (Phase 2) | Hidden args clutter generated docs | Filter `is_hide_set()` args during extraction |

## Sources

- [clap ColorChoice docs](https://docs.rs/clap/latest/clap/enum.ColorChoice.html)
- [clap Styles docs](https://docs.rs/clap/latest/clap/builder/struct.Styles.html)
- [clap features list](https://docs.rs/clap/latest/clap/_features/index.html)
- [Rain's Rust CLI Recommendations: Managing Colors](https://rust-cli-recommendations.sunshowers.io/managing-colors-in-rust.html)
- [NO_COLOR standard](https://no-color.org/)
- [anstream: simplifying terminal styling](https://epage.github.io/blog/2023/03/anstream-simplifying-terminal-styling/)
- [askama template syntax](https://askama.rs/en/latest/template_syntax.html)
- [navigator.clipboard secure context requirement (MDN)](https://developer.mozilla.org/en-US/docs/Web/API/Clipboard_API)
- [Microsoft: Console Virtual Terminal Sequences](https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences)
- [colorchoice-clap crate](https://docs.rs/colorchoice-clap)
