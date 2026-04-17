use config::{Config, ConfigError, Environment, File};
use dotenv::dotenv;
use serde::Deserialize;
use tracing::info;

fn default_strict_mode() -> bool {
    true
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
