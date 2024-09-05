use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use crate::{
    env::state::AppState, services::file::process_file_request, utils::http::response_error,
};

pub async fn get(State(state): State<AppState>, Path(path): Path<String>) -> impl IntoResponse {
    match process_file_request(&state, &path).await {
        Ok(response) => response,
        Err(status) => response_error(status),
    }
}
