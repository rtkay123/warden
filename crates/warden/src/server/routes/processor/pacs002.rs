use axum::{extract::State, response::IntoResponse};
use opentelemetry_semantic_conventions::attribute;
use prost::Message as _;
use serde::Serialize;
use tracing::{Instrument, debug, error, info, info_span, trace};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;
use warden_core::{
    google::r#type::Money,
    iso20022::{TransactionType, pacs002::Pacs002Document},
    message::{DataCache, Payload},
    pseudonyms::transaction_relationship::{CreatePseudonymRequest, TransactionRelationship},
};
use warden_stack::redis::AsyncCommands;

use crate::{
    error::AppError,
    server::{
        publish::publish_message,
        routes::{
            PACS002_001_12,
            processor::pacs008::{build_data_cache, set_cache},
        },
    },
    state::AppHandle,
    version::Version,
};

#[derive(Serialize)]
struct Row {
    id: Uuid,
    document: sqlx::types::Json<serde_json::Value>,
}

/// Submit a pacs.002.001.12 transaction
#[utoipa::path(
    post,
    responses((
        status = CREATED,
        body = Pacs002Document
    )),
    operation_id = "post_pacs_002", // https://github.com/juhaku/utoipa/issues/1170
    path = "/{version}/pacs002",
    params(
        ("version" = Version, Path, description = "API version, e.g., v1, v2, v3")
    ),
    tag = PACS002_001_12,
    request_body(
        content = Pacs002Document
    ))
]
#[tracing::instrument(
    skip(state, request),
    err(Debug),
    fields(method = "POST", end_to_end_id, msg_id, tx_tp)
)]
pub async fn post_pacs002(
    State(state): State<AppHandle>,
    axum::Json(request): axum::Json<Pacs002Document>,
) -> Result<impl IntoResponse, AppError> {
    let tx_tp = TransactionType::PACS002.to_string();
    tracing::Span::current().record("tx_tp", &tx_tp);

    let cre_dt_tm = request.f_i_to_f_i_pmt_sts_rpt.grp_hdr.cre_dt_tm;
    let end_to_end_id = request.f_i_to_f_i_pmt_sts_rpt.tx_inf_and_sts[0]
        .orgnl_end_to_end_id
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("end_to_end_id is expected"))?;
    tracing::Span::current().record("end_to_end_id", end_to_end_id);

    let msg_id = &request.f_i_to_f_i_pmt_sts_rpt.grp_hdr.msg_id;
    tracing::Span::current().record("msg_id", msg_id);

    let pmt_inf_id = &request.f_i_to_f_i_pmt_sts_rpt.tx_inf_and_sts[0].orgnl_instr_id;
    let tx_sts = &request.f_i_to_f_i_pmt_sts_rpt.tx_inf_and_sts[0].tx_sts;

    let mut cache = state.services.cache.get().await?;
    trace!(end_to_end_id = end_to_end_id, "getting data cache");
    let cache = cache
        .get::<_, Vec<u8>>(&end_to_end_id)
        .await
        .map(|value| DataCache::decode(value.as_ref()));

    let data_cache = match cache {
        Ok(Ok(data_cache)) => {
            debug!(end_to_end_id = end_to_end_id, "cache hit");
            rebuild_entities(end_to_end_id, &state, Some(data_cache)).await?
        }
        _ => {
            debug!(end_to_end_id = end_to_end_id, "cache miss");
            rebuild_entities(end_to_end_id, &state, None).await?
        }
    };

    let amount = data_cache.instd_amt.as_ref().map(|value| value.value);

    let ccy = data_cache
        .instd_amt
        .as_ref()
        .map(|value| value.ccy.as_str());

    debug!(%msg_id, %end_to_end_id, "parsed transaction identifiers");

    let money = if let (Some(amt), Some(ccy)) = (amount, ccy) {
        Some(Money::try_from((amt, ccy)).map_err(|_e| anyhow::anyhow!("invalid currency"))?)
    } else {
        trace!(msg_id, "transaction has no amount or currency");
        None
    };

    let transaction_relationship = TransactionRelationship {
        from: data_cache.cdtr_acct_id.to_string(),
        to: data_cache.dbtr_acct_id.to_string(),
        amt: money,
        cre_dt_tm: Some(cre_dt_tm),
        end_to_end_id: end_to_end_id.to_string(),
        msg_id: msg_id.to_string(),
        pmt_inf_id: pmt_inf_id
            .as_ref()
            .ok_or_else(|| {
                error!("missing pmt_inf_id");
                anyhow::anyhow!("missing pmt_inf_id")
            })?
            .to_string(),
        tx_tp: tx_tp.to_string(),
        tx_sts: tx_sts.clone(),
        ..Default::default()
    };

    debug!(%msg_id, %end_to_end_id, "constructed transaction relationship");

    // TODO: remove debtor_account_id from create request, use from TR
    trace!("updating pseudonyms");

    let pseudonyms_request = CreatePseudonymRequest {
        transaction_relationship: Some(transaction_relationship),
        debtor_id: data_cache.dbtr_id.to_string(),
        debtor_account_id: data_cache.dbtr_acct_id.to_string(),
        creditor_id: data_cache.cdtr_id.to_string(),
        creditor_account_id: data_cache.cdtr_acct_id.to_string(),
    };

    let mut pseudonyms_client = state.mutate_pseudonym_client.clone();

    let pseudonyms_fut = async {
        debug!("creating pseudonyms");
        let span = info_span!("create.pseudonyms.account");
        span.set_attribute(attribute::RPC_SERVICE, "pseudonyms");
        pseudonyms_client
            .create_pseudonym(pseudonyms_request)
            .instrument(span)
            .await
            .map_err(|e| {
                error!(error = %e, "failed to create pseudonyms");
                anyhow::anyhow!("could not create pseudonyms")
            })
    };

    let id = Uuid::now_v7();

    let tr_fut = async {
        let span = info_span!("create.transaction_history.pacs002");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "insert");
        span.set_attribute(attribute::DB_COLLECTION_NAME, "pacs002");

        sqlx::query!(
            "insert into pacs002 (id, document) values ($1, $2)",
            id,
            sqlx::types::Json(&request) as _
        )
        .execute(&state.services.postgres)
        .instrument(span)
        .await
        .map_err(|e| {
            error!("{e}");
            anyhow::anyhow!("could not insert transaction_history")
        })
    };
    let (_result, _resp) = tokio::try_join!(tr_fut, pseudonyms_fut)?;
    debug!(%id, %msg_id, %tx_tp, "transaction added to history");

    trace!(%msg_id, "publishing payload to ");

    let payload = Payload {
        tx_tp: tx_tp.to_string(),
        data_cache: Some(data_cache),
        transaction: Some(warden_core::message::payload::Transaction::Pacs002(
            request.clone(),
        )),
        ..Default::default()
    };

    publish_message(&state, payload, msg_id).await?;
    info!(%msg_id, "published transaction to router");
    Ok((axum::http::StatusCode::CREATED, axum::Json(request)))
}

