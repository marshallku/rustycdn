use axum::{
    body::Body,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use std::path::PathBuf;
use tokio_util::io::ReaderStream;

pub fn get_cache_header(age: u32) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let cache_age = if age <= 0 {
        "no-cache"
    } else {
        "public, max-age=31536000"
    };

    headers.insert("Cache-Control", cache_age.parse().unwrap());

    headers
}

pub fn response_error(status_code: StatusCode) -> Response {
    (status_code, get_cache_header(0)).into_response()
}

pub async fn response_file(file_path: &PathBuf) -> Response {
    let file = match tokio::fs::File::open(file_path).await {
        Ok(file) => file,
        Err(_) => {
            return response_error(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    (get_cache_header(31536000), body).into_response()
}
