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
                let category = extract_category(token);
                let display_value = if value.len() > 40 {
                    format!("{}...", &value[..40])
                } else {
                    (*value).clone()
                };
                let escaped_token = escape_html(token);
                let escaped_value = escape_html(&display_value);
                rows.push_str(&format!(
                    "<tr class='border-b border-zinc-800/50 hover:bg-brand-500/[0.03] transition-colors'>\
                       <td class='py-3 px-4'><code class='text-brand-400 text-xs font-mono'>[{}]</code></td>\
                       <td class='py-3 px-4'><span class='text-xs font-medium px-2 py-0.5 rounded-full bg-zinc-800 text-zinc-300'>{}</span></td>\
                       <td class='py-3 px-4'><code class='text-xs font-mono text-zinc-400'>{}</code></td>\
                     </tr>",
                    escaped_token, category, escaped_value
                ));
            }
            Html(format!(
                "<div>\
                   <div class='flex items-center justify-between mb-4'>\
                     <span class='text-sm text-zinc-400'>{} tokens stored</span>\
                     <button hx-get='/api/store' hx-target='#store-content' \
                             class='px-3 py-1.5 text-xs font-medium rounded-lg border border-zinc-700 text-zinc-400 hover:text-zinc-200 hover:border-zinc-600 transition-colors'>Refresh</button>\
                   </div>\
                   <div class='rounded-2xl border border-zinc-800 overflow-hidden'>\
                     <table class='w-full text-sm'>\
                       <thead><tr class='border-b border-zinc-800 text-zinc-500'>\
                         <th class='text-left py-3 px-4 text-xs font-semibold uppercase tracking-wider'>Token</th>\
                         <th class='text-left py-3 px-4 text-xs font-semibold uppercase tracking-wider'>Category</th>\
                         <th class='text-left py-3 px-4 text-xs font-semibold uppercase tracking-wider'>Value</th>\
                       </tr></thead>\
                       <tbody>{}</tbody>\
                     </table>\
                   </div>\
                 </div>",
                entries.len(), rows
            ))
        }
        _ => Html(
            "<div class='text-center py-16'>\
               <svg class='mx-auto mb-4 text-zinc-600' width='48' height='48' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.5'>\
                 <path d='M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4'/>\
               </svg>\
               <p class='text-zinc-400 text-sm'>No tokens in store</p>\
               <p class='text-zinc-600 text-xs mt-1'>Tokenize a file to see mappings here</p>\
             </div>".to_string(),
        ),
    }
}

