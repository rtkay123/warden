use axum::Json;
use utoipa_axum::{router::OpenApiRouter, routes};
use warden_core::iso20022::pacs008::Pacs008Document;

use super::PACS008;

/// expose the Customer OpenAPI to parent module
pub fn router() -> OpenApiRouter {
    OpenApiRouter::new().routes(routes!(create_order))
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
async fn create_order(Json(order): Json<Pacs008Document>) -> Json<Pacs008Document> {
    let pacs = Pacs008Document::default();
    Json(pacs)
}
