use axum::{extract::State, response::IntoResponse};
use warden_core::configuration::routing::RoutingConfiguration;

use crate::{
    server::{error::AppError, http_svc::TAG_ROUTING, version::Version},
    state::AppHandle,
};

#[utoipa::path(
    get,
    responses((
        status = OK,
        body = RoutingConfiguration
    )),
    operation_id = "get_active_routing", // https://github.com/juhaku/utoipa/issues/1170
    path = "/{version}/routing",
    params(
        ("version" = Version, Path, description = "API version, e.g., v1, v2, v3")
    ),
    tag = TAG_ROUTING,
    )
]
#[axum::debug_handler]
#[tracing::instrument(skip(state), err(Debug), fields(method = "GET"))]
pub(super) async fn active_routing(
    version: Version,
    State(state): State<AppHandle>,
) -> Result<impl IntoResponse, AppError> {
    Ok(String::default().into_response())
}
