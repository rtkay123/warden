use axum::Json;
use utoipa_axum::{router::OpenApiRouter, routes};
use warden_core::iso20022::pain001;

/// expose the Customer OpenAPI to parent module
pub fn router() -> OpenApiRouter {
    OpenApiRouter::new().routes(routes!(create_order))
}

/// Create an order.
///
/// Create an order by basically passing through the name of the request with static id.
#[utoipa::path(post, path = "", responses((status = OK, body = pain001::Document)), tag = "pain001")]
async fn create_order(Json(order): Json<pain001::Document>) -> Json<pain001::Document> {
    let pacs = pain001::Document::default();
    Json(pacs)
}
