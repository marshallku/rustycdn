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
        img::{save_image_to_webp, save_resized_image},
        path::{get_original_path, get_resize_width_from_path},
    },
};

pub async fn get(State(state): State<AppState>, Path(path): Path<String>) -> impl IntoResponse {
    let file_path = PathBuf::from(format!("{}/images/{}", CDN_ROOT, path));

    if file_path.exists() {
        error!("File exists but respond with Rust: {:?}", file_path);
        return response_file(&file_path).await;
    }

    let resize_width = get_resize_width_from_path(&path);
    let convert_to_webp = path.ends_with(".webp");
    let original_path = get_original_path(&path, resize_width.is_some());
    let original_file_path = PathBuf::from(format!("{}/images/{}", CDN_ROOT, original_path));

    if !original_file_path.exists() {
        if let Err(_) = fetch_and_cache(state.host, &original_file_path, &original_path).await {
            return response_error(StatusCode::NOT_FOUND);
        }
    }

    if resize_width.is_none() && !convert_to_webp {
        return response_file(&file_path).await;
    }

    let image = match image::open(&original_file_path) {
        Ok(image) => image,
        Err(_) => {
            return response_error(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if !convert_to_webp {
        return save_resized_image(image, resize_width, &original_file_path, &file_path).await;
    }

    let path_with_webp = format!("{}.webp", original_path);
    let file_path_with_webp = PathBuf::from(format!("{}/images/{}", CDN_ROOT, path_with_webp));

    if let Err(_) = save_image_to_webp(&image, &file_path_with_webp) {
        return response_error(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let image_webp = match image::open(&file_path_with_webp) {
        Ok(image) => image,
        Err(_) => {
            return response_error(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    save_resized_image(image_webp, resize_width, &file_path_with_webp, &file_path).await
}
