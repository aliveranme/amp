use std::sync::Arc;

use axum::{
    extract::State,
    response::sse::{Event, Sse},
    Json,
};
use futures::stream::Stream;
use futures::StreamExt;

use amp_proxy::transformer::ChatRequest;

use super::AppState;

pub async fn chat_completion(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let model = if req.model.is_empty() {
        &state.config.default_model
    } else {
        &req.model
    };
    let route = state.router.route(model).clone();

    let stream = amp_proxy::streamer::stream_chat_completion(
        state.client.clone(),
        route,
        state.config.api_key.clone(),
        state.config.url.clone(),
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
