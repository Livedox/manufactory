use crate::{models::{animated_model::AnimatedModel, model::Model}, texture::TextureAtlas};

pub struct Resources {
    pub models: Box<[Model]>,
    pub animated_models: Box<[AnimatedModel]>,
    pub texture_atlas: TextureAtlas,
    pub block_bind_group: wgpu::BindGroup,
    pub player_bind_group: wgpu::BindGroup,
}