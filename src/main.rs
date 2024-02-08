use axum::{
    body::Body,
    extract::{Path, Request},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use reqwest::Client;
use std::{env, fs, path::PathBuf};
use tokio;
use tokio_util::io::ReaderStream;
use tower_http::trace::{self, TraceLayer};
use tracing::{error, info, Level, Span};

async fn fetch_and_cache(file_path: &PathBuf, path: &str) -> Result<(), reqwest::Error> {
    let url = format!("https://marshallku.com/{}", path);
    let response = Client::new().get(&url).send().await?.bytes().await?;
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(file_path, &response).ok();
    Ok(())
}

async fn handle_files_request(Path(path): Path<String>) -> impl IntoResponse {
    let file_path = PathBuf::from(format!("cdn_root/files/{}", path));

    if !file_path.exists() {
        if let Err(_) = fetch_and_cache(&file_path, &path).await {
            return StatusCode::NOT_FOUND.into_response();
        }
    }

    let file = match tokio::fs::File::open(file_path).await {
        Ok(file) => file,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    body.into_response()
}

async fn handle_image_request(Path(path): Path<String>) -> impl IntoResponse {
    let file_path = PathBuf::from(format!("cdn_root/images/{}", path));

    if file_path.exists() {
        error!("File exists but respond with Rust: {:?}", file_path);
        let file = match tokio::fs::File::open(file_path).await {
            Ok(file) => file,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        let stream = ReaderStream::new(file);
        let body = Body::from_stream(stream);

        return body.into_response();
    }

    let extension = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let original_path = path.split('.').next().unwrap().to_string() + "." + extension;
    let original_file_path = PathBuf::from(format!("cdn_root/images/{}", original_path));

    if !original_file_path.exists() {
        if let Err(_) = fetch_and_cache(&original_file_path, &original_path).await {
            return StatusCode::NOT_FOUND.into_response();
        }
    }

    let resize_width = path
        .split('.')
        .find(|s| s.starts_with("w"))
        .and_then(|s| s.strip_prefix("w"))
        .and_then(|s| s.parse::<u32>().ok());

    if resize_width.is_none() {
        let file = match tokio::fs::File::open(file_path).await {
            Ok(file) => file,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        let stream = ReaderStream::new(file);
        let body = Body::from_stream(stream);

        return body.into_response();
    }

    let image = match image::open(&original_file_path) {
        Ok(image) => image,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let resize_height = resize_width.unwrap() * image.height() / image.width();
    let resized_image = image.thumbnail(resize_width.unwrap(), resize_height);

    match resized_image.save(file_path.clone()) {
        Ok(_) => {
            let file = match tokio::fs::File::open(file_path).await {
                Ok(file) => file,
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };
            let stream = ReaderStream::new(file);
            let body = Body::from_stream(stream);

            body.into_response()
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

fn trace_layer_on_request(request: &Request<Body>, _span: &Span) {
    let user_agent = request
        .headers()
        .get("user-agent")
        .map_or("<no user-agent>", |h| {
            h.to_str().unwrap_or("<invalid utf8>")
        });

    let referer = request
        .headers()
        .get("referer")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("<no referer>");

    let ip_address = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|value| value.to_str().ok())
        .unwrap_or("<no ip>");

    tracing::info!(
        "User-Agent: {:?} Referrer: {:?} IP: {:?}",
        user_agent,
        referer,
        ip_address
    )
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let app = Router::new()
        .route("/files/*path", get(handle_files_request))
        .route("/images/*path", get(handle_image_request))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
                .on_request(trace_layer_on_request),
        );

    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| String::from("127.0.0.1"));
    let addr = format!("{}:41890", bind_address);
    let listener = tokio::net::TcpListener::bind(addr.as_str()).await.unwrap();
    info!("Server running at http://{}", addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
