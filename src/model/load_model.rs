use std::{collections::HashMap, path::Path};

use russimp::scene::{Scene, PostProcess};

use crate::{vertices::{model_vertex::ModelVertex}};

use super::{model::Model, load_texture::load_texture};


pub fn load_model(
  device: &wgpu::Device,
  queue: &wgpu::Queue,
  texture_layout: &wgpu::BindGroupLayout,
  src: &str,
  src_texture: &str,
  name: &str
) -> Model {
    let scene = Scene::from_file(src, vec![PostProcess::FlipUVs, PostProcess::MakeLeftHanded]).unwrap();

    let mut model_vertex: Vec<ModelVertex> = vec![];
    scene.meshes[0].vertices.iter().for_each(|vertex| {
        model_vertex.push(ModelVertex {
            position: [vertex.x + 0.5, vertex.y, vertex.z + 0.5],
            uv: [0.0, 0.0]});
    });
    
    scene.meshes[0].texture_coords.iter().for_each(|coords| {
        if let Some(coords) = coords.as_ref() {
            coords.iter().enumerate().for_each(|(index, coords)| {
                model_vertex[index].uv = [coords.x, coords.y]})}
    });

    let texture = load_texture(device, queue, texture_layout, src_texture, name);

    Model::new(device, &model_vertex, texture, name)
}


pub fn load_models(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_layout: &wgpu::BindGroupLayout,
    models_textures_names: &[(&str, &str, &str)],
  ) -> HashMap<String, Model> {
    let mut models: HashMap<String, Model> = HashMap::new();
    models_textures_names.iter().for_each(|mtn| {
        models.insert(
            mtn.2.to_string(), 
            load_model(device, queue, texture_layout, mtn.0, mtn.1, mtn.2)
        );
    });

    models
}