/// GET /api/docs -- returns HTML fragment with workflow, command reference, and token categories.
pub async fn api_docs() -> Html<String> {
    let commands_html = build_commands_html();
    let categories_html = build_categories_html();

    Html(format!(
        "<div class='max-w-4xl'>\
           <div class='mb-10'>\
             <div class='flex flex-col gap-0 max-w-2xl'>\
               <div class='flex items-start gap-4 p-5 rounded-2xl border border-zinc-800 bg-surface-800'>\
                 <div class='w-9 h-9 flex items-center justify-center rounded-full bg-brand-500 text-white font-bold text-sm shrink-0'>1</div>\
                 <div>\
                   <p class='font-semibold text-zinc-100'>Tokenize your logs</p>\
                   <p class='text-sm text-zinc-400 mt-0.5'>Replace sensitive data with safe tokens</p>\
                   <code class='inline-block mt-2 px-3 py-1 text-xs font-mono rounded-lg bg-surface-900 text-brand-400'>logtok tokenize app.log -o safe.log</code>\
                 </div>\
               </div>\
               <div class='flex justify-center py-1 text-zinc-600 pl-4'>\
                 <svg width='20' height='20' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'><line x1='12' y1='5' x2='12' y2='19'/><polyline points='19 12 12 19 5 12'/></svg>\
               </div>\
               <div class='flex items-start gap-4 p-5 rounded-2xl border border-zinc-800 bg-surface-800'>\
                 <div class='w-9 h-9 flex items-center justify-center rounded-full bg-brand-500 text-white font-bold text-sm shrink-0'>2</div>\
                 <div>\
                   <p class='font-semibold text-zinc-100'>Send to any AI</p>\
                   <p class='text-sm text-zinc-400 mt-0.5'>Claude, ChatGPT, or any LLM can analyze without seeing secrets</p>\
                   <code class='inline-block mt-2 px-3 py-1 text-xs font-mono rounded-lg bg-surface-900 text-brand-400'>claude &quot;diagnose errors in safe.log&quot;</code>\
                 </div>\
               </div>\
               <div class='flex justify-center py-1 text-zinc-600 pl-4'>\
                 <svg width='20' height='20' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'><line x1='12' y1='5' x2='12' y2='19'/><polyline points='19 12 12 19 5 12'/></svg>\
               </div>\
               <div class='flex items-start gap-4 p-5 rounded-2xl border border-zinc-800 bg-surface-800'>\
                 <div class='w-9 h-9 flex items-center justify-center rounded-full bg-brand-500 text-white font-bold text-sm shrink-0'>3</div>\
                 <div>\
                   <p class='font-semibold text-zinc-100'>Detokenize the response</p>\
                   <p class='text-sm text-zinc-400 mt-0.5'>Restore real values for a full, readable diagnosis report</p>\
                   <code class='inline-block mt-2 px-3 py-1 text-xs font-mono rounded-lg bg-surface-900 text-brand-400'>logtok detokenize -f ai-response.md</code>\
                 </div>\
               </div>\
             </div>\
           </div>\
           <div class='mb-10'>\
             <h2 class='text-xl font-bold text-zinc-100 mb-1'>CLI Reference</h2>\
             <p class='text-sm text-zinc-400 mb-5'>All available commands and options</p>\
             {}\
           </div>\
           <div class='mb-10'>\
             <h2 class='text-xl font-bold text-zinc-100 mb-1'>Token Categories</h2>\
             <p class='text-sm text-zinc-400 mb-5'>19 built-in categories of sensitive data</p>\
             <div class='rounded-2xl border border-zinc-800 overflow-hidden'>\
               <table class='w-full text-sm'>\
                 <thead><tr class='border-b border-zinc-800 text-zinc-500'>\
                   <th class='text-left py-3 px-4 text-xs font-semibold uppercase tracking-wider'>Prefix</th>\
                   <th class='text-left py-3 px-4 text-xs font-semibold uppercase tracking-wider'>Detects</th>\
                 </tr></thead>\
                 <tbody>{}</tbody>\
               </table>\
             </div>\
           </div>\
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
            "<div class='rounded-2xl border border-zinc-800 bg-surface-800 p-5 mb-3'>\
               <h3 class='font-semibold text-zinc-100 mb-1'><code class='text-brand-400 font-mono'>logtok {}</code></h3>\
               <p class='text-sm text-zinc-400'>{}</p>",
            escape_html(&cmd.name),
            escape_html(&cmd.about)
        ));

        if let Some(long) = &cmd.long_about {
            html.push_str(&format!("<p class='text-xs text-zinc-500 mt-1'>{}</p>", escape_html(long)));
        }

        if !cmd.args.is_empty() {
            html.push_str(
                "<table class='w-full text-sm mt-3'>\
                   <thead><tr class='border-b border-zinc-800 text-zinc-500'>\
                     <th class='text-left py-2 text-xs font-semibold uppercase tracking-wider'>Flag</th>\
                     <th class='text-left py-2 text-xs font-semibold uppercase tracking-wider'>Description</th>\
                     <th class='text-left py-2 text-xs font-semibold uppercase tracking-wider'>Default</th>\
                   </tr></thead>\
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
                    "<tr class='border-b border-zinc-800/50'>\
                       <td class='py-2 pr-4'><code class='text-xs font-mono text-brand-400'>{}</code></td>\
                       <td class='py-2 pr-4 text-zinc-400 text-xs'>{}</td>\
                       <td class='py-2 text-zinc-500 text-xs font-mono'>{}</td>\
                     </tr>",
                    flag,
                    escape_html(&arg.help),
                    default
                ));
            }
            html.push_str("</tbody></table>");
        }

        if let Some(after) = &cmd.after_long_help {
            html.push_str(&format!(
                "<div class='mt-3 p-3 rounded-lg bg-surface-900 text-xs font-mono text-zinc-500 whitespace-pre-wrap'>{}</div>",
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
            "<tr class='border-b border-zinc-800/50 hover:bg-brand-500/[0.03] transition-colors'>\
               <td class='py-2.5 px-4'><code class='text-xs font-mono font-medium text-brand-400'>{}</code></td>\
               <td class='py-2.5 px-4 text-sm text-zinc-400'>{}</td>\
             </tr>",
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
