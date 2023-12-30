use crate::{engine::vertices::block_vertex::BlockVertex, graphic::complex_object::{ComplexObjectSide, ComplexObject, ComplexObjectParts}, voxels::chunk::Chunk};
use super::Buffer;

const INDICES: [[usize; 6]; 2] = [[0,1,2,0,2,3], [3,2,0,2,1,0]];

fn render_side(
  buffer: &mut Buffer,
  part: &[Option<ComplexObjectSide>; 6],
  light: [f32; 4],
  xyz: (f32, f32, f32),
  rotation_index: usize
) {
    part.iter().enumerate().for_each(|(i, side)| {
        let Some(side) = side else {return};
        side.vertex_groups.iter().for_each(|group| {
            let vertices: [BlockVertex; 4] = [0, 1, 2, 3].map(|i| {
                let position = group.sum_position(xyz.0, xyz.1, xyz.2, rotation_index, i);
                BlockVertex::new(position, group.uv(i), side.texture_layer, light)
            });
            buffer.manage_vertices(&vertices, &INDICES[i%2]);
        });
    });
}

#[inline]
pub fn render_complex_object(
  complex_object: &ComplexObject,
  buffer: &mut Buffer,
  belt_buffer: &mut Buffer,
  chunk: &Chunk,
  lx: usize,
  ly: usize,
  lz: usize
) {
    let voxel_data = chunk.voxel_data((lx, ly, lz).into()).unwrap();
    let rotation_index = voxel_data.rotation_index().unwrap_or(0) as usize;
    let light = chunk.get_light((lx, ly, lz).into()).get_normalized();
    let global = chunk.xyz.to_global((lx, ly, lz).into()).into();

    complex_object.parts.iter().for_each(|parts| {
        if let ComplexObjectParts::Block(part) = parts {
            render_side(buffer, part, light, global, rotation_index);
        } else if let ComplexObjectParts::TransportBelt(part) = parts {
            render_side(belt_buffer, part, light, global, rotation_index);
        }
    });
}