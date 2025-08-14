use opentelemetry_semantic_conventions::attribute;
use tracing::{Instrument, debug, info, info_span, instrument, warn};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use warden_core::{configuration::routing::RoutingConfiguration, google};

use crate::{cnfg::CACHE_KEY, state::AppHandle};

#[instrument(skip(state))]
pub async fn get_routing_config(state: AppHandle) -> Option<RoutingConfiguration> {
    debug!("getting routing config");
    {
        let span = info_span!("local_cache.get");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "moka");
        let local_cache = state.local_cache.read().await;
        if let Some(value) = local_cache.get(&CACHE_KEY).await {
            return Some(value);
        }
    }

    let mut client = state.query_routing_client.clone();

    let span = info_span!("get.routing.config");
    span.set_attribute(attribute::RPC_SERVICE, env!("CARGO_PKG_NAME"));
    span.set_attribute("otel.kind", "client");

    if let Ok(config) = client
        .get_active_routing_configuration(google::protobuf::Empty::default())
        .instrument(span)
        .await
    {
        debug!("fetched routing config");
        let span = info_span!("local_cache.insert");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "moka");
        if let Some(config) = config.into_inner().configuration {
            debug!("updating cache");
            let local_cache = state.local_cache.write().await;
            local_cache
                .insert(CACHE_KEY, config.clone())
                .instrument(span)
                .await;
            info!("cache refreshed");
            return Some(config);
        } else {
            warn!("no routing config is active");
            return None;
        }
    } else {
        warn!("no routing config is active");
        return None;
    }
}
