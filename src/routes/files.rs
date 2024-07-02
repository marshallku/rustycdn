use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use reqwest::StatusCode;
use std::path::PathBuf;
use tracing::error;

use crate::{
    constants::CDN_ROOT,
    env::state::AppState,
    utils::{
        fetch::fetch_and_cache,
        http::{response_error, response_file},
    },
};

pub async fn get(State(state): State<AppState>, Path(path): Path<String>) -> impl IntoResponse {
    let file_path = PathBuf::from(format!("{}/files/{}", CDN_ROOT, path));

    if file_path.exists() {
        error!("File exists but respond with Rust: {:?}", file_path);
        return response_file(&file_path).await;
    }

    if let Err(_) = fetch_and_cache(state.host, &file_path, &path).await {
        return response_error(StatusCode::NOT_FOUND);
    }

    response_file(&file_path).await
}
