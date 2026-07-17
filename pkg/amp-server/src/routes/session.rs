use std::sync::Arc;

use axum::{extract::State, Json};
use serde::Deserialize;
use uuid::Uuid;

use amp_core::Session;
use amp_storage;

use super::AppState;

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub thread_id: String,
    pub agent_mode: Option<String>,
}

pub async fn create_session(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSessionRequest>,
) -> Json<Session> {
    let thread_id = Uuid::parse_str(&req.thread_id).expect("Invalid thread_id");
    let mode = req.agent_mode.as_deref().unwrap_or("medium");
    let session = amp_storage::sqlite::create_session(&state.pool, &thread_id, mode)
        .await
        .expect("Failed to create session");
    Json(session)
}
