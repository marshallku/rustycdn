use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use std::path::PathBuf;

use crate::{
    constants::CDN_ROOT, env::state::AppState, services::image::process_image_request,
    utils::http::response_error,
};

pub async fn get(State(state): State<AppState>, Path(path): Path<String>) -> impl IntoResponse {
    let file_path = PathBuf::from(format!("{}/images/{}", CDN_ROOT, path));

    match process_image_request(&state, &path, &file_path).await {
        Ok(response) => response,
        Err(status) => response_error(status),
    }
}
