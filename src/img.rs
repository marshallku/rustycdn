use axum::{http::StatusCode, response::Response};
use std::{
    fs::{copy, write},
    path::PathBuf,
};
use webp::Encoder;

use crate::http::{response_error, response_file};

pub fn save_image_to_webp(image: &image::DynamicImage, path: &PathBuf) -> Result<(), String> {
    let encoder = match Encoder::from_image(&image) {
        Ok(e) => e,
        Err(e) => {
            return Err(e.to_string());
        }
    };
    let webp_memory = encoder.encode(100f32);

    write(&path, &*webp_memory).ok();
    Ok(())
}

pub async fn save_resized_image(
    image: image::DynamicImage,
    width: Option<u32>,
    original_path: &PathBuf,
    target_path: &PathBuf,
) -> Response {
    if width.is_none() {
        return response_file(&target_path).await;
    }

    if image.width() <= width.unwrap() {
        copy(&original_path, &target_path).ok();
        return response_file(&target_path).await;
    }

    let resize_height = width.unwrap() * image.height() / image.width();
    let resized_image = image.thumbnail(width.unwrap(), resize_height);

    match resized_image.save(target_path.clone()) {
        Ok(_) => response_file(&target_path).await,
        Err(_) => response_error(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[cfg(test)]
mod tests {
    use crate::path;

    use super::*;
    use image::{io::Reader as ImageReader, DynamicImage, RgbImage};
    use std::{
        fs::{self, read, set_permissions},
        os::unix::fs::PermissionsExt,
    };
    use tempfile::tempdir;
    use webp::Decoder;

    const IMAGE_PATH: &str = "src/tests/fixtures";
    const IMAGE_NAME: &str = "image";
    const IMAGE_EXT: &str = "jpg";

    #[tokio::test]
    async fn test_save_image_to_webp() {
        const IMAGE_SIZE: u32 = 100;

        let dir = tempdir().unwrap();
        let output_path = dir.path().join("test_image.webp");

        let image = DynamicImage::ImageRgb8(image::RgbImage::new(IMAGE_SIZE, IMAGE_SIZE));

        match save_image_to_webp(&image, &output_path) {
            Ok(_) => {}
            Err(e) => {
                panic!("Failed to save image as WebP: {}", e);
            }
        };

        assert!(output_path.exists());

        let webp_data = read(&output_path).unwrap();
        let decoded_webp = Decoder::new(&webp_data).decode().unwrap();

        assert_eq!(decoded_webp.width(), IMAGE_SIZE);
        assert_eq!(decoded_webp.width(), decoded_webp.height());
    }

    #[tokio::test]
    async fn test_save_image_to_webp_large_image() {
        let image = DynamicImage::ImageRgb8(RgbImage::new(10000, 10000));
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("large_image.webp");

        match save_image_to_webp(&image, &output_path) {
            Ok(_) => {}
            Err(e) => {
                panic!("Failed to save large image as WebP: {}", e);
            }
        };

        assert!(output_path.exists());
    }

    #[tokio::test]
    async fn test_save_image_to_webp_read_only_file_system() {
        let image = DynamicImage::ImageRgb8(RgbImage::new(100, 100));
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("read_only_image.webp");

        set_permissions(dir.path(), std::fs::Permissions::from_mode(0o444)).unwrap();

        let result = save_image_to_webp(&image, &output_path);

        assert!(result.is_err());
        assert!(!output_path.exists());
    }

    #[tokio::test]
    async fn test_save_image_to_webp_invalid_dimension() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("test_image.webp");

        let invalid_cases = [
            DynamicImage::ImageLuma8(image::GrayImage::new(0, 0)),
            DynamicImage::ImageLuma8(image::GrayImage::new(0, 100)),
            DynamicImage::ImageLuma8(image::GrayImage::new(100, 0)),
        ];

        for invalid_image in invalid_cases.iter() {
            let result = save_image_to_webp(&invalid_image, &output_path);
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_save_resized_image() {
        const TARGET_WIDTH: u32 = 100;

        let dir = tempdir().unwrap();

        let image_path = format!("{}.{}", IMAGE_NAME, IMAGE_EXT);
        let path = format!("{}.w{}.{}", IMAGE_NAME, TARGET_WIDTH, IMAGE_EXT);

        let file_path = dir.path().join(path.clone());
        let original_path = path::get_original_path(&path, true);
        let original_file_path = dir.path().join(format!("image{}", original_path.clone()));

        match fs::copy(
            format!("{}/{}", IMAGE_PATH, image_path.clone()),
            file_path.clone(),
        ) {
            Ok(_) => {}
            Err(_) => {
                panic!("Failed to copy image file");
            }
        };

        let image = ImageReader::open(file_path.clone())
            .unwrap()
            .decode()
            .unwrap();

        let response =
            save_resized_image(image, Some(TARGET_WIDTH), &original_file_path, &file_path).await;

        assert_eq!(response.status(), StatusCode::OK);

        let resized_image = ImageReader::open(file_path).unwrap().decode().unwrap();

        assert_eq!(resized_image.width(), TARGET_WIDTH);
    }
}
