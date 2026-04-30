# Phase 06 -- UI Review

**Audited:** 2026-04-30
**Baseline:** Abstract 6-pillar standards (no UI-SPEC.md exists)
**Screenshots:** Not captured (no dev server detected on ports 3000, 5173, 8080)

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Copywriting | 3/4 | Context-specific labels throughout; minor i18n gap in server-rendered fragments |
| 2. Visuals | 3/4 | Clean visual hierarchy with nav/tab structure; many CSS classes referenced but undefined |
| 3. Color | 4/4 | Well-structured CSS custom property system; dark/light themes complete with proper semantic tokens |
| 4. Typography | 4/4 | Disciplined 6-size scale (0.6875rem-1.25rem) with 2 weights (500, 600) |
| 5. Spacing | 3/4 | Consistent rem-based spacing using logical properties; some rgba() values hardcoded instead of using CSS variables |
| 6. Experience Design | 3/4 | Loading states, error handling, empty states all present; language switch calls missing API endpoint |

**Overall: 20/24**

---

## Top 3 Priority Fixes

1. **Missing CSS class definitions for server-rendered HTML** -- Users will see unstyled result panels, store tables, config forms, and docs sections because ~25 CSS classes used in Rust handler HTML output and templates have no corresponding CSS rules in styles.css. Add styles for: `.result-success`, `.result-error`, `.result-stats`, `.result-actions`, `.result-output`, `.btn-copy`, `.result-area`, `.store-panel`, `.store-empty`, `.panel-header`, `.config-panel`, `.config-mode-toggle`, `.config-error`, `.config-success`, `.category-grid`, `.patterns-list`, `.pattern-row`, `.form-actions`, `.toggle-label`, `.docs-panel`, `.command-block`, `.args-table`, `.long-about`, `.after-help`, `.toml-editor`, `.input-mode-tabs`, `.mode-tab`, `.input-section`, `.filepath-row`, `.input-hint`, `.input-textarea`, `.input-text`, `.file-input-hidden`, `.file-picker-label`, `.drop-zone-content`, `.drop-icon`, `.drop-text`, `.drop-or`, `.drop-active`, `.selected-file`, `.recent-label`.

2. **`/api/translations` endpoint missing** -- `app.js:52` calls `fetch('/api/translations?lang=' + ...)` when the user switches language, but no such route exists in `routes.rs`. Hebrew language switching will silently fail (fetch returns 404, falls back to inlined English translations). Add a `GET /api/translations` handler that returns JSON from `i18n::translations(lang)`, or change the approach to reload the page with a lang query parameter.

3. **Handler always renders English translations** -- `handlers.rs:15` hardcodes `i18n::translations("en")` regardless of the user's stored language preference. When a Hebrew user loads the page, the inline `window.__i18n` object contains English strings, and since `/api/translations` does not exist, the UI remains in English despite the `lang` being set to `he` in localStorage. Pass the user's language preference (via cookie or query param) to the handler, or render both language sets inline.

---

## Detailed Findings

### Pillar 1: Copywriting (3/4)

The copywriting is context-appropriate for a developer tool dashboard. Labels are specific rather than generic:

**Strengths:**
- "Paste log content here..." rather than generic "Enter text" (tokenize.html:28)
- "Best for large files -- the server reads the file directly without uploading" -- helpful contextual hint (tokenize.html:46)
- "Paste tokenized text (from Claude Code response, etc.)" -- tells user exactly what input is expected (detokenize.html:9)
- "No tokens in store. Tokenize a file first." -- actionable empty state (api_store.rs:59)
- "No content provided. Paste tokenized text to detokenize." -- tells user what to do (api_tokenize.rs:218)
- "Cannot load token store: ... Tokenize some content first." -- actionable error (api_tokenize.rs:231)
- 34 i18n translation keys covering both English and Hebrew (i18n.rs)
- "Tokenize" / "Detokenize" as CTA labels instead of generic "Submit" on the form buttons

