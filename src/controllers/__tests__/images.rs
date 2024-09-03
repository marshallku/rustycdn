#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    use crate::{constants::CDN_ROOT, controllers::images, env::state::AppState};

    const URI: &str = "/images";

    #[tokio::test]
    async fn test_response_file() {
        let app: Router<AppState> =
            Router::new().route(&format!("{}/*path", URI), get(images::get));
        let state = AppState::from_env();
        let file_path = "/images/hpp/ic_wahlberg_product_core_48.png8.png";
        let response = app
            .with_state(state)
            .oneshot(
                Request::builder()
                    .uri(format!("{}{}", URI, file_path))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let local_file_path = PathBuf::from(format!("{}{}{}", CDN_ROOT, URI, file_path));

        assert_eq!(response.status(), StatusCode::OK);
        assert!(local_file_path.exists());
    }
}
