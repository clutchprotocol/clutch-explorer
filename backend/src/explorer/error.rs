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
