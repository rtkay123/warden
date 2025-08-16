use axum::extract::State;
use warden_core::configuration::typology::{
    TypologyConfiguration, UpdateTypologyConfigRequest, mutate_typologies_server::MutateTypologies,
};

use crate::{
    server::{error::AppError, http_svc::TAG_TYPOLOGIES, version::Version},
    state::AppHandle,
};

/// Update typology configuration
#[utoipa::path(
    put,
    path = "/{version}/typology",
    params(
        ("version" = Version, Path, description = "API version, e.g., v1, v2, v3"),
    ),
    responses((
        status = OK,
        body = TypologyConfiguration
    )),
    operation_id = "update_typology_configuration", // https://github.com/juhaku/utoipa/issues/1170
    tag = TAG_TYPOLOGIES,
    )
]
#[axum::debug_handler]
#[tracing::instrument(skip(state))]
pub async fn update(
    State(state): State<AppHandle>,
    axum::Json(body): axum::Json<TypologyConfiguration>,
) -> Result<axum::Json<TypologyConfiguration>, AppError> {
    let response = state
        .update_typology_configuration(tonic::Request::new(UpdateTypologyConfigRequest {
            configuration: Some(body),
        }))
        .await?
        .into_inner();
    Ok(axum::Json(response))
}
