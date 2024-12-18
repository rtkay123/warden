use std::sync::Arc;

use async_nats::HeaderMap;
use axum::{Json, extract::State};
use opentelemetry::global;
use prost::Message;
use redis::AsyncCommands;
use tracing::{Instrument, Span, info_span, instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;
use warden_core::{
    iso20022::pacs008::Pacs008Document,
    pseudonyms::{account::CreateAccount, transaction_relationship::TransactionRelationship},
};
use warden_infra::{cache::CacheService, tracing::opentelemetry::NatsMetadataInjector};

use crate::state::AppState;

use super::{PACS008, error::ApiError};

/// expose the Customer OpenAPI to parent module
pub fn router(state: Arc<AppState>) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(post_pacs008))
        .with_state(state)
}

/// Submit a pacs.008.001.12 transaction
#[utoipa::path(
    post,
    path = "",
    responses((
        status = OK,
        body = Pacs008Document
    )),
    operation_id = "post_pacs_008", // https://github.com/juhaku/utoipa/issues/1170
    tag = PACS008,
    request_body(
        content = Pacs008Document
    ))
]
#[instrument(skip(state))]
pub async fn post_pacs008(
    State(state): State<Arc<AppState>>,
    Json(transaction): Json<Pacs008Document>,
) -> Result<Json<Pacs008Document>, ApiError> {
    let mut headers = HeaderMap::new();

    global::get_text_map_propagator(|propagator| {
        let context = Span::current().context();
        propagator.inject_context(&context, &mut NatsMetadataInjector(&mut headers))
    });

    let transaction_id = Uuid::now_v7();
    let me = Message::encode_to_vec(&transaction);

    let pacs008_type = "pacs.008.001.12";

    let cre_dt_tm = &transaction.f_i_to_f_i_cstmr_cdt_trf.grp_hdr.cre_dt_tm;
    let msg_id = &transaction.f_i_to_f_i_cstmr_cdt_trf.grp_hdr.msg_id;

    let mut accounts = vec![];
    let mut trs = vec![];

    transaction
        .f_i_to_f_i_cstmr_cdt_trf
        .cdt_trf_tx_inf
        .iter()
        .for_each(|cdt_trf_tx_inf| {
            let end_to_end_id = cdt_trf_tx_inf.pmt_id.end_to_end_id.clone();
            let instr_id = cdt_trf_tx_inf.pmt_id.instr_id();

            let (amt, ccy) = if let Some(ref amt) = cdt_trf_tx_inf.instd_amt {
                (Some(amt.value), Some(amt.ccy.to_string()))
            } else {
                (None, None)
            };

            accounts.push((
                format!("account.{}", String::default()),
                Message::encode_to_vec(&CreateAccount::default()),
            ));

            accounts.push((
                format!("entity.{}", String::default()),
                Message::encode_to_vec(&CreateAccount::default()),
            ));

            //
            // let debtor = CreateAccountRequest::default();
            //
            // acc_futs.push(
            //     create_account(state.account_client.clone(), debtor)
            //         .instrument(info_span!("grpc.save_debtors")),
            // );
            //
            // let creditor = CreateAccountRequest::default();
            //
            // acc_futs.push(
            //     create_account(state.account_client.clone(), creditor)
            //         .instrument(info_span!("grpc.save_creditors")),
            // );

            let tr = TransactionRelationship {
                from: String::default(),
                to: String::default(),
                amt,
                ccy,
                end_to_end_id,
                pmt_inf_id: instr_id.to_string(),
                cre_dt_tm: cre_dt_tm.clone(),
                msg_id: msg_id.clone(),
                tx_tp: pacs008_type.to_string(),
                ..Default::default()
            };
            trs.push(tr);
        });

    match state.cache.clone() {
        CacheService::Clustered(_con) => {
            todo!()
        }
        CacheService::NonClustered(mut con) => {
            con.mset::<_, _, ()>(&accounts)
                .instrument(info_span!("cache.set.accounts.entities"))
                .await?;

            let accounts_fut = accounts.iter().map(|(key, value)| {
                let subject = format!("{}.{key}", state.nats_subjects.accounts);
                state
                    .jetstream
                    .publish_with_headers(subject, headers.clone(), value.to_vec().into())
                    .instrument(info_span!("jetstream.publish.accounts"))
            });
            futures_util::future::join_all(accounts_fut).await;

            let trs: Vec<_> = trs.iter().map(Message::encode_to_vec).collect();

            let mut tr_con = con.clone();
            let tr_key = format!("tr.{transaction_id}");
            let tr = con
                .rpush::<_, _, ()>(tr_key, trs)
                .instrument(info_span!("cache.set.relationship"));

            let tx = tr_con
                .set::<_, _, ()>(transaction_id.to_string(), me.as_slice())
                .instrument(info_span!("cache.set.transaction"));
            tokio::try_join!(tr, tx)?;
        }
    }

    let nats_subject = format!(
        "{}.{transaction_id}",
        state.nats_subjects.transaction_history
    );

    let a = state
        .jetstream
        .publish_with_headers(nats_subject.to_string(), headers, me.into())
        .instrument(info_span!("jetstream.publish.transaction"))
        .await?;
    let fut = tokio::spawn(async move { a.await }.instrument(info_span!("jetstream.ack")))
        .in_current_span();
    let _ = fut.await;

    Ok(Json(transaction))
}
