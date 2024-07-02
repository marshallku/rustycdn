pub fn get_resize_width_from_path(path: &str) -> Option<u32> {
    path.split('.').find_map(|part| {
        if part.starts_with('w') && part[1..].chars().all(char::is_numeric) {
            part[1..].parse::<u32>().ok()
        } else {
            None
        }
    })
}

pub fn get_original_path(path: &str, has_resize: bool) -> String {
    let (dir, filename) = match path.rfind('/') {
        Some(index) => (&path[..=index], &path[index + 1..]),
        None => ("", path),
    };

    let mut parts: Vec<&str> = filename.split('.').collect();

    if parts.last() == Some(&"webp") {
        parts.pop();
    }

    if has_resize {
        parts.remove(parts.len() - 2);
    }

    format!("{}{}", dir, parts.join("."))
}
