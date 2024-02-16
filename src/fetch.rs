use reqwest::Client;
use std::{fs, path::PathBuf};
use tracing::error;

pub async fn fetch_and_cache(
    host: String,
    file_path: &PathBuf,
    path: &str,
) -> Result<(), reqwest::Error> {
    let url = format!("{}{}", host, path);
    let response = match Client::new().get(&url).send().await?.error_for_status() {
        Ok(response) => response.bytes().await?,
        Err(err) => {
            error!("Failed to fetch {}", url);
            return Err(err);
        }
    };

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    fs::write(file_path, &response).ok();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn should_fetch_file() {
        let host = "https://marshallku.com".to_string();
        let path = "/favicon.ico";
        let file_path = PathBuf::from("cdn_root/images/favicon.ico");

        let result = fetch_and_cache(host, &file_path, path).await;

        assert!(result.is_ok());
        assert!(file_path.exists());

        fs::remove_file(file_path).ok();
    }

    #[tokio::test]
    async fn should_not_create_file_if_fetch_failed() {
        let host = "https://marshallku.com".to_string();
        let path = "/must-be-404.ico";
        let file_path = PathBuf::from("cdn_root/images/must-be-404.ico");

        let result = fetch_and_cache(host, &file_path, path).await;

        assert!(result.is_err());
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn should_create_parent_directory() {
        let host = "https://marshallku.com".to_string();
        let path = "/logo/logo.svg";
        let file_path = PathBuf::from("cdn_root/images/logo/logo.svg");

        let result = fetch_and_cache(host, &file_path, path).await;

        assert!(result.is_ok());
        assert!(file_path.exists());

        fs::remove_file(file_path).ok();
    }
}
