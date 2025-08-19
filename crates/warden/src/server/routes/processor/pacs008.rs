use axum::{extract::State, response::IntoResponse};
use tracing::{Instrument, Span, debug, error, info, instrument, trace, trace_span, warn};
use uuid::Uuid;
use warden_core::{
    google::r#type::Money,
    iso20022::{TransactionType, pacs008::Pacs008Document},
    message::DataCache,
    pseudonyms::transaction_relationship::{CreatePseudonymRequest, TransactionRelationship},
};
use warden_stack::{
    opentelemetry_semantic_conventions::attribute, redis::AsyncCommands,
    tracing_opentelemetry::OpenTelemetrySpanExt,
};

use crate::{
    error::AppError,
    server::{publish::publish_message, routes::PACS008_001_12},
    state::AppHandle,
    version::Version,
};

/// Submit a pacs.008.001.12 transaction
#[utoipa::path(
    post,
    responses((
        status = CREATED,
        body = Pacs008Document
    )),
    operation_id = "post_pacs_008", // https://github.com/juhaku/utoipa/issues/1170
    path = "/{version}/pacs008",
    params(
        ("version" = Version, Path, description = "API version, e.g., v1, v2, v3")
    ),
    tag = PACS008_001_12,
    request_body(
        content = Pacs008Document
    ))
]
#[axum::debug_handler]
#[tracing::instrument(
    skip(state, transaction),
    err(Debug),
    fields(method = "POST", end_to_end_id, msg_id, tx_tp)
)]
pub(super) async fn post_pacs008(
    version: Version,
    State(state): State<AppHandle>,
    axum::Json(transaction): axum::Json<Pacs008Document>,
) -> Result<impl IntoResponse, AppError> {
    let tx_tp = TransactionType::PACS008.to_string();
    tracing::Span::current().record("tx_tp", &tx_tp);
    let data_cache = build_data_cache(&transaction)?;

    let tx_count = transaction.f_i_to_f_i_cstmr_cdt_trf.cdt_trf_tx_inf.len();
    let msg_id = &transaction.f_i_to_f_i_cstmr_cdt_trf.grp_hdr.msg_id;

    if transaction.f_i_to_f_i_cstmr_cdt_trf.cdt_trf_tx_inf.len() > 1 {
        warn!(
            msg_id,
            tx_count,
            "found more than 1 transaction for this message. Only the first will be evaluated"
        );
    }

    // take the first - guaranteed by utoipa
    trace!("extracting first credit transfer transaction info");
    let cdt_trf_tx_inf = transaction
        .f_i_to_f_i_cstmr_cdt_trf
        .cdt_trf_tx_inf
        .first()
        .ok_or_else(|| anyhow::anyhow!("required cdt_trf_tx_inf missing"))?;

    let amount = cdt_trf_tx_inf.instd_amt.as_ref().map(|value| value.value);

    let ccy = cdt_trf_tx_inf
        .instd_amt
        .as_ref()
        .map(|value| value.ccy.as_str());

    let end_to_end_id = cdt_trf_tx_inf.pmt_id.end_to_end_id.as_str();

    tracing::Span::current().record("end_to_end_id", end_to_end_id);
    let end_to_end_id = String::from(end_to_end_id);

    let msg_id = &transaction.f_i_to_f_i_cstmr_cdt_trf.grp_hdr.msg_id;
    tracing::Span::current().record("msg_id", msg_id);

    let pmt_inf_id = cdt_trf_tx_inf.pmt_id.instr_id.as_ref().ok_or_else(|| {
        error!("missing pmt_inf_id");
        anyhow::anyhow!("missing pmt_inf_id id")
    })?;

    debug!(%msg_id, %end_to_end_id, "extracted transaction identifiers");

    let money = if let (Some(amt), Some(ccy)) = (amount, ccy) {
        Some(Money::try_from((amt, ccy)).map_err(|_e| anyhow::anyhow!("invalid currency"))?)
    } else {
        trace!(msg_id, "transaction has no amount or currency");
        None
    };

    let transaction_relationship = TransactionRelationship {
        from: data_cache.dbtr_acct_id.to_string(),
        to: data_cache.cdtr_acct_id.to_string(),
        amt: money,
        cre_dt_tm: data_cache.cre_dt_tm,
        end_to_end_id: end_to_end_id.to_string(),
        msg_id: msg_id.to_string(),
        pmt_inf_id: pmt_inf_id.into(),
        tx_tp: tx_tp.to_owned(),
        ..Default::default()
    };

    let request = CreatePseudonymRequest {
        transaction_relationship: Some(transaction_relationship),
        debtor_id: data_cache.dbtr_id.to_string(),
        debtor_account_id: data_cache.dbtr_acct_id.to_string(),
        creditor_id: data_cache.cdtr_id.to_string(),
        creditor_account_id: data_cache.cdtr_acct_id.to_string(),
    };

    debug!(%msg_id, %end_to_end_id, "constructed transaction relationship");

    let mut pseudonyms_client = state.mutate_pseudonym_client.clone();

    trace!("updating pseudonyms");

    let pseudonyms_fut = async {
        let span = trace_span!(
            "create.pseudonyms.account",
            "otel.kind" = "client",
            "rpc.service" = "pseudonyms"
        );
        pseudonyms_client
            .create_pseudonym(request)
            .instrument(span)
            .await
            .map_err(|e| {
                error!(error = %e, "failed to create pseudonyms");
                anyhow::anyhow!("could not create pseudonyms")
            })
    };

    let (_, _) = tokio::try_join!(
        pseudonyms_fut,
        set_cache(&end_to_end_id, &state, &data_cache)
    )?;
    trace!("pseudonyms saved");

    let id = Uuid::now_v7();
    debug!(%id, "inserting transaction into history");

    let span = trace_span!("create.transaction_history.pacs008");
    span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
    span.set_attribute("otel.kind", "client");
    span.set_attribute(attribute::DB_OPERATION_NAME, "insert");
    span.set_attribute(attribute::DB_COLLECTION_NAME, "pacs008");

    trace!(id = ?id, "saving transaction history");
    sqlx::query!(
        "insert into pacs008 (id, document) values ($1, $2)",
        id,
        sqlx::types::Json(&transaction) as _
    )
    .execute(&state.services.postgres)
    .instrument(span)
    .await?;
    info!(%id, %msg_id, "transaction added to history");

    let payload = warden_core::message::Payload {
        tx_tp: tx_tp.to_string(),
        transaction: Some(warden_core::message::payload::Transaction::Pacs008(
            transaction.clone(),
        )),
        data_cache: Some(data_cache),
        ..Default::default()
    };

    publish_message(&state, payload, msg_id).await?;
    trace!(%msg_id, "published transaction to stream");

    Ok((axum::http::StatusCode::CREATED, axum::Json(transaction)))
}

