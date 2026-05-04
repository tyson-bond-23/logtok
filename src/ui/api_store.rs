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
        "<div class='max-w-4xl space-y-12'>\
           <section>\
             <div class='flex items-center gap-3 mb-6'>\
               <div class='w-10 h-10 flex items-center justify-center rounded-xl bg-brand-500/10 text-brand-400'>\
                 <svg width='20' height='20' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'><path d='M13 2L3 14h9l-1 8 10-12h-9l1-8z'/></svg>\
               </div>\
               <div>\
                 <h2 class='text-xl font-bold docs-heading'>Quick Start</h2>\
                 <p class='text-sm text-zinc-500'>Three steps to safe AI log diagnosis</p>\
               </div>\
             </div>\
             <div class='rounded-2xl' style='display:flex;flex-direction:column;gap:0;padding:2rem;background:radial-gradient(ellipse at center, rgba(223,208,184,0.08) 0%, transparent 70%);'>\
               <div class='rounded-2xl border border-zinc-800 bg-surface-800 p-5 docs-card'>\
                 <div class='flex items-center gap-4 mb-3'>\
                   <span style='width:3rem;height:3rem;font-size:1.25rem;' class='flex-shrink-0 flex items-center justify-center rounded-xl font-bold' :class=\"theme === 'light' ? 'bg-brand-500 text-white' : 'bg-[#DFD0B8] text-[#222831]'\">1</span>\
                   <div>\
                     <h3 class='font-semibold docs-heading' style='font-size:1rem;'>Tokenize</h3>\
                     <p class='text-xs text-zinc-500'>Replace sensitive data with safe tokens</p>\
                   </div>\
                 </div>\
                 <div class='relative group/copy'>\
                   <code class='block px-3 py-2 text-xs font-mono rounded-lg bg-zinc-900/60 text-brand-400 docs-code cursor-pointer' \
                         onclick='navigator.clipboard.writeText(this.textContent.trim());var b=this.nextElementSibling;b.textContent=\"Copied!\";setTimeout(function(){{b.textContent=\"Click to copy\"}},1500)'>logtok tokenize app.log -o safe.log</code>\
                   <span class='absolute -top-6 left-1/2 -translate-x-1/2 text-[10px] px-2 py-0.5 rounded bg-zinc-700 text-zinc-300 opacity-0 group-hover/copy:opacity-100 transition-opacity pointer-events-none'>Click to copy</span>\
                 </div>\
               </div>\
               <div class='flex justify-center' style='padding:0.5rem 0 0.5rem 1rem;'>\
                 <svg width='20' height='28' viewBox='0 0 20 28' fill='none' stroke='#948979' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'><path d='M10 2 L10 22'/><path d='M5 18 L10 24 L15 18'/></svg>\
               </div>\
               <div class='rounded-2xl border border-zinc-800 bg-surface-800 p-5 docs-card'>\
                 <div class='flex items-center gap-4 mb-3'>\
                   <span style='width:3rem;height:3rem;font-size:1.25rem;' class='flex-shrink-0 flex items-center justify-center rounded-xl font-bold' :class=\"theme === 'light' ? 'bg-brand-500/20 text-brand-500' : 'bg-[#DFD0B8]/20 text-[#DFD0B8]'\">2</span>\
                   <div>\
                     <h3 class='font-semibold docs-heading' style='font-size:1rem;'>Analyze with AI</h3>\
                     <p class='text-xs text-zinc-500'>Send to any AI without exposing secrets</p>\
                   </div>\
                 </div>\
                 <div class='relative group/copy'>\
                   <code class='block px-3 py-2 text-xs font-mono rounded-lg bg-zinc-900/60 text-brand-400 docs-code cursor-pointer' \
                         onclick='navigator.clipboard.writeText(this.textContent.trim());var b=this.nextElementSibling;b.textContent=\"Copied!\";setTimeout(function(){{b.textContent=\"Click to copy\"}},1500)'>claude &quot;diagnose errors in safe.log&quot;</code>\
                   <span class='absolute -top-6 left-1/2 -translate-x-1/2 text-[10px] px-2 py-0.5 rounded bg-zinc-700 text-zinc-300 opacity-0 group-hover/copy:opacity-100 transition-opacity pointer-events-none'>Click to copy</span>\
                 </div>\
               </div>\
               <div class='flex justify-center' style='padding:0.5rem 0 0.5rem 1rem;'>\
                 <svg width='20' height='28' viewBox='0 0 20 28' fill='none' stroke='#948979' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'><path d='M10 2 L10 22'/><path d='M5 18 L10 24 L15 18'/></svg>\
               </div>\
               <div class='rounded-2xl border border-zinc-800 bg-surface-800 p-5 docs-card'>\
                 <div class='flex items-center gap-4 mb-3'>\
                   <span style='width:3rem;height:3rem;font-size:1.25rem;' class='flex-shrink-0 flex items-center justify-center rounded-xl font-bold' :class=\"theme === 'light' ? 'bg-brand-500/20 text-brand-500' : 'bg-[#DFD0B8]/20 text-[#DFD0B8]'\">3</span>\
                   <div>\
                     <h3 class='font-semibold docs-heading' style='font-size:1rem;'>Detokenize</h3>\
                     <p class='text-xs text-zinc-500'>Restore real values in the AI response</p>\
                   </div>\
                 </div>\
                 <div class='relative group/copy'>\
                   <code class='block px-3 py-2 text-xs font-mono rounded-lg bg-zinc-900/60 text-brand-400 docs-code cursor-pointer' \
                         onclick='navigator.clipboard.writeText(this.textContent.trim());var b=this.nextElementSibling;b.textContent=\"Copied!\";setTimeout(function(){{b.textContent=\"Click to copy\"}},1500)'>logtok detokenize -f ai-response.md</code>\
                   <span class='absolute -top-6 left-1/2 -translate-x-1/2 text-[10px] px-2 py-0.5 rounded bg-zinc-700 text-zinc-300 opacity-0 group-hover/copy:opacity-100 transition-opacity pointer-events-none'>Click to copy</span>\
                 </div>\
               </div>\
             </div>\
           </section>\
           <section>\
             <div class='flex items-center gap-3 mb-6'>\
               <div class='w-10 h-10 flex items-center justify-center rounded-xl bg-brand-500/10 text-brand-400'>\
                 <svg width='20' height='20' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'><polyline points='4 17 10 11 4 5'/><line x1='12' y1='19' x2='20' y2='19'/></svg>\
               </div>\
               <div>\
                 <h2 class='text-xl font-bold docs-heading'>CLI Reference</h2>\
                 <p class='text-sm text-zinc-500'>All available commands and options</p>\
               </div>\
             </div>\
             <div class='space-y-3'>{}</div>\
           </section>\
           <section>\
             <div class='flex items-center gap-3 mb-6'>\
               <div class='w-10 h-10 flex items-center justify-center rounded-xl bg-brand-500/10 text-brand-400'>\
                 <svg width='20' height='20' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'><rect x='3' y='3' width='7' height='7'/><rect x='14' y='3' width='7' height='7'/><rect x='3' y='14' width='7' height='7'/><rect x='14' y='14' width='7' height='7'/></svg>\
               </div>\
               <div>\
                 <h2 class='text-xl font-bold docs-heading'>Token Categories</h2>\
                 <p class='text-sm text-zinc-500'>19 built-in categories of sensitive data</p>\
               </div>\
             </div>\
             <div class='rounded-2xl border border-zinc-800 overflow-hidden docs-table'>\
               <table class='w-full text-sm'>\
                 <thead><tr class='border-b border-zinc-700 bg-zinc-800/50'>\
                   <th class='text-left py-3 px-4 text-xs font-semibold uppercase tracking-wider text-zinc-400'>Prefix</th>\
                   <th class='text-left py-3 px-4 text-xs font-semibold uppercase tracking-wider text-zinc-400'>Detects</th>\
                 </tr></thead>\
                 <tbody>{}</tbody>\
               </table>\
             </div>\
           </section>\
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
            "<div class='rounded-2xl border border-zinc-800 bg-surface-800 p-5 mb-3 docs-card'>\
               <div class='flex items-center gap-2 mb-1'>\
                 <h3 class='font-semibold docs-heading'><code class='text-brand-400 font-mono'>logtok {}</code></h3>\
                 <button type='button' onclick='navigator.clipboard.writeText(\"logtok {}\");this.textContent=\"Copied!\";setTimeout(()=>this.textContent=\"Copy\",1500)' \
                         class='px-2 py-0.5 text-[10px] font-medium rounded border border-zinc-700 text-zinc-500 hover:text-zinc-200 hover:border-zinc-600 transition-colors'>Copy</button>\
               </div>\
               <p class='text-sm text-zinc-400'>{}</p>",
            escape_html(&cmd.name),
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
            // Parse example lines and make each command clickable to copy
            let mut examples_html = String::new();
            for line in after.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("logtok ") || trimmed.starts_with("echo ") || trimmed.starts_with("cat ") {
                    // Extract just the command part (before description separated by multiple spaces)
                    let cmd_text = if let Some(idx) = trimmed.find("  ") {
                        trimmed[..idx].trim()
                    } else {
                        trimmed
                    };
                    let desc_text = if let Some(idx) = trimmed.find("  ") {
                        trimmed[idx..].trim()
                    } else {
                        ""
                    };
                    examples_html.push_str(&format!(
                        "<div class='flex items-center gap-2 py-0.5 group/ex'>\
                           <code class='text-brand-400 cursor-pointer hover:text-brand-300 transition-colors' \
                                 onclick='navigator.clipboard.writeText(this.textContent.trim());var s=this.nextElementSibling;if(s){{s.textContent=\"Copied!\";setTimeout(()=>s.textContent=\"{}\",1500)}}'>{}</code>\
                           <span class='text-zinc-600 text-[10px]'>{}</span>\
                         </div>",
                        escape_html(desc_text),
                        escape_html(cmd_text),
                        escape_html(desc_text)
                    ));
                } else if !trimmed.is_empty() {
                    examples_html.push_str(&format!(
                        "<div class='text-zinc-500 py-0.5 mt-1 font-semibold'>{}</div>",
                        escape_html(trimmed)
                    ));
                }
            }
            if !examples_html.is_empty() {
                html.push_str(&format!(
                    "<div class='mt-3 p-3 rounded-lg bg-surface-900 text-xs font-mono docs-code'>{}</div>",
                    examples_html
                ));
            }
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
