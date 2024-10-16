use std::{sync::{Arc}, time::Instant};


use itertools::Itertools;
use rayon::prelude::*;

use crate::{bytes::BytesCoder, content::Content, coords::{chunk_coord::ChunkCoord, global_coord::GlobalCoord}, light::light_solvers::{LightSolvers, ADD_QUEUE_CAP, REMOVE_QUEUE_CAP}, save_load::{EncodedChunk, WorldRegions}, voxels::{chunk::{Chunk, CHUNK_VOLUME}, chunks::Chunks, generator::Generator, voxel::Voxel}};

pub mod sun;
pub mod loader;


#[derive(Debug)]
pub struct World {
    generator: Generator,
    pub chunks: Arc<Chunks>,
}

impl World {
    pub fn new(content: Arc<Content>, seed: u64, render_radius: i32, ox: i32, oz: i32) -> Self {
        Self {
            generator: Generator::new(&content, seed),
            chunks: Arc::new(Chunks::new(Arc::clone(&content), render_radius, ox, oz)),
        }
    }

    #[inline(never)]
    pub fn load_column_of_chunks(&self, regions: &WorldRegions, cx: i32, cz: i32, light: &mut LightSolvers) {
        let start = Instant::now();
        let chunk = match regions.chunk((cx, cz).into()) {
            None => Chunk::new(&self.generator, cx, cz),
            Some(EncodedChunk::Some(b)) => Chunk::decode_bytes(&self.chunks.content, &b),
        };
        self.chunks.chunks.write().unwrap().insert((cx, cz).into(), Arc::new(chunk));
        self.build_chunk(light, cx, cz);
        light.solve(&self.chunks);
        println!("Load column: {:?}", start.elapsed().as_secs_f32());
    }

    pub fn load_chunks(&self, regions: &WorldRegions, solvers: &mut [LightSolvers], ccs: Vec<ChunkCoord>) {
        let start = Instant::now();
        let chunks: Vec<Arc<Chunk>> = ccs.par_iter()
            .map(|cc| {
                let chunk = match regions.chunk(*cc) {
                    None => Chunk::new(&self.generator, cc.x, cc.z),
                    Some(EncodedChunk::Some(b)) => Chunk::decode_bytes(&self.chunks.content, &b),
                };
                Arc::new(chunk)
            })
            .collect();

        let mut lock = self.chunks.chunks.write().unwrap();
        chunks.into_iter()
            .for_each(|chunk| {lock.insert(chunk.coord, chunk);});
        drop(lock);

        let mut s: Vec<&[ChunkCoord]> = vec![&[]; solvers.len()];
        ccs.chunks((ccs.len() as f32 / solvers.len() as f32).ceil() as usize)
            .enumerate()
            .for_each(|(i, slice)| s[i] = slice);
        solvers.par_iter_mut().zip_eq(s).for_each(|(solver, ccs)| {
            ccs.iter().for_each(|cc| {
                self.build_chunk(solver, cc.x, cc.z);
                solver.solve(&self.chunks);
            });
        });
        println!("Load column: {:?}", start.elapsed().as_secs_f32());
    }

    pub fn build_chunk(&self, light: &mut LightSolvers, cx: i32, cz: i32) {
        light.build_sky_light_chunk(&self.chunks, cx, cz);
        light.on_chunk_loaded(&self.chunks, cx, cz);
    }


    pub fn break_voxel(&self, xyz: &GlobalCoord) {
        Chunks::set_voxel(&self.chunks, *xyz, 0);
        LightSolvers::new(&self.chunks.content).on_block_break(&self.chunks, xyz.x, xyz.y, xyz.z);
    }

    pub fn voxel(&self, xyz: &GlobalCoord) -> Option<Voxel> {
        self.chunks.voxel_global(*xyz)
    }
}