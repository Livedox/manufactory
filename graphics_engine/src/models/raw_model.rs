use crate::{raw_texture::RawTexture, vertices::model_vertex::ModelVertex};
use super::model::Model;


#[derive(Debug, Clone)]
pub struct RawModel {
    pub vertices: Vec<ModelVertex>,
    pub texture: RawTexture,
}


impl RawModel {
    pub fn into_model(self, device: &wgpu::Device, queue: &wgpu::Queue, layout: &wgpu::BindGroupLayout) -> Model {
        let bind_group = self.texture.into_default_texture(device, queue)
            .into_model_bind_group(device, layout, Some("model"));

        Model::new(device, &self.vertices, bind_group, Some("model"))
    }
}