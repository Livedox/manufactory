use std::{marker::PhantomPinned, pin::Pin, sync::{Arc, RwLock}, time::Instant};
use itertools::iproduct;

use crate::{bytes::BytesCoder, content::Content, direction::Direction, light::light::{LightSolvers, ADD_QUEUE_CAP, REMOVE_QUEUE_CAP}, save_load::{WorldRegions, EncodedChunk}, voxels::{chunk::{Chunk, CHUNK_VOLUME}, chunks::{Chunks, WORLD_HEIGHT}, generator::Generator, voxel::Voxel}};

use self::global_coords::GlobalCoords;

pub mod global_coords;
pub mod chunk_coords;
pub mod local_coords;
pub mod coords;
pub mod sun;
pub mod loader;


#[derive(Debug)]
pub struct World {
    generator: Generator,
    pub chunks: Arc<Chunks>,
    pub light: LightSolvers,
    pub player_light_solver: LightSolvers
}

impl World {
    pub fn new(content: Arc<Content>, seed: u64, width: i32, height: i32, depth: i32, ox: i32, oy: i32, oz: i32) -> Self {
        Self {
            generator: Generator::new(&content, seed),
            chunks: Arc::new(Chunks::new(Arc::clone(&content), width, height, depth, ox, oy, oz)),
            light: LightSolvers::new(ADD_QUEUE_CAP, REMOVE_QUEUE_CAP, Arc::clone(&content)),
            player_light_solver: LightSolvers::new(CHUNK_VOLUME, CHUNK_VOLUME, content),
        }
    }

    pub fn solve_rgbs(&self) {
        self.light.solve_rgbs(&self.chunks);
    }

    pub fn load_column_of_chunks(&self, regions: &mut WorldRegions, cx: i32, cz: i32) {
        let start = Instant::now();
        for cy in (0..WORLD_HEIGHT as i32).rev() {
            let chunk = match regions.chunk((cx, cy, cz).into()) {
                EncodedChunk::None => Chunk::new(&self.generator, cx, cy, cz),
                EncodedChunk::Some(b) => Chunk::decode_bytes(&self.chunks.content, b),
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