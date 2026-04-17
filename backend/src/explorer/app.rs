use crate::explorer::handlers;
use crate::explorer::state::AppState;
use axum::http::{HeaderValue, Method};
use axum::routing::get;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

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
        .route("/api/v1/validators", get(handlers::list_validators))
        .route("/api/v1/search", get(handlers::search))
        .route("/api/v1/stats", get(handlers::get_stats))
        .with_state(app_state)
        .layer(cors);

    Ok(router)
}
