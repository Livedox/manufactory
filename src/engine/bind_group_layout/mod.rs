pub mod texture;
pub mod vertex_uniform;
pub mod vertex_storage;

pub(crate) struct Layouts {
    sun: wgpu::BindGroupLayout,
    block_texture: wgpu::BindGroupLayout,
    model_texture: wgpu::BindGroupLayout,
    crosshair_u_ar_scale: wgpu::BindGroupLayout,
}

impl Layouts {
    pub(crate) fn new(device: &wgpu::Device) -> Self {Self {
        sun: self::vertex_uniform::get(&device, "sun_bgl"),
        block_texture: self::texture::get(&device, wgpu::TextureViewDimension::D2Array, "block_texture_bgl"),
        model_texture: self::texture::get(&device, wgpu::TextureViewDimension::D2, "model_texture_bgl"),
        crosshair_u_ar_scale: self::vertex_uniform::get(&device, "crosshair_u_scale_bgl"),
    }}
}