use axum::response::IntoResponse;

pub async fn health_check() -> impl IntoResponse {
    let name = env!("CARGO_PKG_NAME");
    let ver = env!("CARGO_PKG_VERSION");

    format!("{name} v{ver} is live")
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use crate::{
        server::{self, test_config},
        state::AppState,
    };

    #[tokio::test]
    async fn health_check() {
        let state = AppState::create( &test_config()).await.unwrap();
        let app = server::router(state);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}

