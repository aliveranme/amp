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

use super::{extract_api_key, AppState};

/// Build default route from global config (fallback when no user or no user route)
fn default_upstream(state: &Arc<AppState>, model: &str) -> (String, String, String) {
    let route = state.router.route(model);
    let endpoint = state.config.url.clone().unwrap_or_else(|| route.endpoint.clone());
    (endpoint, state.config.api_key.clone(), "Authorization".to_string())
}

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
        users::find_user_by_key(&state.pool, key).await.unwrap_or(None)
    } else { None };

    // Determine upstream endpoint + auth parameters
    let (upstream_endpoint, provider_key, auth_header) = if let Some(ref u) = user {
        match users::find_user_route(&state.pool, &u.user_id, model).await {
            Ok(Some(r)) if !r.api_key_encrypted.is_empty() => {
                (r.endpoint, r.api_key_encrypted, r.auth_header)
            }
            _ => default_upstream(&state, model),
        }
    } else {
        default_upstream(&state, model)
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
