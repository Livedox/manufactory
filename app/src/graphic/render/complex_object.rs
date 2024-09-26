use graphics_engine::vertices::block_vertex::BlockVertex;
use crate::{graphic::complex_object::{ComplexObject, ComplexObjectSide}, voxels::new_chunk::Chunk};
use super::{animated_model::{render_animated_model, AnimatedModels}, model::{render_model, Models}, Buffer};

const INDICES: [[usize; 6]; 2] = [[0,1,2,0,2,3], [3,2,0,2,1,0]];

fn render_side(
  buffer: &mut Buffer,
  side_index: usize,
  sides: &[ComplexObjectSide],
  light: [f32; 4],
  xyz: (f32, f32, f32),
  rotation_index: usize
) {
    sides.iter().for_each(|side| {
        let vertices: [BlockVertex; 4] = [0, 1, 2, 3].map(|i| {
            let position = side.vertex_group.sum_position(xyz.0, xyz.1, xyz.2, rotation_index, i);
            BlockVertex::new(position, side.vertex_group.uv(i), side.texture_layer, light)
        });
        buffer.manage_vertices(&vertices, &INDICES[side_index%2]);
    });
}

#[inline]
pub fn render_complex_object(
  complex_object: &ComplexObject,
  models: &mut Models,
  animated_models: &mut AnimatedModels,
  buffer: &mut Buffer,
  belt_buffer: &mut Buffer,
  chunk: &Chunk,
  lx: usize,
  ly: usize,
  lz: usize
) {
    let rotation_index = match chunk.live_voxel((lx, ly, lz).into()) {
        Some(live_voxel) => live_voxel.rotation_index().unwrap_or(0),
        _ => 0,
    } as usize;
    let light = chunk.light_map()[(lx, ly, lz).into()].get_normalized();
    let global = chunk.coord.to_global((lx, ly, lz).into()).into();

    complex_object.block.iter().enumerate().for_each(|(i, sides)| {
        render_side(buffer, i, sides, light, global, rotation_index);
    });
    complex_object.transport_belt.iter().enumerate().for_each(|(i, sides)| {
        render_side(belt_buffer, i, sides, light, global, rotation_index);
    });
    complex_object.models.iter().for_each(|id| {
        render_model(models, chunk, *id, lx, ly, lz);
    });
    complex_object.animated_models.iter().for_each(|id| {
        render_animated_model(animated_models, chunk, *id, lx, ly, lz);
    });
}