pub mod chat;
pub mod session;
pub mod thread;

use sqlx::sqlite::SqlitePool;

use amp_core::AppConfig;
use amp_proxy::router::Router;

pub struct AppState {
    pub config: AppConfig,
    pub router: Router,
    pub pool: SqlitePool,
    pub client: reqwest::Client,
}
