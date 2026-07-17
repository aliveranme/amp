use std::sync::Arc;

use axum::{
    extract::State,
    response::sse::{Event, Sse},
    Json,
};
use axum::http::HeaderMap;
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
        match users::find_user_by_key(&state.pool, key).await {
            Ok(u) => u,
            Err(e) => {
                tracing::warn!("DB error looking up user: {e}");
                None
            }
        }
    } else { None };

    // Build stream: use stream_chat_completion (w/ extra_headers) for default routes,
    // stream_chat_completion_simple for user DB routes.
    let stream: std::pin::Pin<Box<dyn Stream<Item = Result<amp_proxy::transformer::ChatChunk, amp_proxy::ProxyError>> + Send>> = if let Some(ref u) = user {
        match users::find_user_route(&state.pool, &u.user_id, model).await {
            Ok(Some(r)) if !r.api_key_encrypted.is_empty() => {
                // User DB route — use simple path (no extra_headers)
                let s = amp_proxy::streamer::stream_chat_completion_simple(
                    state.client.clone(),
                    r.endpoint,
                    r.auth_header,
                    r.api_key_encrypted,
                    req,
                );
                Box::pin(s)
            }
            _ => {
                // Fallback to global config — use full path with extra_headers
                let route = default_upstream(&state, model);
                let key = state.config.api_key.clone();
                Box::pin(amp_proxy::streamer::stream_chat_completion(
                    state.client.clone(),
                    route,
                    key,
                    None,
                    req,
                ))
            }
        }
    } else {
        let route = default_upstream(&state, model);
        let key = state.config.api_key.clone();
        Box::pin(amp_proxy::streamer::stream_chat_completion(
            state.client.clone(),
            route,
            key,
            None,
            req,
        ))
    };

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
