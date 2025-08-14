use axum::{extract::State, response::IntoResponse};
use warden_core::configuration::rule::{
    RuleConfiguration, mutate_rule_configuration_server::MutateRuleConfiguration,
};

use crate::{
    server::{error::AppError, http_svc::TAG_RULES, version::Version},
    state::AppHandle,
};

/// Create rule configuration
#[utoipa::path(
    post,
    path = "/{version}/rule",
    params(
        ("version" = Version, Path, description = "API version, e.g., v1, v2, v3"),
    ),
    responses((
        status = CREATED,
        body = RuleConfiguration,
    )),
    operation_id = "create_rule_configuration", // https://github.com/juhaku/utoipa/issues/1170
    tag = TAG_RULES,
    )
]
#[axum::debug_handler]
#[tracing::instrument(skip(state))]
pub async fn create_rule(
    version: Version,
    State(state): State<AppHandle>,
    axum::Json(body): axum::Json<RuleConfiguration>,
) -> Result<impl IntoResponse, AppError> {
    let response = state
        .create_rule_configuration(tonic::Request::new(body))
        .await?
        .into_inner();
    Ok((axum::http::StatusCode::CREATED, axum::Json(response)))
}
