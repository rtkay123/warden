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
    async fn create_routing(pool: PgPool) {
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

        let routing = serde_json::json!({
            "active": true,
            "name": "Public Network Map",
            "version": "1.0.0",
            "messages": [
                {
                    "id": "004",
                    "version": "1.0.0",
                    "tx_tp": "pacs.002.001.12",
                    "typologies": [
                    {
                        "id": "999",
                        "version": "1.0.0",
                        "rules": [
                        {
                            "id": "901",
                            "version": "1.0.0"
                        }
                        ]
                    }
                    ]
                }
            ]
        });

        let body = serde_json::to_vec(&routing).unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/routing")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        // should have an active one
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/routing")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        // let routing_info: RoutingConfiguration = serde_json::from_slice(&bytes).unwrap();
        //
    }
}
