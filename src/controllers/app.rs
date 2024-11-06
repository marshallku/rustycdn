use axum::{routing::get, Router};

use crate::env::state::AppState;

pub fn app() -> Router<AppState> {
    Router::new()
        .route("/health", get(super::health::get))
        .route("/files/*path", get(super::files::get))
        .route("/images/*path", get(super::images::get))
}
