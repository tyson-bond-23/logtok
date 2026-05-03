use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::ui::AppState;

pub async fn ws_heartbeat(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_heartbeat(socket, state))
}

async fn handle_heartbeat(mut socket: WebSocket, state: Arc<AppState>) {
    // Track this connection
    state.ws_connections.fetch_add(1, Ordering::SeqCst);

    // Client sends ping every 5 seconds
    // If no message received for 15 seconds, this connection is dead
    loop {
        match tokio::time::timeout(std::time::Duration::from_secs(15), socket.recv()).await {
            Ok(Some(Ok(Message::Text(_)))) => continue,
            Ok(Some(Ok(Message::Ping(_)))) => continue,
            _ => break,
        }
    }

    // Connection lost — decrement counter
    state.ws_connections.fetch_sub(1, Ordering::SeqCst);

    // Grace period: wait for a new connection (e.g., page reload)
    // before triggering shutdown
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Only shutdown if no connections remain after the grace period
    if state.ws_connections.load(Ordering::SeqCst) == 0 {
        if let Some(tx) = state.shutdown_tx.lock().await.take() {
            let _ = tx.send(());
        }
    }
}
