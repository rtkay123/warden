use async_nats::jetstream::consumer;
use futures_util::StreamExt;
use prost::Message as _;
use tracing::{error, info, trace};
use uuid::Uuid;
use warden_core::configuration::{ConfigKind, ReloadEvent};

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
                trace!("got reload cache event");
                if let Ok(res) = ReloadEvent::decode(message.payload.as_ref())
                    && let Ok(kind) = ConfigKind::try_from(res.kind)
                {
                    match kind {
                        ConfigKind::Routing => {
                            trace!("update triggered, invalidating active routing config");
                            let local_cache = state.local_cache.write().await;
                            local_cache.invalidate_all();
                            let _ = message.ack().await.inspect_err(|e| error!("{e}"));
                        }
                        ConfigKind::Rule => {
                            trace!(kind = ?kind, "detected reload event, nothing to do here, acknowledging");
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
