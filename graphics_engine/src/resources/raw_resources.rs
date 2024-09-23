use crate::{bind_group, bind_group_layout::Layouts, models::{raw_animated_model::RawAnimatedModel, raw_model::RawModel}, raw_texture::RawTexture, texture::{self, TextureAtlas}};

use super::resources::Resources;

pub struct Blocks {
    pub data: Vec<Vec<u8>>,
    pub count: u32,
}

pub struct Atlas {
    pub data: Vec<u8>,
    pub size: u32,
}

pub struct RawResources {
    pub models: Vec<RawModel>,
    pub animated_models: Vec<RawAnimatedModel>,

    pub atlas: Atlas,
    pub player: RawTexture,
    pub blocks: Blocks,
}

impl RawResources {
    pub fn into_resources(
        self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layouts: &Layouts,
        render_pass: &mut egui_wgpu::Renderer
    ) -> Resources {
        let models = self.models.into_iter()
            .map(|m| m.into_model(device, queue, &layouts.model_texture))
            .collect();

        let animated_models = self.animated_models.into_iter()
            .map(|m| m.into_animated_model(device, queue, &layouts.model_texture))
            .collect();

        let texture_atlas = TextureAtlas::new(render_pass, device, queue, self.atlas.size, &self.atlas.data, 4);

        let bb: Vec<&[u8]> = self.blocks.data.iter().map(Vec::as_slice).collect();
        let block_texture = texture::Texture::block_array(device, queue, &bb, self.blocks.count);
        let block_bind_group = bind_group::block_texture::get(device, &layouts.block_texture, &block_texture);

        let player_bind_group = self.player.into_default_texture(device, queue)
            .into_model_bind_group(device, &layouts.player_texture, Some("player"));

        Resources {
            models,
            animated_models,
            texture_atlas,
            block_bind_group,
            player_bind_group,
        }
    }
}