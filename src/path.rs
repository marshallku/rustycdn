pub fn get_resize_width_from_path(path: &str) -> Option<u32> {
    path.split('.')
        .rfind(|s| s.starts_with("w"))
        .and_then(|s| s.strip_prefix("w"))
        .and_then(|s| s.parse::<u32>().ok())
}

pub fn get_original_path(path: &str, has_resize: bool) -> String {
    let extension = path.split('.').last().unwrap_or("");
    let original_path = if has_resize {
        path.split('.').collect::<Vec<&str>>()[..path.split('.').count() - 2].join(".")
            + "."
            + extension
    } else {
        path.to_string()
    };

    original_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_resize_width_from_path() {
        assert_eq!(
            get_resize_width_from_path("/path/to/file.w100.jpg"),
            Some(100)
        );
        assert_eq!(
            get_resize_width_from_path("/path/to/file.with.dot.w200.jpg"),
            Some(200)
        );
        assert_eq!(get_resize_width_from_path("/path/to/file.jpg"), None);
        assert_eq!(
            get_resize_width_from_path("/path/to/file.with.dot.jpg"),
            None
        );
    }

    #[test]
    fn test_get_original_path() {
        let paths = vec![
            "/path/to/file.w100.jpg",
            "/path/to/file.with.dot.w100.jpg",
            "/path/to/file.jpg",
            "/path/to/file.with.dot.jpg",
        ];

        let expected = vec![
            "/path/to/file.jpg",
            "/path/to/file.with.dot.jpg",
            "/path/to/file.jpg",
            "/path/to/file.with.dot.jpg",
        ];

        for (i, path) in paths.iter().enumerate() {
            assert_eq!(
                get_original_path(path, get_resize_width_from_path(path).is_some()),
                expected[i]
            );
        }
    }
}