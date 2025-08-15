use async_nats::jetstream::consumer;
use futures_util::StreamExt;
use tracing::{debug, error, info};
use uuid::Uuid;
use warden_core::configuration::ReloadEvent;

use crate::state::AppHandle;

pub async fn reload(state: AppHandle) -> anyhow::Result<()> {
    let id = Uuid::now_v7().to_string();
    info!(durable = id, "listening for configuration changes");

    let durable = &id;
    let consumer = state
        .services
        .jetstream
        .get_stream(state.config.nats.config.stream.to_string())
        .await?
        .get_or_create_consumer(
            durable,
            consumer::pull::Config {
                durable_name: Some(durable.to_string()),
                filter_subject: state.config.nats.config.reload_subject.to_string(),
                deliver_policy: consumer::DeliverPolicy::LastPerSubject,
                ..Default::default()
            },
        )
        .await?;

    let mut messages = consumer.messages().await?;
    while let Some(value) = messages.next().await {
        match value {
            Ok(message) => {
                debug!("got reload cache event",);
                if let Ok(Some(event)) = String::from_utf8(message.payload.to_vec())
                    .map(|value| ReloadEvent::from_str_name(&value))
                {
                    match event {
                        // TODO: find exact rule
                        ReloadEvent::Rule => {
                            let local_cache = state.local_cache.write().await;
                            local_cache.invalidate_all();
                            let _ = message.ack().await.inspect_err(|e| error!("{e}"));
                        }
                        _ => {
                            debug!(event = ?event, "detected reload event, acknowledging");
                            let _ = message.ack().await.inspect_err(|e| error!("{e}"));
                        }
                    }
                }
            }
            Err(e) => {
                error!("{e}")
            }
        }
    }

    Ok(())
}
