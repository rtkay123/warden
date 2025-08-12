pub mod grpc;
mod reload;
mod route;
mod publish;
mod load;

use std::sync::Arc;

use anyhow::Result;
use async_nats::jetstream::{consumer::{pull, Consumer}, Context};
use tokio::signal;
use tracing::{error, trace};
use warden_stack::{Configuration, tracing::SdkTracerProvider};
use futures_util::StreamExt;

use crate::{cnfg::Nats, state::{AppHandle, AppState, Services}};


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
    let (consumer, _) = tokio::join!(
        get_or_create_stream(&state.services.jetstream, &state.config.nats),
        load::get_routing_config(Arc::clone(&config))
    );

    let consumer = consumer?;

    // Consume messages from the consumer
    while let Some(Ok(message)) = consumer.messages().await?.next().await {
        if let Err(e) = route::route(message, Arc::clone(&state)).await {
            error!("{}", e.to_string());
        }
    }

    Ok(())
}

async fn get_or_create_stream(
    jetstream: &Context,
    nats: &Nats,
) -> anyhow::Result<Consumer<pull::Config>> {
    trace!(name = ?nats.name, "getting or creating stream");
    let stream = jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: nats.name.to_string(),
            subjects: nats.subjects.iter().map(Into::into).collect(),
            max_messages: nats.max_messages,
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
