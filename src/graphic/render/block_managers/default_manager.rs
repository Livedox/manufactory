use std::ops::Range;

use crate::graphic::render::{Buffer};
use crate::graphic::render::block::BlockFace;

use super::face_managers::{manage_z, manage_y, manage_x};

#[derive(Debug)]
pub struct DefaultManager {
    // nx = 0, px = 1, ny = 2, py = 3, nz = 4, pz = 5
    sides: [Vec<((f32, f32, f32), BlockFace)>; 6],
}

impl DefaultManager {
    #[inline]
    pub fn new() -> Self {Self::default()}
    const INDICES: [[usize; 6]; 2] = [[0,1,2,0,2,3], [3,2,0,2,1,0]];
    fn iter_sides(&self, range: Range<usize>, mut c: impl FnMut(&BlockFace, (f32, f32, f32), f32, &[usize])) {
        self.sides[range].iter().enumerate().for_each(|(i, faces)| {
            faces.iter().for_each(|(lrw, face)| {
                c(face, *lrw, (i % 2) as f32, Self::INDICES.get(i % 2).unwrap());
            });
        });
    }
    /// Sides: nx = 0, px = 1, ny = 2, py = 3, nz = 4, pz = 5
    pub(crate) fn set(&mut self, side: usize, layer: usize, row: usize, column: usize, face: BlockFace) {
        self.sides[side].push(((layer as f32, row as f32, column as f32), face));
    }

    pub(crate) fn manage_vertices(&mut self, buffer: &mut Buffer, global: (f32, f32, f32)) {
        self.iter_sides(0..2, |face, lrw, offset, indices| {
            manage_x(buffer, global, lrw, offset, indices, face)});

        self.iter_sides(2..4, |face, lrw, offset, indices| {
            manage_y(buffer, global, lrw, offset, indices, face)});

        self.iter_sides(4..6, |face, lrw, offset, indices| {
            manage_z(buffer, global, lrw, offset, indices, face)});
    }
}

impl Default for DefaultManager {
    fn default() -> Self {Self {sides: Default::default()}}
}