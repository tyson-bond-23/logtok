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
        Ok(Ok((output, stats))) => {
            // After successful tokenization with file_path, send HX-Trigger for recent files (D-25)
            if is_file_path {
                if let Some(path) = path_for_trigger {
                    let trigger = format!(r#"{{"addRecentFile":"{}"}}"#, path.replace('"', r#"\""#));
                    if let Ok(val) = trigger.parse() {
                        headers.insert("HX-Trigger", val);
                    }
                }
            }
            let html = format!(
                "<div class='result-success'>\
                   <div class='result-stats'>{}</div>\
                   <div class='result-actions'>\
                     <button onclick='navigator.clipboard.writeText(this.closest(\".result-success\").querySelector(\"pre\").textContent)' class='btn-copy'>Copy</button>\
                   </div>\
                   <pre class='result-output'>{}</pre>\
                 </div>",
                html_escape(&stats),
                html_escape(&output)
            );
            (headers, Html(html)).into_response()
        }
        Ok(Err(e)) => {
            let html = format!(
                "<div class='result-error'><p>{}</p></div>",
                html_escape(&e)
            );
            (headers, Html(html)).into_response()
        }
        Err(e) => {
            let html = format!(
                "<div class='result-error'><p>Processing failed: {}</p></div>",
                html_escape(&e.to_string())
            );
            (headers, Html(html)).into_response()
        }
    }
}

/// Synchronous tokenization logic, runs inside spawn_blocking.
fn tokenize_input(
    file_path: Option<String>,
    content: Option<String>,
    file_bytes: Option<Vec<u8>>,
    session_key: &str,
) -> Result<(String, String), String> {
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

    // Create temp output file
    let temp_out = tempfile::NamedTempFile::new()
        .map_err(|e| format!("Cannot create temp file: {}", e))?;
    let temp_out_path = temp_out.path().to_path_buf();

    // Determine input source and create input file
    let (input_path, _temp_in) = if let Some(ref fp) = file_path {
        let p = PathBuf::from(fp);
        // T-06-02: Validate path is a regular file
        let meta = std::fs::metadata(&p)
            .map_err(|e| format!("Cannot access file '{}': {}", fp, e))?;
        if !meta.is_file() {
            return Err(format!("Not a regular file: {}", fp));
        }
        (p, None)
    } else if let Some(ref text) = content {
        // Write pasted content to temp file
        let temp_in = tempfile::NamedTempFile::new()
            .map_err(|e| format!("Cannot create temp file: {}", e))?;
        std::fs::write(temp_in.path(), text)
            .map_err(|e| format!("Cannot write temp file: {}", e))?;
        let p = temp_in.path().to_path_buf();
        (p, Some(temp_in))
    } else if let Some(ref bytes) = file_bytes {
        // Write uploaded bytes to temp file
        let temp_in = tempfile::NamedTempFile::new()
            .map_err(|e| format!("Cannot create temp file: {}", e))?;
        std::fs::write(temp_in.path(), bytes)
            .map_err(|e| format!("Cannot write temp file: {}", e))?;
        let p = temp_in.path().to_path_buf();
        (p, Some(temp_in))
    } else {
        return Err(
            "No input provided. Enter a file path, paste content, or upload a file.".to_string(),
        );
    };

    // Process via the core tokenization engine
    crate::processor::process_file_with_config(
        &input_path,
        Some(&temp_out_path),
        65536, // default block size
        true,  // quiet (no progress bar in server context)
        false, // not dry run
        &detection_config,
        Some(&store),
        cfg.ttl_seconds(),
    )
    .map_err(|e| format!("Tokenization failed: {}", e))?;

    let output = std::fs::read_to_string(&temp_out_path)
        .map_err(|e| format!("Cannot read output: {}", e))?;

    // Count tokens found in output for stats
    let token_count = output.matches('[').count(); // rough estimate
    let line_count = output.lines().count();
    let stats = format!("Tokenized {} lines ({} tokens detected)", line_count, token_count);

    Ok((output, stats))
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
                "<div class='result-error'><p>File is empty or not valid text.</p></div>".to_string(),
            ),
        }
    } else {
        return Html(
            "<div class='result-error'><p>No content provided. Paste tokenized text or upload a file.</p></div>"
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
            "<div class='result-success'>\
               <div class='result-stats'>{} tokens replaced, {} unresolved</div>\
               <div class='result-actions'>\
                 <button onclick='navigator.clipboard.writeText(this.closest(\".result-success\").querySelector(\"pre\").textContent)' class='btn-copy'>Copy</button>\
               </div>\
               <pre class='result-output'>{}</pre>\
             </div>",
            dr.replaced_count,
            dr.unresolved_count,
            html_escape(&dr.text)
        )),
        Ok(Err(e)) => Html(format!(
            "<div class='result-error'><p>{}</p></div>",
            html_escape(&e)
        )),
        Err(e) => Html(format!(
            "<div class='result-error'><p>Processing failed: {}</p></div>",
            html_escape(&e.to_string())
        )),
    }
}
