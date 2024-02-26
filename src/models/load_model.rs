use std::{path::Path};

use russimp::scene::{Scene, PostProcess};

use crate::engine::vertices::model_vertex::ModelVertex;

use super::{model::Model, load_texture::load_texture};


pub fn load_model(
  device: &wgpu::Device,
  queue: &wgpu::Queue,
  texture_layout: &wgpu::BindGroupLayout,
  src: impl AsRef<Path>,
  src_texture: impl AsRef<Path>,
  name: &str
) -> Model {
    let scene = Scene::from_file(src.as_ref().to_str().unwrap(),
        vec![PostProcess::FlipUVs, PostProcess::MakeLeftHanded]).unwrap();

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
