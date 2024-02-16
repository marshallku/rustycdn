use axum::{
    body::Body,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use std::path::PathBuf;
use tokio_util::io::ReaderStream;

pub fn get_headers_without_cache() -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert("Cache-Control", "no-cache".parse().unwrap());

    headers
}

pub fn get_headers_with_cache() -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert("Cache-Control", "public, max-age=31536000".parse().unwrap());

    headers
}

pub async fn response_file(file_path: &PathBuf) -> Response {
    let file = match tokio::fs::File::open(file_path).await {
        Ok(file) => file,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                get_headers_without_cache(),
            )
                .into_response();
        }
    };
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    (get_headers_with_cache(), body).into_response()
}
