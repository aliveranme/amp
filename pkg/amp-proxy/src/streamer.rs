use futures::Stream;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::Client;
use tokio::sync::mpsc;

use amp_core::ModelRoute;

use super::injector::build_headers;
use super::transformer::{ChatChunk, ChatRequest};

pub fn stream_chat_completion(
    client: Client,
    route: ModelRoute,
    api_key: String,
    api_url: Option<String>,
    request: ChatRequest,
) -> impl Stream<Item = Result<ChatChunk, super::ProxyError>> + Send + 'static {
    let (tx, rx) = mpsc::channel::<Result<ChatChunk, super::ProxyError>>(64);

    tokio::spawn(async move {
        let endpoint = api_url.unwrap_or(route.endpoint.clone());
        let mut headers = build_headers(&route, &api_key);
        headers.insert("content-type", HeaderValue::from_static("application/json"));

        let http_request = client.post(&endpoint).headers(headers).json(&request).send().await;
        forward_stream(http_request, tx).await;
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}

/// Simplified variant: takes endpoint + auth header name + API key directly.
/// Used when routing through per-user config from the database.
pub fn stream_chat_completion_simple(
    client: Client,
    endpoint: String,
    auth_header: String,
    api_key: String,
    request: ChatRequest,
) -> impl Stream<Item = Result<ChatChunk, super::ProxyError>> + Send + 'static {
    let (tx, rx) = mpsc::channel::<Result<ChatChunk, super::ProxyError>>(64);

    tokio::spawn(async move {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(auth_header.as_bytes()),
            HeaderValue::from_str(&api_key),
        ) {
            headers.insert(name, val);
        }

        let http_request = client.post(&endpoint).headers(headers).json(&request).send().await;
        forward_stream(http_request, tx).await;
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}

/// Shared SSE response reader.
async fn forward_stream(
    http_request: Result<reqwest::Response, reqwest::Error>,
    tx: mpsc::Sender<Result<ChatChunk, super::ProxyError>>,
) {
    match http_request {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                let _ = tx.send(Err(super::ProxyError::Upstream(format!("HTTP {status}: {body}")))).await;
                return;
            }

            let mut stream = resp.bytes_stream();
            use futures::StreamExt;
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        for line in text.lines() {
                            let line = line.trim();
                            if line.is_empty() || line == "data: [DONE]" { continue; }
                            if let Some(data) = line.strip_prefix("data: ") {
                                match serde_json::from_str::<ChatChunk>(data) {
                                    Ok(chunk) => { if tx.send(Ok(chunk)).await.is_err() { return; } }
                                    Err(e) => tracing::warn!("Failed to parse SSE: {e}, data: {data}"),
                                }
                            }
                        }
                    }
                    Err(e) => { let _ = tx.send(Err(super::ProxyError::Upstream(e.to_string()))).await; break; }
                }
            }
        }
        Err(e) => { let _ = tx.send(Err(super::ProxyError::Upstream(e.to_string()))).await; }
    }
}
