use futures::Stream;
use reqwest::header::HeaderValue;
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

        let http_request = client
            .post(&endpoint)
            .headers(headers)
            .json(&request)
            .send()
            .await;

        match http_request {
            Ok(resp) => {
                if !resp.status().is_success() {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    let _ = tx
                        .send(Err(super::ProxyError::Upstream(format!(
                            "HTTP {status}: {body}"
                        ))))
                        .await;
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
                                if line.is_empty() || line == "data: [DONE]" {
                                    continue;
                                }
                                if let Some(data) = line.strip_prefix("data: ") {
                                    if let Ok(chunk) = serde_json::from_str::<ChatChunk>(data) {
                                        let _ = tx.send(Ok(chunk)).await;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx
                                .send(Err(super::ProxyError::Upstream(e.to_string())))
                                .await;
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                let _ = tx
                    .send(Err(super::ProxyError::Upstream(e.to_string())))
                    .await;
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}
