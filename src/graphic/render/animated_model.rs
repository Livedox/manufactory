use std::collections::HashMap;

use crate::voxels::chunk::{Chunk, CHUNK_SIZE};

#[derive(Debug, Clone)]
pub struct AnimatedModelRenderResult {
    pub position: [f32; 3],
    pub light: [f32; 4],
    pub progress: f32,
    pub rotation_index: u32,
}

pub type AnimatedModels = HashMap::<u32, Vec<AnimatedModelRenderResult>>;

#[inline]
pub fn render_animated_model(animated_models: &mut AnimatedModels, chunk: &Chunk, model_id: u32, lx: usize, ly: usize, lz: usize) {
    let mut progress = 0.0;
    let mut rotation_index = 0;
    
    if let Some(live_voxel) = chunk.live_voxels.0.read().unwrap().get(&((ly*CHUNK_SIZE+lz)*CHUNK_SIZE+lx)).cloned() {
        progress = live_voxel.animation_progress();
        if let Some(rotation) = live_voxel.rotation_index() {
            rotation_index = rotation;
        }
    }
    
    let light = chunk.get_light((lx, ly, lz).into()).get_normalized();

    let data = AnimatedModelRenderResult {
        position: chunk.xyz.to_global((lx, ly, lz).into()).into(),
        light,
        progress,
        rotation_index
    };

    if let Some(model) = animated_models.get_mut(&model_id) {
        model.push(data);
    } else {
        animated_models.insert(model_id, vec![data]);
    }
}