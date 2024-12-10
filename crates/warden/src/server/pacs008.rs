use std::sync::Arc;

use axum::{extract::State, Json};
use utoipa_axum::{router::OpenApiRouter, routes};
use warden_core::iso20022::pacs008::Pacs008Document;

use crate::state::AppState;

use super::PACS008;

/// expose the Customer OpenAPI to parent module
pub fn router(state: Arc<AppState>) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(create_order))
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
    tag = PACS008,
    request_body(
        content = Pacs008Document
    ))
]
async fn create_order(
    State(state): State<Arc<AppState>>,
    Json(order): Json<Pacs008Document>,
) -> Json<Pacs008Document> {
    let pacs = Pacs008Document::default();
    Json(pacs)
}
