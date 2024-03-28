#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct AnimatedModelVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub bone_id: [i32; 3],
    pub bone_weight: [f32; 3],
}

impl AnimatedModelVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Sint32x3, 3 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<AnimatedModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl From<resources::animated_model::AnimatedModelVertex> for AnimatedModelVertex {
    fn from(value: resources::animated_model::AnimatedModelVertex) -> Self {
        Self { position: value.position, uv: value.uv, bone_id: value.bone_id, bone_weight: value.bone_weight }
    }
}