pub fn build_data_cache(transaction: &Pacs008Document) -> anyhow::Result<DataCache> {
    trace!("building data cache object");
    let cdt_trf_tx_inf = transaction.f_i_to_f_i_cstmr_cdt_trf.cdt_trf_tx_inf.first();

    let instd_amt = cdt_trf_tx_inf.and_then(|value| value.instd_amt.clone());

    let intr_bk_sttlm_amt = cdt_trf_tx_inf.and_then(|value| value.intr_bk_sttlm_amt.clone());

    let xchg_rate = cdt_trf_tx_inf.and_then(|value| value.xchg_rate);
    let cre_dt_tm = transaction.f_i_to_f_i_cstmr_cdt_trf.grp_hdr.cre_dt_tm;

    let dbtr_othr = cdt_trf_tx_inf.and_then(|value| {
        value
            .dbtr
            .id
            .as_ref()
            .and_then(|value| value.prvt_id.othr.first())
    });

    let debtor_id = dbtr_othr
        .and_then(|value| {
            value
                .schme_nm
                .as_ref()
                .map(|schme_nm| format!("{}{}", value.id, schme_nm.prtry))
        })
        .ok_or_else(|| anyhow::anyhow!("missing debtor id"))?;

    let cdtr_othr = cdt_trf_tx_inf.and_then(|value| {
        value.cdtr.as_ref().and_then(|value| {
            value
                .id
                .as_ref()
                .and_then(|value| value.prvt_id.othr.first())
        })
    });

    let creditor_id = cdtr_othr
        .and_then(|value| {
            value
                .schme_nm
                .as_ref()
                .map(|schme_nm| format!("{}{}", value.id, schme_nm.prtry))
        })
        .ok_or_else(|| anyhow::anyhow!("missing creditor id"))?;

    let dbtr_acct_othr = cdt_trf_tx_inf.and_then(|value| {
        value
            .dbtr_acct
            .as_ref()
            .and_then(|value| value.id.as_ref().map(|value| value.othr.clone()))
    });
    let dbtr_mmb_id = cdt_trf_tx_inf.and_then(|value| {
        value.dbtr_agt.as_ref().and_then(|value| {
            value
                .fin_instn_id
                .clr_sys_mmb_id
                .as_ref()
                .map(|value| value.mmb_id.as_str())
        })
    });

    let debtor_acct_id = if let (Some(a), Some(b)) = (dbtr_acct_othr, dbtr_mmb_id) {
        Some(format!("{}{b}", a.id))
    } else {
        None
    }
    .ok_or_else(|| anyhow::anyhow!("missing debtor_acct_id"))?;

    let cdtr_acct_othr = cdt_trf_tx_inf.and_then(|value| {
        value
            .cdtr_acct
            .as_ref()
            .and_then(|value| value.id.as_ref().map(|value| value.othr.clone()))
    });
    let cdtr_mmb_id = cdt_trf_tx_inf.and_then(|value| {
        value.cdtr_agt.as_ref().and_then(|value| {
            value
                .fin_instn_id
                .clr_sys_mmb_id
                .as_ref()
                .map(|value| value.mmb_id.as_str())
        })
    });

    let creditor_acct_id = if let (Some(a), Some(b)) = (cdtr_acct_othr, cdtr_mmb_id) {
        Some(format!("{}{b}", a.id))
    } else {
        None
    }
    .ok_or_else(|| anyhow::anyhow!("missing creditor_acct_id"))?;

    let data_cache = DataCache {
        cdtr_id: creditor_id.to_string(),
        dbtr_id: debtor_id.to_string(),
        dbtr_acct_id: debtor_acct_id.to_string(),
        cdtr_acct_id: creditor_acct_id.to_string(),
        cre_dt_tm: Some(cre_dt_tm),
        instd_amt,
        intr_bk_sttlm_amt,
        xchg_rate,
    };

    Ok(data_cache)
}

