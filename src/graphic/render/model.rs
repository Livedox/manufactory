use std::collections::HashMap;

use crate::voxels::chunk::{Chunk, CHUNK_SIZE};

#[derive(Debug, Clone)]
pub struct ModelRenderResult {
    pub position: [f32; 3],
    pub light: [f32; 4],
    pub rotation_index: u32,
}

pub type Models = HashMap::<String, Vec<ModelRenderResult>>;

#[inline]
pub fn render_model(models: &mut Models, chunk: &Chunk, name: &str, lx: usize, ly: usize, lz: usize) {
    let rotation_index = chunk.voxels_data.get(&((ly*CHUNK_SIZE+lz)*CHUNK_SIZE+lx))
        .and_then(|vd| vd.rotation_index()).unwrap_or(0);

    let light = chunk.get_light((lx, ly, lz).into()).get_normalized();

    let data = ModelRenderResult {
        position: chunk.xyz.to_global((lx, ly, lz).into()).into(),
        light,
        rotation_index
    };

    if let Some(model) = models.get_mut(name) {
        model.push(data);
    } else {
        models.insert(name.to_string(), vec![data]);
    }
}