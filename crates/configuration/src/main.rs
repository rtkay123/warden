mod cnfg;
mod server;
mod state;

use std::net::{Ipv6Addr, SocketAddr};

use crate::{server::error::AppError, state::AppState};
use axum::http::header::CONTENT_TYPE;
use clap::Parser;
use tokio::signal;
use tower::{make::Shared, steer::Steer};
use tracing::{error, info, trace};
use warden_stack::{
    Configuration, Services,
    tracing::{SdkTracerProvider, Tracing},
};

/// warden-config
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to config file
    #[arg(short, long)]
    config_file: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
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

    let mut services = Services::builder()
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

    let state = AppState::create(
        crate::state::Services {
            postgres,
            cache,
            jetstream,
        },
        &config,
    )
    .await?;

    trace!("running migrations");
    sqlx::migrate!("./migrations")
        .run(&state.services.postgres)
        .await?;
    trace!("migrations updated");

    let (app, grpc_server) = server::serve(state)?;

    let service = Steer::new(
        vec![app, grpc_server],
        |req: &axum::extract::Request, _services: &[_]| {
            if req
                .headers()
                .get(CONTENT_TYPE)
                .map(|content_type| content_type.as_bytes())
                .filter(|content_type| content_type.starts_with(b"application/grpc"))
                .is_some()
            {
                // grpc service
                1
            } else {
                // http service
                0
            }
        },
    );

    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, config.application.port));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(port = addr.port(), "starting config-api");

    axum::serve(listener, Shared::new(service))
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
