use crate::explorer::app::build_router;
use crate::explorer::configuration::AppConfig;
use crate::explorer::db::{cleanup_database, run_migrations};
use crate::explorer::indexer::IndexerService;
use crate::explorer::ingestion::NodeHttpIngestionSource;
use crate::explorer::shutdown::wait_for_shutdown;
use crate::explorer::state::{AppState, ExplorerService};
use crate::explorer::tracing::setup_tracing;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

pub async fn run_api(config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing(&config.log_level, &config.seq_url, &config.seq_api_key)?;

    let pg_pool = if config.data_source == "postgres" {
        let pool = PgPool::connect(&config.database_url).await?;

        if config.cleanup_on_start {
            info!("Cleanup on start enabled, clearing database data...");
            if let Err(e) = cleanup_database(&pool).await {
                error!("Failed to cleanup database on start: {}", e);
            } else {
                info!("Database cleared successfully on start");
            }
        }

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
            wait_for_shutdown().await;

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

pub async fn run_indexer(config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
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
        config.ride_request_referrer_fee_percent,
        config.ride_offer_referrer_fee_percent,
    );

    let developer_mode = config.developer_mode;

    info!("explorer indexer started");

    tokio::select! {
        res = indexer.run() => {
            if let Err(e) = res {
                error!("Indexer service failed: {}", e);
            }
        }
        _ = wait_for_shutdown() => {
            if developer_mode {
                info!("Developer mode enabled, clearing database data...");
                if let Err(e) = cleanup_database(&pool).await {
                    error!("Failed to cleanup database: {}", e);
                } else {
                    info!("Database cleared successfully");
                }
            }
        }
    }

    Ok(())
}
