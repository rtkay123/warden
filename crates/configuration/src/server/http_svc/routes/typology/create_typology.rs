use axum::{extract::State, response::IntoResponse};
use warden_core::configuration::typology::{
    TypologyConfiguration, mutate_typologies_server::MutateTypologies,
};

use crate::{
    server::{error::AppError, http_svc::TAG_TYPOLOGIES, version::Version},
    state::AppHandle,
};

/// Create rule configuration
#[utoipa::path(
    post,
    path = "/{version}/typology",
    params(
        ("version" = Version, Path, description = "API version, e.g., v1, v2, v3"),
    ),
    responses((
        status = CREATED,
        body = TypologyConfiguration,
    )),
    operation_id = "create_typology_configuration", // https://github.com/juhaku/utoipa/issues/1170
    tag = TAG_TYPOLOGIES,
    )
]
#[axum::debug_handler]
#[tracing::instrument(skip(state))]
pub async fn create_typology(
    version: Version,
    State(state): State<AppHandle>,
    axum::Json(body): axum::Json<TypologyConfiguration>,
) -> Result<impl IntoResponse, AppError> {
    let response = state
        .create_typology_configuration(tonic::Request::new(body))
        .await?
        .into_inner();
    Ok((axum::http::StatusCode::CREATED, axum::Json(response)))
}
