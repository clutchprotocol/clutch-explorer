use crate::explorer::handlers;
use crate::explorer::postgres_repository::PostgresRepository;
use crate::explorer::reserve::ReserveClient;
use axum::http::{HeaderValue, Method};
use axum::routing::get;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub repo: PostgresRepository,
    /// Separate from `repo` on purpose: everything there reads this explorer's own database,
    /// and this reaches another service that may be absent or down. Keeping them apart means a
    /// treasury outage cannot present itself as an explorer failure.
    pub reserve: ReserveClient,
}

pub fn build_router(
    app_state: AppState,
    allowed_origins: &str,
) -> Result<Router, Box<dyn std::error::Error>> {
    let cors = if allowed_origins.trim() == "*" {
        CorsLayer::new().allow_origin(Any)
    } else {
        let origins = allowed_origins
            .split(',')
            .map(|v| HeaderValue::from_str(v.trim()))
            .collect::<Result<Vec<_>, _>>()?;
        CorsLayer::new().allow_origin(origins)
    }
    .allow_methods([Method::GET])
    .allow_headers(Any);

    let router = Router::new()
        .route("/health", get(handlers::health))
        .route("/ready", get(handlers::ready))
        .route("/api/v1/blocks", get(handlers::list_blocks))
        .route("/api/v1/blocks/:id", get(handlers::get_block))
        .route("/api/v1/transactions", get(handlers::list_transactions))
        .route("/api/v1/transactions/:hash", get(handlers::get_transaction))
        .route("/api/v1/accounts/:address", get(handlers::get_account))
        .route(
            "/api/v1/accounts/:address/activity",
            get(handlers::get_account_activity),
        )
        .route("/api/v1/validators", get(handlers::list_validators))
        .route("/api/v1/search", get(handlers::search))
        .route("/api/v1/stats", get(handlers::get_stats))
        .route("/api/v1/reserve", get(handlers::get_reserve))
        .with_state(app_state)
        .layer(cors);

    Ok(router)
}
