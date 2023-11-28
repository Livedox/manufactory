use itertools::iproduct;

use crate::{world::chunk_coords::ChunkCoords, rev_qumark};

use super::{chunks::Chunks, chunk::Chunk};

impl Chunks {
    fn find_chunk_position<X, Y, Z, P>(&self, iters: (X, Y, Z), predicat: P)
        -> Option<ChunkCoords>
        where
            X: Iterator<Item = i32> + Clone,
            Y: Iterator<Item = i32> + Clone,
            Z: Iterator<Item = i32> + Clone,
            P: Fn(Option<&Chunk>) -> bool
    {
        for (cx, cy, cz) in iproduct!(iters.0, iters.1, iters.2) {
            if (cx - self.ox >= 0 && cy >= 0 && cz - self.oz >= 0
              && cx - self.ox < self.width && cy < self.height && cz - self.oz < self.depth)
              && predicat(self.chunk(ChunkCoords(cx, cy, cz))) {
                return Some(ChunkCoords(cx, cy, cz));
            }
        }
        None
    }


    pub fn find_nearest_position_xyz<P>(&self, start_chunk_coords: ChunkCoords, predicat: &P)
        -> Option<ChunkCoords>
        where P: Fn(Option<&Chunk>) -> bool,
    {
        let sx = start_chunk_coords.0;
        let sy = start_chunk_coords.1;
        let sz = start_chunk_coords.2;
        for i in 0..(self.depth.max(self.width).max(self.height)) {
            let min_x = if sx > i {sx-i} else {0};
            let max_x = if i+sx < self.width {i+sx} else {self.width-1};
            let min_y = if sy > i {sy-i} else {0};
            let max_y = if i+sy < self.height {i + sy} else {self.height-1};
            let x_side: Box<[i32]> = Self::check_size(i, sx, self.width);
            let y_side: Box<[i32]> = Self::check_size(i, sy, self.height);
            let z_side: Box<[i32]> = Self::check_size(i, sz, self.depth);

            rev_qumark!(self.find_chunk_position(
                (min_x..=max_x, min_y..=max_y, z_side.iter().map(|i| *i)), predicat));
    
            let min_z = if sz > i {sz-i+1} else {0};
            let max_z = if i+sz < self.depth-1 {sz+i-1} else {self.depth-1};
            rev_qumark!(self.find_chunk_position(
                (x_side.iter().map(|i| *i), min_y..=max_y, min_z..=max_z), predicat));
    
            let min_x = if sx > i {sx-i+1} else {0};
            let max_x = if i+sx < self.width {sx+i-1} else {self.width-1};
            rev_qumark!(self.find_chunk_position(
                (min_x..=max_x, y_side.iter().map(|i| *i), min_z..=max_z), predicat));
        }
        None
    }

    /// Looks only at the chunk with y = 0
    pub fn find_nearest_position_xz<P>(&self, start_chunk_coords: ChunkCoords, predicat: &P)
        -> Option<ChunkCoords>
        where P: Fn(Option<&Chunk>) -> bool,
    {
        let sx = start_chunk_coords.0;
        let sz = start_chunk_coords.2;
        for i in 0..(self.depth.max(self.width)) {
            let min_x = if sx > i {sx-i+self.ox} else {self.ox};
            let max_x = if i+sx < self.width {i+sx-self.ox} else {self.width-1+self.ox};
            let min_z = if sz > i {sz-i+1+self.oz} else {self.oz};
            let max_z = if i+sz < self.depth-1 {sz+i-1-self.oz} else {self.depth-1+self.oz};
            let x_side = Self::check_size(i, sx, self.width);
            let z_side = Self::check_size(i, sz, self.depth);

            println!("{:?} {:?} {:?}", min_x..=max_x, min_z..=max_z, z_side);

            rev_qumark!(self.find_chunk_position(
                (min_x..=max_x, 0..=0, z_side.iter().map(|i| *i)), predicat));
            rev_qumark!(self.find_chunk_position(
                (x_side.iter().map(|i| *i), 0..=0, min_z..=max_z), predicat));
        }
        None
    }


    pub fn find_pos_stable_xz<P>(&self, predicat: &P) -> Option<ChunkCoords>
        where P: Fn(Option<&Chunk>) -> bool
    {
        for (cx, cz) in iproduct!(0..self.width, 0..self.depth) {
            if let Some(chunk) = self.chunks.get(ChunkCoords(cx, 0, cz)
                .index_without_offset(self.width, self.depth)) {
                    if predicat(chunk.as_ref().map(|c| c.as_ref())) {
                        return Some(ChunkCoords(cx+self.ox, 0, cz+self.oz));
                    }
                }
        }
        None
    }


    pub fn find_pos_stable_xyz<P>(&self, predicat: &P) -> Option<ChunkCoords>
        where P: Fn(Option<&Chunk>) -> bool
    {
        for (cx, cy, cz) in iproduct!(0..self.width, 0..self.height, 0..self.depth) {
            if let Some(chunk) = self.chunks.get(ChunkCoords(cx, cy, cz)
                .index_without_offset(self.width, self.depth)) {
                    if predicat(chunk.as_ref().map(|c| c.as_ref())) {
                        return Some(ChunkCoords(cx+self.ox, cy, cz+self.oz));
                    }
                }
        }
        None
    }


    fn check_size(i: i32, p: i32, size: i32) -> Box<[i32]> {
        if p < i {
            Box::new([i + p])
        } else if i + p > size {
            Box::new([-i + p])
        } else {
            Box::new([-i + p, i + p])
        }
    }
}