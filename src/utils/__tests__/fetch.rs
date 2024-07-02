#[cfg(test)]
mod tests {
    use crate::utils::fetch::fetch_and_cache;

    use std::{fs, path::PathBuf};

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
