# Phase 6: Local UI Interface - Research

**Researched:** 2026-04-29
**Domain:** Rust web server with embedded HTML dashboard (axum + Askama + HTMX + Alpine.js)
**Confidence:** HIGH

## Summary

Phase 6 adds a `logtok ui` subcommand that starts a local axum web server on `127.0.0.1`, embeds all frontend assets into the single binary via `rust-embed`, and serves a five-tab dashboard (Tokenize, Detokenize, Token Store, Config, Docs) with dark/light themes and English/Hebrew RTL support. The frontend stack is HTMX 2.0 for server-driven interactivity plus Alpine.js for client-side reactivity (tabs, theme, language, RTL flip), totaling ~30KB with no build step. A WebSocket heartbeat detects browser tab close and triggers server auto-stop.

The project already uses tokio (synchronous `main()` but tokio is in Cargo.toml scope), askama for HTML templating, and clap for CLI. axum is built on tokio so there is zero runtime conflict. The existing `docs.rs` Askama pattern (compile-time template rendering with `CommandInfo` structs) directly ports to the dashboard's Docs tab. Core functions (`process_file_with_config`, `detokenize`, `Store::load/save`, `LoktokConfig`) are exposed via JSON API endpoints.

**Primary recommendation:** Use axum 0.8 + askama 0.15 (already in Cargo.toml) + askama_web 0.15 (axum-0.8 feature) + rust-embed 8.x + tower-http 0.6 (cors, compression) + open 5.x (browser launch). Embed HTMX 2.0.10 and Alpine.js 3.x as static JS files compiled into the binary. Keep all HTML in Askama templates under `templates/ui/`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- D-01: Local web server approach. `logtok ui` starts an axum HTTP server on `127.0.0.1`, auto-opens the default browser. Interactive dashboard served from embedded assets in the single binary.
- D-02: This supersedes the PROJECT.md "Web UI or dashboard -- Out of Scope" entry. The UI is a localhost-only developer tool, not a hosted web dashboard.
- D-03: Five dashboard sections: Docs (carried from Phase 5), Tokenize panel, Detokenize panel, Token Store browser, Config editor.
- D-04: Config editor includes blacklist management (names, variables to always tokenize) and custom pattern editing. Visual form view by default with raw TOML toggle for advanced users.
- D-05: Top tab bar navigation: Tokenize | Detokenize | Token Store | Config | Docs. Default landing tab is Tokenize (primary action).
- D-06: Top nav bar includes: theme toggle button (dark/light), language selector (English/Hebrew).
- D-07: Two languages at launch: English and Hebrew.
- D-08: Full RTL layout flip when Hebrew is selected -- navigation, panels, text direction all mirror.
- D-09: Modern dev tool aesthetic (VS Code / Linear / Vercel style). Not carrying forward the Phase 5 docs palette.
- D-10: Dark theme palette: Background #0f0f10, Surface #1c1c1e, Border #2a2a2c, Primary #6366f1, Success #22c55e, Error #ef4444, Text #e4e4e7, Muted #71717a.
- D-11: Warm white light theme (#fafaf5 range). Same accent colors adapted for light backgrounds.
- D-12: Follow OS preference via `prefers-color-scheme` media query.
- D-13: Backend: axum.
- D-14: Templating: Askama.
- D-15: Asset embedding: rust-embed.
- D-16: Frontend: HTMX + Alpine.js (~30KB total, no build step).
- D-17: Middleware: tower-http for static file serving and middleware.
- D-18: Four input methods: drag-and-drop zone, file picker button, paste text area, file path input field.
- D-19: Encryption key is auto-generated per session. No manual key entry.
- D-20: Server binds to 127.0.0.1 only. Not accessible from the network.
- D-21: `logtok ui` starts server, auto-opens browser to the dashboard.
- D-22: Server auto-stops when the browser tab is closed (WebSocket heartbeat detection).
- D-23: Language preference persists across sessions (localStorage).
- D-24: Last active tab persists across sessions (localStorage).
- D-25: Recent file paths persist for quick re-selection (localStorage).
- D-26: Theme does NOT persist -- follows OS preference each time.
- D-27: Dual-mode config editor: form-based view by default, with raw TOML editor toggle.

### Claude's Discretion
- Exact CSS values (spacing, font sizes, border radius) for light theme adaptation
- Responsive breakpoints for mobile/tablet
- Warm white exact hex values within the #fafaf5 range
- WebSocket heartbeat interval for auto-stop
- Default port selection strategy (8080 or next available)
- Internal HTML structure and component organization

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

## Standard Stack

### Core (New Dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| axum | 0.8.8 | HTTP server & routing | Built on tokio (already in project). De facto Rust web framework. WebSocket support via `ws` feature. [VERIFIED: docs.rs/crate/axum/latest] |
| askama_web | 0.15.2 | Askama-to-axum IntoResponse bridge | Replaces deprecated `askama_axum`. Derive `WebTemplate` for automatic `IntoResponse` impl. Feature `axum-0.8`. [VERIFIED: docs.rs/crate/askama_web/latest] |
| rust-embed | 8.11.0 | Embed static assets into binary | Compile-time embedding of JS/CSS/images. 66 versions, mature. [VERIFIED: docs.rs/crate/rust-embed/latest] |
| tower-http | 0.6.8 | HTTP middleware (CORS, compression, static) | Standard middleware for axum. Features: `cors`, `compression-gzip`. [VERIFIED: docs.rs/crate/tower-http/latest] |
| tower | 0.5.x | Service trait (transitive, needed for middleware layers) | Required by tower-http, usually pulled transitively. [ASSUMED] |
| open | 5.3.4 | Open URL in default browser | Cross-platform browser launch. 150M+ downloads. [VERIFIED: docs.rs/crate/open/latest] |
| tokio-tungstenite | (transitive) | WebSocket protocol | Pulled in by axum's `ws` feature. [ASSUMED] |

### Frontend (Embedded as static files, not Rust crates)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| HTMX | 2.0.10 | Server-driven interactivity | ~14KB minified. Declarative AJAX via HTML attributes. Stable 2.x. [VERIFIED: htmx.org] |
| Alpine.js | 3.x (latest 3.14+) | Client-side reactivity | ~15KB minified. Tabs, theme toggle, RTL, localStorage. No build step. [VERIFIED: alpinejs.dev] |

### Already in Cargo.toml (reused)

| Library | Current Version | Role in Phase 6 |
|---------|----------------|-----------------|
| askama | 0.15 | Template engine for dashboard HTML |
| tokio | (not explicit in Cargo.toml but pulled by deps) | Async runtime for axum server |
| serde | 1.0.228 | JSON request/response serialization for API endpoints |
| serde_json | 1.0.149 | JSON parsing for API payloads |
| toml | 0.8 | Config file read/write for config editor |
| anyhow | 1.0.102 | Error handling in server code |
| tracing | 0.1.44 | Server request logging |
| tracing-subscriber | 0.3 | Log output formatting |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| askama_web | Manual `Html(template.render()?)` | Works but askama_web gives proper error handling and content-type for free |
| rust-embed | include_bytes! macros | rust-embed handles MIME types, directory traversal, debug-mode file reloading |
| open crate | webbrowser crate | `open` is simpler, more downloads; `webbrowser` guarantees browser (not editor) for local HTML files but we're opening a URL not a file |
| HTMX 2.0 | HTMX 4.0-beta2 | 4.0 uses fetch() API, but beta -- stick with stable 2.0.10 |
| axum-embed | Manual rust-embed handler | axum-embed is 0.1.0 from 2023, unmaintained. Write a thin handler instead. |

**Installation (additions to Cargo.toml):**
```toml
[dependencies]
axum = { version = "0.8", features = ["ws"] }
askama_web = { version = "0.15", features = ["axum-0.8"] }
rust-embed = "8"
tower-http = { version = "0.6", features = ["cors", "compression-gzip"] }
open = "5"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net"] }
```

**Note on tokio:** The project currently does not list tokio explicitly in Cargo.toml (it is pulled transitively). For `#[tokio::main]` and `TcpListener::bind`, tokio must be an explicit dependency with `rt-multi-thread`, `macros`, and `net` features. The `logtok ui` command will need an async entry point while other commands remain synchronous.

## Architecture Patterns

### Recommended Project Structure
```
src/
  ui/
    mod.rs          # Re-exports, server startup, browser open, shutdown
    routes.rs       # axum Router definition (all routes)
    handlers.rs     # Handler functions for each endpoint
    api.rs          # JSON API handlers (tokenize, detokenize, store, config)
    ws.rs           # WebSocket heartbeat handler for auto-stop
    assets.rs       # rust-embed struct + static file serving handler
    i18n.rs         # English/Hebrew translation strings
  cli.rs            # Add Ui variant to Commands enum
  main.rs           # Add Commands::Ui match arm
templates/
  ui/
    base.html       # Base layout (nav bar, theme, RTL, Alpine.js root)
    tokenize.html   # Tokenize tab content
    detokenize.html # Detokenize tab content  
    store.html      # Token Store browser tab
    config.html     # Config editor tab (form + TOML)
    docs.html       # Docs tab (reuse Phase 5 content)
static/
  htmx.min.js       # HTMX 2.0.10
  alpine.min.js     # Alpine.js 3.x
  styles.css         # All CSS (dark/light themes, RTL)
```

### Pattern 1: axum Server Startup with Auto-Open
**What:** Start an axum server, find an available port, open browser, wait for shutdown signal.
**When to use:** `logtok ui` command handler.
**Example:**
```rust
// Source: axum docs + open crate docs
use axum::Router;
use tokio::net::TcpListener;
use std::net::SocketAddr;

pub async fn start_server(port: Option<u16>) -> anyhow::Result<()> {
    let addr = find_available_port(port.unwrap_or(8080))?;
    let listener = TcpListener::bind(addr).await?;
    let actual_port = listener.local_addr()?.port();
    
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let app = build_router(shutdown_tx);
    
    eprintln!("logtok: dashboard at http://127.0.0.1:{}", actual_port);
    let _ = open::that(format!("http://127.0.0.1:{}", actual_port));
    
    axum::serve(listener, app)
        .with_graceful_shutdown(async { let _ = shutdown_rx.await; })
        .await?;
    Ok(())
}

fn find_available_port(preferred: u16) -> anyhow::Result<SocketAddr> {
    // Try preferred, then scan upward
    for port in preferred..preferred + 100 {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        if std::net::TcpListener::bind(addr).is_ok() {
            return Ok(addr);
        }
    }
    anyhow::bail!("No available port found in range {}..{}", preferred, preferred + 100)
}
```
[VERIFIED: axum docs.rs, open crate docs.rs]

### Pattern 2: WebSocket Heartbeat Auto-Stop
**What:** Client sends periodic pings; if server misses N heartbeats, trigger shutdown.
**When to use:** D-22 -- server auto-stops when browser tab closes.
**Example:**
```rust
// Source: axum WebSocket docs
use axum::extract::ws::{WebSocket, WebSocketUpgrade, Message};
use axum::response::Response;
use std::sync::Arc;
use tokio::sync::oneshot;

pub async fn ws_heartbeat(
    ws: WebSocketUpgrade,
    shutdown: Arc<tokio::sync::Mutex<Option<oneshot::Sender<()>>>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_heartbeat(socket, shutdown))
}

async fn handle_heartbeat(
    mut socket: WebSocket, 
    shutdown: Arc<tokio::sync::Mutex<Option<oneshot::Sender<()>>>>,
) {
    // Client sends ping every 5 seconds
    // If we don't hear for 15 seconds, shut down
    loop {
        match tokio::time::timeout(
            std::time::Duration::from_secs(15), 
            socket.recv()
        ).await {
            Ok(Some(Ok(_))) => continue,  // heartbeat received
            _ => break,                    // timeout or disconnect
        }
    }
    // Trigger graceful shutdown
    if let Some(tx) = shutdown.lock().await.take() {
        let _ = tx.send(());
    }
}
```
**Client-side (Alpine.js):**
```javascript
// In base.html template
<div x-data x-init="
  let ws = new WebSocket('ws://' + location.host + '/ws/heartbeat');
  setInterval(() => { if (ws.readyState === 1) ws.send('ping'); }, 5000);
  ws.onclose = () => { /* reconnect or show 'server stopped' */ };
">
```
[VERIFIED: axum::extract::ws docs]

### Pattern 3: Embedded Static Assets
**What:** Use `rust-embed` to compile JS/CSS into the binary, serve via axum handler.
**When to use:** Serving htmx.min.js, alpine.min.js, styles.css.
**Example:**
```rust
// Source: rust-embed docs + axum routing
use rust_embed::Embed;
use axum::response::{IntoResponse, Response};
use axum::http::{header, StatusCode};

#[derive(Embed)]
#[folder = "static/"]
struct StaticAssets;

async fn static_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response {
    match StaticAssets::get(&path) {
        Some(file) => {
            let mime = mime_guess::from_path(&path)
                .first_or_octet_stream()
                .to_string();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime)],
                file.data.to_vec(),
            ).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
```
**Note:** `rust-embed` includes `mime_guess` transitively. In debug mode, rust-embed reads files from disk (hot reload during development). In release mode, files are embedded in the binary. [VERIFIED: rust-embed docs]

### Pattern 4: Askama + axum Template Rendering
**What:** Render HTML templates as axum responses using askama_web.
**When to use:** All page-rendering endpoints.
**Example:**
```rust
// Source: askama_web docs + askama axum example
use askama::Template;

#[derive(Template)]
#[template(path = "ui/base.html")]
struct DashboardTemplate {
    lang: String,      // "en" or "he"
    active_tab: String, // "tokenize", "detokenize", etc.
    version: String,
}

async fn dashboard() -> Result<impl axum::response::IntoResponse, AppError> {
    let template = DashboardTemplate {
        lang: "en".into(),
        active_tab: "tokenize".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    };
    Ok(axum::response::Html(template.render()?))
}
```
[VERIFIED: askama_web 0.15.2 docs, askama axum-app example]

### Pattern 5: RTL Layout with CSS + Alpine.js
**What:** Full RTL flip controlled by Alpine.js reactive state, CSS logical properties.
**When to use:** D-07/D-08 -- Hebrew support.
**Example:**
```html
<!-- base.html: Alpine.js controls dir attribute -->
<html :lang="lang" :dir="lang === 'he' ? 'rtl' : 'ltr'" x-data="app()">

<style>
  /* Use CSS logical properties for automatic RTL */
  .nav-tabs { display: flex; gap: 0; }
  .panel { padding-inline: 1.5rem; margin-inline-start: 0; }
  .input-group { text-align: start; }
  
  /* RTL-specific overrides only when needed */
  [dir="rtl"] .icon-before { transform: scaleX(-1); }
</style>
```
**Key insight:** CSS logical properties (`padding-inline`, `margin-inline-start`, `border-inline-end`) handle most RTL automatically. Only a few elements (directional icons, specific layouts) need `[dir="rtl"]` overrides. [ASSUMED]

### Pattern 6: HTMX API Integration for Tokenize Panel
**What:** HTMX sends form data to server, receives tokenized HTML fragment.
**When to use:** D-18 -- file tokenization via UI.
**Example:**
```html
<!-- tokenize.html fragment -->
<form hx-post="/api/tokenize" 
      hx-target="#result" 
      hx-encoding="multipart/form-data"
      hx-indicator="#spinner">
  <!-- File path input (large files -- server reads directly) -->
  <input type="text" name="file_path" placeholder="Path to log file...">
  
  <!-- OR paste text area -->
  <textarea name="content" rows="10" placeholder="Paste log content..."></textarea>
  
  <!-- OR file picker -->
  <input type="file" name="file">
  
  <button type="submit">Tokenize</button>
</form>
<div id="spinner" class="htmx-indicator">Processing...</div>
<div id="result"></div>
```
**Server handler returns HTML fragment (not JSON) for HTMX swap:**
```rust
async fn api_tokenize(/* multipart form */) -> impl IntoResponse {
    // Process file, return HTML fragment with tokenized output
    Html(format!("<pre class='output'>{}</pre>", tokenized_content))
}
```
[VERIFIED: htmx.org/examples/file-upload]

### Pattern 7: Auto-Generated Session Encryption Key
**What:** Generate a random encryption key per UI session so the user never enters one manually.
**When to use:** D-19 -- no manual key entry.
**Example:**
```rust
use rand::RngCore;

fn generate_session_key() -> String {
    let mut key_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut key_bytes);
    // Use hex encoding for a clean passphrase
    hex::encode(key_bytes)
}

// When starting the UI server:
let session_key = generate_session_key();
std::env::set_var("LOGTOK_KEY", &session_key);
// Now Store::new() will pick up LOGTOK_KEY from env
```
**Note:** This means token stores created via `logtok ui` are ephemeral -- they cannot be decrypted after the session ends unless the key is persisted. This is intentional for the UI use case. The `hex` crate (or manual hex encoding) may be needed for clean key formatting. [ASSUMED]

### Anti-Patterns to Avoid
- **Serving templates from disk in release:** All assets MUST be embedded via rust-embed for single-binary distribution. Never use `ServeDir` pointing to filesystem paths.
- **Blocking the tokio runtime:** The existing `process_file_with_config` is synchronous and does heavy CPU work. Wrap in `tokio::task::spawn_blocking()` to avoid blocking the async runtime.
- **Global mutable state for session key:** Use axum's `State` extractor with `Arc` to share server state. Do not use global statics or `once_cell` for runtime-generated values.
- **HTMX returning JSON:** HTMX expects HTML fragments by default. Return `Html(...)` from API endpoints consumed by HTMX, not `Json(...)`. Only return JSON for Alpine.js fetch calls or non-HTMX consumers.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Static asset embedding | Custom include_bytes! macros | rust-embed | Handles MIME types, debug hot-reload, directory traversal |
| Browser launch | Platform-specific `Command::new("xdg-open")` | open 5.x | Cross-platform (Windows, macOS, Linux), handles edge cases |
| WebSocket protocol | Raw TCP upgrade handling | axum `ws` feature | Handles upgrade, framing, ping/pong, close codes |
| CORS headers | Manual header injection | tower-http `CorsLayer` | Handles preflight, allowed origins, methods correctly |
| RTL layout | Manual CSS for every element | CSS logical properties | Browser handles RTL natively with `dir="rtl"` + logical properties |
| Reactive UI state | Custom JS state management | Alpine.js | 15KB, declarative, handles localStorage persist, no build step |
| Server-driven updates | Custom fetch/XHR wrappers | HTMX 2.0 | 14KB, declarative HTML attributes, handles indicators/swaps |
| Template rendering | String concatenation or format! | Askama | Compile-time checked, auto-escaping, template inheritance |
| Port scanning | Manual socket binding loop | (keep simple -- 10 lines) | Port scanning is genuinely simple enough; see Pattern 1 above |

**Key insight:** The entire frontend is ~30KB of JS (HTMX + Alpine.js) with no build step, no npm, no bundler. This is essential for the single-binary philosophy -- JS files are embedded via rust-embed at compile time.

## Common Pitfalls

### Pitfall 1: Blocking the Async Runtime with Synchronous Processing
**What goes wrong:** `process_file_with_config()` is synchronous and CPU-intensive. Calling it directly in an axum handler blocks the tokio runtime, stalling all other connections (including the WebSocket heartbeat).
**Why it happens:** The existing codebase was designed for CLI (synchronous main). axum runs on tokio.
**How to avoid:** Wrap all CPU-intensive work in `tokio::task::spawn_blocking()`:
```rust
let result = tokio::task::spawn_blocking(move || {
    process_file_with_config(&path, Some(&out), block_size, true, false, &config, store.as_ref(), ttl)
}).await??;
```
**Warning signs:** WebSocket heartbeat timeouts during large file processing.

### Pitfall 2: Askama Template Path Resolution
**What goes wrong:** Askama looks for templates in `$CARGO_MANIFEST_DIR/templates/` by default. If templates are in a subdirectory, the `path` attribute must include it.
**Why it happens:** Askama resolves paths at compile time relative to the templates root.
**How to avoid:** Use `#[template(path = "ui/base.html")]` not `#[template(path = "base.html")]` if files are in `templates/ui/`.
**Warning signs:** Compile errors about missing template files.

### Pitfall 3: rust-embed Debug vs Release Behavior
**What goes wrong:** In debug mode, rust-embed reads files from disk (great for development). In release mode, files are embedded in the binary. If you add/change static files, debug works but release has stale content.
**Why it happens:** Different embedding strategies per profile.
**How to avoid:** Always `cargo build --release` for distribution testing. During development, the disk-read behavior is actually helpful for CSS/JS iteration.
**Warning signs:** Static files work in debug but are missing or stale in release builds.

### Pitfall 4: HTMX + Alpine.js Attribute Conflicts
**What goes wrong:** Both HTMX and Alpine.js use HTML attributes for behavior. They can conflict if an element has both `hx-*` and `x-*` attributes that try to control the same DOM.
**Why it happens:** HTMX intercepts form submissions and link clicks; Alpine.js manages reactive state.
**How to avoid:** Clear separation: HTMX handles server communication (forms, data loading). Alpine.js handles client-only state (tabs, theme, language, localStorage). Never put `hx-trigger` and `x-on:click` on the same element for the same action.
**Warning signs:** Double-firing events, unexpected DOM updates.

### Pitfall 5: Tokio Runtime for CLI + Server Hybrid
**What goes wrong:** The current `main()` is `fn main()` (synchronous). Adding `#[tokio::main]` for the `ui` command changes the function signature. Other commands that don't need async may behave unexpectedly.
**Why it happens:** Mixing sync and async entry points.
**How to avoid:** Use `#[tokio::main]` on `main()` and keep all existing synchronous command handlers as-is -- they work fine inside an async context. OR use `tokio::runtime::Runtime::new()` only for the `ui` command path:
```rust
Commands::Ui { port } => {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(ui::start_server(port))?;
}
```
The second approach is cleaner because it keeps the existing sync main unchanged and only creates the async runtime when needed.
**Warning signs:** Compile errors about async in non-async context, or "cannot start a runtime from within a runtime" panic.

### Pitfall 6: Session Key Scope
**What goes wrong:** The auto-generated encryption key (D-19) is per-session. If the user tokenizes via UI, then tries `logtok detokenize` from CLI, the keys don't match.
**Why it happens:** UI generates its own key; CLI reads `LOGTOK_KEY` env var.
**How to avoid:** Document this clearly in the UI. Consider printing the session key to stderr on server start so users can set `LOGTOK_KEY` if they want CLI interop. Or: if `LOGTOK_KEY` is already set in env, use that instead of generating a new one.
**Warning signs:** "Decryption failed" errors when mixing UI and CLI workflows.

### Pitfall 7: File Path Security
**What goes wrong:** The file path input (D-18) lets users type an arbitrary path that the server reads. Even on localhost, this is a local file read endpoint.
**Why it happens:** Server reads files directly for large file performance.
**How to avoid:** Since the server binds to 127.0.0.1 only (D-20), the attack surface is limited to the local user. Still, validate paths are regular files (not directories, devices, or symlinks to sensitive files). Log all file access via tracing.
**Warning signs:** Path traversal attempts, reading /etc/shadow or similar.

## Code Examples

### axum Router Setup (complete)
```rust
// Source: axum docs, pattern composition
use axum::{Router, routing::{get, post, put}};
use tower_http::compression::CompressionLayer;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};

struct AppState {
    shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
    session_key: String,
    config: crate::config::LoktokConfig,
}

fn build_router(shutdown_tx: oneshot::Sender<()>) -> Router {
    let state = Arc::new(AppState {
        shutdown_tx: Mutex::new(Some(shutdown_tx)),
        session_key: generate_session_key(),
        config: load_or_default_config(),
    });

    Router::new()
        // Pages
        .route("/", get(dashboard))
        // API endpoints (return HTML fragments for HTMX)
        .route("/api/tokenize", post(api_tokenize))
        .route("/api/detokenize", post(api_detokenize))
        .route("/api/store", get(api_store))
        .route("/api/config", get(api_config_get))
        .route("/api/config", put(api_config_put))
        // WebSocket
        .route("/ws/heartbeat", get(ws_heartbeat))
        // Static assets
        .route("/static/{*path}", get(static_handler))
        .layer(CompressionLayer::new())
        .with_state(state)
}
```
[VERIFIED: axum routing docs]

### Alpine.js App Skeleton (client-side state)
```javascript
// Source: Alpine.js docs
function app() {
  return {
    tab: localStorage.getItem('logtok-tab') || 'tokenize',
    lang: localStorage.getItem('logtok-lang') || 'en',
    // Theme follows OS -- no localStorage
    theme: window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light',
    recentFiles: JSON.parse(localStorage.getItem('logtok-recent') || '[]'),
    
    setTab(t) {
      this.tab = t;
      localStorage.setItem('logtok-tab', t);
    },
    setLang(l) {
      this.lang = l;
      localStorage.setItem('logtok-lang', l);
      document.documentElement.dir = l === 'he' ? 'rtl' : 'ltr';
      document.documentElement.lang = l;
    },
    toggleTheme() {
      this.theme = this.theme === 'dark' ? 'light' : 'dark';
      // Note: D-26 says theme doesn't persist, so no localStorage
    },
    addRecentFile(path) {
      this.recentFiles = [path, ...this.recentFiles.filter(f => f !== path)].slice(0, 10);
      localStorage.setItem('logtok-recent', JSON.stringify(this.recentFiles));
    },
    
    init() {
      // Watch OS theme changes
      window.matchMedia('(prefers-color-scheme: dark)')
        .addEventListener('change', e => { this.theme = e.matches ? 'dark' : 'light'; });
      // Apply stored language direction
      if (this.lang === 'he') {
        document.documentElement.dir = 'rtl';
        document.documentElement.lang = 'he';
      }
    }
  }
}
```
[VERIFIED: Alpine.js docs for x-data patterns]

### i18n Pattern (simple object-based)
```rust
// src/ui/i18n.rs
use std::collections::HashMap;

pub fn translations(lang: &str) -> HashMap<&'static str, &'static str> {
    match lang {
        "he" => HashMap::from([
            ("tab.tokenize", "\u{05d8}\u{05d5}\u{05e7}\u{05e0}\u{05d9}\u{05d6}\u{05e6}\u{05d9}\u{05d4}"),
            ("tab.detokenize", "\u{05e9}\u{05d7}\u{05d6}\u{05d5}\u{05e8}"),
            ("tab.store", "\u{05de}\u{05d0}\u{05d2}\u{05e8} \u{05d8}\u{05d5}\u{05e7}\u{05e0}\u{05d9}\u{05dd}"),
            ("tab.config", "\u{05d4}\u{05d2}\u{05d3}\u{05e8}\u{05d5}\u{05ea}"),
            ("tab.docs", "\u{05ea}\u{05d9}\u{05e2}\u{05d5}\u{05d3}"),
            // ... more keys
        ]),
        _ => HashMap::from([
            ("tab.tokenize", "Tokenize"),
            ("tab.detokenize", "Detokenize"),
            ("tab.store", "Token Store"),
            ("tab.config", "Config"),
            ("tab.docs", "Docs"),
            // ... more keys
        ]),
    }
}
```
**Note:** For a two-language app, a simple HashMap approach is sufficient. No need for i18n frameworks like `fluent` or `i18n-embed`. Hebrew strings should be reviewed by a Hebrew speaker before release. [ASSUMED]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| askama_axum crate | askama_web 0.15+ with feature flags | askama 0.13+ | Must use askama_web, not askama_axum (deprecated) |
| axum 0.7 routing | axum 0.8 routing (minor API changes) | Late 2025 | Route syntax unchanged, some extractor changes |
| HTMX 1.x | HTMX 2.0 (stable) | June 2024 | Breaking: removed some deprecated attributes, IE support dropped |
| Alpine.js 2.x | Alpine.js 3.x | 2021 | Breaking: new directive syntax, x-data changes |

**Deprecated/outdated:**
- `askama_axum` crate: deprecated as of askama 0.13, replaced by `askama_web`
- HTMX 1.x: replaced by 2.0, significant improvements
- `axum-embed` 0.1.0: last updated Dec 2023, do not use -- write a thin handler instead

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | tower 0.5.x pulled transitively by axum | Standard Stack | LOW -- may need explicit dep if not |
| A2 | tokio-tungstenite pulled by axum ws feature | Standard Stack | LOW -- axum docs confirm ws feature includes it |
| A3 | CSS logical properties handle most RTL layout automatically | Architecture Patterns | MEDIUM -- some elements may need manual RTL fixes |
| A4 | Simple HashMap-based i18n is sufficient for 2 languages | Code Examples | LOW -- works for small scale, would need framework for 5+ languages |
| A5 | hex encoding for session key format | Architecture Patterns | LOW -- any string encoding works for passphrase |
| A6 | Session key ephemeral by design (UI-only workflow) | Common Pitfalls | MEDIUM -- user may expect CLI interop, need clear UX messaging |

## Open Questions

1. **Session key and CLI interop**
   - What we know: D-19 says auto-generated key, no manual entry. Existing CLI uses LOGTOK_KEY env var.
   - What's unclear: Should UI session be compatible with subsequent CLI `detokenize`? If yes, key needs to be communicated to user.
   - Recommendation: If LOGTOK_KEY is already set in environment, use it. Otherwise generate one and print it to stderr. This gives users a path to CLI interop without requiring manual entry.

2. **File upload size limits**
   - What we know: D-18 includes file picker and drag-drop (browser upload) plus file path input (server reads directly).
   - What's unclear: What's the max upload size for browser-based file upload? The file path method bypasses this.
   - Recommendation: Set a reasonable multipart limit (e.g., 50MB) for browser upload. For larger files, the file path input is the recommended method. Document this in the UI.

3. **Docs tab content source**
   - What we know: D-03 says "Docs (carried from Phase 5)". Phase 5 generates a standalone HTML file.
   - What's unclear: Should the Docs tab re-render the askama docs template inline, or embed the Phase 5 HTML output?
   - Recommendation: Re-render using the same `CommandInfo`/`TokenCategory` data from `docs.rs` but in a new template that fits the dashboard layout (no sidebar, no standalone styling). Reuse the data extraction, not the HTML.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust (stable) | Build | Yes | 1.94.1 | -- |
| cargo | Build | Yes | 1.94.1 | -- |
| Web browser | Auto-open (D-21) | Yes | (system) | Print URL to stderr, user opens manually |

**Missing dependencies with no fallback:** None.
**Missing dependencies with fallback:** None.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | Localhost-only, single user, no auth needed (D-20) |
| V3 Session Management | No | No user sessions -- single localhost user |
| V4 Access Control | Partial | Bind to 127.0.0.1 only (D-20), validate file paths |
| V5 Input Validation | Yes | Validate file paths (no traversal), sanitize TOML input, limit upload size |
| V6 Cryptography | Yes | AES-256-GCM for token store (existing), auto-generated key (D-19) |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| File path traversal via path input | Tampering | Canonicalize paths, verify regular file, reject symlinks to sensitive files |
| XSS in tokenized output display | Tampering | Askama auto-escapes by default; use `|escape` filter for dynamic content |
| TOML injection in config editor | Tampering | Parse TOML server-side before writing; reject invalid TOML |
| Port hijacking on localhost | Spoofing | Low risk on single-user machine; log port and PID on startup |

## Project Constraints (from CLAUDE.md)

- **Single binary:** All assets must be embedded (rust-embed). No runtime file dependencies.
- **Performance:** Large file tokenization must not block the async runtime (spawn_blocking).
- **Security:** Token mappings never transmitted. Server binds 127.0.0.1 only. Encrypted at rest.
- **Portability:** Cross-platform (Windows, macOS, Linux). `open` crate handles browser launch cross-platform.
- **Stack:** Rust stable 1.85+, clap 4.6, tokio, serde, askama 0.15, aes-gcm, argon2. Phase 6 adds axum, rust-embed, tower-http, open.
- **No build step for frontend:** HTMX + Alpine.js are static JS files, not npm packages.

## Sources

### Primary (HIGH confidence)
- [axum 0.8.8 docs](https://docs.rs/axum/latest/axum/) - Router, WebSocket, State patterns
- [axum WebSocket module](https://docs.rs/axum/latest/axum/extract/ws/index.html) - WebSocketUpgrade, Message types
- [askama_web 0.15.2 docs](https://docs.rs/crate/askama_web/latest) - Version verified, axum-0.8 feature
- [rust-embed 8.11.0 docs](https://docs.rs/crate/rust-embed/latest) - Version verified
- [tower-http 0.6.8 docs](https://docs.rs/crate/tower-http/latest) - Version verified
- [open 5.3.4 docs](https://docs.rs/crate/open/latest) - Version verified
- [HTMX 2.0 docs](https://htmx.org/docs/) - File upload examples, attribute reference
- [Alpine.js docs](https://alpinejs.dev/) - x-data, localStorage patterns
- [askama axum-app example](https://github.com/askama-rs/askama/tree/main/examples/axum-app) - Template rendering pattern

### Secondary (MEDIUM confidence)
- [axum static-file-server example](https://github.com/tokio-rs/axum/blob/main/examples/static-file-server/src/main.rs) - Static file serving patterns
- [axum WebSocket example](https://github.com/tokio-rs/axum/blob/main/examples/websockets/src/main.rs) - WebSocket handler patterns

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all crates verified on docs.rs/crates.io with current versions
- Architecture: HIGH - patterns verified against official docs and examples
- Pitfalls: HIGH - based on verified API constraints (sync vs async, template paths, embed behavior)

**Research date:** 2026-04-29
**Valid until:** 2026-05-29 (stable ecosystem, no major releases expected)
