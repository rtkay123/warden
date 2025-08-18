mod cnfg;
mod error;
mod server;
mod state;
mod version;

use std::net::{Ipv6Addr, SocketAddr};
use tokio::signal;

use clap::{Parser, command};
use tracing::{error, info, trace};
use warden_stack::{
    Configuration, Services,
    tracing::{SdkTracerProvider, Tracing},
};

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

    let provider = tracing.otel_provider;

    tokio::spawn(tracing.loki_task);

    let mut services = Services::builder()
        .postgres(&config.database)
        .await
        .inspect_err(|e| error!("database: {e}"))?
        .cache(&config.cache)
        .await
        .inspect_err(|e| error!("cache: {e}"))?
        .nats_jetstream(&config.nats)
        .await
        .inspect_err(|e| error!("nats: {e}"))?
        .build();

    let mut more = vec![];
    more.push(2);

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

    let services = state::Services {
        postgres,
        cache,
        jetstream,
    };

    let state = AppState::create(services, &config).await?;

    trace!("running migrations");
    sqlx::migrate!("./migrations")
        .run(&state.services.postgres)
        .await?;
    trace!("migrations updated");

    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, config.application.port));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(port = addr.port(), "starting warden");

    let router = server::router(state).merge(server::metrics_app());
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal(provider))
        .await?;

    Ok(())
}

async fn shutdown_signal(provider: SdkTracerProvider) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    provider
        .shutdown()
        .expect("failed to shutdown trace provider");
}
