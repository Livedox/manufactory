use std::path::Path;

use image::ImageError;

pub struct ModelTexture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub fn load_texture(src: impl AsRef<Path>) -> Result<ModelTexture, ImageError> {
    let img = image::open(src)?;

    Ok(ModelTexture {
        width: img.width(),
        height: img.height(),
        data: img.into_bytes()
    })
}