use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use amp_storage::users;

use super::AppState;

#[derive(Deserialize)]
pub struct LoginQuery {
    #[serde(rename = "authToken")]
    pub auth_token: String,
}

/// GET /auth/cli-login?authToken=xxx — amp CLI browser auth
pub async fn cli_login(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LoginQuery>,
) -> Html<String> {
    let code = Uuid::new_v4().to_string();
    let mut store = state.auth_codes.lock().await;
    store.insert(code.clone(), query.auth_token.clone());
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<body style="font-family:sans-serif;padding:2em;text-align:center">
<h1>amp code BYOK</h1>
<p>认证成功！请在终端粘贴以下代码：</p>
<pre style="background:#f4f4f4;padding:1em;font-size:1.5em;display:inline-block">{code}</pre>
<p style="color:#666;margin-top:2em">仅一次性有效</p>
</body></html>"#
    ))
}

/// POST /auth/token — exchange api_key or code for access_token
pub async fn exchange_token(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // 1. Try direct API key auth
    if let Some(api_key) = query.get("api_key") {
        match users::find_user_by_key(&state.pool, api_key).await {
            Ok(Some(user)) => {
                return Ok(Json(serde_json::json!({
                    "access_token": Uuid::new_v4().to_string(),
                    "token_type": "Bearer", "expires_in": 86400,
                    "user": { "id": user.user_id, "name": user.name }
                })));
            }
            Ok(None) => {
                return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                    "error": "invalid_api_key", "message": "API key 无效"
                }))));
            }
            Err(e) => {
                tracing::error!("DB error during auth: {e}");
                return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": "server_error", "message": "认证服务内部错误"
                }))));
            }
        }
    }

    // 2. Code exchange (flow from cli_login page)
    if let Some(code) = query.get("code") {
        state.auth_codes.lock().await.remove(code);
        return Ok(Json(serde_json::json!({
            "access_token": Uuid::new_v4().to_string(),
            "token_type": "Bearer", "expires_in": 86400,
            "user": { "id": "byok-user", "name": "BYOK User" }
        })));
    }

    Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({
        "error": "missing_parameter", "message": "缺少 api_key 或 code 参数"
    }))))
}

/// GET /api/user — return user info (called by amp CLI after auth)
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let api_key = super::extract_api_key(&headers);

    if let Some(ref key) = api_key {
        match users::find_user_by_key(&state.pool, key).await {
            Ok(Some(user)) => {
                return Ok(Json(serde_json::json!({
                    "id": user.user_id, "name": user.name,
                    "email": format!("{}@byok.local", user.user_id), "credits": 999999.0
                })));
            }
            Ok(None) => {} // fall through to anonymous
            Err(e) => {
                tracing::error!("DB error in get_user: {e}");
                return Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": "server_error"
                }))));
            }
        }
    }
    Ok(Json(serde_json::json!({
        "id":"anonymous","name":"Anonymous","email":"anon@byok.local","credits":0.0
    })))
}
