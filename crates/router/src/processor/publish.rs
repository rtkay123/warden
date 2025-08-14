use opentelemetry::global;
use opentelemetry_semantic_conventions::attribute;
use tracing::{Instrument, Span, debug, info, info_span, warn};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use warden_core::{configuration::routing::RoutingConfiguration, message::Payload};
use warden_stack::tracing::telemetry::nats::injector;

use crate::state::AppHandle;

pub(crate) async fn to_rule(
    (rule_id, rule_version): (&String, &str),
    state: AppHandle,
    mut payload: Payload,
    routing: RoutingConfiguration,
) -> anyhow::Result<()> {
    // send transaction to next with nats
    let subject = format!(
        "{}.{rule_id}.v{rule_version}",
        state.config.nats.destination_prefix
    );
    debug!(subject = ?subject, "publishing");

    payload.routing = Some(routing);

    let payload = prost::Message::encode_to_vec(&payload);

    let mut headers = async_nats::HeaderMap::new();

    let cx = Span::current().context();

    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut injector::HeaderMap(&mut headers))
    });

    let span = info_span!("nats.publish");
    span.set_attribute(
        attribute::MESSAGING_DESTINATION_SUBSCRIPTION_NAME,
        subject.to_string(),
    );
    state
        .services
        .jetstream
        .publish_with_headers(subject.clone(), headers, payload.into())
        .instrument(span)
        .await
        .inspect_err(|e| warn!(subject = ?subject, "failed to publish: {e}"))?;

    info!("published to rule");

    Ok(())
}
