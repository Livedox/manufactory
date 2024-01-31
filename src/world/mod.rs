use std::{marker::PhantomPinned, pin::Pin, sync::{Arc, RwLock}, time::Instant};
use itertools::iproduct;

use crate::{light::light::LightSolvers, voxels::{chunks::{Chunks, WORLD_HEIGHT}, voxel::Voxel, chunk::{Chunk, CHUNK_VOLUME}}, direction::Direction, save_load::{WorldRegions, EncodedChunk}, bytes::BytesCoder};

use self::global_coords::GlobalCoords;

pub mod global_coords;
pub mod chunk_coords;
pub mod local_coords;
pub mod coords;
pub mod sun;
pub mod loader;


#[derive(Debug)]
pub struct World {
    pub chunks: Arc<Chunks>,
    pub light: LightSolvers,
    pub player_light_solver: LightSolvers
}

impl World {
    pub fn new(width: i32, height: i32, depth: i32, ox: i32, oy: i32, oz: i32) -> Self {
        Self {
            chunks: Arc::new(Chunks::new(width, height, depth, ox, oy, oz)),
            light: LightSolvers::default(),
            player_light_solver: LightSolvers::new(CHUNK_VOLUME, CHUNK_VOLUME),
        }
    }

    pub fn solve_rgbs(&self) {
        self.light.solve_rgbs(&self.chunks);
    }

    pub fn load_column_of_chunks(&self, regions: &mut WorldRegions, cx: i32, cz: i32) {
        let start = Instant::now();
        for cy in (0..WORLD_HEIGHT as i32).rev() {
            let chunk = match regions.chunk((cx, cy, cz).into()) {
                EncodedChunk::None => Chunk::new(cx, cy, cz),
                EncodedChunk::Some(b) => Chunk::decode_bytes(b),
            };
            
            let index = chunk.xyz.chunk_index(&self.chunks);
            let chunks = unsafe {&mut *self.chunks.chunks.get()};
            chunks[index] = Some(Arc::new(chunk));
            self.build_chunk(cx, cy, cz);
        }
        self.solve_rgbs();
        println!("Load column: {:?}", start.elapsed().as_secs_f32());
    }

    pub fn build_chunk(&self, cx: i32, cy: i32, cz: i32) {
        self.light.build_sky_light_chunk(&self.chunks, cx, cy, cz);
        self.light.on_chunk_loaded(&self.chunks, cx, cy, cz);
    }


    pub fn break_voxel(&self, xyz: &GlobalCoords) {
        Chunks::set(&self.chunks, *xyz, 0, None);
        // self.chunks.set(*xyz, 0, None);
        self.player_light_solver.on_block_break(&self.chunks, xyz.0, xyz.1, xyz.2);
    }

    pub fn set_voxel(&self, xyz: &GlobalCoords, id: u32, dir: &Direction) {
        Chunks::set(&self.chunks, *xyz, id, Some(dir));
        // self.chunks.set(*xyz, id, Some(dir));
        self.player_light_solver.on_block_set(&self.chunks, xyz.0, xyz.1, xyz.2, id);
    }

    pub fn voxel(&self, xyz: &GlobalCoords) -> Option<Voxel> {
        self.chunks.voxel_global(*xyz)
    }
}