use anyhow::{Result, anyhow};
use tracing::{Instrument, instrument, trace, trace_span};
use warden_core::configuration::rule::{RuleConfiguration, RuleConfigurationRequest};

use crate::state::AppHandle;

#[instrument(skip(state))]
pub(super) async fn get_configuration(
    request: RuleConfigurationRequest,
    state: AppHandle,
) -> Result<RuleConfiguration> {
    trace!("checking cache for rule configuration");
    let config = {
        let cache = state.local_cache.read().await;
        cache.get(&request).await
    };
    if let Some(config) = config {
        trace!("cache hit");
        return Ok(config);
    }
    trace!("cache miss, asking config service");

    let mut client = state.query_rule_client.clone();

    let span = trace_span!(
        "get.rule.config",
        "otel.kind" = "client",
        "rpc.service" = "configuration"
    );
    let resp = client
        .get_rule_configuration(request.clone())
        .instrument(span)
        .await?
        .into_inner();

    let config = resp
        .configuration
        .ok_or_else(|| anyhow!("missing configuration"))?;

    let cache = state.local_cache.write().await;
    cache.insert(request, config.clone()).await;

    Ok(config)
}
