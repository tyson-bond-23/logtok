mod api_config;
mod api_store;
mod api_tokenize;
mod assets;
mod handlers;
mod i18n;
mod routes;
mod ws;

use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};

pub struct AppState {
    pub shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
    pub session_key: String,
}

pub async fn start_server(port: Option<u16>) -> Result<()> {
    // D-19: Auto-generate encryption key per session
    let session_key = generate_session_key();
    // If LOGTOK_KEY is set, use it instead (for CLI interop)
    #[allow(deprecated)]
    let key = std::env::var("LOGTOK_KEY").unwrap_or_else(|_| {
        // Set it so Store::new() picks it up
        #[allow(deprecated)]
        unsafe {
            std::env::set_var("LOGTOK_KEY", &session_key);
        }
        eprintln!("logtok: session key generated (set LOGTOK_KEY for CLI interop)");
        session_key
    });

    // D-20: Bind to 127.0.0.1 only
    let addr = find_available_port(port.unwrap_or(8080))?;
    let listener = TcpListener::bind(addr).await?;
    let actual_port = listener.local_addr()?.port();

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let state = Arc::new(AppState {
        shutdown_tx: Mutex::new(Some(shutdown_tx)),
        session_key: key,
    });
    let app = routes::build_router(state);

    eprintln!("logtok: dashboard at http://127.0.0.1:{}", actual_port);

    // D-21: Auto-open browser
    let url = format!("http://127.0.0.1:{}", actual_port);
    if let Err(e) = open::that(&url) {
        eprintln!(
            "logtok: could not open browser ({}), visit {} manually",
            e, url
        );
    }

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        })
        .await?;

    eprintln!("logtok: server stopped");
    Ok(())
}

fn find_available_port(preferred: u16) -> Result<SocketAddr> {
    for port in preferred..preferred.saturating_add(100) {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        if std::net::TcpListener::bind(addr).is_ok() {
            return Ok(addr);
        }
    }
    anyhow::bail!(
        "No available port in range {}..{}",
        preferred,
        preferred.saturating_add(100)
    )
}

fn generate_session_key() -> String {
    use rand::RngCore;
    let mut key_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut key_bytes);
    hex::encode(key_bytes)
}
