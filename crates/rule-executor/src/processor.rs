mod publish;
mod reload;
mod rule;

use std::sync::Arc;

use anyhow::Result;
use async_nats::jetstream::{
    Context,
    consumer::{Consumer, pull},
};
use futures_util::{future, StreamExt};
use tokio::signal;
use tracing::trace;
use warden_stack::{Configuration, tracing::SdkTracerProvider};

use crate::{
    cnfg::Nats,
    state::{AppHandle, AppState, Services},
};

pub async fn serve(
    services: Services,
    config: Configuration,
    provider: SdkTracerProvider,
) -> anyhow::Result<()> {
    let state = Arc::new(AppState::new(services, config).await?);

    tokio::select! {
        _ = futures_util::future::try_join(reload::reload(Arc::clone(&state)), run(state)) => {}
        _ = shutdown_signal(provider) => {}
    };

    Ok(())
}

async fn run(state: AppHandle) -> anyhow::Result<()> {
    let config = Arc::clone(&state);
    let consumer = get_or_create_stream(&state.services.jetstream, &state.config.nats).await?;

    let limit = None;

    consumer
        .messages()
        .await?
        .for_each_concurrent(limit, |message| {
            let state = Arc::clone(&state);
            // tokio::spawn(async move {
            //     if let Ok(message) = message
            //         && let Err(e) = route::route(message, Arc::clone(&state)).await
            //     {
            //         error!("{}", e.to_string());
            //     }
            // });
            future::ready(())
        })
        .await;

    Ok(())
}

async fn get_or_create_stream(
    jetstream: &Context,
    nats: &Nats,
) -> anyhow::Result<Consumer<pull::Config>> {
    trace!(name = ?nats.name, "getting or creating stream");
    let stream = jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: format!("{}.v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            subjects: nats.subjects.iter().map(Into::into).collect(),
            ..Default::default()
        })
        .await?;
    let durable = nats.durable_name.to_string();
    // Get or create a pull-based consumer
    Ok(stream
        .get_or_create_consumer(
            durable.as_ref(),
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some(durable.to_string()),
                ..Default::default()
            },
        )
        .await?)
}

async fn shutdown_signal(provider: SdkTracerProvider) -> Result<()> {
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
        _ = ctrl_c => {
        },
        _ = terminate => {
        },
    }
    let _ = provider.shutdown();

    Ok(())
}
