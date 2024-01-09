use std::{marker::PhantomPinned, pin::Pin};

use itertools::iproduct;

use crate::{light::light::LightSolvers, voxels::{chunks::{Chunks, WORLD_HEIGHT}, voxel::Voxel, chunk::Chunk}, direction::Direction, save_load::{WorldRegions, EncodedChunk}, bytes::BytesCoder};

use self::global_coords::GlobalCoords;

pub mod global_coords;
pub mod chunk_coords;
pub mod local_coords;
pub mod coords;
pub mod sun;
pub mod loader;


#[derive(Debug)]
pub struct World {
    pub chunks: Pin<Box<Chunks>>,
    pub light: Pin<Box<LightSolvers>>,
}

impl World {
    pub fn new(width: i32, height: i32, depth: i32, ox: i32, oy: i32, oz: i32) -> Self {
        Self {
            chunks: Box::pin(Chunks::new(width, height, depth, ox, oy, oz)),
            light: Box::pin(LightSolvers::new()),
        }
    }

    pub fn solve_rgbs(&mut self) {
        self.light.solve_rgbs(&mut self.chunks);
    }

    pub fn load_column_of_chunks(&mut self, regions: &mut WorldRegions, cx: i32, cz: i32) {
        for cy in (0..WORLD_HEIGHT as i32).rev() {
            let chunk = match regions.chunk((cx, cy, cz).into()) {
                EncodedChunk::None => Chunk::new(cx, cy, cz),
                EncodedChunk::Some(b) => Chunk::decode_bytes(b),
            };
            let index = chunk.xyz.chunk_index(&self.chunks);
            self.chunks.chunks[index] = Some(Box::new(chunk));
            self.build_chunk(cx, cy, cz);
        }
        self.solve_rgbs();
    }

    pub fn build_chunk(&mut self, cx: i32, cy: i32, cz: i32) {
        self.light.build_sky_light_chunk(&mut self.chunks, cx, cy, cz);
        self.light.on_chunk_loaded(&mut self.chunks, cx, cy, cz);
    }


    pub fn break_voxel(&mut self, xyz: &GlobalCoords) {
        Chunks::set(&mut self.chunks, *xyz, 0, None);
        // self.chunks.set(*xyz, 0, None);
        self.light.on_block_break(&mut self.chunks, xyz.0, xyz.1, xyz.2);
    }

    pub fn set_voxel(&mut self, xyz: &GlobalCoords, id: u32, dir: &Direction) {
        Chunks::set(&mut self.chunks, *xyz, id, Some(dir));
        // self.chunks.set(*xyz, id, Some(dir));
        self.light.on_block_set(&mut self.chunks, xyz.0, xyz.1, xyz.2, id);
    }

    pub fn voxel(&self, xyz: &GlobalCoords) -> Option<&Voxel> {
        self.chunks.voxel_global(*xyz)
    }
}