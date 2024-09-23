use crate::{animator::Animator, raw_texture::RawTexture, vertices::animated_model_vertex::AnimatedModelVertex};

use super::animated_model::AnimatedModel;

#[derive(Debug, Clone)]
pub struct RawAnimatedModel {
    pub vertices: Vec<AnimatedModelVertex>,
    pub texture: RawTexture,
    pub animator: Animator,
}

impl RawAnimatedModel {
    pub fn new(
        vertices: Vec<AnimatedModelVertex>,
        texture: RawTexture,
        animator: Animator,
    ) -> Self {
        Self { vertices, animator, texture }
    }

    pub fn into_animated_model(
        self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout
    ) -> AnimatedModel {
        let bind_group = self.texture.into_default_texture(device, queue)
            .into_model_bind_group(device, layout, Some("animated_model"));

        AnimatedModel::new(device, &self.vertices, bind_group, self.animator, Some("animated_model"))
    }
}