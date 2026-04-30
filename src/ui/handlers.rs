use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "ui/base.html")]
struct DashboardTemplate {
    version: String,
}

pub async fn dashboard() -> Result<Html<String>, axum::http::StatusCode> {
    let template = DashboardTemplate {
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    template
        .render()
        .map(Html)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}