**Issues:**
- Server-rendered HTML fragments (api_tokenize.rs, api_store.rs, api_config.rs) use hardcoded English strings with no i18n support. Examples: "Token" / "Category" / "Value" table headers (api_store.rs:49), "Save Configuration" button (api_config.rs:86), "Form view" / "Raw TOML" toggle labels (api_config.rs:69-70), "Command Reference" / "Token Categories" headings (api_store.rs:73-74), "Refresh" button (api_store.rs:47), "+ Add Pattern" (api_config.rs:79).
- The `data-i18n` attributes on template elements are not wired to any JS translation swap logic -- they serve as documentation only.

### Pillar 2: Visuals (3/4)

**Strengths:**
- Clear visual hierarchy: fixed nav bar (56px) with tab navigation, main content area below
- Active tab uses primary color bottom border indicator (styles.css:132-134)
- Theme toggle has sun/moon SVG icons with proper `aria-label` (base.html:51)
- Language selector has `aria-label` (base.html:75)
- Tab content uses `x-cloak` to prevent FOUC (base.html:89-111)
- Fade-in animation on tab switch (styles.css:196-199)
- Upload has a drag-and-drop zone with upload icon SVG (tokenize.html:84-88)
- Version badge in nav provides orientation (base.html:82)

**Issues:**
- Major styling gap: ~40 CSS classes used across templates and server-rendered HTML have no definitions in styles.css (see Priority Fix 1). This means the tokenize result panel, detokenize result panel, store table wrapper, config form, docs layout, and all input mode tabs will render with browser defaults -- no custom styling.
- The `.drop-active` class referenced in tokenize.html:69 has no CSS rule (`.drag-over` exists at styles.css:331 but the HTML uses `.drop-active`).
- No favicon defined -- browser tab shows generic icon.

### Pillar 3: Color (4/4)

