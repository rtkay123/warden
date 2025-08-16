mod aggregate;

use anyhow::Result;
use async_nats::{
    self,
    jetstream::{
        Context,
        consumer::{Consumer, pull::Config},
    },
};
use futures_util::{StreamExt as _, future};
use tokio::signal;
use tracing::{debug, error, info};
use warden_stack::tracing::SdkTracerProvider;

use crate::{cnfg::NatsConfig, state::AppHandle};

pub async fn serve(state: AppHandle, provider: SdkTracerProvider) -> Result<()> {
    tokio::select! {
        _ = run(state) => {}
        _ = shutdown_signal(provider) => {}
    };
    Ok(())
}

async fn run(state: AppHandle) -> anyhow::Result<()> {
    let consumer = get_or_create_stream(&state.services.jetstream, &state.config.nats).await?;

    let limit = None;

    consumer
        .messages()
        .await?
        .for_each_concurrent(limit, |message| {
            let state = state.clone();
            tokio::spawn(async move {
                if let Ok(message) = message
                    && let Err(e) = aggregate::handle(message, state).await
                {
                    error!("{}", e.to_string());
                }
            });
            future::ready(())
        })
        .await;

    Ok(())
}

async fn get_or_create_stream(
    jetstream: &Context,
    nats: &NatsConfig,
) -> anyhow::Result<Consumer<Config>> {
    debug!(name = ?nats.name, subjects = ?nats.subjects, "getting or creating stream");
    let stream = jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: nats.name.to_string(),
            subjects: nats.subjects.iter().map(|v| v.to_string()).collect(),
            ..Default::default()
        })
        .await?;
    let durable = nats.durable_name.to_string();
    // Get or create a pull-based consumer
    let consumer = stream
        .get_or_create_consumer(
            durable.as_ref(),
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some(durable.to_string()),
                ..Default::default()
            },
        )
        .await?;

    info!(subject = ?nats.subjects, "ready to receive messages");
    Ok(consumer)
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
