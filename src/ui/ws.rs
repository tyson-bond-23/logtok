use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use std::sync::Arc;

use crate::ui::AppState;

pub async fn ws_heartbeat(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_heartbeat(socket, state))
}

async fn handle_heartbeat(mut socket: WebSocket, state: Arc<AppState>) {
    // Client sends ping every 5 seconds
    // If no message received for 15 seconds, trigger shutdown
    loop {
        match tokio::time::timeout(std::time::Duration::from_secs(15), socket.recv()).await {
            Ok(Some(Ok(Message::Text(_)))) => continue,
            Ok(Some(Ok(Message::Ping(_)))) => continue,
            _ => break,
        }
    }
    // Trigger graceful shutdown
    if let Some(tx) = state.shutdown_tx.lock().await.take() {
        let _ = tx.send(());
    }
}
