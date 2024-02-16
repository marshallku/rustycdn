use std::{fs::write, path::PathBuf};
use webp::Encoder;

pub fn save_image_to_webp(image: image::DynamicImage, path: &PathBuf) -> Result<(), String> {
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
