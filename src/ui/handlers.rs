use askama::Template;
use axum::response::Html;
use std::collections::HashMap;

use crate::ui::i18n;

#[derive(Template)]
#[template(path = "ui/base.html")]
struct DashboardTemplate {
    version: String,
    translations: HashMap<String, String>,
}

pub async fn dashboard() -> Result<Html<String>, axum::http::StatusCode> {
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
