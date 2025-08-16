use axum::extract::{Query, State};
use tonic::IntoRequest;
use warden_core::configuration::typology::{
    DeleteTypologyConfigurationRequest, TypologyConfiguration,
    mutate_typologies_server::MutateTypologies,
};

use crate::{
    server::{error::AppError, http_svc::TAG_TYPOLOGIES, version::Version},
    state::AppHandle,
};

/// Get the typology configuration
#[utoipa::path(
    delete,
    path = "/{version}/typology",
    responses((
        status = OK,
        body = TypologyConfiguration
    )),
    params(
        ("version" = Version, Path, description = "API version, e.g., v1, v2, v3"),
        DeleteTypologyConfigurationRequest
    ),
    operation_id = "delete_typology_configuration", // https://github.com/juhaku/utoipa/issues/1170
    tag = TAG_TYPOLOGIES,
    )
]
#[axum::debug_handler]
#[tracing::instrument(skip(state))]
pub async fn delete_typology(
    State(state): State<AppHandle>,
    Query(params): Query<DeleteTypologyConfigurationRequest>,
) -> Result<axum::Json<TypologyConfiguration>, AppError> {
    let config = state
        .delete_typology_configuration(params.into_request())
        .await?
        .into_inner();

    Ok(axum::Json(config))
}
