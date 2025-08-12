mod cnfg;
mod server;
mod state;

use clap::Parser;
use tracing::error;
use warden_config::state::AppState;
use warden_stack::{Configuration, Services, tracing::Tracing};

/// warden-config
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to config file
    #[arg(short, long)]
    config_file: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = include_str!("../warden-config.toml");

    let mut config = config::Config::builder()
        .add_source(config::File::from_str(config, config::FileFormat::Toml));

    if let Some(cf) = args.config_file.as_ref().and_then(|v| v.to_str()) {
        config = config.add_source(config::File::new(cf, config::FileFormat::Toml));
    };

    let mut config: Configuration = config.build()?.try_deserialize()?;
    config.application.name = env!("CARGO_CRATE_NAME").into();
    config.application.version = env!("CARGO_PKG_VERSION").into();

    let tracing = Tracing::builder()
        .opentelemetry(&config.application, &config.monitoring)?
        .loki(&config.application, &config.monitoring)?
        .build(&config.monitoring);

    let provider = tracing.otel_provider;

    tokio::spawn(tracing.loki_task);

    let services = Services::builder()
        .postgres(&config.database)
        .await
        .inspect_err(|e| error!("database: {e}"))?
        .nats_jetstream(&config.nats)
        .await
        .inspect_err(|e| error!("nats: {e}"))?
        .cache(&config.cache)
        .await
        .inspect_err(|e| error!("cache: {e}"))?
        .build();

    let postgres = services
        .postgres
        .take()
        .ok_or_else(|| anyhow::anyhow!("database is not ready"))?;

    let cache = services
        .cache
        .take()
        .ok_or_else(|| anyhow::anyhow!("cache is not ready"))?;

    let jetstream = services
        .jetstream
        .take()
        .ok_or_else(|| anyhow::anyhow!("jetstream is not ready"))?;

    let state = AppState::new(services, config, Some(provider))
        .await
        .inspect_err(|e| error!("{e}"))?;

    server::serve(state);
}
