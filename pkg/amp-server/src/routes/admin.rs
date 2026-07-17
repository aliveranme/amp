use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use amp_core::AppError;
use amp_storage::users;
use amp_storage::users::{UserRouteRow, UserRow};

use super::AppState;

// ─── User CRUD ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct CreateUserResponse {
    pub api_key: String,
    pub user_id: String,
    pub name: String,
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, AppError> {
    let name = req.name.unwrap_or_else(|| "default".to_string());
    let user = users::create_user(&state.pool, &name).await?;
    Ok(Json(CreateUserResponse {
        api_key: user.api_key,
        user_id: user.user_id,
        name: user.name,
    }))
}

#[derive(Serialize)]
pub struct ListUsersResponse {
    pub users: Vec<UserRow>,
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ListUsersResponse>, AppError> {
    let users = users::list_users(&state.pool).await?;
    Ok(Json(ListUsersResponse { users }))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Json<serde_json::Value> {
    let deleted = users::delete_user(&state.pool, &user_id).await;
    Json(serde_json::json!({"deleted": deleted}))
}

// ─── Route CRUD ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateRouteRequest {
    pub model: String,
    pub provider: String,
    pub endpoint: String,
    pub api_key: Option<String>,
    pub auth_header: Option<String>,
}

pub async fn create_route(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(req): Json<CreateRouteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    users::upsert_user_route(
        &state.pool,
        &user_id,
        &req.model,
        &req.provider,
        &req.endpoint,
        &req.api_key.unwrap_or_default(),
        &req.auth_header.unwrap_or_else(|| "Authorization".to_string()),
    )
    .await?;
    Ok(Json(serde_json::json!({"status": "ok", "model": req.model, "user_id": user_id})))
}

pub async fn list_routes(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<UserRouteRow>>, AppError> {
    let routes = users::get_user_routes(&state.pool, &user_id).await?;
    Ok(Json(routes))
}

pub async fn delete_route(
    State(state): State<Arc<AppState>>,
    Path((user_id, model)): Path<(String, String)>,
) -> Json<serde_json::Value> {
    let deleted = users::delete_user_route(&state.pool, &user_id, &model).await;
    Json(serde_json::json!({"deleted": deleted}))
}