**Strengths:**
- Clean CSS custom property system with semantic variable names (styles.css:6-18)
- Dark theme: #0f0f10 bg, #1c1c1e surface, #6366f1 primary -- proper near-black dev tool aesthetic
- Light theme: #fafaf5 bg, #ffffff surface, #4f46e5 primary -- warm white with adjusted contrast
- OS preference detection via `prefers-color-scheme` media query (styles.css:72-84) handles pre-Alpine hydration state
- No hardcoded colors outside the CSS variable system in stylesheets -- all element styles reference `var(--xxx)`
- Accent color (primary) used intentionally: active tab indicator, button backgrounds, focus rings, toggle switches
- Success (#22c55e) and error (#ef4444) semantically applied
- 6 rgba() uses are all for subtle hover/focus backgrounds using the primary/success/error colors at low opacity -- appropriate usage
- `#ffffff` used only for button text on colored backgrounds -- correct for contrast

**Minor note:**
- The `:root` and `.theme-dark` blocks are identical (styles.css:6-18 vs 42-52). The `:root` block could be removed since Alpine.js always applies `.theme-dark` or `.theme-light` class. Not a defect -- it provides a sensible default before JS hydrates.

### Pillar 4: Typography (4/4)

**Strengths:**
- Base font: `system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif` -- proper system font stack
- Monospace for code: `'Cascadia Code', 'Fira Code', 'JetBrains Mono', monospace` -- developer-appropriate
- Exactly 6 font sizes forming a clear scale:
  - 0.6875rem (11px) -- version badge
  - 0.75rem (12px) -- table headers, remove buttons
  - 0.8125rem (13px) -- labels, monospace content, lang selector
  - 0.875rem (14px) -- body text, buttons, inputs, status messages (workhorse size)
  - 1rem (16px) -- config section headings
  - 1.25rem (20px) -- panel headings
- 2 font weights: 500 (medium, for tabs/labels/buttons) and 600 (semibold, for headings/table headers)
- `font-family: inherit` applied to buttons and inputs to prevent browser defaults

### Pillar 5: Spacing (3/4)

**Strengths:**
- Consistent rem-based spacing throughout -- no px values except the base `font-size: 14px`
- CSS logical properties used consistently: `padding-inline`, `margin-block-start`, `margin-inline-start`, `inset-inline-start`, `border-block-end` -- excellent RTL foundation
- Spacing values cluster around a coherent scale: 0.25, 0.375, 0.5, 0.625, 0.75, 1, 1.25, 1.5, 2, 3 rem
- `[dir="rtl"]` overrides minimal -- only for toggle switch transform direction (styles.css:452-454) and icon flip (styles.css:549)
- Responsive breakpoints at 768px and 480px with appropriate adjustments

**Issues:**
- No declared spacing scale (8-point, 4-point, etc.) -- the 10+ distinct spacing values work but are not constrained to a formal scale. Values like 0.375rem (6px) and 0.625rem (10px) break a pure 4-point or 8-point grid.
- `calc(var(--nav-height) + 40px)` at styles.css:582 uses a hardcoded `40px` magic number for mobile nav offset.

### Pillar 6: Experience Design (3/4)

**Strengths:**
- **Loading states:** HTMX indicators on tokenize (tokenize.html:110-113) and detokenize (detokenize.html:19-22) panels with spinner animation (styles.css:471-483). Store, docs, and config panels have "Loading..." text indicators (store.html:7, docs.html:7, config.html:7).
- **Error states:** All API handlers return styled `.result-error` or `.config-error` divs with specific error messages (api_tokenize.rs:99-112, api_store.rs:58-61, api_config.rs:120-125, 183-185, 198-208).
- **Empty states:** Token store shows "No tokens in store. Tokenize a file first." (api_store.rs:59). No-input error gives actionable message (api_tokenize.rs:170, 217-218).
- **Copy to clipboard:** Result panels include a copy button using `navigator.clipboard.writeText()` (api_tokenize.rs:89-91, 243-244).
- **Drag-and-drop:** Auto-submits on drop with visual feedback via Alpine.js reactive state (tokenize.html:73-82).
- **File picker:** Auto-submits on file selection (tokenize.html:96-100).
- **Recent files:** localStorage persistence with 10-file limit and deduplication (tokenize.html:34-38, app.js:37-41).
- **WebSocket heartbeat:** Auto-stop server on browser tab close (app.js:84-98, ws.rs).
- **Tab/language persistence:** localStorage survives page reload (app.js:6-8).
- **OS theme following:** Real-time theme updates via matchMedia listener (app.js:71-72).
- **Input validation:** File path validated as regular file server-side (api_tokenize.rs:147-151). TOML validated before write (api_config.rs:119).
- **XSS prevention:** All user-facing content HTML-escaped (api_tokenize.rs:12-18, api_store.rs:170-176).

**Issues:**
- `/api/translations` endpoint referenced in app.js:52 does not exist -- language switching to Hebrew will fail silently. The fallback (app.js:58) reads `window.__i18n` which only has English strings.
- `handlers.rs:15` always passes English translations to the template, ignoring user language preference.
- No confirmation dialog for destructive actions (e.g., saving config overwrites existing .logtok.toml without warning).
- No disabled state on submit buttons during HTMX request -- user can double-submit.
- Copy button feedback: no visual confirmation that copy succeeded (toast or button text change).

---

## Files Audited

- `templates/ui/base.html` -- Main dashboard template (117 lines)
- `templates/ui/tokenize.html` -- Tokenize panel (117 lines)
- `templates/ui/detokenize.html` -- Detokenize panel (27 lines)
- `templates/ui/store.html` -- Token store panel (9 lines)
- `templates/ui/docs.html` -- Docs panel (9 lines)
- `templates/ui/config.html` -- Config panel with addPatternRow (25 lines)
- `static/styles.css` -- Full theme CSS (677 lines)
- `static/app.js` -- Alpine.js app state (100 lines)
- `src/ui/handlers.rs` -- Dashboard handler (29 lines)
- `src/ui/api_tokenize.rs` -- Tokenize/detokenize API (259 lines)
- `src/ui/api_store.rs` -- Store/docs API (177 lines)
- `src/ui/api_config.rs` -- Config API (211 lines)
- `src/ui/i18n.rs` -- Translation strings (referenced via summaries)
