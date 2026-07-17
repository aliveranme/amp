use std::sync::Arc;

use axum::{
    extract::State,
    response::sse::{Event, Sse},
    Json,
};
use axum::http::{HeaderMap, StatusCode};
use futures::stream::Stream;
use tokio_stream::StreamExt;

use amp_core::ModelRoute;
use amp_proxy::transformer::ChatRequest;
use amp_storage::users;

use super::{extract_api_key, AppState};

/// Build default route from global config (fallback when no user or no user route)
fn default_upstream(state: &Arc<AppState>, model: &str) -> ModelRoute {
    let route = state.router.route(model);
    let mut r = route.clone();
    if let Some(ref url) = state.config.url {
        r.endpoint = url.clone();
    }
    r
}

/// Simple rate limiter: check requests in the last 60s.
/// Returns true if allowed, false if rate limited.
async fn check_rate_limit(state: &Arc<AppState>, user_id: &str, limit: i64) -> bool {
    if limit <= 0 { return true; }
    let row: Result<Option<(i64,)>, _> = sqlx::query_as(
        "SELECT COUNT(*) FROM usage_logs WHERE user_id = ? AND created_at > datetime('now', '-1 minute')"
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await;
    match row {
        Ok(Some((c,))) => c < limit,
        _ => true,
    }
}

/// POST /v1/chat/completions — route by user config, fallback to global defaults
pub async fn chat_completion(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, (StatusCode, Json<serde_json::Value>)> {
    let model = if req.model.is_empty() { &state.config.default_model } else { &req.model };

    // Identify user from API key
    let api_key = extract_api_key(&headers);
    let (user_id, _rate_limit) = if let Some(ref key) = api_key {
        match users::find_user_by_key(&state.pool, key).await {
            Ok(Some(u)) => (Some(u.user_id), None::<i64>),
            _ => (None, None),
        }
    } else { (None, None) };

    let (upstream_endpoint, provider_key, auth_header, provider_name) = if let Some(ref uid) = user_id {
        match users::find_user_route(&state.pool, uid, model).await {
            Ok(Some(r)) if r.enabled && !r.api_key_encrypted.is_empty() => {
                // Rate limit check
                if !check_rate_limit(&state, uid, r.rate_limit).await {
                    return Err((StatusCode::TOO_MANY_REQUESTS, Json(serde_json::json!({
                        "error": "rate_limit_exceeded",
                        "message": format!("超过限流: {} 请求/分钟", r.rate_limit)
                    }))));
                }
                (r.endpoint, r.api_key_encrypted, r.auth_header, r.provider)
            }
            Ok(Some(r)) if !r.enabled => {
                return Err((StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                    "error": "route_disabled",
                    "message": "该路由已停用"
                }))));
            }
            _ => {
                let route = default_upstream(&state, model);
                (route.endpoint, state.config.api_key.clone(), "Authorization".to_string(), route.provider)
            }
        }
    } else {
        let route = default_upstream(&state, model);
        (route.endpoint, state.config.api_key.clone(), "Authorization".to_string(), route.provider)
    };

    let effective_key = if provider_key.is_empty() { state.config.api_key.clone() } else { provider_key };

    // Log usage asynchronously
    if let Some(ref uid) = user_id {
        let pool = state.pool.clone();
        let m = model.to_string();
        let p = provider_name.clone();
        let uid_clone = uid.clone();
        tokio::spawn(async move {
            let _ = users::log_usage(&pool, &uid_clone, &m, &p, 0, 0, 0, "success").await;
        });
    }

    let stream = amp_proxy::streamer::stream_chat_completion_simple(
        state.client.clone(),
        upstream_endpoint,
        auth_header,
        effective_key,
        req,
    );

    let event_stream = stream.map(|result| match result {
        Ok(chunk) => {
            let json = serde_json::to_string(&chunk).unwrap_or_default();
            Ok(Event::default().data(json))
        }
        Err(e) => Ok(Event::default().data(format!("error: {e}"))),
    });

    Ok(Sse::new(event_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text(": keepalive"),
    ))
}
