use std::sync::Arc;

use crate::{models::{animated_model::AnimatedModel, model::Model}, texture::TextureAtlas};

pub struct Resources {
    pub models: Box<[Model]>,
    pub animated_models: Box<[AnimatedModel]>,
    pub texture_atlas: Arc<TextureAtlas>,
    pub block_bind_group: wgpu::BindGroup,
    pub player_bind_group: wgpu::BindGroup,
}

impl Resources {
    pub fn models(&self) -> &[Model] {
        &self.models
    }

    pub fn animated_models(&self) -> &[AnimatedModel] {
        &self.animated_models
    }

    pub fn block_bind_group(&self) -> &wgpu::BindGroup {
        &self.block_bind_group
    }

    pub fn player_bind_group(&self) -> &wgpu::BindGroup {
        &self.player_bind_group
    }

    pub fn atlas(&self) -> &TextureAtlas {
        &self.texture_atlas
    }

    pub fn clone_atlas(&self) -> Arc<TextureAtlas> {
        Arc::clone(&self.texture_atlas)
    }
}