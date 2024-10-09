use std::{sync::Arc};

use itertools::iproduct;

use crate::{content::Content, coords::{chunk_coord::ChunkCoord, local_coord::LocalCoord}, voxels::{chunk::{Chunk, CHUNK_SIZE, CHUNK_SQUARE, CHUNK_VOLUME}, chunks::{Chunks, WORLD_BLOCK_HEIGHT, WORLD_HEIGHT}}};

use super::{light::Light, light_solver::LightSolver};

const MAX_LIGHT: u8 = 15;
const SIDE_COORDS_OFFSET: [(i32, i32, i32); 6] = [
    (1,0,0), (-1,0,0),
    (0,1,0), (0,-1,0),
    (0,0,1), (0,0,-1),
];

pub const ADD_QUEUE_CAP: usize = 262_144;
pub const REMOVE_QUEUE_CAP: usize = 131_072;

#[derive(Debug)]
pub struct LightSolvers<'content> {
    content: &'content Content,
    solver: LightSolver,
}


impl<'content> LightSolvers<'content> {
    pub fn new(content: &'content Content) -> Self {
        Self {content, solver: LightSolver::new()}
    }

    pub fn with_capacity<'cont>(add_cap: usize, remove_cap: usize, content: &'content Content) -> Self {Self {
        content,
        solver: LightSolver::with_capacity(add_cap, remove_cap),
    }}

    pub fn build_sky_light_chunk(&mut self, chunks: &Chunks, cx: i32, cz: i32) {
        let Some(chunk) = chunks.chunk(ChunkCoord::new(cx, cz)) else {return};

        for i in (CHUNK_VOLUME-CHUNK_SQUARE)..CHUNK_VOLUME {
            unsafe {chunk.light_map().0.get_unchecked(i)}.set_sun(15);
        }

        for idx in (0..CHUNK_VOLUME-CHUNK_SQUARE).rev() {
            let id = unsafe {chunk.voxels().0.get_unchecked(idx)}.id() as usize;
            if unsafe {chunk.light_map().0.get_unchecked(idx + CHUNK_SQUARE)}.get_sun() == 15
                && self.content.blocks[id].is_light_passing()
            {
                unsafe {chunk.light_map().0.get_unchecked(idx)}.set_sun(15);
                let global = ChunkCoord::new(cx, cz)
                    .to_global(LocalCoord::from_index(idx));
                self.solver.add_max_sun_to_solve(global);
            }
        }

        self.solver.solve(chunks, &self.content);
        chunk.modify(true);
    }


    pub fn on_chunk_loaded(&mut self, chunks: &Chunks, cx: i32, cz: i32) {
        let cc = ChunkCoord::new(cx, cz);
        let Some(chunk) = chunks.chunk(cc) else {return};
        for idx in 0..CHUNK_VOLUME {
            let id = unsafe {chunk.voxels().0.get_unchecked(idx)}.id() as usize;
            let emission = self.content.blocks[id].emission();
            if emission.iter().any(|e| *e > 0) {
                let light = Light::new(emission[0], emission[1], emission[2], 0);
                self.solver.add_with_emission_and_chunk(
                    &chunk, LocalCoord::from_index(idx), light);
            }
        }

        self.solve(chunks);
        self.build_nearby_light(chunks, cx, cz);
    }


    #[inline(never)]
    fn build_nearby_light(&mut self, chunks: &Chunks, cx: i32, cz: i32) {
        let buffer: [Option<Arc<Chunk>>; 4] = [
            chunks.chunk((cx - 1, cz).into()),
            chunks.chunk((cx, cz - 1).into()),
            chunks.chunk((cx + 1, cz).into()),
            chunks.chunk((cx, cz + 1).into())
        ];
        for (ly, mut lz, mut lx) in iproduct!(0..WORLD_BLOCK_HEIGHT as i32, -1..=CHUNK_SIZE as i32, -1..=CHUNK_SIZE as i32) {
            if lx == lz || (lx == -1 && lz == CHUNK_SIZE as i32) || (lx == CHUNK_SIZE as i32 && lz == -1) {continue};
            let Some(chunk) = 
                (if lx == -1 {
                    lx = CHUNK_SIZE as i32 - 1;
                    &buffer[0]
                } else if lx == CHUNK_SIZE as i32 {
                    lx = 0;
                    &buffer[2]
                } else if lz == -1 {
                    lz = CHUNK_SIZE as i32 - 1;
                    &buffer[1]
                } else if lz == CHUNK_SIZE as i32 {
                    lz = 0;
                    &buffer[3]
                } else {
                    continue;
                }) else {continue};

            let lc = (lx as usize, ly as usize, lz as usize).into();
            self.solver.add_with_chunk(chunk, lc);
            self.solve(chunks);
        }
    }


    pub fn on_block_break(&mut self, chunks: &Chunks, x: i32, y: i32, z: i32) {
        self.remove_rgb(chunks, x, y, z);
        self.solve(chunks);
        if chunks.get_sun((x, y+1, z).into()) == MAX_LIGHT || (y+1) as usize == WORLD_HEIGHT*CHUNK_SIZE {
            for i in (0..=y).rev() {
                if chunks.voxel_global((x, i, z).into()).map_or(true, |v| v.id != 0) {break};
                self.solver.add_with_emission(chunks, (x, i, z).into(), Light::new(0, 0, 0, MAX_LIGHT));
            }
        }
        for (ax, ay, az) in SIDE_COORDS_OFFSET {
            self.add_rgbs(chunks, x+ax, y+ay, z+az);
        }
        self.solve(chunks);
    }


    pub fn on_block_set(&mut self, chunks: &Chunks, x: i32, y: i32, z: i32, id: u32) {
        let emission = self.content.blocks[id as usize].emission();
        self.remove_rgbs(chunks, x, y, z);
        self.solver.solve(chunks, &self.content);

        for ny in (0..y).rev() {
            if chunks.voxel_global((x, ny, z).into()).map_or(0, |v| v.id) != 0 {break};
            self.solver.remove_s(chunks, (x, ny, z).into());
            self.solver.solve(chunks, &self.content);
        }

        if emission.iter().any(|e| *e > 0) {
            self.add_with_emission_rgb(chunks, x, y, z, emission);
        }
        self.solve(chunks);
    }


    pub fn add_rgb(&mut self, chunks: &Chunks, x: i32, y: i32, z: i32) {
        self.solver.add(chunks, (x, y, z).into());
    }

    pub fn add_rgbs(&mut self, chunks: &Chunks, x: i32, y: i32, z: i32) {
        self.solver.add(chunks, (x, y, z).into());
    }

    pub fn add_with_emission_rgb(&mut self, chunks: &Chunks, x: i32, y: i32, z: i32, emission: &[u8; 3]) {
        self.solver.add_with_emission(chunks, (x, y, z).into(), Light::new(emission[0], emission[1], emission[2], 0));
    }

    pub fn solve(&mut self, chunks: &Chunks) {
        self.solver.solve(chunks, &self.content);
    }

    pub fn remove_rgb(&mut self, chunks: &Chunks, x: i32, y: i32, z: i32) {
        self.solver.remove_rgb(chunks, (x, y, z).into());
    }

    pub fn remove_rgbs(&mut self, chunks: &Chunks, x: i32, y: i32, z: i32) {
        self.solver.remove_all(chunks, (x, y, z).into());
    }
}