#[tracing::instrument(skip(state, data_cache))]
async fn rebuild_entities(
    end_to_end_id: &str,
    state: &AppHandle,
    mut data_cache: Option<DataCache>,
) -> anyhow::Result<DataCache> {
    let span = info_span!("get.transaction_history.pacs008");
    span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
    span.set_attribute(attribute::DB_OPERATION_NAME, "select");
    span.set_attribute(attribute::DB_COLLECTION_NAME, "pacs008");
    span.set_attribute(attribute::DB_OPERATION_PARAMETER, end_to_end_id.to_string());
    tracing::info!(end_to_end = end_to_end_id, "rebuilding cache");

    let transaction = sqlx::query_as!(
        Row,
        r#"select id, document as "document: sqlx::types::Json<serde_json::Value>" from pacs008 where exists (
            select 1
            from jsonb_array_elements(document->'f_i_to_f_i_cstmr_cdt_trf'->'cdt_trf_tx_inf') as elem
            where elem->'pmt_id'->>'end_to_end_id' = $1
        ) limit 1"#,
        end_to_end_id
    )
    .fetch_one(&state.services.postgres)
    .instrument(span)
    .await?;

    debug!(id = ?transaction.id, "found transaction");

    let document = serde_json::from_value(transaction.document.0)?;

    if data_cache.is_none() {
        debug!(e2e_id = end_to_end_id, "attempting to rebuild data cache");
        let data_cache_value = build_data_cache(&document)?;

        set_cache(end_to_end_id, state, &data_cache_value).await?;

        let _old_value = data_cache.replace(data_cache_value);
    };

    data_cache.ok_or_else(|| anyhow::anyhow!("no pacs008 found"))
}
