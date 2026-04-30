use axum::routing::{get, post, put};
use axum::Router;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;

use crate::ui::api_config::{api_config_get, api_config_put};
use crate::ui::api_store::{api_docs, api_store};
use crate::ui::api_tokenize::{api_detokenize, api_tokenize};
use crate::ui::assets::static_handler;
use crate::ui::handlers::{api_translations, dashboard};
use crate::ui::ws::ws_heartbeat;
use crate::ui::AppState;

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Page routes
        .route("/", get(dashboard))
        // API routes (return HTML fragments for HTMX)
        .route("/api/tokenize", post(api_tokenize))
        .route("/api/detokenize", post(api_detokenize))
        .route("/api/store", get(api_store))
        .route("/api/docs", get(api_docs))
        .route("/api/config", get(api_config_get))
        .route("/api/config", put(api_config_put))
        .route("/api/translations", get(api_translations))
        // WebSocket heartbeat for auto-stop (D-22)
        .route("/ws/heartbeat", get(ws_heartbeat))
        // Static assets served via rust-embed (D-15)
        .route("/static/{*path}", get(static_handler))
        // Middleware (D-17)
        .layer(CompressionLayer::new())
        .with_state(state)
}
