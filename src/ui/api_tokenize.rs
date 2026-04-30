use axum::extract::{Multipart, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse};
use std::path::PathBuf;
use std::sync::Arc;

use crate::config;
use crate::store::Store;
use crate::ui::AppState;

/// HTML-escape a string for safe rendering in <pre> blocks (T-06-06).
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// POST /api/tokenize -- accepts multipart form data with three possible input sources:
/// 1. `file_path` field (string) -- server reads file directly (best for large files)
/// 2. `content` field (string) -- pasted text content
/// 3. `file` field (uploaded file bytes) -- file picker or drag-and-drop
pub async fn api_tokenize(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut file_path: Option<String> = None;
    let mut content: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().map(|n| n.to_string()).unwrap_or_default();
        match name.as_str() {
            "file_path" => {
                if let Ok(text) = field.text().await {
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() {
                        file_path = Some(trimmed);
                    }
                }
            }
            "content" => {
                if let Ok(text) = field.text().await {
                    let t: String = text;
                    if !t.trim().is_empty() {
                        content = Some(t);
                    }
                }
            }
            "file" => {
                if let Ok(bytes) = field.bytes().await {
                    let b: axum::body::Bytes = bytes;
                    if !b.is_empty() {
                        file_bytes = Some(b.to_vec());
                    }
                }
            }
            _ => {}
        }
    }

    // Track whether this was a file_path request for the HX-Trigger header
    let is_file_path = file_path.is_some();
    let path_for_trigger = file_path.clone();

    let session_key = state.session_key.clone();
    let result = tokio::task::spawn_blocking(move || {
        tokenize_input(file_path, content, file_bytes, &session_key)
    })
    .await;

    let mut headers = HeaderMap::new();

    match result {
        Ok(Ok((output, stats, saved_path))) => {
            // After successful tokenization with file_path, send HX-Trigger for recent files (D-25)
            if is_file_path {
                if let Some(path) = path_for_trigger {
                    let trigger = format!(r#"{{"addRecentFile":"{}"}}"#, path.replace('"', r#"\""#));
                    if let Ok(val) = trigger.parse() {
                        headers.insert("HX-Trigger", val);
                    }
                }
            }
            let saved_badge = if let Some(ref sp) = saved_path {
                format!(
                    "<div class='flex items-center gap-2 mt-3 px-3 py-2 rounded-lg bg-brand-500/10 border border-brand-500/20'>\
                       <svg width='14' height='14' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='text-brand-400 shrink-0'>\
                         <path d='M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z'/>\
                         <polyline points='17 21 17 13 7 13 7 21'/><polyline points='7 3 7 8 15 8'/>\
                       </svg>\
                       <span class='text-xs font-mono text-brand-400'>Saved to: {}</span>\
                     </div>",
                    html_escape(sp)
                )
            } else {
                String::new()
            };
            let html = format!(
                "<div class='rounded-2xl border border-emerald-500/20 bg-emerald-500/[0.04] p-4 mt-4'>\
                   <div class='flex items-center justify-between mb-3'>\
                     <span class='text-xs font-medium text-emerald-400'>{}</span>\
                     <button onclick='navigator.clipboard.writeText(this.closest(\"div\").querySelector(\"pre\").textContent);this.textContent=\"Copied!\";setTimeout(()=>this.textContent=\"Copy\",2000)' \
                             class='px-3 py-1 text-xs font-medium rounded-md border border-zinc-700 text-zinc-400 hover:text-zinc-200 hover:border-zinc-600 transition-colors'>Copy</button>\
                   </div>\
                   <pre class='text-sm font-mono leading-relaxed text-zinc-200 bg-zinc-900/60 rounded-xl p-4 overflow-x-auto whitespace-pre-wrap break-words'>{}</pre>\
                   {}\
                 </div>",
                html_escape(&stats),
                html_escape(&output),
                saved_badge
            );
            (headers, Html(html)).into_response()
        }
        Ok(Err(e)) => {
            let html = format!(
                "<div class='mt-4 p-3 rounded-lg border border-red-500/30 bg-red-500/10 text-red-400 text-sm'>{}</div>",
                html_escape(&e)
            );
            (headers, Html(html)).into_response()
        }
        Err(e) => {
            let html = format!(
                "<div class='mt-4 p-3 rounded-lg border border-red-500/30 bg-red-500/10 text-red-400 text-sm'>Processing failed: {}</div>",
                html_escape(&e.to_string())
            );
            (headers, Html(html)).into_response()
        }
    }
}

