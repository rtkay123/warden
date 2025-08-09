mod cnfg;
mod error;
mod server;
mod state;
mod version;

use std::net::{Ipv6Addr, SocketAddr};

use clap::{Parser, command};
use stack_up::{Configuration, tracing::Tracing};
use tracing::info;

use crate::state::AppState;

/// warden
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to config file
    #[arg(short, long)]
    config_file: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), error::AppError> {
    let args = Args::parse();
    let config = include_str!("../warden.toml");

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

    tokio::spawn(tracing.loki_task);

    let state = AppState::create(&config).await?;

    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, config.application.port));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(port = addr.port(), "starting warden");

    axum::serve(listener, server::router(state)).await?;

    Ok(())
}
