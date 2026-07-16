use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Storage error: {0}")]
    Storage(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Upstream API error: {0}")]
    Upstream(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::NotFound(_) => (axum::http::StatusCode::NOT_FOUND, self.to_string()),
            AppError::InvalidRequest(_) => (axum::http::StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Config(_) => (axum::http::StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Upstream(_) => (axum::http::StatusCode::BAD_GATEWAY, self.to_string()),
            _ => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, axum::Json(serde_json::json!({"error": message}))).into_response()
    }
}
