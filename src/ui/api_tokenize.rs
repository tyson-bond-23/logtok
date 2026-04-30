use axum::response::Html;

pub async fn api_tokenize() -> Html<String> {
    Html("<div class='result'><p>Tokenize API ready (plan 03 implements)</p></div>".to_string())
}

pub async fn api_detokenize() -> Html<String> {
    Html("<div class='result'><p>Detokenize API ready (plan 03 implements)</p></div>".to_string())
}
