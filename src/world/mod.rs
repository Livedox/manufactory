use std::sync::mpsc::{Sender, Receiver};

use itertools::iproduct;

use crate::{light::light::Light, voxels::{chunks::{Chunks, WORLD_HEIGHT}, voxel::Voxel, chunk::CHUNK_SIZE}, direction::Direction, world_loader::{self, WorldLoader}};

use self::global_coords::GlobalCoords;

pub mod global_coords;
pub mod chunk_coords;
pub mod local_coords;
pub mod coords;
pub mod sun;

#[derive(Debug)]
pub struct World {
    pub waiting_chunks: Vec<(i32, i32)>,
    pub chunks: Chunks,
    pub light: Light
}

impl World {
    pub fn new(width: i32, height: i32, depth: i32, ox: i32, oy: i32, oz: i32) -> Self {
        Self {
            chunks: Chunks::new(width, height, depth, ox, oy, oz),
            light: Light::new(),
            waiting_chunks: vec![]
        }
    }


    pub fn build_sky_light(&mut self) {
        let height = self.chunks.height;
        let depth = self.chunks.depth;
        let width = self.chunks.width;
        for (cy, cz, cx) in iproduct!(0..height, 0..depth, 0..width) {
            self.light.on_chunk_loaded(&mut self.chunks, cx, cy, cz);
        }
        self.light.build_sky_light(&mut self.chunks);
    }


    pub fn break_voxel(&mut self, xyz: &GlobalCoords) {
        self.chunks.set(*xyz, 0, None);
        self.light.on_block_break(&mut self.chunks, xyz.0, xyz.1, xyz.2);
    }

    pub fn set_voxel(&mut self, xyz: &GlobalCoords, id: u32, dir: &Direction) {
        self.chunks.set(*xyz, id, Some(dir));
        self.light.on_block_set(&mut self.chunks, xyz.0, xyz.1, xyz.2, id);
    }


    pub fn voxel(&self, xyz: &GlobalCoords) -> Option<&Voxel> {
        self.chunks.voxel_global(*xyz)
    }


    pub fn load_chunks(&mut self, world_loader: &WorldLoader) {
        for (cy, cz, cx) in iproduct!(0..self.chunks.height, 0..self.chunks.depth, 0..self.chunks.width) {
            if self.chunks.chunk((cx, cy, cz)).is_none() && !self.waiting_chunks.contains(&(cx, cz)) {
                self.waiting_chunks.push((cx, cz));
                let _ = world_loader.send((cx, cz));
            }
        }
    }

    pub fn receive_world(&mut self, world_loader: &WorldLoader) {
        if let Ok(world) = world_loader.try_recv() {
            let chunks = world.chunks;
            let cxz = (chunks.chunks[0].as_ref().unwrap().xyz.0, chunks.chunks[0].as_ref().unwrap().xyz.2);
            for chunk in chunks.chunks.into_iter() {
                let Some(chunk) = chunk else {continue};
                let index = chunk.xyz.index(self.chunks.depth, self.chunks.width);
                self.chunks.chunks[index] = Some(chunk);
            };

            let min_x = cxz.0*CHUNK_SIZE as i32-1;
            let max_x = (cxz.0+1)*CHUNK_SIZE as i32+1;

            let min_z = cxz.1*CHUNK_SIZE as i32-1;
            let max_z = (cxz.1+1)*CHUNK_SIZE as i32+1;
            for (gy, gz, gx) in iproduct!(0i32..((WORLD_HEIGHT*CHUNK_SIZE) as i32), min_z..max_z, min_x..max_x) {
                if gx == min_x || gx == max_x || gz == max_z || gz == min_z {
                    self.light.add_rgbs(&mut self.chunks, gx, gy, gz); 
                }
            }
            self.light.solve_rgbs(&mut self.chunks);
        }
    }
}