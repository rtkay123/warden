pub mod create_typology;
pub mod delete_typology;
pub mod get_typology;
pub mod post_typology;

#[cfg(test)]
mod tests {
    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
    };
    use sqlx::PgPool;
    use tower::ServiceExt;
    use warden_core::configuration::typology::TypologyConfiguration;
    use warden_stack::cache::RedisManager;

    use crate::{
        server::http_svc::{build_router, routes::test_config},
        state::{AppState, Services},
    };

    #[sqlx::test]
    async fn all_operations(pool: PgPool) {
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

        let response = app
            .clone()
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

        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/typology?id=999&version=1.0.0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let config: TypologyConfiguration = serde_json::from_slice(&body).unwrap();

        assert_eq!(&config.id, "999");
        assert_eq!(&config.version, "1.0.0");

        let typology = serde_json::json!({
              "description": "Test description",
              "typology_name": "Rule-901-Typology-999",
              "id": "901",
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
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/typology?id=999&version=1.0.0")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/typology?id=901&version=1.0.0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let config: TypologyConfiguration = serde_json::from_slice(&body).unwrap();

        assert_eq!(&config.id, "901");

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/typology?id=901&version=1.0.0")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
