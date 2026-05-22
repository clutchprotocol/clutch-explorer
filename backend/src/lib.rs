//! Shared explorer backend library (HTTP API and indexer).

pub mod explorer;

/// HTTP API entrypoint (re-exported so library-only checks keep the Axum stack reachable).
pub use explorer::run::run_api;
/// Indexer entrypoint (re-exported for the same reason as [`run_api`]).
pub use explorer::run::run_indexer;
