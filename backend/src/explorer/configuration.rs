use config::{Config, ConfigError, Environment, File};
use dotenv::dotenv;
use serde::Deserialize;
use tracing::info;

fn default_strict_mode() -> bool {
    true
}

fn default_data_source() -> String {
    "postgres".to_string()
}

fn default_database_url() -> String {
    "postgres://postgres:postgres@localhost:5432/clutch_explorer".to_string()
}

fn default_node_metrics_url() -> String {
    "http://node1:3001/metrics".to_string()
}

fn default_node_ws_url() -> String {
    "http://node1:8081".to_string()
}

fn default_indexer_poll_interval_ms() -> u64 {
    4000
}

fn default_indexer_start_height() -> u64 {
    0
}

fn default_developer_mode() -> bool {
    false
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub log_level: String,
    pub listen_addr: String,
    pub seq_url: String,
    pub seq_api_key: String,
    pub clutch_node_api_url: String,
    pub allowed_origins: String,
    #[serde(default = "default_strict_mode")]
    pub strict_mode: bool,
    #[serde(default = "default_developer_mode")]
    pub developer_mode: bool,
    #[serde(default = "default_data_source")]
    pub data_source: String,
    #[serde(default = "default_database_url")]
    pub database_url: String,
    #[serde(default = "default_node_metrics_url")]
    pub node_metrics_url: String,
    #[serde(default = "default_node_ws_url")]
    pub node_ws_url: String,
    #[serde(default = "default_indexer_poll_interval_ms")]
    pub indexer_poll_interval_ms: u64,
    #[serde(default = "default_indexer_start_height")]
    pub indexer_start_height: u64,
}

impl AppConfig {
    fn from_env(env: &str) -> Result<Self, ConfigError> {
        dotenv().ok();
        let file_path = format!("config/{}.toml", env);
        let builder = Config::builder()
            .add_source(File::with_name(&file_path))
            .add_source(Environment::with_prefix("APP"));
        builder.build()?.try_deserialize::<Self>()
    }

    pub fn load_configuration(env: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = AppConfig::from_env(env)?;
        info!("Loaded explorer configuration from {}: {:?}", env, config);
        Ok(config)
    }
}
