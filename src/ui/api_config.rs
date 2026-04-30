use axum::response::Html;

pub async fn api_config_get() -> Html<String> {
    Html("<div class='result'><p>Config API ready (plan 04 implements)</p></div>".to_string())
}

pub async fn api_config_put() -> Html<String> {
    Html("<div class='result'><p>Config saved (plan 04 implements)</p></div>".to_string())
}
