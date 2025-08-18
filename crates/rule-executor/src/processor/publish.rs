use warden_stack::tracing::telemetry::nats::injector;

use opentelemetry::global;
use opentelemetry_semantic_conventions::attribute;
use tracing::{Instrument, Span, debug, info_span, warn};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use warden_core::message::Payload;

use crate::state::AppHandle;

pub(super) async fn to_typologies(
    subject: &str,
    state: AppHandle,
    payload: Payload,
) -> anyhow::Result<()> {
    // send transaction to next with nats
    let subject = format!("{}.{}", state.config.nats.destination_prefix, subject);
    debug!(subject = ?subject, "publishing");

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
    span.set_attribute(
        "otel.kind",
        "producer"
    );
    state
        .services
        .jetstream
        .publish_with_headers(subject.clone(), headers, payload.into())
        .instrument(span)
        .await
        .inspect_err(|e| warn!(subject = ?subject, "failed to publish: {e}"))?;

    Ok(())
}
