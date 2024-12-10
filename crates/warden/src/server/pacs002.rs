use std::sync::Arc;

use axum::{Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};
use warden_core::iso20022::pacs002::Pacs002Document;

use crate::state::AppState;

use super::PACS002;

/// expose the Customer OpenAPI to parent module
pub fn router(state: Arc<AppState>) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(create_order))
        .with_state(state)
}

/// Submit a pacs.002.001.12 transaction
#[utoipa::path(
    post,
    path = "",
    responses((
        status = OK,
        body = Pacs002Document
    )),
    tag = PACS002,
    request_body(
        content = Pacs002Document
    ))
]
async fn create_order(
    State(state): State<Arc<AppState>>,
    Json(order): Json<Pacs002Document>,
) -> Json<Pacs002Document> {
    let pacs = Pacs002Document::default();
    Json(pacs)
}
