use axum::response::Html;

use crate::config::{CustomPatternToml, DetectionSection, LoktokConfig, StoreSection};
use crate::ui::api_store::escape_html;

/// All 19 detection categories.
const CATEGORIES: [&str; 19] = [
    "IP", "HOST", "URL", "PATH", "PORT", "EMAIL", "USER", "PHONE", "KEY", "PASS", "CONN", "JWT",
    "PEM", "UUID", "MAC", "CC", "SSN", "DOB", "CUSTOM",
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

    // Build category toggles
    let mut category_toggles = String::new();
    for cat in &CATEGORIES {
        let disabled = cfg
            .detection
            .disabled
            .iter()
            .any(|d| d.eq_ignore_ascii_case(cat));
        let checked = if disabled { "" } else { "checked" };
        category_toggles.push_str(&format!(
            "<label class='toggle-label'>\
               <input type='checkbox' name='enabled_{}' {} value='1'> {}\
             </label>",
            cat.to_lowercase(),
            checked,
            cat
        ));
    }

    // Custom patterns section
    let mut patterns_html = String::new();
    for (i, p) in cfg.detection.custom_patterns.iter().enumerate() {
        patterns_html.push_str(&format!(
            "<div class='pattern-row'>\
               <input type='text' name='pattern_{}_name' value='{}' placeholder='Name'>\
               <input type='text' name='pattern_{}_regex' value='{}' placeholder='Regex'>\
               <input type='number' name='pattern_{}_group' value='{}' placeholder='Group'>\
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
        "<div class='config-panel' x-data=\"{{ configMode: 'form' }}\">\
           <div class='config-mode-toggle'>\
             <button type='button' @click=\"configMode = 'form'\" :class=\"{{ 'active': configMode === 'form' }}\" class='btn-secondary'>Form view</button>\
             <button type='button' @click=\"configMode = 'toml'\" :class=\"{{ 'active': configMode === 'toml' }}\" class='btn-secondary'>Raw TOML</button>\
           </div>\
           <form hx-put='/api/config' hx-target='#config-result' hx-encoding='multipart/form-data'>\
             <div x-show=\"configMode === 'form'\">\
               <h3>Detection Categories</h3>\
               <div class='category-grid'>{}</div>\
               <h3>Custom Patterns</h3>\
               <div class='patterns-list' id='patterns-list'>{}</div>\
               <button type='button' onclick='addPatternRow()' class='btn-secondary'>+ Add Pattern</button>\
               <h3>Store Settings</h3>\
               <label>TTL (days): <input type='number' name='ttl_days' value='{}' min='1' max='365'></label>\
             </div>\
             <div x-show=\"configMode === 'toml'\">\
               <textarea name='raw_toml' rows='20' class='toml-editor'>{}</textarea>\
             </div>\
             <div class='form-actions'>\
               <button type='submit' class='btn-primary'>Save Configuration</button>\
             </div>\
           </form>\
           <div id='config-result'></div>\
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
                    "<div class='config-error'><p>Invalid TOML: {}</p></div>",
                    escape_html(&e.to_string())
                ));
            }
        }
    } else {
        // Form mode: build config from form fields
        let mut disabled: Vec<String> = Vec::new();
        for cat in &CATEGORIES {
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
                    "<div class='config-error'><p>Serialization error: {}</p></div>",
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
            "<div class='config-success'><p>Configuration saved to .logtok.toml</p></div>"
                .to_string(),
        ),
        Ok(Err(e)) => Html(format!(
            "<div class='config-error'><p>Write failed: {}</p></div>",
            escape_html(&e.to_string())
        )),
        Err(e) => Html(format!(
            "<div class='config-error'><p>Internal error: {}</p></div>",
            escape_html(&e.to_string())
        )),
    }
}
