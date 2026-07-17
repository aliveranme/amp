use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Query, State};
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

/// POST /auth/token?code=xxx — exchange code for token
pub async fn exchange_token(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    // Direct API key auth
    if let Some(api_key) = query.get("api_key") {
        if let Some(user) = users::find_user_by_key(&state.pool, api_key).await {
            return Json(serde_json::json!({
                "access_token": Uuid::new_v4().to_string(),
                "token_type": "Bearer", "expires_in": 86400,
                "user": { "id": user.user_id, "name": user.name }
            }));
        }
    }
    // Code exchange
    if let Some(code) = query.get("code") {
        state.auth_codes.lock().await.remove(code);
    }
    Json(serde_json::json!({
        "access_token": Uuid::new_v4().to_string(),
        "token_type": "Bearer", "expires_in": 86400,
        "user": { "id": "byok-user", "name": "BYOK User" }
    }))
}

/// GET /api/user — return user info (called by amp CLI after auth)
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Json<serde_json::Value> {
    let api_key = extract_api_key(&headers);

    if let Some(ref key) = api_key {
        if let Some(user) = users::find_user_by_key(&state.pool, key).await {
            return Json(serde_json::json!({
                "id": user.user_id, "name": user.name,
                "email": format!("{}@byok.local", user.user_id), "credits": 999999.0
            }));
        }
    }
    Json(serde_json::json!({
        "id":"anonymous","name":"Anonymous","email":"anon@byok.local","credits":0.0
    }))
}

fn extract_api_key(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.strip_prefix("Bearer ").unwrap_or(v).to_string())
        .or_else(|| headers.get("x-api-key")?.to_str().ok().map(|s| s.to_string()))
}
