use wgpu::util::DeviceExt;

use crate::vertices::model_vertex::ModelVertex;

#[derive(Debug)]
pub struct Model {
    pub vertex_count: usize,
    pub vertex_buffer: wgpu::Buffer,
    pub texture: wgpu::BindGroup,
}

impl Model {
    pub fn new(
      device: &wgpu::Device,
      model: &[ModelVertex],
      texture: wgpu::BindGroup,
      label: Option<&str>
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(model),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {vertex_count: model.len(), vertex_buffer, texture}
    }
}