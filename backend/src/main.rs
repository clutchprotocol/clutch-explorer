mod explorer;

use clap::Parser;
use explorer::app::build_router;
use explorer::configuration::AppConfig;
use explorer::db::{cleanup_database, run_migrations};
use explorer::state::{AppState, ExplorerService};
use explorer::tracing::setup_tracing;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{error, info};

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

    let service = Arc::new(ExplorerService::new(config.clone(), pg_pool.clone())?);
    let app_state = AppState { service };
    let app = build_router(app_state, &config.allowed_origins)?;

    let listener = TcpListener::bind(&config.listen_addr).await?;
    info!("explorer backend listening on {}", config.listen_addr);

    let developer_mode = config.developer_mode;
    let pool_for_shutdown = pg_pool.clone();

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C handler");
            info!("Shutdown signal received");

            if developer_mode {
                if let Some(pool) = pool_for_shutdown {
                    info!("Developer mode enabled, clearing database data...");
                    if let Err(e) = cleanup_database(&pool).await {
                        error!("Failed to cleanup database: {}", e);
                    } else {
                        info!("Database cleared successfully");
                    }
                }
            }
        })
        .await?;

    Ok(())
}
