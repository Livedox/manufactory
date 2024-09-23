use crate::texture::Texture;

#[derive(Debug, Clone)]
pub struct RawTexture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl RawTexture {
    pub fn into_default_texture(self, device: &wgpu::Device, queue: &wgpu::Queue) -> Texture {
        Texture::image(device, queue, self.width, self.height, &self.data)
    }
}