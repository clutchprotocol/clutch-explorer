#[path = "../explorer/mod.rs"]
mod explorer;

use clap::Parser;
use explorer::configuration::AppConfig;
use explorer::db::run_migrations;
use explorer::indexer::IndexerService;
use explorer::ingestion::NodeHttpIngestionSource;
use explorer::tracing::setup_tracing;
use sqlx::PgPool;
use std::sync::Arc;
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

    let pool = PgPool::connect(&config.database_url).await?;
    run_migrations(&pool).await?;

    let source = Arc::new(NodeHttpIngestionSource::new(
        config.node_metrics_url.clone(),
        config.node_ws_url.clone(),
    ));
    let indexer = IndexerService::new(
        source,
        pool.clone(),
        config.indexer_poll_interval_ms,
        config.indexer_start_height,
    );

    let developer_mode = config.developer_mode;

    info!("explorer indexer started");

    tokio::select! {
        res = indexer.run() => {
            if let Err(e) = res {
                error!("Indexer service failed: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            info!("Shutdown signal received");
            if developer_mode {
                info!("Developer mode enabled, clearing database data...");
                if let Err(e) = explorer::db::cleanup_database(&pool).await {
                    error!("Failed to cleanup database: {}", e);
                } else {
                    info!("Database cleared successfully");
                }
            }
        }
    }

    Ok(())
}
