mod cnfg;

mod processor;
mod state;

use anyhow::Result;
use clap::Parser;
use tracing::error;
use warden_stack::{Configuration, Services, tracing::Tracing};

/// rule-executor
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to config file
    #[arg(short, long)]
    config_file: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = include_str!("../rule-executor.toml");

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

    let mut services = Services::builder()
        .nats_jetstream(&config.nats)
        .await
        .inspect_err(|e| error!("nats: {e}"))?
        .postgres(&config.database)
        .await
        .inspect_err(|e| error!("postgres: {e}"))?
        .build();

    let jetstream = services
        .jetstream
        .take()
        .ok_or_else(|| anyhow::anyhow!("jetstream is not ready"))?;

    let postgres = services
        .postgres
        .take()
        .ok_or_else(|| anyhow::anyhow!("database is not ready"))?;

    let services = state::Services {
        jetstream,
        postgres,
    };

    processor::serve(services, config, provider)
        .await
        .inspect_err(|e| error!("{e}"))
}
