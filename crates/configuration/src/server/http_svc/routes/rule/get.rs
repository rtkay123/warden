use axum::extract::{Query, State};
use warden_core::configuration::rule::{
    RuleConfiguration, RuleConfigurationRequest,
    query_rule_configuration_server::QueryRuleConfiguration,
};

use crate::{
    server::{error::AppError, http_svc::TAG_RULES, version::Version},
    state::AppHandle,
};

/// Get rule configuration
#[utoipa::path(
    get,
    path = "/{version}/rule",
    responses((
        status = OK,
        body = RuleConfiguration
    )),
    params(
        ("version" = Version, Path, description = "API version, e.g., v1, v2, v3"),
        RuleConfigurationRequest
    ),
    operation_id = "get_rule_configuration", // https://github.com/juhaku/utoipa/issues/1170
    tag = TAG_RULES,
    )
]
#[axum::debug_handler]
#[tracing::instrument(skip(state))]
pub async fn get_rule(
    version: Version,
    State(state): State<AppHandle>,
    Query(body): Query<RuleConfigurationRequest>,
) -> Result<axum::Json<Option<RuleConfiguration>>, AppError> {
    let response = state
        .get_rule_configuration(tonic::Request::new(body))
        .await?
        .into_inner()
        .configuration;

    Ok(axum::Json(response))
}
