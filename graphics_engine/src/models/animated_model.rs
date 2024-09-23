use wgpu::util::DeviceExt;

use crate::{animator::Animator, vertices::animated_model_vertex::AnimatedModelVertex};

#[derive(Debug)]
pub struct AnimatedModel {
    pub vertex_count: usize,
    pub vertex_buffer: wgpu::Buffer,
    pub texture: wgpu::BindGroup,

    animator: Animator
}


impl AnimatedModel {
    pub fn new(
      device: &wgpu::Device,
      model: &[AnimatedModelVertex],
      texture: wgpu::BindGroup,
      animator: Animator,
      label: Option<&str>
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(model),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {vertex_count: model.len(), vertex_buffer, texture, animator}
    }

    pub fn joint_count(&self) -> usize {
        self.animator.joint_count()
    }

    pub fn calculate_bytes_transforms(&self, animation_name: Option<&str>, progress: f32) -> Vec<u8> {
        self.animator.calculate_transforms(animation_name, progress)
            .iter().map(|mat| bytemuck::bytes_of(mat)).flatten().cloned().collect()
    }
}