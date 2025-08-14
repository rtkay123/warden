use axum::{extract::State, response::IntoResponse};
use warden_core::configuration::routing::{
    RoutingConfiguration, mutate_routing_server::MutateRouting,
};

use crate::{
    server::{error::AppError, http_svc::TAG_ROUTING, version::Version},
    state::AppHandle,
};

/// Create routing configuration
#[utoipa::path(
    post,
    responses((
        status = CREATED,
        body = RoutingConfiguration
    )),
    operation_id = "post_routing_configuration", // https://github.com/juhaku/utoipa/issues/1170
    path = "/{version}/routing",
    params(
        ("version" = Version, Path, description = "API version, e.g., v1, v2, v3")
    ),
    tag = TAG_ROUTING,
)
]
#[axum::debug_handler]
#[tracing::instrument(skip(state), err(Debug), fields(method = "POST"))]
pub async fn post_routing(
    version: Version,
    State(state): State<AppHandle>,
    axum::Json(body): axum::Json<RoutingConfiguration>,
) -> Result<impl IntoResponse, AppError> {
    let response = state
        .create_routing_configuration(tonic::Request::new(body))
        .await?
        .into_inner();

    Ok((axum::http::StatusCode::CREATED, axum::Json(response)))
}
