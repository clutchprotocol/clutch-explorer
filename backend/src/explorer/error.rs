use crate::explorer::models::ApiErrorDto;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExplorerError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("upstream error: {0}")]
    Upstream(String),
    #[error("storage error: {0}")]
    Storage(String),
}

impl From<sqlx::Error> for ExplorerError {
    fn from(err: sqlx::Error) -> Self {
        Self::Storage(err.to_string())
    }
}

impl IntoResponse for ExplorerError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::NotFound(m) => (StatusCode::NOT_FOUND, "not_found", m),
            Self::InvalidRequest(m) => (StatusCode::BAD_REQUEST, "invalid_request", m),
            Self::Upstream(m) => (StatusCode::BAD_GATEWAY, "upstream_error", m),
            Self::Storage(m) => (StatusCode::SERVICE_UNAVAILABLE, "storage_error", m),
        };
        let body = ApiErrorDto {
            code: code.to_string(),
            message,
        };
        (status, Json(body)).into_response()
    }
}
