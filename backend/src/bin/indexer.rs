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

    let pool = PgPool::connect(&config.database_url).await?;
    run_migrations(&pool).await?;

    let source = Arc::new(NodeHttpIngestionSource::new(
        config.node_metrics_url.clone(),
        config.node_ws_url.clone(),
    ));
    let indexer = IndexerService::new(
        source,
        pool,
        config.indexer_poll_interval_ms,
        config.indexer_start_height,
    );

    info!("explorer indexer started");
    indexer.run().await?;
    Ok(())
}
