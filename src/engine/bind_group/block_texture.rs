use crate::engine::texture::Texture;

pub(crate) fn get(device: &wgpu::Device, textrue_bgl: &wgpu::BindGroupLayout, texture: &Texture) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: textrue_bgl,
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
        label: Some("block_texutre_bind_group"),
    })
}