use std::{cell::RefCell, rc::Rc};

use itertools::iproduct;

use crate::{light::light::Light, voxels::{chunks::Chunks, voxel::Voxel}, direction::Direction};

use self::global_xyz::GlobalXYZ;

pub mod global_xyz;
pub mod xyz;
pub mod sun;


pub struct World {
    pub chunks: Chunks,
    pub light: Light
}

impl World {
    pub fn new(width: i32, height: i32, depth: i32) -> Self {
        Self {
            chunks: Chunks::new(width, height, depth, 0, 0, 0),
            light: Light::new()
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


    pub fn break_voxel(&mut self, xyz: &GlobalXYZ) {
        self.chunks.set(xyz.0, xyz.1, xyz.2, 0, None);
        self.light.on_block_break(&mut self.chunks, xyz.0, xyz.1, xyz.2);
    }

    pub fn set_voxel(&mut self, xyz: &GlobalXYZ, id: u32, dir: &Direction) {
        self.chunks.set(xyz.0, xyz.1, xyz.2, id, Some(dir));
        self.light.on_block_set(&mut self.chunks, xyz.0, xyz.1, xyz.2, id);
    }


    pub fn voxel(&self, xyz: &GlobalXYZ) -> Option<&Voxel> {
        self.chunks.voxel_global(xyz.0, xyz.1, xyz.2)
    }
}