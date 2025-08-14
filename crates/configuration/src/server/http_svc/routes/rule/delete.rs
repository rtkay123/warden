use axum::extract::{Query, State};
use tonic::IntoRequest;
use warden_core::configuration::rule::{
    DeleteRuleConfigurationRequest, RuleConfiguration,
    mutate_rule_configuration_server::MutateRuleConfiguration,
};

use crate::{
    server::{error::AppError, http_svc::TAG_RULES, version::Version},
    state::AppHandle,
};

/// Delete rule configuration
#[utoipa::path(
    delete,
    path = "/{version}/rule",
    responses((
        status = OK,
        body = RuleConfiguration
    )),
    params(
        ("version" = Version, Path, description = "API version, e.g., v1, v2, v3"),
        DeleteRuleConfigurationRequest,
    ),
    operation_id = "delete_rule_configuration", // https://github.com/juhaku/utoipa/issues/1170
    tag = TAG_RULES,
    )
]
#[axum::debug_handler]
#[tracing::instrument(skip(state))]
pub async fn delete_rule_config(
    version: Version,
    State(state): State<AppHandle>,
    Query(body): Query<DeleteRuleConfigurationRequest>,
) -> Result<axum::Json<RuleConfiguration>, AppError> {
    let body = state
        .delete_rule_configuration(body.into_request())
        .await?
        .into_inner();

    Ok(axum::Json(body))
}
