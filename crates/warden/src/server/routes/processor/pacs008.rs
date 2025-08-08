use axum::{extract::State, response::IntoResponse};
use warden_core::iso20022::pacs008::Pacs008Document;

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
    Ok(String::default())
}
