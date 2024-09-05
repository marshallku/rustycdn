use axum::response::Response;
use reqwest::StatusCode;
use std::path::PathBuf;
use tracing::error;

use crate::{
    constants::CDN_ROOT,
    env::state::AppState,
    utils::{fetch::fetch_and_cache, http::response_file},
};

pub async fn process_file_request(state: &AppState, path: &str) -> Result<Response, StatusCode> {
    let file_path = PathBuf::from(format!("{}/files/{}", CDN_ROOT, path));

    if file_path.exists() {
        error!("File exists but respond with Rust: {:?}", file_path);
        return Ok(response_file(&file_path).await);
    }

    if let Err(_) = fetch_and_cache(state.host.clone(), &file_path, &path).await {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(response_file(&file_path).await)
}
