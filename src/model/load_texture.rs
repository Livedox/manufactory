use crate::texture::Texture;

pub fn load_texture(
  device: &wgpu::Device,
  queue: &wgpu::Queue,
  texture_layout: &wgpu::BindGroupLayout,
  src: &str,
  name: &str
) -> wgpu::BindGroup {
    let image = Texture::image(&device, &queue, src).unwrap();
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&image.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&image.sampler),
            },
        ],
        label: Some(&format!("Model texture diffuse_bind_group ({})", name)),
    })
}