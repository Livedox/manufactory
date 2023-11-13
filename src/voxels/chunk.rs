use std::{collections::HashMap, rc::Rc};

use itertools::iproduct;

use crate::{light::light_map::LightMap, direction::Direction};

use super::{voxel::{self, Voxel}, voxel_data::{VoxelData, VoxelAdditionalData}, chunks::Chunks, block::blocks::BLOCKS};

pub const CHUNK_SIZE: usize = 32; // must be a power of two
pub const HALF_CHUNK_SIZE: usize = CHUNK_SIZE/2;
pub const CHUNK_SQUARE: usize = CHUNK_SIZE.pow(2);
pub const CHUNK_VOLUME: usize = CHUNK_SIZE.pow(3);
pub const CHUNK_BIT_SHIFT: usize = CHUNK_SIZE.ilog2() as usize;
pub const CHUNK_BITS: usize = CHUNK_SIZE - 1_usize;


#[derive(Debug)]
pub struct Chunk {
    pub voxels: [voxel::Voxel; CHUNK_VOLUME],
    pub sun_map: [[bool; CHUNK_SIZE]; CHUNK_SIZE],
    pub voxels_data: HashMap<usize, VoxelData>,
    pub modified: bool,
    pub lightmap: LightMap,
    pub xyz: (i32, i32, i32)
}


impl Chunk {
    pub fn new(pos_x: i32, pos_y: i32, pos_z: i32) -> Chunk {
        let mut voxels = [Voxel::new(0); CHUNK_VOLUME];
        let mut voxels_data = HashMap::new();
        let mut sun_map = [[true; CHUNK_SIZE]; CHUNK_SIZE];

        for (y, z, x) in iproduct!(0..CHUNK_SIZE, 0..CHUNK_SIZE, 0..CHUNK_SIZE) {
            let real_x = x as i32 + pos_x*CHUNK_SIZE as i32;
            let real_y = y as i32 + pos_y*CHUNK_SIZE as i32;
            let real_z = z as i32 + pos_z*CHUNK_SIZE as i32;
            if real_y as f64 <= ((real_x as f64 *0.3).sin() * 0.5 + 0.5) * 10. {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 2;
                voxels_data.remove(&((y*CHUNK_SIZE+z)*CHUNK_SIZE+x));
                sun_map[x][z] = false;
            }
            if real_y <= 3 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 0;
                voxels_data.remove(&((y*CHUNK_SIZE+z)*CHUNK_SIZE+x));
                // sun_map[x][z] = false;
            }
            if real_y <= 2 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 5;
                voxels_data.remove(&((y*CHUNK_SIZE+z)*CHUNK_SIZE+x));
                sun_map[x][z] = false;
            }
            // if real_y <= 1 {
            //     voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 9;
            //     voxels_data.remove(&((y*CHUNK_SIZE+z)*CHUNK_SIZE+x));
            //     sun_map[x][z] = false;
            // }
            if x == 1 && y == 0 && z == 0 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 9;
                voxels_data.insert((y*CHUNK_SIZE+z)*CHUNK_SIZE+x, VoxelData {
                    id: 9,
                    global_coords: (real_x, real_y, real_z),
                    additionally: Rc::new(VoxelAdditionalData::new(9, &Direction::new_x())),
                });
                sun_map[x][z] = false;
            }
            if real_y == 3 && real_x == 11 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 5;
                voxels_data.remove(&((y*CHUNK_SIZE+z)*CHUNK_SIZE+x));
                // sun_map[x][z] = false;
            }
            if real_y < 1 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 10;
            }
        }

        Chunk {
            voxels,
            xyz: (pos_x, pos_y, pos_z),
            sun_map,
            voxels_data,
            modified: true,
            lightmap: LightMap::new()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.voxels.iter().all(|voxel| { voxel.id == 0 })
    }


    pub fn is_air(&self, local_coords: (usize, usize, usize)) -> bool {
        self.voxel(local_coords).id == 0
    }


    pub fn voxel_index(local_coords: (usize, usize, usize)) -> usize {
        (local_coords.1*CHUNK_SIZE+local_coords.2)*CHUNK_SIZE+local_coords.0
    }


    pub fn voxel(&self, local_coords: (usize, usize, usize)) -> &Voxel {
        &self.voxels[Self::voxel_index(local_coords)]
    }

    fn mut_voxel(&mut self, local_coords: (usize, usize, usize)) -> &mut Voxel {
        &mut self.voxels[Self::voxel_index(local_coords)]
    }

    pub fn modify(&mut self) {
        self.modified = true;
    }

    pub fn set_voxel_id(&mut self, local_coords: (usize, usize, usize), id: u32, direction: Option<&Direction>) {
        self.voxels_data.remove(&((local_coords.1*CHUNK_SIZE+local_coords.2)*CHUNK_SIZE+local_coords.0));
        self.mut_voxel(local_coords).id = id;
        if BLOCKS()[id as usize].is_additional_data() {
            self.voxels_data.insert((local_coords.1*CHUNK_SIZE+local_coords.2)*CHUNK_SIZE+local_coords.0, VoxelData {
                id,
                global_coords: Chunks::global_coords(self.xyz, local_coords),
                additionally: Rc::new(VoxelAdditionalData::new(id, direction.unwrap_or(&Direction::new_x()))),
            });
        }
    }


    pub fn mut_voxel_data(&mut self, local_coords: (usize, usize, usize)) -> Option<&mut VoxelData> {
        let index = Chunk::voxel_index(local_coords);
        self.voxels_data.get_mut(&index)
    }


    pub fn voxel_data(&self, local_coords: (usize, usize, usize)) -> Option<&VoxelData> {
        let index = Chunk::voxel_index(local_coords);
        self.voxels_data.get(&index)
    }


    pub fn voxels_data(&self) -> &HashMap<usize, VoxelData> {
        &self.voxels_data
    }


    pub fn mut_voxels_data(&mut self) -> &mut HashMap<usize, VoxelData> {
        &mut self.voxels_data
    }


    pub fn add_voxel_data(&mut self, local_coords: (usize, usize, usize), voxel_data: VoxelData) -> Option<VoxelData> {
        self.voxels_data.insert(Chunk::voxel_index(local_coords), voxel_data)
    }
}