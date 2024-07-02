#[cfg(test)]
mod tests {
    use http_body_util::BodyExt;
    use reqwest::StatusCode;
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    use crate::utils::http::{get_cache_header, response_error, response_file};

    #[tokio::test]
    async fn test_response_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(file_path.clone())
            .and_then(|file| Ok(file))
            .unwrap();

        file.write_all(b"test").unwrap();

        let response = response_file(&file_path).await;
        let body = response.collect().await.unwrap().to_bytes();

        assert_eq!(
            body,
            "test".as_bytes(),
            "File content does not match expected content"
        );
    }

    #[tokio::test]
    async fn test_response_file_with_empty_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("empty.txt");
        File::create(&file_path).unwrap();

        let response = response_file(&file_path).await;
        let body = response.collect().await.unwrap().to_bytes();

        assert_eq!(
            body,
            "".as_bytes(),
            "Response body is not empty for empty file"
        );
    }

    #[tokio::test]
    async fn test_response_file_with_large_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large.txt");
        let mut file = File::create(&file_path).unwrap();

        let large_content = vec![b'a'; 10_000];
        file.write_all(&large_content).unwrap();

        let response = response_file(&file_path).await;
        let body = response.collect().await.unwrap().to_bytes();

        assert_eq!(
            body,
            large_content.as_slice(),
            "Response body does not match large file content"
        );
    }

    #[tokio::test]
    async fn test_response_file_with_error() {
        let file_path = PathBuf::from("non-existing-file.txt");
        let response = response_file(&file_path).await;

        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Expected internal server error for non-existing file"
        );
    }

    #[test]
    fn test_get_cache_header() {
        let headers = get_cache_header(100);
        let cache_control = headers.get("Cache-Control").unwrap().to_str().unwrap();

        assert_eq!(
            cache_control, "public, max-age=100",
            "Cache-Control header does not match with positive cache age"
        );
    }

    #[test]
    fn test_get_cache_header_with_falsy_age() {
        let headers = get_cache_header(0);
        let cache_control = headers.get("Cache-Control").unwrap().to_str().unwrap();

        assert_eq!(
            cache_control, "no-cache",
            "Cache-Control header does not match 'no-cache' for age 0"
        );
    }

    #[test]
    fn test_response_error() {
        let response = response_error(StatusCode::NOT_FOUND);
        let status = response.status();

        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "Status code does not match NOT_FOUND"
        );
    }

    #[test]
    fn test_response_error_with_cache() {
        let response = response_error(StatusCode::NOT_FOUND);
        let headers = response.headers();
        let cache_control = headers.get("Cache-Control").unwrap().to_str().unwrap();

        assert_eq!(
            cache_control, "no-cache",
            "Cache-Control header does not match 'no-cache' in error response"
        );
    }
}
