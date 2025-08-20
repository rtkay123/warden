use axum::extract::{Query, State};
use tonic::IntoRequest;
use warden_core::configuration::rule::{
    RuleConfiguration, RuleConfigurationRequest, UpdateRuleRequest,
    mutate_rule_configuration_server::MutateRuleConfiguration,
};

use crate::{
    server::{error::AppError, http_svc::TAG_RULES, version::Version},
    state::AppHandle,
};

/// Update the routing configuration
#[utoipa::path(
    put,
    path = "/{version}/rule",
    params(
        ("version" = Version, Path, description = "API version, e.g., v1, v2, v3"),
        RuleConfigurationRequest
    ),
    responses((
        status = OK,
        body = RuleConfiguration
    )),
    operation_id = "update rule configuration", // https://github.com/juhaku/utoipa/issues/1170
    tag = TAG_RULES,
)
]
#[axum::debug_handler]
#[tracing::instrument(skip(state))]
pub async fn update_rule_config(
    version: Version,
    Query(params): Query<RuleConfigurationRequest>,
    State(state): State<AppHandle>,
    axum::Json(body): axum::Json<RuleConfiguration>,
) -> Result<axum::Json<RuleConfiguration>, AppError> {
    let config = state
        .update_rule_configuration(
            UpdateRuleRequest {
                id: params.id,
                version: params.version,
                configuration: Some(body),
            }
            .into_request(),
        )
        .await?
        .into_inner();

    Ok(axum::Json(config))
}
