use clap::Parser;
use clutch_explorer_backend::explorer::configuration::AppConfig;
use clutch_explorer_backend::explorer::run::run_api;

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
    run_api(config).await
}
