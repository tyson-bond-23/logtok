use askama::Template;
use axum::extract::Query;
use axum::response::{Html, Json};
use std::collections::HashMap;

use crate::ui::i18n;

#[derive(Template)]
#[template(path = "ui/base.html")]
struct DashboardTemplate {
    version: String,
    translations: HashMap<String, String>,
}

pub async fn dashboard() -> Result<Html<String>, axum::http::StatusCode> {
    // Default to English for initial page load; client-side Alpine.js
    // will call /api/translations if a different language is stored
    let t = i18n::translations("en");
    let translations: HashMap<String, String> = t
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let template = DashboardTemplate {
        version: env!("CARGO_PKG_VERSION").to_string(),
        translations,
    };
    template
        .render()
        .map(Html)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(serde::Deserialize)]
pub struct TranslationsQuery {
    #[serde(default = "default_lang")]
    lang: String,
}

fn default_lang() -> String {
    "en".to_string()
}

/// GET /api/translations?lang=xx -- returns JSON translation map for the requested language.
pub async fn api_translations(
    Query(params): Query<TranslationsQuery>,
) -> Json<HashMap<String, String>> {
    let t = i18n::translations(&params.lang);
    let map: HashMap<String, String> = t
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    Json(map)
}
