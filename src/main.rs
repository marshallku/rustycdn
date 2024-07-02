mod constants;
mod env;
mod utils;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use constants::CDN_ROOT;
use env::state::AppState;
use reqwest::StatusCode;
use std::path::PathBuf;
use tokio;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{error, info, Level};
use utils::{
    fetch::fetch_and_cache,
    http::{response_error, response_file},
    img::{save_image_to_webp, save_resized_image},
    log::trace_layer_on_request,
    path::{get_original_path, get_resize_width_from_path},
};

async fn handle_files_request(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
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

async fn handle_image_request(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let state = AppState::from_env();
    let addr = format!("{}:{}", state.address.to_string(), state.port.to_string());
    let app = Router::new()
        .route("/files/*path", get(handle_files_request))
        .route("/images/*path", get(handle_image_request))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_request(trace_layer_on_request),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr.as_str()).await.unwrap();

    info!("Server running at http://{}", addr);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap()
}
