---
phase: 06-local-ui-interface
reviewed: 2026-04-30T12:00:00Z
depth: standard
files_reviewed: 21
files_reviewed_list:
  - src/cli.rs
  - src/main.rs
  - src/ui/api_config.rs
  - src/ui/api_store.rs
  - src/ui/api_tokenize.rs
  - src/ui/assets.rs
  - src/ui/handlers.rs
  - src/ui/i18n.rs
  - src/ui/mod.rs
  - src/ui/routes.rs
  - src/ui/ws.rs
  - static/app.js
  - static/styles.css
  - tailwind.config.js
  - templates/ui/base.html
  - templates/ui/config.html
  - templates/ui/detokenize.html
  - templates/ui/docs.html
  - templates/ui/store.html
  - templates/ui/tokenize.html
  - tests/ui_server.rs
findings:
  critical: 2
  warning: 4
  info: 2
  total: 8
status: issues_found
---

# Phase 06: Code Review Report

**Reviewed:** 2026-04-30T12:00:00Z
**Depth:** standard
**Files Reviewed:** 21
**Status:** issues_found

## Summary

The local UI interface implements a localhost-only web dashboard using Axum, HTMX, and Alpine.js. The architecture is sound: session keys for encryption, 127.0.0.1 binding, HTML escaping on outputs, and graceful shutdown via WebSocket heartbeat. However, there are two critical issues -- a path traversal vulnerability in the tokenize API and a broken HTML template -- along with several warnings around UTF-8 safety, inaccurate metrics, and missing WebSocket reconnection logic.

## Critical Issues

### CR-01: Path Traversal in Tokenize API

**File:** `src/ui/api_tokenize.rs:145-152`
**Issue:** The `file_path` field from user-submitted multipart form data is used directly as a filesystem path with no validation beyond checking it is a regular file. Although the server binds to 127.0.0.1, a user (or malicious script on localhost) can read any file the process has permission to access by submitting paths like `/etc/shadow`, `C:\Windows\System32\config\SAM`, or `../../sensitive.conf`. The comment references "T-06-02: Validate path is a regular file" but the check only verifies `is_file()`, not that the path is within an allowed directory.
**Fix:** Canonicalize the path and validate it resides within an allowed directory (e.g., CWD or a configured root). At minimum, reject paths containing `..` segments:
```rust
let p = PathBuf::from(fp);
let canonical = std::fs::canonicalize(&p)
    .map_err(|e| format!("Cannot access file '{}': {}", fp, e))?;
let cwd = std::env::current_dir()
    .map_err(|e| format!("Cannot determine CWD: {}", e))?;
let cwd_canonical = std::fs::canonicalize(&cwd)
    .map_err(|e| format!("Cannot canonicalize CWD: {}", e))?;
if !canonical.starts_with(&cwd_canonical) {
    return Err(format!("Path '{}' is outside the working directory", fp));
}
let meta = std::fs::metadata(&canonical)
    .map_err(|e| format!("Cannot access file '{}': {}", fp, e))?;
if !meta.is_file() {
    return Err(format!("Not a regular file: {}", fp));
}
```

### CR-02: Unclosed Body Tag in base.html

**File:** `templates/ui/base.html:19-22`
**Issue:** The `<body>` opening tag is missing its closing `>`. Line 19 has the opening `<body` with a `class` attribute, line 20 has a `:class` attribute, but line 21 is empty and line 22 starts `<!-- Top Navigation -->`. The `>` that should close the body tag after the `:class` attribute value is absent. This causes the browser to parse the navigation and all subsequent content as part of the body tag's attributes, resulting in a blank page or severely broken rendering.
**Fix:** Add the closing `>` after the `:class` attribute on line 20:
```html
<body class="bg-surface-900 text-zinc-100 font-sans antialiased min-h-screen"
      :class="theme === 'light' ? 'light-mode bg-surface-50 text-zinc-900' : 'bg-surface-900 text-zinc-100'">
```

## Warnings

### WR-01: UTF-8 Byte Slicing Panic in Token Store Display

**File:** `src/ui/api_store.rs:29-30`
**Issue:** `&value[..40]` slices by byte index, not by character boundary. If a token's stored value contains multi-byte UTF-8 characters (common in internationalized logs, URLs with encoded characters, or PEM data with certain byte sequences), this will panic at runtime with "byte index is not a char boundary."
**Fix:** Use `char_indices` to find a safe truncation point:
```rust
let display_value = if value.len() > 40 {
    let end = value.char_indices()
        .take_while(|(i, _)| *i <= 40)
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(40.min(value.len()));
    format!("{}...", &value[..end])
} else {
    (*value).clone()
};
```

