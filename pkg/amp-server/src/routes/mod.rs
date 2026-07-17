pub mod admin;
pub mod auth;
pub mod chat;
pub mod session;
pub mod thread;

use std::collections::HashMap;

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
