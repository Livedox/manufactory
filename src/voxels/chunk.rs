use crate::light::light_map::LightMap;

use super::voxel::{self, Voxel};

pub const CHUNK_SIZE: usize = 16; // must be a power of two
pub const CHUNK_SQUARE: usize = CHUNK_SIZE.pow(2);
pub const CHUNK_VOLUME: usize = CHUNK_SIZE.pow(3);
pub const CHUNK_BIT_SHIFT: usize = CHUNK_SIZE.ilog2() as usize;
pub const CHUNK_BITS: usize = CHUNK_SIZE - 1 as usize;


#[derive(Debug)]
pub struct Chunk {
    pub voxels: [voxel::Voxel; CHUNK_VOLUME],
    pub modified: bool,
    pub lightmap: LightMap,
    pub chunk_coords: (i32, i32, i32)
}


impl Chunk {
    pub fn new(pos_x: i32, pos_y: i32, pos_z: i32) -> Chunk {
        let mut voxels = [Voxel::new(0); CHUNK_VOLUME];
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let real_x = x + pos_x as usize*CHUNK_SIZE;
                    let real_y = y + pos_y as usize*CHUNK_SIZE;
                    let _real_z = z + pos_z as usize*CHUNK_SIZE;
                    if real_y as f64 <= ((real_x as f64 *0.3).sin() * 0.5 + 0.5) * 10. {
                        voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 1;
                    }
                    if real_y <= 1 {
                        voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 3;
                    }
                }
            }
        }
        Chunk {
            voxels,
            chunk_coords: (pos_x, pos_y, pos_z),
            modified: true,
            lightmap: LightMap::new()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.voxels.iter().all(|voxel| { voxel.id == 0 })
    }


    fn voxel_index(local_coords: (usize, usize, usize)) -> usize {
        (local_coords.1*CHUNK_SIZE+local_coords.2)*CHUNK_SIZE+local_coords.0
    }


    pub fn voxel(&self, local_coords: (usize, usize, usize)) -> &Voxel {
        &self.voxels[Self::voxel_index(local_coords)]
    }
    pub fn mut_voxel(&mut self, local_coords: (usize, usize, usize)) -> &mut Voxel {
        &mut self.voxels[Self::voxel_index(local_coords)]
    }

    pub fn modify(&mut self) {
        self.modified = true;
    }

    pub fn set_voxel_id(&mut self, local_coords: (usize, usize, usize), id: u32) {
        self.mut_voxel(local_coords).id = id
    }
}