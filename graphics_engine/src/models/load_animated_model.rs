// use std::{collections::HashMap, path::Path};

// use crate::vertices::animated_model_vertex::AnimatedModelVertex;

// use super::{animated_model::{AnimatedModel}, load_texture::load_texture};

// pub fn load_animated_model(
//   device: &wgpu::Device,
//   queue: &wgpu::Queue,
//   texture_layout: &wgpu::BindGroupLayout,
//   animated_models: Vec<resources::animated_model::AnimatedModel>,
//   name: &str
// ) -> Box<[AnimatedModel]> {
//     animated_models.into_iter().map(|model| {
//         let texture = load_texture(device, queue, texture_layout,
//             &model.texture.data, model.texture.width, model.texture.height, name);
        
//         let vertices: Vec::<AnimatedModelVertex> = model.vertices.into_iter()
//             .map(AnimatedModelVertex::from).collect();

//         AnimatedModel::new(device, &vertices, texture, model.animator, Some(name))
//     }).collect()
// }
