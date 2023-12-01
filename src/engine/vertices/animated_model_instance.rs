#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct AnimatedModelInstance {
    pub position: [f32; 3],
    pub light: [f32; 4],
    pub start_matrix: u32,
    pub rotation_matrix_index: u32,
}

impl AnimatedModelInstance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<AnimatedModelInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl AnimatedModelInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![4 => Float32x3, 5 => Float32x4, 6 => Uint32, 7 => Uint32];
}