/// Build the _TOKENIZED output path from an input path.
/// e.g., "app.log" -> "app_TOKENIZED.log", "data.txt" -> "data_TOKENIZED.txt"
fn tokenized_output_path(input: &std::path::Path) -> PathBuf {
    let stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());
    let ext = input
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let new_name = format!("{}_TOKENIZED{}", stem, ext);
    input
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join(new_name)
}

/// Synchronous tokenization logic, runs inside spawn_blocking.
/// Returns (output_text, stats_message, Option<saved_file_path>).
fn tokenize_input(
    file_path: Option<String>,
    content: Option<String>,
    file_bytes: Option<Vec<u8>>,
    session_key: &str,
) -> Result<(String, String, Option<String>), String> {
    // Load config
    let cfg = if let Some(found) = config::find_config() {
        config::load_config(&found).unwrap_or_default()
    } else {
        config::LoktokConfig::default()
    };
    let detection_config = cfg.to_detection_config();

    // Create store with session key (D-19)
    let store_dir = std::env::current_dir()
        .map_err(|e| format!("Cannot determine CWD: {}", e))?
        .join(".loktok");
    let store = Store::with_passphrase(&store_dir, session_key.to_string())
        .map_err(|e| format!("Store error: {}", e))?;

    // Determine input source and create input file
    let (input_path, _temp_in, source_name) = if let Some(ref fp) = file_path {
        let p = PathBuf::from(fp);
        // T-06-02: Validate path is a regular file
        let meta = std::fs::metadata(&p)
            .map_err(|e| format!("Cannot access file '{}': {}", fp, e))?;
        if !meta.is_file() {
            return Err(format!("Not a regular file: {}", fp));
        }
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());
        (p, None, Some(name))
    } else if let Some(ref text) = content {
        // Write pasted content to temp file
        let temp_in = tempfile::NamedTempFile::new()
            .map_err(|e| format!("Cannot create temp file: {}", e))?;
        std::fs::write(temp_in.path(), text)
            .map_err(|e| format!("Cannot write temp file: {}", e))?;
        let p = temp_in.path().to_path_buf();
        (p, Some(temp_in), None)
    } else if let Some(ref bytes) = file_bytes {
        // Write uploaded bytes to temp file
        let temp_in = tempfile::NamedTempFile::new()
            .map_err(|e| format!("Cannot create temp file: {}", e))?;
        std::fs::write(temp_in.path(), bytes)
            .map_err(|e| format!("Cannot write temp file: {}", e))?;
        let p = temp_in.path().to_path_buf();
        (p, Some(temp_in), None)
    } else {
        return Err(
            "No input provided. Enter a file path, paste content, or upload a file.".to_string(),
        );
    };

    // Compute the _TOKENIZED output path
    let save_path = tokenized_output_path(&input_path);

    // Process via the core tokenization engine — write directly to _TOKENIZED file
    crate::processor::process_file_with_config(
        &input_path,
        Some(&save_path),
        65536, // default block size
        true,  // quiet (no progress bar in server context)
        false, // not dry run
        &detection_config,
        Some(&store),
        cfg.ttl_seconds(),
    )
    .map_err(|e| format!("Tokenization failed: {}", e))?;

    let output = std::fs::read_to_string(&save_path)
        .map_err(|e| format!("Cannot read output: {}", e))?;

    // Count tokens found in output for stats
    let token_count = output.matches('[').count(); // rough estimate
    let line_count = output.lines().count();

    // Build stats with saved file info
    let saved_display = save_path.to_string_lossy().to_string();
    let stats = if source_name.is_some() {
        format!(
            "Tokenized {} lines ({} tokens detected) — saved to {}",
            line_count, token_count, saved_display
        )
    } else {
        format!(
            "Tokenized {} lines ({} tokens detected)",
            line_count, token_count
        )
    };

    let saved = if source_name.is_some() {
        Some(saved_display)
    } else {
        None
    };

    Ok((output, stats, saved))
}

