use anyhow::Result;
use std::{collections::HashSet, sync::Arc};

use opentelemetry::global;
use prost::Message;
use tracing::{Instrument, Span, info_span, instrument, trace, trace_span, warn};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use warden_core::{google, message::Payload};
use warden_stack::tracing::telemetry::nats;

use crate::{cnfg::CACHE_KEY, processor::publish, state::AppHandle};

#[instrument(skip(message, state), err(Debug), fields(msg_id))]
pub async fn route(message: async_nats::jetstream::Message, state: AppHandle) -> Result<()> {
    let span = Span::current();

    if let Some(ref headers) = message.headers {
        let context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&nats::extractor::HeaderMap(headers))
        });
        span.set_parent(context);
    };

    let payload: Payload = Message::decode(message.payload.as_ref())?;

    match payload.transaction {
        Some(ref transaction) => {
            let msg_id = match transaction {
                warden_core::message::payload::Transaction::Pacs008(pacs008_document) => {
                    &pacs008_document.f_i_to_f_i_cstmr_cdt_trf.grp_hdr.msg_id
                }
                warden_core::message::payload::Transaction::Pacs002(pacs002_document) => {
                    &pacs002_document.f_i_to_f_i_pmt_sts_rpt.grp_hdr.msg_id
                }
            };
            span.record("msg_id", msg_id);

            let routing = {
                let local_cache = state.local_cache.read().await;
                local_cache.get(&CACHE_KEY).await
            };

            let routing = match routing {
                Some(local) => Some(local),
                None => {
                    let span = trace_span!(
                        "get.active.routing",
                        "otel.kind" = "client",
                        "rpc.service" = "configuration"
                    );
                    let mut client = state.query_routing_client.clone();
                    client
                        .get_active_routing_configuration(google::protobuf::Empty::default())
                        .instrument(span)
                        .await?
                        .into_inner()
                        .configuration
                }
            }
            .ok_or_else(|| anyhow::anyhow!("no routing configuration available"))?;

            trace!(tx_tp = ?payload.tx_tp, "finding all rules from configuration");
            let set: HashSet<_> = routing
                .messages
                .iter()
                .filter(|msg| msg.tx_tp == payload.tx_tp)
                .flat_map(|msg| &msg.typologies)
                .flat_map(|typ| &typ.rules)
                .map(|rule| (&rule.id, rule.version()))
                .collect();

            let futs = set.into_iter().map(|value| {
                publish::to_rule(value, Arc::clone(&state), payload.clone(), routing.clone())
            });

            futures_util::future::join_all(futs).await;
        }
        None => {
            warn!("transaction is empty - proceeding with ack");
        }
    }

    let span = info_span!("nats.ack");
    message
        .ack()
        .instrument(span)
        .await
        .map_err(|_| anyhow::anyhow!("ack error"))?;

    Ok(())
}
