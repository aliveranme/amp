use std::sync::Arc;

use axum::{extract::State, Json};
use serde::Deserialize;

use amp_core::Thread;
use amp_storage;

use super::AppState;

#[derive(Deserialize)]
pub struct CreateThreadRequest {
    pub title: Option<String>,
}

pub async fn list_threads(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<Thread>> {
    let threads = amp_storage::sqlite::list_threads(&state.pool)
        .await
        .unwrap_or_default();
    Json(threads)
}

pub async fn create_thread(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateThreadRequest>,
) -> Json<Thread> {
    let title = req.title.unwrap_or_default();
    let thread = amp_storage::sqlite::create_thread(&state.pool, &title)
        .await
        .expect("Failed to create thread");
    Json(thread)
}
