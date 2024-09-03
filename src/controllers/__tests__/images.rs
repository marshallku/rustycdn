#[cfg(test)]
mod tests {

    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    use crate::{controllers::images, env::state::AppState};

    const URI: &str = "/images";

    #[tokio::test]
    async fn test_response_file() {
        let app: Router<AppState> =
            Router::new().route(&format!("{}/*path", URI), get(images::get));
        let state = AppState::from_env();
        let response = app
            .with_state(state)
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "{}/images/hpp/ic_wahlberg_product_core_48.png8.png",
                        URI
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
