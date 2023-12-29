use crate::{graphic::render::{BlockFace, Buffer, IS_GREEDY_MESHING}, engine::vertices::block_vertex::BlockVertex};

// At the moment this is almost corrected with the help of post-processing, the operative word is almost

// Fix pixel gaps 
//     Bigger number => less pixel gaps, more artifacts associated with block enlargement
//     Less number => more pixel gaps, fewer artifacts associated with block enlargement
// https://stackoverflow.com/questions/39958039/where-do-pixel-gaps-come-from-in-opengl
// https://blackflux.wordpress.com/2014/03/02/meshing-in-voxel-engines-part-3/
const STITCHING: f32 = if IS_GREEDY_MESHING {0.0005} else {0.0}; //0.0025 0.0014

#[inline]
pub(crate) fn manage_x(
  buffer: &mut Buffer,
  global: (f32, f32, f32),
  lrw: (f32, f32, f32),
  offset: f32,
  indices: &[usize],
  face: &BlockFace
) {
    let x = global.0 as f32 + lrw.0 + offset;
    let py = global.1 as f32 + lrw.1 + 1.0 + STITCHING;
    let ny = py - face.size[1] as f32 - STITCHING;
    let pz = global.2 as f32 + lrw.2 + 1.0 + STITCHING;
    let nz = pz - face.size[0] as f32 - STITCHING;

    insert_vertices_into_buffer(buffer, face.size[0] as f32, face.size[1] as f32, face.layer,
        face.light.get(), indices, [[x, ny, nz], [x, py, nz], [x, py, pz], [x, ny, pz]]);
}

#[inline]
pub(crate) fn manage_y(
  buffer: &mut Buffer,
  global: (f32, f32, f32),
  lrw: (f32, f32, f32),
  offset: f32,
  indices: &[usize],
  face: &BlockFace
) {
    let y = global.1 as f32 + lrw.0 + offset;
    let px = global.0 as f32 + lrw.1 + 1.0 + STITCHING;
    let nx = px - face.size[1] as f32 - STITCHING;
    let pz = global.2 as f32 + lrw.2 + 1.0 + STITCHING;
    let nz = pz - face.size[0] as f32 - STITCHING;

    insert_vertices_into_buffer(buffer, face.size[1] as f32, face.size[0] as f32, face.layer,
        face.light.get(), indices, [[nx, y, nz], [nx, y, pz], [px, y, pz], [px, y, nz]]);
}

#[inline]
pub(crate) fn manage_z(
  buffer: &mut Buffer,
  global: (f32, f32, f32),
  lrw: (f32, f32, f32),
  offset: f32,
  indices: &[usize],
  face: &BlockFace
) {
    let z = global.2 as f32 + lrw.0 + offset;
    let px = global.0 as f32 + lrw.1 + 1.0 + STITCHING;
    let nx = px - face.size[1] as f32 - STITCHING;
    let py = global.1 as f32 + lrw.2 + 1.0 + STITCHING;
    let ny = py - face.size[0] as f32 - STITCHING;

    insert_vertices_into_buffer(buffer, face.size[0] as f32, face.size[1] as f32, face.layer,
        face.light.get(), indices, [[nx, ny, z], [px, ny, z], [px, py, z], [nx, py, z]]);
}

#[inline]
pub(crate) fn insert_vertices_into_buffer(
  buffer: &mut Buffer,
  pu: f32,
  pv: f32,
  layer: u32, 
  lights: [[f32; 4]; 4],
  indices: &[usize],
  coords: [[f32; 3]; 4],
) {
    buffer.manage_vertices(&[
        BlockVertex::new(coords[0], [0., 0.], layer, lights[0]),
        BlockVertex::new(coords[1], [0., pv], layer, lights[1]),
        BlockVertex::new(coords[2], [pu, pv], layer, lights[2]),
        BlockVertex::new(coords[3], [pu, 0.], layer, lights[3]),
    ], indices);
}