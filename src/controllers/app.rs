use axum::{routing::get, Router};

use crate::env::state::AppState;

pub fn app() -> Router<AppState> {
    Router::new()
        .route("/files/*path", get(super::files::get))
        .route("/images/*path", get(super::images::get))
}
