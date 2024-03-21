use std::path::Path;

use crate::texture::Texture;

pub fn load_texture(
  device: &wgpu::Device,
  queue: &wgpu::Queue,
  texture_layout: &wgpu::BindGroupLayout,
  src: impl AsRef<Path>,
  name: &str
) -> wgpu::BindGroup {
    let img = image::open(src).expect("Failed to open file!");
    let texture = Texture::image(device, queue, img.width(), img.height(), img.as_bytes());
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: texture_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            },
        ],
        label: Some(&format!("Model texture diffuse_bind_group ({})", name)),
    })
}