use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};

/// Start a test server on a random port and return the base URL and shutdown sender.
async fn start_test_server() -> (String, oneshot::Sender<()>) {
    // Set LOGTOK_KEY for test
    #[allow(deprecated)]
    unsafe {
        std::env::set_var("LOGTOK_KEY", "test-key-for-integration-tests-0123456789abcdef");
    }

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let state = Arc::new(logtok::ui::AppState {
        shutdown_tx: Mutex::new(None),
        session_key: "test-key-for-integration-tests-0123456789abcdef".to_string(),
    });

    let app = logtok::ui::routes::build_router(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async { let _ = shutdown_rx.await; })
            .await
            .unwrap();
    });

    (format!("http://127.0.0.1:{}", port), shutdown_tx)
}

#[tokio::test]
async fn test_dashboard_returns_html() {
    let (base_url, _shutdown) = start_test_server().await;
    let resp = reqwest::get(&base_url).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("logtok"), "Dashboard should contain 'logtok'");
    assert!(body.contains("</html>"), "Dashboard should be HTML");
}

#[tokio::test]
async fn test_static_css_served() {
    let (base_url, _shutdown) = start_test_server().await;
    let resp = reqwest::get(format!("{}/static/styles.css", base_url))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("x-cloak") || body.contains("htmx-indicator"), "CSS should contain base styles");
}

#[tokio::test]
async fn test_static_js_served() {
    let (base_url, _shutdown) = start_test_server().await;
    let resp = reqwest::get(format!("{}/static/app.js", base_url))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("function app()"),
        "JS should contain app function"
    );
}

#[tokio::test]
async fn test_api_store_empty() {
    let (base_url, _shutdown) = start_test_server().await;
    let resp = reqwest::get(format!("{}/api/store", base_url))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_api_docs_returns_content() {
    let (base_url, _shutdown) = start_test_server().await;
    let resp = reqwest::get(format!("{}/api/docs", base_url))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(!body.is_empty(), "Docs response should not be empty");
}

#[tokio::test]
async fn test_api_config_returns_content() {
    let (base_url, _shutdown) = start_test_server().await;
    let resp = reqwest::get(format!("{}/api/config", base_url))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(!body.is_empty(), "Config response should not be empty");
}

#[tokio::test]
async fn test_api_tokenize_with_content() {
    let (base_url, _shutdown) = start_test_server().await;
    let client = reqwest::Client::new();
    let form = reqwest::multipart::Form::new()
        .text("content", "Connection from 192.168.1.1 by admin@example.com");
    let resp = client
        .post(format!("{}/api/tokenize", base_url))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("[IP_") || body.contains("token") || body.contains("pre"),
        "Tokenize should return result HTML"
    );
}

#[tokio::test]
async fn test_api_tokenize_no_input() {
    let (base_url, _shutdown) = start_test_server().await;
    let client = reqwest::Client::new();
    let form = reqwest::multipart::Form::new().text("content", "");
    let resp = client
        .post(format!("{}/api/tokenize", base_url))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("error") || body.contains("No input"),
        "Empty input should return error message"
    );
}

#[tokio::test]
async fn test_static_htmx_served() {
    let (base_url, _shutdown) = start_test_server().await;
    let resp = reqwest::get(format!("{}/static/htmx.min.js", base_url))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}
