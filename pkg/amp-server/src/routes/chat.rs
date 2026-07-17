use std::sync::Arc;

use axum::{
    extract::State,
    response::sse::{Event, Sse},
    Json,
};
use axum::http::HeaderMap;
use futures::stream::Stream;
use tokio_stream::StreamExt;

use amp_proxy::transformer::ChatRequest;
use amp_storage::users;

use super::AppState;

/// POST /v1/chat/completions — route by user config, fallback to global defaults
pub async fn chat_completion(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let model = if req.model.is_empty() { &state.config.default_model } else { &req.model };

    // Identify user from API key
    let api_key = extract_api_key(&headers);
    let user = if let Some(ref key) = api_key {
        users::find_user_by_key(&state.pool, key).await
    } else { None };

    // Determine upstream endpoint + API key
    let (upstream_endpoint, provider_key, auth_header) = if let Some(ref u) = user {
        match users::find_user_route(&state.pool, &u.user_id, model).await {
            Ok(Some(r)) if !r.api_key_encrypted.is_empty() => {
                (r.endpoint, r.api_key_encrypted, r.auth_header)
            }
            _ => {
                let route = state.router.route(model);
                (route.endpoint.clone(), state.config.api_key.clone(), "Authorization".to_string())
            }
        }
    } else {
        let route = state.router.route(model);
        (route.endpoint.clone(), state.config.api_key.clone(), "Authorization".to_string())
    };

    let effective_key = if provider_key.is_empty() { state.config.api_key.clone() } else { provider_key };

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

    Sse::new(event_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text(": keepalive"),
    )
}

fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.strip_prefix("Bearer ").unwrap_or(v).to_string())
        .or_else(|| {
            headers
                .get("x-api-key")?
                .to_str()
                .ok()
                .map(|s| s.to_string())
        })
}
