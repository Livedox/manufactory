use std::ops::Range;
use crate::graphic::render::{Buffer};
use crate::graphic::render::block::BlockFace;
use crate::CHUNK_SIZE;
use super::face_managers::{manage_z, manage_y, manage_x};


#[derive(Debug)]
pub struct GreedManager {
    // nx = 0, px = 1, ny = 2, py = 3, nz = 4, pz = 5
    sides: [[RowColumn; CHUNK_SIZE]; 6],
    faces: Vec<BlockFace>,
}

impl GreedManager {
    const INDICES: [[usize; 6]; 2] = [[0,1,2,0,2,3], [3,2,0,2,1,0]];

    #[inline]
    pub fn new() -> Self {Self::default()}

    fn greed(&mut self) {
        for side in self.sides.iter_mut() {
            for layer in side.iter_mut() {
                for row in layer.row.iter_mut() {
                    for s in row.windows(2) {
                        let (column0, index0) = unsafe {s.get_unchecked(0)};
                        let (column1, index1) = unsafe {s.get_unchecked(1)};
                        if *column1 != *column0 + 1 {continue};
                        let prev = unsafe {&mut *(self.faces.get_unchecked_mut(*index0) as *mut BlockFace)};
                        let curr = unsafe {self.faces.get_unchecked_mut(*index1)};
                        if prev.layer == curr.layer && prev.light == curr.light && prev.size[1] == curr.size[1] {
                            curr.size[0] += prev.size[0];
                            prev.size = [0, 0];
                        }
                    }
                }

                for column in layer.column.iter_mut() {
                    for s in column.windows(2) {
                        let (row_prev, index_prev) = unsafe {s.get_unchecked(0)};
                        let (row_curr, index_curr) = unsafe {s.get_unchecked(1)};
                        if *row_curr != *row_prev + 1 {continue};
                        let prev = unsafe {&mut *(self.faces.get_unchecked_mut(*index_prev) as *mut BlockFace)};
                        if prev.size == [0, 0] {continue};
                        let curr = unsafe {self.faces.get_unchecked_mut(*index_curr)};
                        if prev.layer == curr.layer && prev.light == curr.light && prev.size[0] == curr.size[0] {
                            curr.size[1] += prev.size[1];
                            prev.size = [0, 0];
                        }
                    }
                }
            }
        }
    }

    fn iter_sides(&self, range: Range<usize>, mut c: impl FnMut(&BlockFace, (f32, f32, f32), f32, &[usize])) {
        // https://www.reddit.com/r/programminghorror/
        self.sides[range].iter().enumerate().for_each(|(i, side)| {
            side.iter().enumerate().for_each(|(layer, row_column)| {
                row_column.row.iter().enumerate().for_each(|(row, columns)| {
                    columns.iter().for_each(|(column, face_index)| {
                        let face = unsafe {self.faces.get_unchecked(*face_index)};
                        if face.size == [0, 0] {return};
                        c(face, (layer as f32, row as f32, *column as f32), i as f32,
                            Self::INDICES.get(i % 2).unwrap());
                    });
                }); 
            })
        });
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