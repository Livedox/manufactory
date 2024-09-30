use std::{sync::{Arc}, time::Instant};


use crate::{bytes::BytesCoder, content::Content, coords::global_coord::GlobalCoord, light::light_solvers::{LightSolvers, ADD_QUEUE_CAP, REMOVE_QUEUE_CAP}, save_load::{EncodedChunk, WorldRegions}, voxels::{chunk::{Chunk, CHUNK_VOLUME}, chunks::Chunks, generator::Generator, voxel::Voxel}};

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
    pub fn new(content: Arc<Content>, seed: u64, render_radius: i32, ox: i32, oz: i32) -> Self {
        Self {
            generator: Generator::new(&content, seed),
            chunks: Arc::new(Chunks::new(Arc::clone(&content), render_radius, ox, oz)),
            light: LightSolvers::new(ADD_QUEUE_CAP, REMOVE_QUEUE_CAP, Arc::clone(&content)),
            player_light_solver: LightSolvers::new(CHUNK_VOLUME, CHUNK_VOLUME, content),
        }
    }

    pub fn solve(&self) {
        self.light.solve(&self.chunks);
    }

    pub fn load_column_of_chunks(&self, regions: &WorldRegions, cx: i32, cz: i32) {
        let start = Instant::now();
        let chunk = match regions.chunk((cx, cz).into()) {
            None => Chunk::new(&self.generator, cx, cz),
            Some(EncodedChunk::Some(b)) => Chunk::decode_bytes(&self.chunks.content, &b),
        };
        let chunks = unsafe {&mut *self.chunks.chunks.get()};
        chunks.insert((cx, cz).into(), Arc::new(chunk));
        self.build_chunk(cx, cz);
        self.solve();
        println!("Load column: {:?}", start.elapsed().as_secs_f32());
    }

    pub fn build_chunk(&self, cx: i32, cz: i32) {
        self.light.build_sky_light_chunk(&self.chunks, cx, cz);
        self.light.on_chunk_loaded(&self.chunks, cx, cz);
    }


    pub fn break_voxel(&self, xyz: &GlobalCoord) {
        Chunks::set_voxel(&self.chunks, *xyz, 0);
        // self.chunks.set(*xyz, 0, None);
        self.player_light_solver.on_block_break(&self.chunks, xyz.x, xyz.y, xyz.z);
    }

    pub fn voxel(&self, xyz: &GlobalCoord) -> Option<Voxel> {
        self.chunks.voxel_global(*xyz)
    }
}