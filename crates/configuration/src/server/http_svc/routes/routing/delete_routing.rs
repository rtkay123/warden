use axum::extract::{Path, State};
use tonic::IntoRequest;
use warden_core::configuration::routing::{
    DeleteConfigurationRequest, RoutingConfiguration, mutate_routing_server::MutateRouting,
};

use crate::{
    server::{error::AppError, http_svc::TAG_ROUTING, version::Version},
    state::AppHandle,
};

/// Delete routing configuration
#[utoipa::path(
    delete,
    path = "/{version}/routing/{id}",
    responses((
        status = OK,
        body = RoutingConfiguration
    )),
    operation_id = "delete_routing_configuration", // https://github.com/juhaku/utoipa/issues/1170
    params(
        ("version" = Version, Path, description = "API version, e.g., v1, v2, v3"),
        ("id" = String, Path, description = "Identifier for item to delete"),
    ),
    tag = TAG_ROUTING,
    )
]
#[axum::debug_handler]
#[tracing::instrument(skip(state))]
pub async fn delete(
    State(state): State<AppHandle>,
    Path(id): Path<String>,
    axum::Json(body): axum::Json<RoutingConfiguration>,
) -> Result<axum::Json<RoutingConfiguration>, AppError> {
    let body = state
        .delete_routing_configuration(DeleteConfigurationRequest { id }.into_request())
        .await?
        .into_inner();

    Ok(axum::Json(body))
}
