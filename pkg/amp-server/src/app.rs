use std::collections::HashMap;
use std::sync::Arc;

use axum::routing::{delete, get, patch, post};
use axum::Router;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::services::fs::ServeFile;

use amp_core::AppConfig;
use amp_proxy::router::Router as AmpRouter;
use amp_storage;

use super::routes::{admin, auth, chat, session, thread, AppState};

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
        auth_codes: Mutex::new(HashMap::new()),
    });

    // Static frontend from web/out/ (if exists)
    let frontend_path = std::path::Path::new("web/out");
    let fallback = if frontend_path.exists() {
        // SPA-style: unmatched paths fall back to index.html
        let index_file = ServeFile::new("web/out/index.html");
        Some(ServeDir::new("web/out").fallback(index_file))
    } else {
        None
    };

    let mut app = Router::new()
        .route("/health", get(health))
        // Auth — amp CLI login flow
        .route("/auth/cli-login", get(auth::cli_login))
        .route("/auth/token", post(auth::exchange_token))
        .route("/api/user", get(auth::get_user))
        // Admin API — multi-user management
        .route("/admin/api/stats", get(admin::dashboard_stats))
        .route("/admin/api/users", get(admin::list_users))
        .route("/admin/api/users", post(admin::create_user))
        .route("/admin/api/users/{user_id}", delete(admin::delete_user))
        .route("/admin/api/users/{user_id}", patch(admin::update_user_name))
        .route("/admin/api/users/{user_id}/routes", get(admin::list_routes))
        .route("/admin/api/users/{user_id}/routes", post(admin::create_route))
        .route("/admin/api/users/{user_id}/routes/{model}", delete(admin::delete_route))
        // Proxy
        .route("/v1/chat/completions", post(chat::chat_completion))
        // Threads
        .route("/api/threads", get(thread::list_threads))
        .route("/api/threads", post(thread::create_thread))
        .route("/api/sessions", post(session::create_session))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Mount static frontend as fallback (serves /, /admin, etc.)
    if let Some(fs) = fallback {
        app = app.fallback_service(fs);
    }

    app
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok"}))
}
