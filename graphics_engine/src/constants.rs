pub const BLOCK_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub const BLOCK_TEXTURE_SIZE: u32 = 32;
// Maximum mipmap_count is BASE_SIZE.ilog2() + 1 (img size 1px) but it's too small
pub const BLOCK_MIPMAP_COUNT: usize = BLOCK_TEXTURE_SIZE.ilog2() as usize;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub const IS_LINE_TOPOLOGY: bool = false;
pub const PRIMITIVE_TOPOLOGY: wgpu::PrimitiveTopology = match IS_LINE_TOPOLOGY {
    true => wgpu::PrimitiveTopology::LineList,
    false => wgpu::PrimitiveTopology::TriangleList,
};