use askama::Template;
use axum::extract::Query;
use axum::response::{Html, Json};
use std::collections::HashMap;

use crate::ui::i18n;

pub struct ChangelogEntry {
    pub kind: &'static str,
    pub title: &'static str,
    pub description: &'static str,
}

/// Changelog for the current version. Update this when bumping the version.
fn current_changelog() -> Vec<ChangelogEntry> {
    vec![
        ChangelogEntry {
            kind: "feat",
            title: "New dark mode palette",
            description: "Warm earth tones — #222831 background, #DFD0B8 text, #948979 muted",
        },
        ChangelogEntry {
            kind: "feat",
            title: "First-time onboarding",
            description: "Welcome overlay with 3-step workflow guide for new users",
        },
        ChangelogEntry {
            kind: "feat",
            title: "Version update notifications",
            description: "See what's new when logtok is updated",
        },
        ChangelogEntry {
            kind: "fix",
            title: "Server survives page reload",
            description: "WebSocket heartbeat no longer kills the server on browser refresh",
        },
        ChangelogEntry {
            kind: "feat",
            title: "Copy button feedback",
            description: "Toast notification and green highlight when copying tokenized output",
        },
        ChangelogEntry {
            kind: "feat",
            title: "Token Store auto-refresh",
            description: "Store tab refreshes automatically when you switch to it",
        },
        ChangelogEntry {
            kind: "feat",
            title: "Smart textarea collapse",
            description: "Textarea shrinks when a file is selected, expands on click. Delete file button added.",
        },
    ]
}

#[derive(Template)]
#[template(path = "ui/base.html")]
struct DashboardTemplate {
    version: String,
    translations: HashMap<String, String>,
    changelog: Vec<ChangelogEntry>,
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
        changelog: current_changelog(),
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
