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

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use sqlx::PgPool;
    use tower::ServiceExt;
    use warden_stack::cache::RedisManager;

    use crate::{
        server::http_svc::{build_router, routes::test_config},
        state::{AppState, Services},
    };

    #[sqlx::test]
    async fn update(pool: PgPool) {
        let config = test_config();

        let cache = RedisManager::new(&config.cache).await.unwrap();
        let client = async_nats::connect(&config.nats.hosts[0]).await.unwrap();
        let jetstream = async_nats::jetstream::new(client);

        let state = AppState::create(
            Services {
                postgres: pool,
                cache,
                jetstream,
            },
            &test_config(),
        )
        .await
        .unwrap();

        let app = build_router(state);

        let rule = serde_json::json!({
              "id": "901",
              "version": "1.0.0",
              "description": "Number of outgoing transactions - debtor",
              "configuration": {
                "parameters": {
                  "max_query_range": 86400000
                },
                "exit_conditions": [
                  {
                    "sub_rule_ref": ".x00",
                    "reason": "Incoming transaction is unsuccessful"
                  }
                ],
                "bands": [
                  {
                    "sub_rule_ref": ".01",
                    "upper_limit": 2,
                    "reason": "The debtor has performed one transaction to date"
                  },
                  {
                    "sub_rule_ref": ".02",
                    "lower_limit": 2,
                    "upper_limit": 3,
                    "reason": "The debtor has performed two transactions to date"
                  },
                  {
                    "sub_rule_ref": ".03",
                    "lower_limit": 3,
                    "reason": "The debtor has performed three or more transactions to date"
                  }
                ]
              }
        });

        let body = serde_json::to_vec(&rule).unwrap();

        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/rule")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        let rule = serde_json::json!({
              "id": "902",
              "version": "1.0.0",
              "description": "Number of outgoing transactions - debtor",
              "configuration": {
                "parameters": {
                  "max_query_range": 86400000
                },
                "exit_conditions": [
                  {
                    "sub_rule_ref": ".x00",
                    "reason": "Incoming transaction is unsuccessful"
                  }
                ],
                "bands": []
              }
        });

        let body = serde_json::to_vec(&rule).unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/rule?id=901&version=1.0.0")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
