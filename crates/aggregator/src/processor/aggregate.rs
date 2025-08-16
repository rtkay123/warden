use async_nats::jetstream::Message;
use opentelemetry::global;
use opentelemetry_semantic_conventions::attribute;
use tracing::{Instrument, Span, debug, error, info, info_span, instrument, trace};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;
use warden_core::{
    configuration::routing::RoutingConfiguration,
    message::{AggregationResult, Payload, TypologyResult, payload::Transaction},
};
use warden_stack::{redis::AsyncCommands, tracing::telemetry::nats::extractor};

use crate::state::AppHandle;

#[instrument(skip(message, state), err(Debug))]
pub async fn handle(message: Message, state: AppHandle) -> anyhow::Result<()> {
    let span = Span::current();

    if let Some(ref headers) = message.headers {
        let context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&extractor::HeaderMap(headers))
        });
        span.set_parent(context);
    };

    let mut payload: Payload = prost::Message::decode(message.payload.as_ref())?;

    if let (Some(ref typology_result), Some(Transaction::Pacs002(document)), Some(routing)) = (
        payload.typology_result.take(),
        &payload.transaction,
        &payload.routing,
    ) {
        let cache_key = format!("tadp_{}_tp", document.f_i_to_f_i_pmt_sts_rpt.grp_hdr.msg_id);
        let (typology_results, review) =
            handle_typologies(typology_result, &state, &cache_key, routing).await?;

        if typology_results
            .len()
            .ne(&routing.messages[0].typologies.len())
        {
            trace!("insufficient typology results for this typology. waiting for more");
            return Ok(());
        }

        let aggs = AggregationResult {
            id: routing.messages[0].id.to_owned(),
            version: routing.messages[0].version.to_owned(),
            typology_results,
            review,
        };

        payload.aggregation_result = Some(aggs);
        let _ = payload.rule_result.take();

        let id = Uuid::now_v7();
        debug!(%id, "inserting evaluation result");

        let span = info_span!("create.evaluations.evaluation");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "insert");
        span.set_attribute(attribute::DB_COLLECTION_NAME, "transaction");
        span.set_attribute("otel.kind", "client");

        sqlx::query!(
            "insert into evaluation (id, document) values ($1, $2)",
            id,
            sqlx::types::Json(&payload) as _
        )
        .execute(&state.services.postgres)
        .instrument(span)
        .await?;
        info!(%id, "evaluation added");

        let mut cache = state.services.cache.get().await?;
        let span = Span::current();
        span.set_attribute(attribute::DB_SYSTEM_NAME, "valkey");
        span.set_attribute(attribute::DB_OPERATION_NAME, "del");
        span.set_attribute(attribute::DB_OPERATION_PARAMETER, cache_key.to_string());
        span.set_attribute("otel.kind", "client");
        debug!("cache cleared");

        cache.del::<_, ()>(&cache_key).await?;
    } else {
        error!("payload has insufficient data");
    }

    let span = info_span!("nats.ack");
    message
        .ack()
        .instrument(span)
        .await
        .map_err(|_| anyhow::anyhow!("ack error"))?;

    Ok(())
}

async fn handle_typologies(
    payload: &TypologyResult,
    state: &AppHandle,
    cache_key: &str,
    routing: &RoutingConfiguration,
) -> anyhow::Result<(Vec<TypologyResult>, bool)> {
    let mut cache = state.services.cache.get().await?;
    let bytes = prost::Message::encode_to_vec(payload);

    let span = Span::current();
    span.set_attribute(attribute::DB_SYSTEM_NAME, "valkey");
    span.set_attribute(attribute::DB_OPERATION_NAME, "sadd+scard");
    span.set_attribute(attribute::DB_OPERATION_PARAMETER, cache_key.to_string());
    span.set_attribute("otel.kind", "client");

    debug!("saving typology result");
    let res = warden_stack::redis::pipe()
        .sadd::<_, _>(cache_key, bytes)
        .ignore()
        .scard(cache_key)
        .query_async::<Vec<usize>>(&mut cache)
        .instrument(span)
        .await?;

    let typology_count = res
        .first()
        .ok_or_else(|| anyhow::anyhow!("smembers did not return anything"))?;

    let typologies = &routing.messages[0].typologies;

    if typology_count.lt(&typologies.len()) {
        return Ok((vec![], false));
    }

    debug!("getting all typology results");
    let span = Span::current();
    span.set_attribute(attribute::DB_SYSTEM_NAME, "valkey");
    span.set_attribute(attribute::DB_OPERATION_NAME, "smembers");
    span.set_attribute(attribute::DB_OPERATION_PARAMETER, cache_key.to_string());
    span.set_attribute("otel.kind", "client");
    let res = cache
        .smembers::<_, Vec<Vec<Vec<u8>>>>(cache_key)
        .instrument(span)
        .await?;

    let members = res
        .first()
        .ok_or_else(|| anyhow::anyhow!("smembers did not return anything"))?;

    let typologies: Result<Vec<TypologyResult>, _> = members
        .iter()
        .map(|value| {
            <TypologyResult as prost::Message>::decode(value.as_ref()).map_err(anyhow::Error::new)
        })
        .collect();

    let typologies = typologies?;

    let mut review = false;
    for typology in routing.messages[0].typologies.iter() {
        if let Some(value) = typologies
            .iter()
            .find(|value| value.id.eq(&typology.id) && value.version.eq(&typology.version))
            && value.review
        {
            review = true;
        }
    }

    Ok((typologies, review))
}
