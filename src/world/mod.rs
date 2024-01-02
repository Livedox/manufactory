use itertools::iproduct;

use crate::{light::light::LightSolvers, voxels::{chunks::Chunks, voxel::Voxel}, direction::Direction};

use self::global_coords::GlobalCoords;

pub mod global_coords;
pub mod chunk_coords;
pub mod local_coords;
pub mod coords;
pub mod sun;


#[derive(Debug)]
pub struct World {
    pub chunks: Chunks,
    pub light: LightSolvers
}

impl World {
    pub fn new(width: i32, height: i32, depth: i32, ox: i32, oy: i32, oz: i32) -> Self {
        Self {
            chunks: Chunks::new(width, height, depth, ox, oy, oz),
            light: LightSolvers::new()
        }
    }


    pub fn add_rgbs(&mut self, x: i32, y: i32, z: i32) {
        self.light.add_rgbs(&mut self.chunks, x, y, z);
    }

    pub fn solve_rgbs(&mut self) {
        self.light.solve_rgbs(&mut self.chunks);
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


    pub fn build_chunk(&mut self, cx: i32, cy: i32, cz: i32) {
        self.light.build_sky_light_chunk(&mut self.chunks, cx, cy, cz);
        self.light.on_chunk_loaded(&mut self.chunks, cx, cy, cz);
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
}