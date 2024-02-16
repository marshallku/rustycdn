mod env;
mod fetch;
mod http;
mod img;
mod log;
mod path;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use std::{fs, path::PathBuf};
use tokio;
use tower_http::trace::{self, TraceLayer};
use tracing::{error, info, Level};

const CDN_ROOT: &str = "cdn_root";

#[derive(Clone)]
pub struct AppState {
    host: String,
    port: u16,
    address: String,
}

impl AppState {
    pub fn from_env() -> Self {
        let env = env::Env::new();

        Self {
            host: env.host.into_owned(),
            port: env.port,
            address: env.address.into_owned(),
        }
    }
}

async fn handle_files_request(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let file_path = PathBuf::from(format!("{}/files/{}", CDN_ROOT, path));

    if file_path.exists() {
        error!("File exists but respond with Rust: {:?}", file_path);
        return http::response_file(&file_path).await;
    }

    if let Err(_) = fetch::fetch_and_cache(state.host, &file_path, &path).await {
        return http::response_error(StatusCode::NOT_FOUND);
    }

    http::response_file(&file_path).await
}

async fn handle_image_request(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let file_path = PathBuf::from(format!("{}/images/{}", CDN_ROOT, path));

    if file_path.exists() {
        error!("File exists but respond with Rust: {:?}", file_path);
        return http::response_file(&file_path).await;
    }

    let resize_width = path::get_resize_width_from_path(&path);
    let convert_to_webp = path.ends_with(".webp");
    let original_path = path::get_original_path(&path, resize_width.is_some());
    let original_file_path = PathBuf::from(format!("{}/images/{}", CDN_ROOT, original_path));

    if !original_file_path.exists() {
        if let Err(_) =
            fetch::fetch_and_cache(state.host, &original_file_path, &original_path).await
        {
            return http::response_error(StatusCode::NOT_FOUND);
        }
    }

    if resize_width.is_none() && !convert_to_webp {
        return http::response_file(&file_path).await;
    }

    let image = match image::open(&original_file_path) {
        Ok(image) => image,
        Err(_) => {
            return http::response_error(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if convert_to_webp {
        let original_path_with_webp = format!("{}.webp", original_path);
        let original_file_path_with_webp =
            PathBuf::from(format!("{}/images/{}", CDN_ROOT, original_path_with_webp));

        if let Err(_) = img::save_image_to_webp(image.clone(), &original_file_path_with_webp) {
            return http::response_error(StatusCode::INTERNAL_SERVER_ERROR);
        }

        let image_webp = match image::open(&original_file_path_with_webp) {
            Ok(image) => image,
            Err(_) => {
                return http::response_error(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        if image_webp.width() <= resize_width.unwrap() {
            fs::copy(&original_file_path_with_webp, &file_path).ok();
            return http::response_file(&file_path).await;
        }

        let resize_height = resize_width.unwrap() * image_webp.height() / image_webp.width();
        let resized_image_webp = image_webp.thumbnail(resize_width.unwrap(), resize_height);

        match resized_image_webp.save(file_path.clone()) {
            Ok(_) => {
                return http::response_file(&file_path).await;
            }
            Err(_) => {
                return http::response_error(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    if image.width() <= resize_width.unwrap() {
        fs::copy(&original_file_path, &file_path).ok();
        return http::response_file(&file_path).await;
    }

    let resize_height = resize_width.unwrap() * image.height() / image.width();
    let resized_image = image.thumbnail(resize_width.unwrap(), resize_height);

    match resized_image.save(file_path.clone()) {
        Ok(_) => {
            return http::response_file(&file_path).await;
        }
        Err(_) => {
            return http::response_error(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
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
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
                .on_request(log::trace_layer_on_request),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr.as_str()).await.unwrap();

    info!("Server running at http://{}", addr);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap()
}
