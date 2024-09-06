#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct PlayerVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub light: [f32; 4]
}

impl PlayerVertex {
    #[inline]
    pub fn new(position: [f32; 3], uv: [f32; 2], light: [f32; 4]) -> Self {Self {
        position,
        uv,
        light
    }}
    
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x4];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}