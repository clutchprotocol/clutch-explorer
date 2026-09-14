use crate::explorer::app::build_router;
use crate::explorer::configuration::AppConfig;
use crate::explorer::db::{cleanup_database, run_migrations};
use crate::explorer::indexer::IndexerService;
use crate::explorer::ingestion::NodeHttpIngestionSource;
use crate::explorer::shutdown::wait_for_shutdown;
use crate::explorer::reserve::ReserveClient;
use crate::explorer::state::{AppState, ExplorerService};
use crate::explorer::tracing::setup_tracing;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

pub async fn run_api(config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing(&config.log_level, &config.seq_url, &config.seq_api_key)?;

    // No cleanup here, on start or on shutdown. The API is a READER of tables the indexer
    // writes, and the two run as separate containers against one database -- so a truncate from
    // this process races a live indexer. On stage it wiped `indexer_cursor` out from under a
    // running indexer, which then kept its position only in memory: the logs showed a cursor
    // walking 295 to 303 while the table held no rows and blocks 1..295 were gone for good.
    // Wiping belongs to whoever owns the data, which is `run_indexer` below.
    let pg_pool = if config.data_source == "postgres" {
        let pool = PgPool::connect(&config.database_url).await?;
        run_migrations(&pool).await?;
        Some(pool)
    } else {
        None
    };

    let service = Arc::new(ExplorerService::new(config.clone(), pg_pool.clone())?);
    let reserve = ReserveClient::new(config.treasury_public_reconciliation_url.clone());
    let app_state = AppState { service, reserve };
    let app = build_router(app_state, &config.allowed_origins)?;

    let listener = TcpListener::bind(&config.listen_addr).await?;
    info!("explorer backend listening on {}", config.listen_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(wait_for_shutdown())
        .await?;

    Ok(())
}

pub async fn run_indexer(config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing(&config.log_level, &config.seq_url, &config.seq_api_key)?;

    let pool = PgPool::connect(&config.database_url).await?;
    run_migrations(&pool).await?;

    // After the migrations, not before: a truncate of tables that do not exist yet fails on a
    // fresh database. The API used to do this the other way round and swallow the error.
    if config.cleanup_on_start {
        info!("Cleanup on start enabled, clearing database data...");
        cleanup_database(&pool).await?;
        info!("Database cleared successfully on start");
    }

    let source = Arc::new(NodeHttpIngestionSource::new(
        config.node_metrics_url.clone(),
        config.node_ws_url.clone(),
    ));
    let indexer = IndexerService::new(
        source,
        pool.clone(),
        config.indexer_poll_interval_ms,
        config.indexer_start_height,
        config.ride_request_referrer_fee_bps,
        config.ride_offer_referrer_fee_bps,
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
