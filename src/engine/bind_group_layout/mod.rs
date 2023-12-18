pub mod texture;
pub mod vertex_uniform;
pub mod vertex_storage;
pub mod post_process;

pub struct Layouts {
    pub block_texture: wgpu::BindGroupLayout,
    pub model_texture: wgpu::BindGroupLayout,

    pub transforms_storage: wgpu::BindGroupLayout,
    
    pub sun: wgpu::BindGroupLayout,
    pub crosshair_aspect_scale: wgpu::BindGroupLayout,
    pub camera: wgpu::BindGroupLayout,
    pub time: wgpu::BindGroupLayout,
    pub post_proccess_test: wgpu::BindGroupLayout,
    pub multisampled_post_proccess: wgpu::BindGroupLayout,
}

impl Layouts {
    pub(crate) fn new(device: &wgpu::Device) -> Self {Self {
        block_texture: self::texture::get(device, wgpu::TextureViewDimension::D2Array, "block_texture_bgl"),
        model_texture: self::texture::get(device, wgpu::TextureViewDimension::D2, "model_texture_bgl"),
        post_proccess_test: self::post_process::get(device, false),
        multisampled_post_proccess: self::post_process::get(device, true),
        transforms_storage: self::vertex_storage::get(device, true, "animated_model_bgl"),
        
        crosshair_aspect_scale: self::vertex_uniform::get(device, "crosshair_aspect_scale_bgl"),
        sun: self::vertex_uniform::get(device, "sun_bgl"),
        camera: self::vertex_uniform::get(device, "camera_bgl"),
        time: self::vertex_uniform::get(device, "transport_belt_bgl"),
    }}
}