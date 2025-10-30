pub mod create;
pub mod delete;
pub mod get;
pub mod update;

#[cfg(test)]
mod tests {
    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
    };
    use sqlx::PgPool;
    use tower::ServiceExt;
    use warden_core::configuration::rule::RuleConfiguration;
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

        let response = app
            .clone()
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

        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/rule?id=901&version=1.0.0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let config: RuleConfiguration = serde_json::from_slice(&body).unwrap();

        assert_eq!(&config.id, "901");
        assert_eq!(&config.version, "1.0.0");

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

        app.clone()
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

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/rule?id=902&version=1.0.0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let config: RuleConfiguration = serde_json::from_slice(&body).unwrap();

        assert_eq!(&config.id, "902");
        assert!(&config.configuration.unwrap().bands.is_empty());

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .header("Content-Type", "application/json")
                    .uri("/api/v0/rule?id=902&version=1.0.0")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
