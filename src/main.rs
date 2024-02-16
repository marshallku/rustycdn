mod env;
mod fetch;
mod http;
mod log;
mod path;

use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use std::{fs, path::PathBuf};
use tokio;
use tokio_util::io::ReaderStream;
use tower_http::trace::{self, TraceLayer};
use tracing::{error, info, Level};

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
    let file_path = PathBuf::from(format!("cdn_root/files/{}", path));

    if !file_path.exists() {
        if let Err(_) = fetch::fetch_and_cache(state.host, &file_path, &path).await {
            return (StatusCode::NOT_FOUND, http::get_headers_without_cache()).into_response();
        }
    }

    let file = match tokio::fs::File::open(file_path).await {
        Ok(file) => file,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    (http::get_headers_with_cache(), body).into_response()
}

async fn handle_image_request(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let file_path = PathBuf::from(format!("cdn_root/images/{}", path));

    if file_path.exists() {
        error!("File exists but respond with Rust: {:?}", file_path);
        return http::response_file(&file_path).await;
    }

    let resize_width = path::get_resize_width_from_path(&path);
    let original_path = path::get_original_path(&path, resize_width.is_some());
    let original_file_path = PathBuf::from(format!("cdn_root/images/{}", original_path));

    if !original_file_path.exists() {
        if let Err(_) =
            fetch::fetch_and_cache(state.host, &original_file_path, &original_path).await
        {
            return (StatusCode::NOT_FOUND, http::get_headers_without_cache()).into_response();
        }
    }

    if resize_width.is_none() {
        return http::response_file(&file_path).await;
    }

    let image = match image::open(&original_file_path) {
        Ok(image) => image,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                http::get_headers_without_cache(),
            )
                .into_response();
        }
    };

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
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                http::get_headers_without_cache(),
            )
                .into_response();
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
