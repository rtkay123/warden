use anyhow::{Result, anyhow};
use determine_outcome::determine_outcome;
use opentelemetry_semantic_conventions::attribute;
use serde::Deserialize;
use sqlx::types::BigDecimal;
use time::OffsetDateTime;
use tracing::{Instrument, error, info_span, trace};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use warden_core::{
    configuration::rule::RuleConfiguration,
    iso20022::TransactionType,
    message::{Payload, RuleResult},
};

use crate::{processor::rule::determine_outcome, state::AppHandle};

#[derive(Deserialize)]
pub struct Parameters {
    max_query_range: f64,
}

pub(super) async fn process_901(
    configuration: &RuleConfiguration,
    payload: &Payload,
    state: AppHandle,
) -> Result<RuleResult> {
    let mut rule_result = RuleResult {
        id: configuration.id.to_string(),
        version: configuration.version.to_string(),
        ..Default::default()
    };
    let c = configuration.configuration.as_ref();

    let bands = c
        .and_then(|value| {
            if value.bands.is_empty() {
                None
            } else {
                Some(&value.bands)
            }
        })
        .ok_or_else(|| anyhow!("no bands available"))?;

    let exit_conditions = c
        .and_then(|value| {
            if value.exit_conditions.is_empty() {
                None
            } else {
                Some(&value.exit_conditions)
            }
        })
        .ok_or_else(|| anyhow!("no exit conditions available"))?;

    let parameters = c
        .and_then(|value| value.parameters.as_ref())
        .ok_or_else(|| anyhow!("no parameters available"))?;

    let params: Parameters = serde_json::from_value(parameters.clone().into())
        .inspect_err(|e| error!("failed to deserailise params: {e:?}"))?;

    let unsuccessful_transaction = exit_conditions
        .iter()
        .find(|value| value.sub_rule_ref.eq(".x00"));

    if let Some(warden_core::message::payload::Transaction::Pacs002(pacs002_document)) =
        payload.transaction.as_ref()
    {
        let tx_sts = pacs002_document
            .f_i_to_f_i_pmt_sts_rpt
            .tx_inf_and_sts
            .first()
            .ok_or_else(|| anyhow::anyhow!("tx sts to be there"))?;

        if tx_sts.tx_sts().ne("ACCC") {
            let unsuccessful_transaction = unsuccessful_transaction
                .ok_or_else(|| anyhow::anyhow!("no unsuccessful transaction ref"))?;
            rule_result.reason = unsuccessful_transaction.reason.to_owned();
            rule_result.sub_rule_ref = unsuccessful_transaction.sub_rule_ref.to_owned();

            return Ok(rule_result);
        }

        let current_pacs002_timeframe: OffsetDateTime = pacs002_document
            .f_i_to_f_i_pmt_sts_rpt
            .grp_hdr
            .cre_dt_tm
            .try_into()?;

        let data_cache = payload
            .data_cache
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("data cache is missing"))?;

        let range = BigDecimal::try_from(params.max_query_range)?;

        let tx_tp = TransactionType::PACS002.to_string();

        let span = info_span!("rule.logic");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "901");
        span.set_attribute("otel.kind", "client");

        trace!("executing rule query");
        let recent_transactions = sqlx::query_scalar!(
            "select count(*) from transaction_relationship tr
                 where tr.destination = $1
                   and tr.tx_tp = $2
                   and extract(epoch from ($3::timestamptz - tr.cre_dt_tm)) * 1000 <= $4
                   and tr.cre_dt_tm <= $3::timestamptz",
            data_cache.dbtr_acct_id,
            tx_tp,
            current_pacs002_timeframe,
            range,
        )
        .fetch_one(&state.services.postgres)
        .instrument(span)
        .await?
        .ok_or_else(|| anyhow::anyhow!("no data"))?;

        determine_outcome(recent_transactions, bands.as_ref(), &mut rule_result);

        Ok(rule_result)
    } else {
        Err(anyhow::anyhow!("no valid transaction"))
    }
}
