use axum::extract::State;
use axum::response::Html;
use std::sync::Arc;

use crate::docs::{extract_commands, get_token_categories};
use crate::store::Store;
use crate::ui::AppState;

/// GET /api/store -- returns HTML fragment with token mappings table or empty-state message.
pub async fn api_store(State(state): State<Arc<AppState>>) -> Html<String> {
    let session_key = state.session_key.clone();
    let result = tokio::task::spawn_blocking(move || {
        let store_dir = std::env::current_dir()
            .map_err(|e| format!("{}", e))?
            .join(".loktok");
        let store =
            Store::with_passphrase(&store_dir, session_key).map_err(|e| format!("{}", e))?;
        store.load().map_err(|e| format!("{}", e))
    })
    .await;

    match result {
        Ok(Ok(data)) if !data.token_to_value.is_empty() => {
            let mut rows = String::new();
            let mut entries: Vec<_> = data.token_to_value.iter().collect();
            entries.sort_by_key(|(k, _)| k.to_string());
            for (token, value) in &entries {
                // Parse category from token format: CATEGORY_NNN
                let category = extract_category(token);
                let display_value = if value.len() > 30 {
                    format!("{}...", &value[..30])
                } else {
                    (*value).clone()
                };
                // T-06-06: HTML-escape all user-facing content
                let escaped_token = escape_html(token);
                let escaped_value = escape_html(&display_value);
                rows.push_str(&format!(
                    "<tr><td><code>[{}]</code></td><td>{}</td><td><code>{}</code></td></tr>",
                    escaped_token, category, escaped_value
                ));
            }
            Html(format!(
                "<div class='store-panel'>\
                   <div class='panel-header'>\
                     <span>{} tokens stored</span>\
                     <button hx-get='/api/store' hx-target='#store-content' class='btn-secondary'>Refresh</button>\
                   </div>\
                   <table class='token-table'>\
                     <thead><tr><th>Token</th><th>Category</th><th>Value</th></tr></thead>\
                     <tbody>{}</tbody>\
                   </table>\
                 </div>",
                entries.len(),
                rows
            ))
        }
        _ => Html(
            "<div class='store-empty'><p>No tokens in store. Tokenize a file first.</p></div>"
                .to_string(),
        ),
    }
}

/// GET /api/docs -- returns HTML fragment with command reference and token categories.
pub async fn api_docs() -> Html<String> {
    let commands_html = build_commands_html();
    let categories_html = build_categories_html();

    Html(format!(
        "<div class='docs-panel'>\
           <h2>Command Reference</h2>{}\
           <h2>Token Categories</h2>\
           <table class='token-table'>\
             <thead><tr><th>Prefix</th><th>Description</th></tr></thead>\
             <tbody>{}</tbody>\
           </table>\
         </div>",
        commands_html, categories_html
    ))
}

/// Build HTML for all CLI commands from clap metadata.
fn build_commands_html() -> String {
    let (commands, _global_args) = extract_commands();
    let mut html = String::new();

    for cmd in &commands {
        html.push_str(&format!(
            "<div class='command-block'>\
               <h3><code>logtok {}</code></h3>\
               <p>{}</p>",
            escape_html(&cmd.name),
            escape_html(&cmd.about)
        ));

        if let Some(long) = &cmd.long_about {
            html.push_str(&format!("<p class='long-about'>{}</p>", escape_html(long)));
        }

        if !cmd.args.is_empty() {
            html.push_str(
                "<table class='args-table'>\
                   <thead><tr><th>Flag</th><th>Description</th><th>Default</th></tr></thead>\
                   <tbody>",
            );
            for arg in &cmd.args {
                let flag = match (&arg.short, &arg.long) {
                    (Some(s), Some(l)) => format!("-{}, --{}", s, escape_html(l)),
                    (None, Some(l)) => format!("--{}", escape_html(l)),
                    (Some(s), None) => format!("-{}", s),
                    (None, None) => escape_html(&arg.name),
                };
                let default = arg
                    .default_value
                    .as_deref()
                    .map(|d| escape_html(d))
                    .unwrap_or_else(|| "-".to_string());
                html.push_str(&format!(
                    "<tr><td><code>{}</code></td><td>{}</td><td>{}</td></tr>",
                    flag,
                    escape_html(&arg.help),
                    default
                ));
            }
            html.push_str("</tbody></table>");
        }

        if let Some(after) = &cmd.after_long_help {
            html.push_str(&format!(
                "<div class='after-help'>{}</div>",
                escape_html(after)
            ));
        }

        html.push_str("</div>");
    }

    html
}

/// Build HTML table rows for all 19 token categories.
fn build_categories_html() -> String {
    let categories = get_token_categories();
    let mut html = String::new();
    for cat in &categories {
        html.push_str(&format!(
            "<tr><td><code>{}</code></td><td>{}</td></tr>",
            escape_html(&cat.prefix),
            escape_html(&cat.description)
        ));
    }
    html
}

/// Extract category name from a token string like "IP_001" -> "IP"
fn extract_category(token: &str) -> String {
    // Token format: CATEGORY_NNN (e.g., IP_001, HOST_002)
    // Find the last underscore followed by digits
    if let Some(idx) = token.rfind('_') {
        let suffix = &token[idx + 1..];
        if suffix.chars().all(|c| c.is_ascii_digit()) && !suffix.is_empty() {
            return token[..idx].to_string();
        }
    }
    token.to_string()
}

/// HTML-escape user-facing content to prevent XSS (T-06-06).
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
