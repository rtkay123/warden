use axum::{extract::State, response::IntoResponse};
use tracing::{error, trace, warn};
use warden_core::{
    iso20022::{TransactionType, pacs008::Pacs008Document},
    message::DataCache,
};

use crate::{error::AppError, server::routes::PACS008_001_12, state::AppHandle, version::Version};

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
    let tx_tp = TransactionType::PACS008;
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

    // take the first
    trace!("extracting first credit transfer transaction info");
    let cdt_trf_tx_inf = transaction.f_i_to_f_i_cstmr_cdt_trf.cdt_trf_tx_inf.first();

    let amount = cdt_trf_tx_inf.and_then(|value| value.instd_amt.as_ref().map(|value| value.value));

    let ccy =
        cdt_trf_tx_inf.and_then(|value| value.instd_amt.as_ref().map(|value| value.ccy.as_str()));

    let end_to_end_id = cdt_trf_tx_inf
        .as_ref()
        .map(|value| value.pmt_id.end_to_end_id.as_str())
        .ok_or_else(|| {
            anyhow::anyhow!("missing end_to_end_id id")
        })?;

    Ok(String::default())
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