### WR-02: Inaccurate Token Count Metric

**File:** `src/ui/api_tokenize.rs:193`
**Issue:** Token count is estimated by counting `[` characters in the output: `output.matches('[').count()`. This over-counts because log content, JSON structures, and other text commonly contain `[` characters that are not tokens. The displayed "tokens detected" number will be misleading.
**Fix:** Use the actual token regex pattern `\[([A-Z]+_\d{3,})\]` to count real tokens:
```rust
let token_re = regex::Regex::new(r"\[[A-Z]+_\d{3,}\]").unwrap();
let token_count = token_re.find_iter(&output).count();
```

### WR-03: No WebSocket Reconnection Causes Premature Server Shutdown

**File:** `static/app.js:84-98`
**Issue:** The WebSocket heartbeat has no reconnection logic. If the connection drops due to a transient network event, OS sleep/wake, or browser tab throttling, the server receives no heartbeat for 15 seconds and triggers graceful shutdown (per `src/ui/ws.rs:19`). The user loses their server while still actively using the dashboard in another tab or after waking their laptop.
**Fix:** Add reconnection with exponential backoff:
```javascript
startHeartbeat() {
  var self = this;
  var retryDelay = 1000;
  function connect() {
    var protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    var ws = new WebSocket(protocol + '//' + location.host + '/ws/heartbeat');
    var interval;
    ws.onopen = function() {
      retryDelay = 1000;
      interval = setInterval(function() {
        if (ws.readyState === WebSocket.OPEN) { ws.send('ping'); }
      }, 5000);
    };
    ws.onclose = function() {
      clearInterval(interval);
      setTimeout(connect, retryDelay);
      retryDelay = Math.min(retryDelay * 2, 30000);
    };
    ws.onerror = function() { ws.close(); };
  }
  connect();
}
```

### WR-04: Unsafe `set_var` Called Without Synchronization Guarantees

**File:** `src/ui/mod.rs:28-33`
**Issue:** `std::env::set_var` is used inside an `unsafe` block to set `LOGTOK_KEY` so that `Store::new()` picks it up. While this runs before the Axum server starts multi-threaded request handling, the `unsafe` marker exists precisely because `set_var` is unsound in multi-threaded programs (it mutates global state without synchronization). If any library or dependency reads environment variables during this window, it creates undefined behavior. The `#[allow(deprecated)]` suppresses the Rust 2024 deprecation warning but does not make it safe.
**Fix:** Instead of modifying the global environment, pass the session key directly to `Store::with_passphrase` everywhere (which is already done in the API handlers). Remove the `set_var` call entirely and ensure `Store::with_passphrase` is the only code path:
```rust
let key = std::env::var("LOGTOK_KEY").unwrap_or_else(|_| {
    eprintln!("logtok: session key generated (set LOGTOK_KEY for CLI interop)");
    session_key
});
// Do NOT call set_var -- handlers already use Store::with_passphrase
```

## Info

### IN-01: Duplicate HTML Escape Functions

**File:** `src/ui/api_tokenize.rs:12-18` and `src/ui/api_store.rs:240-246`
**Issue:** Two nearly identical HTML escape functions exist: `html_escape` in `api_tokenize.rs` and `escape_html` in `api_store.rs`. They differ only in how single quotes are encoded (`&#x27;` vs `&#39;` -- both are valid). This is code duplication that increases maintenance burden and risks divergence.
**Fix:** Remove `html_escape` from `api_tokenize.rs` and use the existing `escape_html` from `api_store.rs` (which is already `pub`):
```rust
// In api_tokenize.rs, replace html_escape calls with:
use crate::ui::api_store::escape_html;
```

### IN-02: JSON Injection in HX-Trigger Header

**File:** `src/ui/api_tokenize.rs:81`
**Issue:** The file path is inserted into a JSON string in the `HX-Trigger` header with only `"` escaped. While this is a localhost-only server, a file path containing backslashes (common on Windows) or other JSON-special characters could produce malformed JSON, causing the HTMX trigger to silently fail. This is not a security issue since the server is local-only, but it is a correctness issue on Windows.
**Fix:** Use `serde_json` to properly serialize the path value:
```rust
let trigger = serde_json::json!({"addRecentFile": path}).to_string();
```

---

_Reviewed: 2026-04-30T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
