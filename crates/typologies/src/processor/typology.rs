mod aggregate_rules;
mod evaluate_expression;

use std::sync::Arc;

use anyhow::Result;
use opentelemetry::global;
use prost::Message;
use tracing::{Instrument, Span, error, info, info_span, instrument, warn};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use warden_core::{
    configuration::{routing::RoutingConfiguration, typology::TypologyConfigurationRequest},
    message::{Payload, RuleResult, TypologyResult},
};
use warden_stack::{redis::AsyncCommands, tracing::telemetry::nats::extractor};

use crate::{
    processor::{driver::GetTypologyConfiguration as _, publish},
    state::AppHandle,
};

#[instrument(skip(message, state), err(Debug))]
pub async fn process_typology(
    message: async_nats::jetstream::Message,
    state: AppHandle,
) -> Result<()> {
    let span = Span::current();

    if let Some(ref headers) = message.headers {
        let context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&extractor::HeaderMap(headers))
        });

        if let Err(e) = span.set_parent(context) {
            error!("{e:?}");
        };
    };

    let payload: Payload = Message::decode(message.payload.as_ref())?;

    if payload.transaction.is_none() {
        warn!("transaction is empty - proceeding with ack");
        let _ = message.ack().await;
        return Ok(());
    }

    let transaction = payload.transaction.as_ref().expect("to have returned");

    match transaction {
        warden_core::message::payload::Transaction::Pacs008(_) => {
            warn!("Pacs008 is unsupported on this version: this should be unreachable");
        }
        warden_core::message::payload::Transaction::Pacs002(pacs002_document) => {
            let key = format!(
                "tp_{}",
                pacs002_document.f_i_to_f_i_pmt_sts_rpt.grp_hdr.msg_id
            );

            let rule_result = &payload
                .rule_result
                .as_ref()
                .expect("rule result should be here");
            let rule_results = cache_and_get_all(&key, rule_result, Arc::clone(&state)).await?;

            let routing = payload
                .routing
                .as_ref()
                .expect("routing missing from payload");

            let (mut typology_result, _rule_count) =
                aggregate_rules::aggregate_rules(&rule_results, routing, rule_result)?;

            let _ = evaluate_typology(&mut typology_result, routing, payload.clone(), &key, state)
                .await
                .inspect_err(|e| error!("{e}"));
        }
    };

    let span = info_span!("nats.ack");
    message
        .ack()
        .instrument(span)
        .await
        .map_err(|_| anyhow::anyhow!("ack error"))?;

    Ok(())
}

#[instrument(skip(routing, payload, state), err(Debug))]
async fn evaluate_typology(
    typology_result: &mut [TypologyResult],
    routing: &RoutingConfiguration,
    mut payload: Payload,
    key: &str,
    state: AppHandle,
) -> Result<()> {
    for typology_result in typology_result.iter_mut() {
        let handle = Arc::clone(&state);
        let routing_rules = routing.messages[0].typologies.iter().find(|typology| {
            typology.version.eq(&typology_result.version) && typology.id.eq(&typology_result.id)
        });
        let typology_result_rules = &typology_result.rule_results;

        if routing_rules.is_some()
            && typology_result_rules.len() < routing_rules.unwrap().rules.len()
        {
            continue;
        }

        let typology_config = handle
            .get_typology_config(TypologyConfigurationRequest {
                id: typology_result.id.to_owned(),
                version: typology_result.version.to_owned(),
            })
            .await?;

        let result = evaluate_expression::evaluate_expression(typology_result, &typology_config)?;

        typology_result.result = result;

        let workflow = typology_config
            .workflow
            .as_ref()
            .expect("no workflow in config");

        if workflow.interdiction_threshold.is_some() {
            typology_result.workflow.replace(*workflow);
        }
        typology_result.review = result.ge(&typology_config.workflow.unwrap().alert_threshold);

        payload.typology_result = Some(typology_result.to_owned());

        let is_interdicting = typology_config
            .workflow
            .unwrap()
            .interdiction_threshold
            .is_some_and(|value| value > 0.0 && result >= value);

        if is_interdicting {
            typology_result.review = true;
        }

        if result >= typology_config.workflow.unwrap().alert_threshold {
            info!("alerting");
        }

        let subj = handle.config.nats.destination_prefix.to_string();
        let _ = publish::to_tadp(&subj, handle, payload.clone())
            .await
            .inspect_err(|e| error!("{e}"));

        let mut c = state.services.cache.get().await?;
        c.del::<_, ()>(key).await?;
    }

    Ok(())
}

async fn cache_and_get_all(
    cache_key: &str,
    rule_result: &RuleResult,
    state: AppHandle,
) -> Result<Vec<RuleResult>> {
    let mut cache = state.services.cache.get().await?;

    let bytes = prost::Message::encode_to_vec(rule_result);

    let res = warden_stack::redis::pipe()
        .sadd::<_, _>(cache_key, bytes)
        .ignore()
        .smembers(cache_key)
        .query_async::<Vec<Vec<Vec<u8>>>>(&mut cache)
        .await?;

    let members = res
        .first()
        .ok_or_else(|| anyhow::anyhow!("smembers did not return anything"))?;

    members
        .iter()
        .map(|value| RuleResult::decode(value.as_ref()).map_err(anyhow::Error::new))
        .collect()
}
