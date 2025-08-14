use axum::extract::{Path, State};
use warden_core::configuration::routing::{
    RoutingConfiguration, UpdateRoutingRequest, mutate_routing_server::MutateRouting,
};

use crate::{
    server::{error::AppError, http_svc::TAG_ROUTING, version::Version},
    state::AppHandle,
};

/// Replace routing configuration
#[utoipa::path(
    put,
    responses((
        status = OK,
        body = RoutingConfiguration
    )),
    path = "/{version}/routing/{id}",
    params(
        ("version" = Version, Path, description = "API version, e.g., v1, v2, v3"),
        ("id" = String, Path, description = "Identifier for item to replace"),
    ),
    operation_id = "replace_routing_configuration", // https://github.com/juhaku/utoipa/issues/1170
    tag = TAG_ROUTING,
    )
]
#[axum::debug_handler]
#[tracing::instrument(skip(state))]
pub async fn replace(
    version: Version,
    State(state): State<AppHandle>,
    Path(id): Path<String>,
    axum::Json(body): axum::Json<RoutingConfiguration>,
) -> Result<axum::Json<RoutingConfiguration>, AppError> {
    let response = state
        .update_routing_configuration(tonic::Request::new(UpdateRoutingRequest {
            id,
            configuration: Some(body),
        }))
        .await?
        .into_inner();
    Ok(axum::Json(response))
}