#[instrument(skip(state), fields(end_to_end_id = end_to_end_id))]
pub async fn set_cache(
    end_to_end_id: &str,
    state: &AppHandle,
    data_cache: &DataCache,
) -> anyhow::Result<()> {
    trace!("updating cache");
    let span = Span::current();
    span.set_attribute(attribute::DB_SYSTEM_NAME, "valkey");
    span.set_attribute(attribute::DB_OPERATION_NAME, "set");
    span.set_attribute(attribute::DB_OPERATION_PARAMETER, end_to_end_id.to_string());
    let mut cache_update = state.services.cache.get().await?;
    let bytes = prost::Message::encode_to_vec(data_cache);
    cache_update
        .set_ex::<_, _, ()>(&end_to_end_id, bytes, state.app_config.cache_ttl)
        .await
        .map_err(|e| {
            error!("cache: {e}");
            anyhow::anyhow!("internal server error")
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
    };
    use sqlx::PgPool;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};
    use tower::ServiceExt;
    use warden_stack::cache::RedisManager;

    use crate::{
        server::{self, generate_id, test_config},
        state::{AppState, Services},
    };

    #[sqlx::test]
    async fn post(pool: PgPool) {
        let config = test_config();

        let cache = RedisManager::new(&config.cache).await.unwrap();
        let client = async_nats::connect(&config.nats.hosts[0]).await.unwrap();
        let jetstream = async_nats::jetstream::new(client);

        let state = AppState::create(
            Services {
                postgres: pool,
                cache,
                jetstream,
            },
            &test_config(),
        )
        .await
        .unwrap();
        let app = server::router(state);

        let pacs = server::test_pacs008();

        let inf = &pacs.f_i_to_f_i_cstmr_cdt_trf.cdt_trf_tx_inf[0];
        let ccy = &inf.intr_bk_sttlm_amt.as_ref().unwrap().ccy;
        let end_to_end_id = &inf.pmt_id.end_to_end_id;
        let debtor_fsp = &inf.chrgs_inf[0]
            .agt
            .fin_instn_id
            .clr_sys_mmb_id
            .as_ref()
            .unwrap()
            .mmb_id;
        let creditor_fsp = &inf
            .cdtr_agt
            .as_ref()
            .unwrap()
            .fin_instn_id
            .clr_sys_mmb_id
            .as_ref()
            .unwrap()
            .mmb_id;

        let body = serde_json::to_vec(&pacs).unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/pacs008")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        post_clearance(app, end_to_end_id, ccy, debtor_fsp, creditor_fsp).await;
    }

    #[sqlx::test]
    async fn post_missing_e2e(pool: PgPool) {
        let config = test_config();

        let cache = RedisManager::new(&config.cache).await.unwrap();
        let client = async_nats::connect(&config.nats.hosts[0]).await.unwrap();
        let jetstream = async_nats::jetstream::new(client);

        let state = AppState::create(
            Services {
                postgres: pool,
                cache,
                jetstream,
            },
            &test_config(),
        )
        .await
        .unwrap();
        let app = server::router(state);
        // no end to end id

        let mut pacs = server::test_pacs008();
        pacs.f_i_to_f_i_cstmr_cdt_trf.cdt_trf_tx_inf = vec![];

        let body = serde_json::to_vec(&pacs).unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/pacs008")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
    async fn post_clearance(
        app: Router,
        end_to_end_id: &str,
        ccy: &str,
        debtor_fsp: &str,
        creditor_fsp: &str,
    ) {
        let msg_id = generate_id();
        let cre_dt_tm = OffsetDateTime::now_utc().format(&Rfc3339).unwrap();

        let v = serde_json::json!({
          "f_i_to_f_i_pmt_sts_rpt": {
            "grp_hdr": {
              "msg_id": msg_id,
              "cre_dt_tm": cre_dt_tm
            },
            "orgnl_grp_inf_and_sts": [
              {
                "grp_sts": "ACCC",
                "nb_of_txs_per_sts": [
                  {
                    "dtld_ctrl_sum": 1,
                    "dtld_nb_of_txs": "1",
                    "dtld_sts": "ACCC"
                  }
                ],
                "orgnl_cre_dt_tm": null,
                "orgnl_ctrl_sum": 2,
                "orgnl_msg_id": "ce569868c865-4986-94ac-906e46022617",
                "orgnl_msg_nm_id": "pacs.008.001.10",
                "orgnl_nb_of_txs": "1",
                "sts_rsn_inf": [
                  {
                    "addtl_inf": [
                      "Transaction accepted and settled"
                    ],
                    "orgtr": null,
                    "rsn": null
                  }
                ]
              }
            ],
            "splmtry_data": [],
            "tx_inf_and_sts": [
              {
                "orgnl_instr_id": generate_id(),
                "orgnl_end_to_end_id": end_to_end_id,
                "tx_sts": "ACCC",
                "sts_rsn_inf": [
                  {
                    "addtl_inf": [
                      "Transaction processed successfully"
                    ],
                    "orgtr": null,
                    "rsn": null
                  }
                ],
                "splmtry_data": [],
                "chrgs_inf": [
                  {
                    "amt": {
                      "value": 0,
                      "ccy": ccy
                    },
                    "agt": {
                      "fin_instn_id": {
                        "clr_sys_mmb_id": {
                          "mmb_id": debtor_fsp
                        },
                        "b_i_c_f_i": "BANKXXX",
                        "l_e_i": "1234567890",
                        "nm": "Bank"
                      }
                    }
                  },
                  {
                    "amt": {
                      "value": 0,
                      "ccy": ccy
                    },
                    "agt": {
                      "fin_instn_id": {
                        "clr_sys_mmb_id": {
                          "mmb_id": debtor_fsp
                        }
                      }
                    }
                  },
                  {
                    "amt": {
                      "value": 0,
                      "ccy": ccy
                    },
                    "agt": {
                      "fin_instn_id": {
                        "clr_sys_mmb_id": {
                          "mmb_id": creditor_fsp
                        }
                      }
                    }
                  }
                ],
                "accptnc_dt_tm": "2023-06-02T07:52:31.000Z",
                "instg_agt": {
                  "fin_instn_id": {
                    "clr_sys_mmb_id": {
                      "mmb_id": debtor_fsp
                    }
                  }
                },
                "instd_agt": {
                  "fin_instn_id": {
                    "clr_sys_mmb_id": {
                      "mmb_id": creditor_fsp
                    }
                  }
                }
              }
            ]
          }
        });
        let body = serde_json::to_vec(&v).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/pacs002")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

        dbg!(body_str);

        assert_eq!(status, StatusCode::CREATED);
    }
}
