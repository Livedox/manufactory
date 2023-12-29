#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct BlockVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub layer: u32,
    pub v_light: [f32; 4]
}

impl BlockVertex {
    #[inline]
    pub fn new(position: [f32; 3], uv: [f32; 2], layer: u32, v_light: [f32; 4]) -> Self {Self {
        position,
        layer,
        uv,
        v_light
    }}
    
    const ATTRIBS: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Uint32, 3 => Float32x4];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BlockVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}