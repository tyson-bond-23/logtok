use axum::response::Html;

pub async fn api_store() -> Html<String> {
    Html("<div class='result'><p>Store API ready (plan 04 implements)</p></div>".to_string())
}

pub async fn api_docs() -> Html<String> {
    Html("<div class='result'><p>Docs API ready (plan 04 implements)</p></div>".to_string())
}
