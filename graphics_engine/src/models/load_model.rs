use std::{path::Path};

use crate::vertices::model_vertex::ModelVertex;

use super::{model::Model, load_texture::load_texture};


pub fn load_models(
  device: &wgpu::Device,
  queue: &wgpu::Queue,
  texture_layout: &wgpu::BindGroupLayout,
  models: Vec<resources::model::Model>,
  name: &str
) -> Box<[Model]> {
    models.into_iter().map(|model| {
        let texture = load_texture(device, queue, texture_layout,
            &model.texture.data, model.texture.width, model.texture.height, name);
        
        let vertices: Vec::<ModelVertex> = model.vertices.into_iter().map(ModelVertex::from).collect();

        Model::new(device, &vertices, texture, name)
    }).collect()
}