/// POST /api/detokenize -- accepts multipart form data with `content` field or file upload.
/// Restores real values from the session token store.
pub async fn api_detokenize(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Html<String> {
    let mut content: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().map(|n| n.to_string()).unwrap_or_default();
        match name.as_str() {
            "content" => {
                if let Ok(t) = field.text().await {
                    if !t.trim().is_empty() {
                        content = Some(t);
                    }
                }
            }
            "file" => {
                if let Ok(bytes) = field.bytes().await {
                    if !bytes.is_empty() {
                        file_bytes = Some(bytes.to_vec());
                    }
                }
            }
            _ => {}
        }
    }

    // Use content first, then file upload
    let text = if let Some(c) = content {
        c
    } else if let Some(bytes) = file_bytes {
        match String::from_utf8(bytes) {
            Ok(s) if !s.trim().is_empty() => s,
            _ => return Html(
                "<div class='mt-4 p-3 rounded-lg border border-red-500/30 bg-red-500/10 text-red-400 text-sm'>File is empty or not valid text.</div>".to_string(),
            ),
        }
    } else {
        return Html(
            "<div class='mt-4 p-3 rounded-lg border border-red-500/30 bg-red-500/10 text-red-400 text-sm'>No content provided. Paste tokenized text or upload a file.</div>"
                .to_string(),
        );
    };

    let session_key = state.session_key.clone();
    let result = tokio::task::spawn_blocking(move || {
        let store_dir = std::env::current_dir()
            .map_err(|e| format!("Cannot determine CWD: {}", e))?
            .join(".loktok");
        let store = Store::with_passphrase(&store_dir, session_key)
            .map_err(|e| format!("Store error: {}", e))?;
        let data = store
            .load()
            .map_err(|e| format!("Cannot load token store: {}. Tokenize some content first.", e))?;
        let result = crate::detokenizer::detokenize(&text, &data.token_to_value);
        Ok::<_, String>(result)
    })
    .await;

    match result {
        Ok(Ok(dr)) => Html(format!(
            "<div class='rounded-2xl border border-emerald-500/20 bg-emerald-500/[0.04] p-4 mt-4'>\
               <div class='flex items-center justify-between mb-3'>\
                 <span class='text-xs font-medium text-emerald-400'>{} tokens replaced, {} unresolved</span>\
                 <button onclick='navigator.clipboard.writeText(this.closest(\"div\").querySelector(\"pre\").textContent);this.textContent=\"Copied!\";setTimeout(()=>this.textContent=\"Copy\",2000)' \
                         class='px-3 py-1 text-xs font-medium rounded-md border border-zinc-700 text-zinc-400 hover:text-zinc-200 hover:border-zinc-600 transition-colors'>Copy</button>\
               </div>\
               <pre class='text-sm font-mono leading-relaxed text-zinc-200 bg-zinc-900/60 rounded-xl p-4 overflow-x-auto whitespace-pre-wrap break-words'>{}</pre>\
             </div>",
            dr.replaced_count,
            dr.unresolved_count,
            html_escape(&dr.text)
        )),
        Ok(Err(e)) => Html(format!(
            "<div class='mt-4 p-3 rounded-lg border border-red-500/30 bg-red-500/10 text-red-400 text-sm'>{}</div>",
            html_escape(&e)
        )),
        Err(e) => Html(format!(
            "<div class='mt-4 p-3 rounded-lg border border-red-500/30 bg-red-500/10 text-red-400 text-sm'>Processing failed: {}</div>",
            html_escape(&e.to_string())
        )),
    }
}
