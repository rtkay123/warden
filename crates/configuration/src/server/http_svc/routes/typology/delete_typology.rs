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
    async fn delete_typology(pool: PgPool) {
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

        let typology = serde_json::json!({
              "description": "Test description",
              "typology_name": "Rule-901-Typology-999",
              "id": "999",
              "version": "1.0.0",
              "workflow": {
                "alert_threshold": 200,
                "interdiction_threshold": 400
              },
              "rules": [
                {
                  "id": "901",
                  "version": "1.0.0",
                  "wghts": [
                    {
                      "ref": ".err",
                      "wght": 0
                    },
                    {
                      "ref": ".x00",
                      "wght": 100
                    },
                    {
                      "ref": ".01",
                      "wght": 100
                    },
                    {
                      "ref": ".02",
                      "wght": 200
                    },
                    {
                      "ref": ".03",
                      "wght": 400
                    }
                  ]
                }
              ],
              "expression": {
                "operator": "ADD",
                "terms": [
                  {
                    "id": "901",
                    "version": "1.0.0"
                  }
                ]
              }

        });

        let body = serde_json::to_vec(&typology).unwrap();

        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/typology")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/typology?id=999&version=1.0.0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
