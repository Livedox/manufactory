use std::collections::HashMap;

use crate::voxels::chunk::{Chunk, CHUNK_SIZE};

#[derive(Debug, Clone)]
pub struct AnimatedModelRenderResult {
    pub position: [f32; 3],
    pub light: [f32; 4],
    pub progress: f32,
    pub rotation_index: u32,
}

pub type AnimatedModels = HashMap::<String, Vec<AnimatedModelRenderResult>>;

#[inline]
pub fn render_animated_model(animated_models: &mut AnimatedModels, chunk: &Chunk, name: &str, lx: usize, ly: usize, lz: usize) {
    let voxel_data = chunk.voxels_data.read().unwrap().get(&((ly*CHUNK_SIZE+lz)*CHUNK_SIZE+lx)).map(|d| d.clone()).unwrap();
    let progress = voxel_data.additionally.animation_progress().unwrap_or(0.0);
    let rotation_index = voxel_data.rotation_index().unwrap_or(0);
    let light = chunk.get_light((lx, ly, lz).into()).get_normalized();

    let data = AnimatedModelRenderResult {
        position: chunk.xyz.to_global((lx, ly, lz).into()).into(),
        light,
        progress,
        rotation_index
    };

    if let Some(model) = animated_models.get_mut(name) {
        model.push(data);
    } else {
        animated_models.insert(name.to_string(), vec![data]);
    }
}