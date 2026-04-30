use axum::response::Html;

use crate::config::{CustomPatternToml, DetectionSection, LoktokConfig, StoreSection};
use crate::ui::api_store::escape_html;

/// All 19 detection categories with descriptions and regex patterns.
const CATEGORIES: [(&str, &str, &str); 19] = [
    ("PEM", "PEM certificates & keys", r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    ("JWT", "JSON Web Tokens", r"eyJ[...].eyJ[...].[...]"),
    ("CONN", "Connection strings", r"(postgresql|mysql|mongodb|redis)://..."),
    ("EMAIL", "Email addresses", r"[a-zA-Z0-9._%+-]+@[...]\.[a-zA-Z]{2,}"),
    ("URL", "URLs & endpoints", r"https?://[^\s]+"),
    ("KEY", "API keys & secrets", r"(api_key|token|secret)\s*[=:]\s*VALUE"),
    ("PASS", "Passwords", r"(password|passwd|pwd)\s*[=:]\s*VALUE"),
    ("CC", "Credit card numbers", r"4xxx-xxxx-xxxx-xxxx (Luhn validated)"),
    ("SSN", "Social security numbers", r"\d{3}-\d{2}-\d{4}"),
    ("ARN", "AWS ARN identifiers", r"arn:aws:[...]:ACCOUNT_ID:[...]"),
    ("NAME", "Usernames & authors", r"(user|username|author)\s*[=:]\s*VALUE"),
    ("UUID", "UUIDs / GUIDs", r"[0-9a-f]{8}-[...]-[0-9a-f]{12}"),
    ("MAC", "MAC addresses", r"[0-9a-f]{2}:[...repeated 6x...]"),
    ("PHONE", "Phone numbers", r"(+1-)?\d{3}-\d{3}-\d{4}"),
    ("IP", "IP addresses (v4)", r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}"),
    ("HOST", "Internal hostnames", r"host.internal|host.local|host.corp"),
    ("DNS", "Domain names", r"sub.domain.tld"),
    ("OS", "OS version strings", r"Linux 5.15|Windows NT 10.0|Darwin 23"),
    ("PATH", "File system paths", r"/var/log/app/error.log"),
];

/// GET /api/config -- returns HTML fragment with config form (category toggles, patterns, TTL)
/// and raw TOML toggle (D-27).
pub async fn api_config_get() -> Html<String> {
    let (cfg, raw_toml) = tokio::task::spawn_blocking(|| {
        if let Some(path) = crate::config::find_config() {
            let raw = std::fs::read_to_string(&path).unwrap_or_default();
            let parsed = crate::config::load_config(&path).unwrap_or_default();
            (parsed, raw)
        } else {
            (LoktokConfig::default(), String::new())
        }
    })
    .await
    .unwrap_or((LoktokConfig::default(), String::new()));

    // Build category toggle cards with regex info popup
    let mut category_toggles = String::new();
    for (cat, desc, regex_example) in &CATEGORIES {
        let disabled = cfg
            .detection
            .disabled
            .iter()
            .any(|d| d.eq_ignore_ascii_case(cat));
        let checked = if disabled { "" } else { "checked" };
        let escaped_regex = escape_html(regex_example);
        category_toggles.push_str(&format!(
            "<div class='relative group rounded-xl border p-4 transition-all {}' x-data='{{ showInfo: false }}'>\
               <div class='flex items-start justify-between gap-2'>\
                 <label class='flex items-center gap-3 cursor-pointer flex-1 min-w-0'>\
                   <input type='checkbox' name='enabled_{}' {} value='1' class='w-4 h-4 accent-brand-500 shrink-0 mt-0.5'>\
                   <div class='min-w-0'>\
                     <span class='block text-sm font-mono font-bold text-brand-400'>{}</span>\
                     <span class='block text-xs text-zinc-500 mt-0.5'>{}</span>\
                   </div>\
                 </label>\
                 <button type='button' @click='showInfo = !showInfo' \
                         class='w-6 h-6 flex items-center justify-center rounded-full text-xs font-bold shrink-0 transition-colors' \
                         :class=\"showInfo ? 'bg-brand-500 text-white' : 'bg-zinc-700 text-zinc-400 hover:bg-zinc-600 hover:text-zinc-200'\">\
                   ?</button>\
               </div>\
               <div x-show='showInfo' x-transition x-cloak \
                    class='mt-3 pt-3 border-t border-zinc-700/50'>\
                 <p class='text-[10px] font-medium uppercase tracking-wider text-zinc-500 mb-1'>Pattern</p>\
                 <code class='block text-xs font-mono text-emerald-400 bg-zinc-900/60 rounded-lg px-3 py-2 break-all'>{}</code>\
               </div>\
             </div>",
            if disabled { "border-zinc-800 bg-surface-800/50 opacity-60" } else { "border-zinc-700 bg-surface-800 hover:border-zinc-600" },
            cat.to_lowercase(),
            checked,
            cat,
            desc,
            escaped_regex
        ));
    }

    // Custom patterns section with remove buttons
    let mut patterns_html = String::new();
    for (i, p) in cfg.detection.custom_patterns.iter().enumerate() {
        patterns_html.push_str(&format!(
            "<div class='pattern-row grid grid-cols-[1fr_2fr_70px_32px] gap-2 mb-2 items-center'>\
               <input type='text' name='pattern_{}_name' value='{}' placeholder='Name' class='px-3 py-2 text-sm rounded-lg border border-zinc-700 bg-surface-800 text-zinc-200 focus:outline-none focus:border-brand-500'>\
               <input type='text' name='pattern_{}_regex' value='{}' placeholder='Regex' class='px-3 py-2 text-sm font-mono rounded-lg border border-zinc-700 bg-surface-800 text-zinc-200 focus:outline-none focus:border-brand-500'>\
               <input type='number' name='pattern_{}_group' value='{}' placeholder='Grp' class='px-3 py-2 text-sm rounded-lg border border-zinc-700 bg-surface-800 text-zinc-200 text-center focus:outline-none focus:border-brand-500'>\
               <button type='button' onclick='this.parentElement.remove()' class='w-8 h-8 flex items-center justify-center rounded-lg border border-zinc-700 text-zinc-500 hover:text-red-400 hover:border-red-400 transition-colors'>&times;</button>\
             </div>",
            i,
            escape_html(&p.name),
            i,
            escape_html(&p.pattern),
            i,
            p.capture_group.unwrap_or(0)
        ));
    }

    let escaped_toml = escape_html(&raw_toml);

    Html(format!(
        "<div class='max-w-3xl' x-data=\"{{ configMode: 'form' }}\">\
           <div class='inline-flex rounded-xl border border-zinc-800 p-1 mb-8 bg-surface-800'>\
             <button type='button' @click=\"configMode = 'form'\" \
                     class='px-4 py-1.5 text-sm font-medium rounded-lg transition-all' \
                     :class=\"configMode === 'form' ? 'bg-brand-500 text-white' : 'text-zinc-400 hover:text-zinc-200'\">Form View</button>\
             <button type='button' @click=\"configMode = 'toml'\" \
                     class='px-4 py-1.5 text-sm font-medium rounded-lg transition-all' \
                     :class=\"configMode === 'toml' ? 'bg-brand-500 text-white' : 'text-zinc-400 hover:text-zinc-200'\">Raw TOML</button>\
           </div>\
           <form hx-put='/api/config' hx-target='#config-result' hx-encoding='multipart/form-data'>\
             <div x-show=\"configMode === 'form'\">\
               <div class='mb-8'>\
                 <h3 class='text-lg font-semibold cfg-heading mb-1'>Detection Categories</h3>\
                 <p class='text-sm text-zinc-500 mb-4'>Toggle which types of sensitive data to detect. Click <span class='inline-flex items-center justify-center w-4 h-4 rounded-full bg-zinc-700 text-zinc-400 text-[9px] font-bold'>?</span> to see the regex pattern.</p>\
                 <div class='grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3'>{}</div>\
               </div>\
               <div class='mb-8'>\
                 <h3 class='text-lg font-semibold cfg-heading mb-1'>Custom Patterns</h3>\
                 <p class='text-sm text-zinc-400 mb-4'>Define your own regex patterns for domain-specific data</p>\
                 <div id='patterns-list' class='mb-3'>{}</div>\
                 <button type='button' onclick='addPatternRow()' \
                         class='inline-flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-lg border border-dashed border-zinc-700 text-zinc-400 hover:text-brand-400 hover:border-brand-500 transition-colors'>\
                   <svg width='14' height='14' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'><line x1='12' y1='5' x2='12' y2='19'/><line x1='5' y1='12' x2='19' y2='12'/></svg>\
                   Add Pattern\
                 </button>\
               </div>\
               <div class='mb-8'>\
                 <h3 class='text-lg font-semibold cfg-heading mb-1'>Store Settings</h3>\
                 <div class='flex items-center gap-3 mt-3'>\
                   <label class='text-sm font-medium text-zinc-300'>Token TTL</label>\
                   <input type='number' name='ttl_days' value='{}' min='1' max='365' \
                          class='w-20 px-3 py-2 text-sm text-center rounded-lg border border-zinc-700 bg-surface-800 text-zinc-200 focus:outline-none focus:border-brand-500'>\
                   <span class='text-sm text-zinc-500'>days</span>\
                 </div>\
               </div>\
             </div>\
             <div x-show=\"configMode === 'toml'\">\
               <textarea name='raw_toml' rows='20' \
                         class='w-full px-4 py-3 text-sm font-mono rounded-2xl border border-zinc-800 bg-surface-800 text-zinc-200 focus:outline-none focus:border-brand-500 resize-y'>{}</textarea>\
             </div>\
             <div class='mt-6 pt-6 border-t border-zinc-800'>\
               <button type='submit' \
                       class='inline-flex items-center gap-2 px-6 py-2.5 bg-brand-500 hover:bg-brand-600 text-white text-sm font-semibold rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-brand-500 focus:ring-offset-2 focus:ring-offset-surface-900'>\
                 <svg width='16' height='16' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'>\
                   <path d='M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z'/>\
                   <polyline points='17 21 17 13 7 13 7 21'/><polyline points='7 3 7 8 15 8'/>\
                 </svg>\
                 Save Configuration\
               </button>\
             </div>\
           </form>\
           <div id='config-result' class='mt-4'></div>\
         </div>",
        category_toggles, patterns_html, cfg.store.ttl_days, escaped_toml
    ))
}

/// PUT /api/config -- accepts multipart form data, validates, and writes .logtok.toml.
/// T-06-03: Validates TOML before writing (parse first, reject invalid).
pub async fn api_config_put(
    mut multipart: axum::extract::Multipart,
) -> Html<String> {
    // Collect all form fields
    let mut fields: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        let value = field.text().await.unwrap_or_default();
        fields.insert(name, value);
    }

    // Determine config path -- write to CWD/.logtok.toml
    let config_path = std::env::current_dir()
        .unwrap_or_default()
        .join(".logtok.toml");

    // Check if raw TOML mode was used (non-empty raw_toml field)
    let raw_toml = fields.get("raw_toml").cloned().unwrap_or_default();
    let raw_toml_trimmed = raw_toml.trim();

    let toml_content = if !raw_toml_trimmed.is_empty() {
        // Raw TOML mode: validate by parsing first (T-06-03)
        match toml::from_str::<LoktokConfig>(raw_toml_trimmed) {
            Ok(_) => raw_toml_trimmed.to_string(),
            Err(e) => {
                return Html(format!(
                    "<div class='mt-4 p-3 rounded-lg border border-red-500/30 bg-red-500/10 text-red-400 text-sm'>Invalid TOML: {}</div>",
                    escape_html(&e.to_string())
                ));
            }
        }
    } else {
        // Form mode: build config from form fields
        let mut disabled: Vec<String> = Vec::new();
        for (cat, _, _) in &CATEGORIES {
            let key = format!("enabled_{}", cat.to_lowercase());
            if !fields.contains_key(&key) {
                // Unchecked checkbox = category disabled
                disabled.push(cat.to_string());
            }
        }

        // Collect custom patterns
        let mut custom_patterns: Vec<CustomPatternToml> = Vec::new();
        for i in 0..50 {
            let name_key = format!("pattern_{}_name", i);
            let regex_key = format!("pattern_{}_regex", i);
            let group_key = format!("pattern_{}_group", i);

            if let (Some(name), Some(regex)) = (fields.get(&name_key), fields.get(&regex_key)) {
                let name = name.trim().to_string();
                let regex = regex.trim().to_string();
                if !name.is_empty() && !regex.is_empty() {
                    let group = fields
                        .get(&group_key)
                        .and_then(|g| g.parse::<usize>().ok())
                        .filter(|&g| g > 0);
                    custom_patterns.push(CustomPatternToml {
                        name,
                        pattern: regex,
                        capture_group: group,
                    });
                }
            } else {
                break;
            }
        }

        let ttl_days = fields
            .get("ttl_days")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(30)
            .clamp(1, 365);

        let config = LoktokConfig {
            detection: DetectionSection {
                disabled,
                custom_patterns,
            },
            store: StoreSection { ttl_days },
        };

        // Serialize to TOML
        match toml::to_string_pretty(&config) {
            Ok(s) => s,
            Err(e) => {
                return Html(format!(
                    "<div class='mt-4 p-3 rounded-lg border border-red-500/30 bg-red-500/10 text-red-400 text-sm'>Serialization error: {}</div>",
                    escape_html(&e.to_string())
                ));
            }
        }
    };

    // Write to .logtok.toml
    let write_result = tokio::task::spawn_blocking(move || {
        std::fs::write(&config_path, &toml_content)
    })
    .await;

    match write_result {
        Ok(Ok(())) => Html(
            "<div class='mt-4 p-3 rounded-lg border border-emerald-500/30 bg-emerald-500/10 text-emerald-400 text-sm'>Configuration saved to .logtok.toml</div>"
                .to_string(),
        ),
        Ok(Err(e)) => Html(format!(
            "<div class='mt-4 p-3 rounded-lg border border-red-500/30 bg-red-500/10 text-red-400 text-sm'>Write failed: {}</div>",
            escape_html(&e.to_string())
        )),
        Err(e) => Html(format!(
            "<div class='mt-4 p-3 rounded-lg border border-red-500/30 bg-red-500/10 text-red-400 text-sm'>Internal error: {}</div>",
            escape_html(&e.to_string())
        )),
    }
}
