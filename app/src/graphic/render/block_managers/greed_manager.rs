use std::ops::Range;
use itertools::iproduct;

use crate::{graphic::render::Buffer, voxels::new_chunk::CHUNK_SIZE};
use crate::graphic::render::block::BlockFace;
use super::face_managers::{manage_z, manage_y, manage_x};


#[derive(Debug)]
pub struct GreedManager {
    // nx = 0, px = 1, ny = 2, py = 3, nz = 4, pz = 5
    sides: [[RowColumn; CHUNK_SIZE]; 6],
    faces: Vec<BlockFace>,
}

// Why is there so much unsafe code in this code?
//     Because it's optimized as fuck.
//     It's optimized down to the CPU cache usage, I had to optimize the "face.size[0] == 0" in "iter_sides",
//     because it was using half the CPU time.
// Why did I optimize this so much?
//     Because I was having fun.
impl GreedManager {
    const INDICES: [[usize; 6]; 2] = [[0,1,2,0,2,3], [3,2,0,2,1,0]];

    #[inline]
    pub fn new() -> Self {Self::default()}

    #[inline]
    fn greed(&mut self) {
        for side in self.sides.iter_mut() {
            for layer in side.iter_mut() {
                for row in layer.row.iter_mut() {
                    for s in row.windows(2) {
                        let (column_prev, index_prev) = unsafe {s.get_unchecked(0)};
                        let (column_curr, index_curr) = unsafe {s.get_unchecked(1)};
                        if *column_curr != *column_prev + 1 {continue};
                        let prev = unsafe {&mut *(self.faces.get_unchecked_mut(*index_prev) as *mut BlockFace)};
                        let curr = unsafe {self.faces.get_unchecked_mut(*index_curr)};
                        if prev.layer == curr.layer && prev.light == curr.light && prev.size[1] == curr.size[1] {
                            curr.size[0] += prev.size[0];
                            prev.size = [0, 0];
                        }
                    }
                }

                for column in layer.column.iter_mut() {
                    if column.len() == 1 {
                        let (_, index) = unsafe {column.get_unchecked_mut(0)};
                        let face = unsafe {self.faces.get_unchecked(*index)};
                        if face.size == [0, 0] {*index = usize::MAX}
                    } else {
                       for s in column.windows(2) {
                            let (row_prev, index_prev) = unsafe {s.get_unchecked(0)};
                            let (row_curr, index_curr) = unsafe {s.get_unchecked(1)};
                            if *row_curr != *row_prev + 1 {continue};

                            let prev = unsafe {&mut *(self.faces.get_unchecked_mut(*index_prev) as *mut BlockFace)};
                            let mut_index_prev = unsafe { (index_prev as *const usize as *mut usize).as_mut().unwrap() };

                            if prev.size == [0, 0] {
                                *mut_index_prev = usize::MAX;
                                continue;
                            };
                            
                            let curr = unsafe {self.faces.get_unchecked_mut(*index_curr)};
                            if prev.layer == curr.layer && prev.light == curr.light && prev.size[0] == curr.size[0] {
                                curr.size[1] += prev.size[1];
                                prev.size = [0, 0];
                                *mut_index_prev = usize::MAX;
                            }
                        }
                    }
                }
            }
        }
    }

    #[inline]
    fn iter_sides(&self, range: Range<usize>, mut c: impl FnMut(&BlockFace, (f32, f32, f32), f32, &[usize; 6])) {
        for ((i, side), layer, column) in iproduct!(self.sides[range].iter().enumerate(), 0..CHUNK_SIZE, 0..CHUNK_SIZE) {
            for (row, face_index) in unsafe {side.get_unchecked(layer).column.get_unchecked(column)}.iter() {
                if *face_index == usize::MAX {continue};
                let face = self.faces.get(*face_index).unwrap();
                c(face, (layer as f32, *row as f32, column as f32), i as f32, &Self::INDICES[i%2]);
            }
        }
    }

    /// Sides: nx = 0, px = 1, ny = 2, py = 3, nz = 4, pz = 5
    pub(crate) fn set(&mut self, side: usize, layer: usize, row: usize, column: usize, face: BlockFace) {
        let index = self.faces.len();
        self.faces.push(face);
        let layer = self.sides[side].get_mut(layer).unwrap();
        layer.row[row].push((column, index));
        layer.column[column].push((row, index));
    }

    pub(crate) fn manage_vertices(&mut self, buffer: &mut Buffer, global: (f32, f32, f32)) {
        self.greed();

        self.iter_sides(0..2, |face, lrw, offset, indices| {
            manage_x(buffer, global, lrw, offset, indices, face)});

        self.iter_sides(2..4, |face, lrw, offset, indices| {
            manage_y(buffer, global, lrw, offset, indices, face)});

        self.iter_sides(4..6, |face, lrw, offset, indices| {
            manage_z(buffer, global, lrw, offset, indices, face)});
    }
}

impl Default for GreedManager {
    #[inline]
    fn default() -> Self {Self {
        sides: Default::default(),
        faces: Vec::new(),
    }}
}

#[derive(Debug)]
struct RowColumn {
    row: [Vec<(usize, usize)>; CHUNK_SIZE],
    column: [Vec<(usize, usize)>; CHUNK_SIZE],
}

impl Default for RowColumn {
    #[inline]
    fn default() -> Self {
        Self { row: Default::default(), column: Default::default() }
    }
}