use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use amp_core::AppError;
use amp_storage::users;
use amp_storage::users::{UserRouteRow, UserRow};

use super::AppState;

// ─── Dashboard Stats ────────────────────────────────────────────

pub async fn dashboard_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_count = users::user_count(&state.pool).await?;
    let route_count = users::route_count(&state.pool).await?;
    let usage = users::get_global_usage(&state.pool, 7).await?;
    Ok(Json(serde_json::json!({
        "user_count": user_count,
        "route_count": route_count,
        "usage_7d": {
            "total_requests": usage.total_requests,
            "total_tokens_in": usage.total_tokens_in,
            "total_tokens_out": usage.total_tokens_out,
        }
    })))
}

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
        api_key: user.api_key, user_id: user.user_id, name: user.name,
    }))
}

#[derive(Serialize)]
pub struct ListUsersResponse { pub users: Vec<UserRow> }

pub async fn list_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ListUsersResponse>, AppError> {
    let users = users::list_users(&state.pool).await?;
    Ok(Json(ListUsersResponse { users }))
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<UserRow>, AppError> {
    let user = users::get_user_by_id(&state.pool, &user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;
    Ok(Json(user))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let deleted = users::delete_user(&state.pool, &user_id).await?;
    Ok(Json(serde_json::json!({"deleted": deleted})))
}

#[derive(Deserialize)]
pub struct UpdateUserNameRequest { pub name: String }

pub async fn update_user_name(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateUserNameRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let updated = users::update_user_name(&state.pool, &user_id, &req.name).await?;
    Ok(Json(serde_json::json!({"updated": updated})))
}

// ─── Usage Stats ────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UsageQuery { pub days: Option<i32> }

pub async fn get_user_usage(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Query(q): Query<UsageQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let days = q.days.unwrap_or(7);
    let usage = users::get_user_usage(&state.pool, &user_id, days).await?;
    Ok(Json(serde_json::json!(usage)))
}

pub async fn get_global_usage(
    State(state): State<Arc<AppState>>,
    Query(q): Query<UsageQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let days = q.days.unwrap_or(7);
    let usage = users::get_global_usage(&state.pool, days).await?;
    Ok(Json(serde_json::json!(usage)))
}

// ─── Route CRUD ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateRouteRequest {
    pub model: String,
    pub provider: String,
    pub endpoint: String,
    pub api_key: Option<String>,
    pub auth_header: Option<String>,
    pub enabled: Option<bool>,
    pub rate_limit: Option<i64>,
    pub max_tokens: Option<i64>,
}

pub async fn create_route(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(req): Json<CreateRouteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    users::upsert_user_route(
        &state.pool, &user_id,
        &req.model, &req.provider, &req.endpoint,
        &req.api_key.unwrap_or_default(),
        &req.auth_header.unwrap_or_else(|| "Authorization".to_string()),
        req.enabled.unwrap_or(true),
        req.rate_limit.unwrap_or(0),
        req.max_tokens.unwrap_or(0),
    ).await?;
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
) -> Result<Json<serde_json::Value>, AppError> {
    let deleted = users::delete_user_route(&state.pool, &user_id, &model).await?;
    Ok(Json(serde_json::json!({"deleted": deleted})))
}

#[derive(Deserialize)]
pub struct ToggleRouteRequest { pub enabled: bool }

pub async fn toggle_route(
    State(state): State<Arc<AppState>>,
    Path((user_id, model)): Path<(String, String)>,
    Json(req): Json<ToggleRouteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ok = users::set_route_enabled(&state.pool, &user_id, &model, req.enabled).await?;
    Ok(Json(serde_json::json!({"updated": ok, "enabled": req.enabled})))
}
