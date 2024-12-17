use std::sync::Arc;

use axum::{Json, extract::State};
use tracing::instrument;
use utoipa_axum::{router::OpenApiRouter, routes};
use warden_core::iso20022::pacs008::Pacs008Document;

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
    let pacs = Pacs008Document::default();
    Ok(Json(pacs))
}
