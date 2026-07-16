use axum::routing::get;
use axum::Router;
use tower_http::cors::CorsLayer;

pub async fn create() -> Router {
    Router::new()
        .route("/health", get(health))
        .layer(CorsLayer::permissive())
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok"}))
}
