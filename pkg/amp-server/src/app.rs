use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;

use amp_core::AppConfig;
use amp_proxy::router::Router as AmpRouter;
use amp_storage;

use super::routes::{chat, session, thread, AppState};

pub async fn create(config: AppConfig) -> Router {
    // Init storage
    let pool = amp_storage::sqlite::init_pool(&config.db_path)
        .await
        .expect("Failed to initialize database");

    // Load route config
    let route_router = if let Some(path) = &config.route_config_path {
        AmpRouter::from_config(path).expect("Invalid route config")
    } else {
        let route_config: amp_core::RouteConfig = toml::from_str(include_str!("../../../route-config.toml"))
            .expect("Invalid default route config");
        AmpRouter::from_hashmap(route_config.routes).expect("Invalid default route config")
    };

    let client = reqwest::Client::new();

    let state = Arc::new(AppState {
        config,
        router: route_router,
        pool,
        client,
    });

    Router::new()
        .route("/health", get(health))
        .route("/v1/chat/completions", post(chat::chat_completion))
        .route("/api/threads", get(thread::list_threads))
        .route("/api/threads", post(thread::create_thread))
        .route("/api/sessions", post(session::create_session))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok"}))
}
