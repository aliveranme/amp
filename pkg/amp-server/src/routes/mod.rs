pub mod admin;
pub mod auth;
pub mod chat;
pub mod session;
pub mod thread;

use std::collections::HashMap;

use axum::http::HeaderMap;
use sqlx::sqlite::SqlitePool;
use tokio::sync::Mutex;

use amp_core::AppConfig;
use amp_proxy::router::Router;

pub struct AppState {
    pub config: AppConfig,
    pub router: Router,
    pub pool: SqlitePool,
    pub client: reqwest::Client,
    /// Auth code store: code → auth_token
    pub auth_codes: Mutex<HashMap<String, String>>,
}

/// Extract API key from Authorization: Bearer or x-api-key header.
pub fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.strip_prefix("Bearer ").unwrap_or(v).to_string())
        .or_else(|| {
            headers.get("x-api-key")?.to_str().ok().map(|s| s.to_string())
        })
}
