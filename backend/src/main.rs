mod explorer;

use clap::Parser;
use explorer::app::build_router;
use explorer::configuration::AppConfig;
use explorer::db::run_migrations;
use explorer::state::{AppState, ExplorerService};
use explorer::tracing::setup_tracing;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "default")]
    env: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = &Args::parse().env;
    let config = AppConfig::load_configuration(env)?;
    setup_tracing(&config.log_level, &config.seq_url, &config.seq_api_key)?;

    let pg_pool = if config.data_source == "postgres" {
        let pool = PgPool::connect(&config.database_url).await?;
        run_migrations(&pool).await?;
        Some(pool)
    } else {
        None
    };

    let service = Arc::new(ExplorerService::new(config.clone(), pg_pool)?);
    let app_state = AppState { service };
    let app = build_router(app_state, &config.allowed_origins)?;

    let listener = TcpListener::bind(&config.listen_addr).await?;
    info!("explorer backend listening on {}", config.listen_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
