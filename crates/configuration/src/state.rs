mod cache_key;
mod routing;
mod rule;

use async_nats::jetstream::Context;
use opentelemetry_semantic_conventions::attribute;
use prost::Message;
use sqlx::PgPool;
use std::{ops::Deref, sync::Arc};
use tracing::{Instrument, info_span, instrument, trace};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use warden_core::configuration::ReloadEvent;
use warden_stack::{Configuration, cache::RedisManager, redis::AsyncCommands};

use crate::{
    cnfg::LocalConfig,
    server::{error::AppError, reload_stream::create_stream},
    state::cache_key::CacheKey,
};

#[derive(Clone)]
pub struct AppHandle(Arc<AppState>);

impl Deref for AppHandle {
    type Target = Arc<AppState>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct Services {
    pub postgres: PgPool,
    pub cache: RedisManager,
    pub jetstream: Context,
}

pub struct AppState {
    pub services: Services,
    pub app_config: LocalConfig,
}

impl AppState {
    pub async fn create(
        services: Services,
        configuration: &Configuration,
    ) -> Result<AppHandle, AppError> {
        let local_config: LocalConfig = serde_json::from_value(configuration.misc.clone())?;

        local_config
            .nats
            .subject
            .split(".")
            .next()
            .ok_or_else(|| anyhow::anyhow!("expected a dot separated config for nats subjects"))?;

        create_stream(&services.jetstream, &local_config.nats).await?;

        Ok(AppHandle(Arc::new(Self {
            services,
            app_config: local_config,
        })))
    }
}

#[instrument(skip(state), err(Debug))]
pub async fn invalidate_cache(state: &AppHandle, key: CacheKey<'_>) -> Result<(), tonic::Status> {
    trace!("invalidating cache");
    let mut cache = state
        .services
        .cache
        .get()
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

    cache
        .del::<_, ()>(key)
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))
}

#[instrument(skip(state), err(Debug))]
pub async fn publish_reload(
    state: &AppHandle,
    prefix: &str,
    event: ReloadEvent,
) -> Result<(), tonic::Status> {
    trace!("publishing reload event");

    let span = info_span!("reload config");
    span.set_attribute(attribute::MESSAGING_SYSTEM, "nats");
    span.set_attribute("otel.kind", "producer");

    let bytes = event.encode_to_vec();
    state
        .services
        .jetstream
        .publish(format!("{prefix}.reload"), bytes.into())
        .instrument(span)
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

    Ok(())
}
