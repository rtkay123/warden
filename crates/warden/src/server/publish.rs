use anyhow::Result;
use opentelemetry::global;
use opentelemetry_semantic_conventions::attribute;
use tracing::{Instrument, Span, info, info_span, trace};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use warden_core::message::Payload;
use warden_stack::tracing::telemetry::nats::injector;

use crate::state::AppHandle;

pub async fn publish_message(state: &AppHandle, payload: Payload, msg_id: &str) -> Result<()> {
    // send transaction to next with nats
    let subject = format!("{}.{}", state.app_config.nats.subject, msg_id);
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
    trace!(%msg_id, "publishing message");

    state
        .services
        .jetstream
        .publish_with_headers(subject, headers, payload.into())
        .instrument(span)
        .await?;

    info!(%msg_id, "message published");

    Ok(())
}
