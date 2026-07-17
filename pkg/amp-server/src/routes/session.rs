use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
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
) -> Result<Json<Session>, (StatusCode, String)> {
    let thread_id = Uuid::parse_str(&req.thread_id).map_err(|_| {
        (StatusCode::BAD_REQUEST, format!("Invalid thread_id: {}", req.thread_id))
    })?;
    let mode = req.agent_mode.as_deref().unwrap_or("medium");
    let session = amp_storage::sqlite::create_session(&state.pool, &thread_id, mode)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create session: {e}"))
        })?;
    Ok(Json(session))
}
