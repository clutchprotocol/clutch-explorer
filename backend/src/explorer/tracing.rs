use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::explorer::seq::{SeqLayer, SeqLogger};

pub fn setup_tracing(
    log_level: &str,
    seq_url: &str,
    seq_api_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let seq_logger = Arc::new(Mutex::new(SeqLogger::new(seq_url, seq_api_key)));
    let seq_layer = SeqLayer::new(seq_logger);

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(seq_layer)
        .with(EnvFilter::new(log_level))
        .try_init()
        .or_else(|_| {
            println!("Global default trace dispatcher has already been set");
            Ok::<(), Box<dyn std::error::Error>>(())
        })?;

    Ok(())
}
