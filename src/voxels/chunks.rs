use std::{collections::HashMap, rc::Rc, cell::RefCell};

use itertools::iproduct;

use crate::{graphic::render::{VoxelRenderer}, vertices::{block_vertex::BlockVertex}, model::animated_model::AnimatedModel, direction::Direction, recipes::recipe::Recipes, world::global_xyz::GlobalXYZ};

use super::{chunk::{Chunk, CHUNK_SIZE, CHUNK_BITS, CHUNK_BIT_SHIFT}, voxel::Voxel, voxel_data::{VoxelAdditionalData, Drill, VoxelData, MultiBlock}};

pub const WORLD_HEIGHT: usize = 16; // In chunks

#[derive(Debug)]
pub struct Chunks {
    pub chunks: Vec<Option<Chunk>>,
    pub meshes: Vec<Option<(Vec<BlockVertex>, Vec<u16>, HashMap::<String, Vec<([f32; 3], [f32; 4], f32, u32)>>, HashMap<String, Vec<([f32; 3], [f32; 4], u32)>>, Vec<BlockVertex>, Vec<u16>)>>,
    pub volume: i32,
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    ox: i32,
    oy: i32,
    oz: i32,
}

impl Chunks {
    pub fn new(width: i32, height: i32, depth: i32, ox: i32, oy: i32, oz: i32) -> Chunks {
        let volume = width*height*depth;
        let mut chunks: Vec<Option<Chunk>> = vec![];
        for _ in 0..volume { chunks.push(None); }
        let mut meshes: Vec<Option<(Vec<BlockVertex>, Vec<u16>, HashMap::<String, Vec<([f32; 3], [f32; 4], f32, u32)>>, HashMap<String, Vec<([f32; 3], [f32; 4], u32)>>, Vec<BlockVertex>, Vec<u16>)>> = vec![];
        for _ in 0..volume { meshes.push(None); }
        // for y in 0..height {
        //     for z in 0..depth {
        //         for x in 0..width {
        //             chunks.push(Some(Chunk::new(x, y, z)));
        //         }
        //     }
        // }

        Chunks { chunks, meshes, volume, width, height, depth, ox, oy, oz }
    }

    pub fn get_nearest_chunk_index(&mut self) -> Option<usize> {
        let mut near_x: i32 = 0;
        let mut near_y: i32 = 0;
        let mut near_z: i32 = 0;
        let mut min_distance = i32::MAX;
        for y in 0..self.height {
            for z in 0..self.depth {
                for x in 0..self.width {
                    let chunk = &self.chunks[((y * self.depth + z) * self.width + x) as usize];
                    if chunk.is_none() { continue; }
                    if !chunk.as_ref().unwrap().modified { continue; }
                       
                    let lx = x - self.width/2;
                    let ly = y - self.height/2;
                    let lz = z - self.depth/2;
                    let distance = lx * lx + ly * ly + lz * lz;
                    if distance < min_distance {
                        min_distance = distance;
                        near_x = x;
                        near_y = y;
                        near_z = z;
                    }
                }
            }
        }
        let index = ((near_y * self.depth + near_z) * self.width + near_x) as usize;
        let chunk = &mut self.chunks[index];
        if chunk.is_none() { return None; }

        if chunk.as_ref().unwrap().modified {
            chunk.as_mut().unwrap().modified = false;
        } else {
            return None;
        }
        Some(index)
    }


    pub fn build_meshes(&mut self, render: &mut VoxelRenderer, animated_models: &HashMap<String, AnimatedModel>) -> Option<usize> {
        let mut near_x: i32 = 0;
        let mut near_y: i32 = 0;
        let mut near_z: i32 = 0;
        let mut min_distance = i32::MAX;
        for y in 0..self.height {
            for z in 0..self.depth {
                for x in 0..self.width {
                    let chunk = &self.chunks[((y * self.depth + z) * self.width + x) as usize];
                    if chunk.is_none() { continue; }
                    let mesh = &self.meshes[((y * self.depth + z) * self.width + x) as usize];
                    if mesh.is_some() && !chunk.as_ref().unwrap().modified { continue; }
                       
                    let lx = x - self.width/2;
                    let ly = y - self.height/2;
                    let lz = z - self.depth/2;
                    let distance = lx * lx + ly * ly + lz * lz;
                    if distance < min_distance {
                        min_distance = distance;
                        near_x = x;
                        near_y = y;
                        near_z = z;
                    }
                }
            }
        }
        let index = ((near_y * self.depth + near_z) * self.width + near_x) as usize;
        let chunk = &mut self.chunks[index];
        if chunk.is_none() { return None; }

        let mesh = &mut self.meshes[index];
        if mesh.is_none() || chunk.as_ref().unwrap().modified {
            if chunk.as_ref().unwrap().is_empty() || mesh.is_some() {
                self.meshes[index] = None;
            }
            chunk.as_mut().unwrap().modified = false;
        } else {
            return None;
        }
        
        let voxels = render.render_test(index, self);
        self.meshes[index] = Some(voxels);
		Some(index)
    }


    pub fn load_visible(&mut self) -> bool {
        let mut near_x: i32 = 0;
        let mut near_y: i32 = 0;
        let mut near_z: i32 = 0;
        let mut min_distance = i32::MAX;
        for y in 0..self.height {
            for z in 0..self.depth {
                for x in 0..self.width {
                    let chunk = &self.chunks[((y * self.depth + z) * self.width + x) as usize];
                    if chunk.is_some() { continue; }

                    let lx = x - self.width/2;
                    let ly = y - self.height/2;
                    let lz = z - self.depth/2;
                    let distance = lx*lx + ly*ly + lz*lz;
                    if distance < min_distance {
                        min_distance = distance;
                        near_x = x;
                        near_y = y;
                        near_z = z;
                    }
                }
            }
        }
        let index = ((near_y * self.depth + near_z) * self.width + near_x) as usize;
        let chunk = &self.chunks[index];
        if chunk.is_some() { return false; }

        self.chunks[index] = Some(Chunk::new(near_x+self.ox, near_y+self.oy, near_z+self.oz));

        true
    }


    pub fn voxel(&self, chunk_coords: (i32, i32, i32), local_coords: (usize, usize, usize)) -> Option<&Voxel> {
        if let Some(chunk) = self.chunk(chunk_coords) { return Some(chunk.voxel(local_coords)) }
        None
    }


    pub fn voxel_global(&self, x: i32, y: i32, z: i32) -> Option<&Voxel> {
        self.voxel(Self::chunk_coords(x, y, z), Self::local_coords(x, y, z))
    }

    pub fn is_air_global(&self, x: i32, y: i32, z: i32) -> bool {
        let Some(voxel) = self.voxel(Self::chunk_coords(x, y, z), Self::local_coords(x, y, z)) else {return false};
        voxel.id == 0
    }


    pub fn set(&mut self, x: i32, y: i32, z: i32, id: u32, direction: Option<&Direction>) -> Option<u32> {
        let coords = Self::chunk_coords(x, y, z);
        let Some(chunk) = self.mut_chunk(coords) else {return None};

        let local = Self::local_coords(x, y, z);
    
        let x_offset = (local.0 >= CHUNK_SIZE-1) as i32 - (local.0 <= 0) as i32;
        let y_offset = (local.1 >= CHUNK_SIZE-1) as i32 - (local.1 <= 0) as i32;
        let z_offset = (local.2 >= CHUNK_SIZE-1) as i32 - (local.2 <= 0) as i32;
        chunk.set_voxel_id(local, id, direction);
        chunk.modify();
        
        
        if x_offset != 0 {
            if let Some(chunk) = self.mut_chunk((coords.0+x_offset, coords.1, coords.2)) {chunk.modify()};
        }
        if y_offset != 0 {
            if let Some(chunk) = self.mut_chunk((coords.0, coords.1+y_offset, coords.2)) {chunk.modify()};
        }
        if z_offset != 0 {
            if let Some(chunk) = self.mut_chunk((coords.0, coords.1, coords.2+z_offset)) {chunk.modify()};
        }
        
        Some(id)
    }


    pub fn light(&self, x: i32, y: i32, z: i32, channel: u8) -> u16 {
        let chunk_coords = Self::chunk_coords(x, y, z);
        let chunk = self.chunk(chunk_coords);

        if !self.is_in_area(chunk_coords) || chunk.is_none() { return 0; }

        chunk.unwrap().lightmap.get(Self::u8_local_coords(x, y, z), channel)
    }


    pub fn is_in_area(&self, chunk_coords: (i32, i32, i32)) -> bool {
        chunk_coords.0 >= 0 && chunk_coords.0 < self.width &&
        chunk_coords.1 >= 0 && chunk_coords.1 < self.height &&
        chunk_coords.2 >= 0 && chunk_coords.2 < self.depth
    }


    fn chunk_index(&self, chunk_coords: (i32, i32, i32)) -> usize {
        ((chunk_coords.1*self.depth+chunk_coords.2) * self.width + chunk_coords.0) as usize
    }


    pub fn chunk(&self, chunk_coords: (i32, i32, i32)) -> Option<&Chunk> {
        if !self.is_in_area(chunk_coords) { return None; }
        let chunk = self.chunks.get(self.chunk_index(chunk_coords));
        if let Some(chunk) = chunk { return chunk.as_ref() }
        None
    }


    pub fn mut_chunk(&mut self, chunk_coords: (i32, i32, i32)) -> Option<&mut Chunk> {
        if !self.is_in_area(chunk_coords) { return None; }
        let index = self.chunk_index(chunk_coords);
        let chunk = self.chunks.get_mut(index);
        if let Some(chunk) = chunk { return chunk.as_mut() }
        None
    }


    pub fn chunk_by_global(&self, x: i32, y: i32, z: i32) -> Option<&Chunk> {
        self.chunk(Self::chunk_coords(x, y, z))
    }


    pub fn mut_chunk_by_global(&mut self, x: i32, y: i32, z: i32) -> Option<&mut Chunk> {
        self.mut_chunk(Self::chunk_coords(x, y, z))
    }


    pub fn global_coords(chunk_coords: (i32, i32, i32), local: (usize, usize, usize)) -> (i32, i32, i32) {(
        (chunk_coords.0*CHUNK_SIZE as i32 + local.0 as i32),
        (chunk_coords.1*CHUNK_SIZE as i32 + local.1 as i32),
        (chunk_coords.2*CHUNK_SIZE as i32 + local.2 as i32),
    )}

    
    pub fn chunk_coords(x: i32, y: i32, z: i32) -> (i32, i32, i32) {(
        x>>CHUNK_BIT_SHIFT,
        y>>CHUNK_BIT_SHIFT,
        z>>CHUNK_BIT_SHIFT
    )}
    
    
    pub fn local_coords(x: i32, y: i32, z: i32) -> (usize, usize, usize) {(
        x as usize % CHUNK_SIZE,
        y as usize % CHUNK_SIZE,
        z as usize % CHUNK_SIZE
    )}

    pub fn u8_local_coords(x: i32, y: i32, z: i32) -> (u8, u8, u8) {
        let local = Self::local_coords(x, y, z);
        (local.0 as u8, local.1 as u8, local.2 as u8)
    }


    pub fn voxels_data(&self, chunk_coords: (i32, i32, i32)) -> Option<&HashMap<usize, VoxelData>> {
        let Some(chunk) = self.chunk(chunk_coords) else {return None};
        Some(chunk.voxels_data())
    }


    pub fn mut_voxels_data(&mut self, chunk_coords: (i32, i32, i32)) -> Option<&mut HashMap<usize, VoxelData>> {
        let Some(chunk) = self.mut_chunk(chunk_coords) else {return None};
        Some(chunk.mut_voxels_data())
    }


    pub fn add_multiblock_structure(&mut self, xyz: &GlobalXYZ, width: i32, height: i32, depth: i32, id: u32, dir: &Direction) -> Option<Vec<(i32, i32, i32)>> {
        let mut coords: Vec<(i32, i32, i32)> = vec![];
        // FIX THIS SHIT
        let width_range = if width > 0 {
            (xyz.0)..(xyz.0+width)
        } else {
            (xyz.0+width+1)..(xyz.0+1)
        };
        let height_range = if height > 0 {
            (xyz.1)..(xyz.1+height)
        } else {
            (xyz.1+height+1)..(xyz.1+1)
        };
        let depth_range = if depth > 0 {
            (xyz.2)..(xyz.2+depth)
        } else {
            (xyz.2+depth+1)..(xyz.2+1)
        };
        coords.push(xyz.clone().into());
        for (nx, nz, ny) in iproduct!(width_range, depth_range, height_range) {
            if nx == xyz.0 && ny == xyz.1 && nz == xyz.2 {continue};
            if !self.is_air_global(nx, ny, nz) {return None};
            coords.push((nx, ny, nz));
        }

        
        let voxel_additional_data = Rc::new(VoxelAdditionalData::new_multiblock(id, dir, coords.clone()));
        coords.iter().enumerate().for_each(|(index, coord)| {
            let id = if index == 0 {id} else {1};
            self.set(coord.0, coord.1, coord.2, id, None);
            let voxels_data = self.mut_voxels_data(Chunks::chunk_coords(coord.0, coord.1, coord.2)).unwrap();
            let local = Chunks::local_coords(coord.0, coord.1, coord.2);
            println!("{:?}", coord);
            voxels_data.insert(Chunk::voxel_index(local), VoxelData {
                id,
                global_coords: *coord,
                additionally: voxel_additional_data.clone()
            });
        });
        Some(coords)
    }


    pub fn remove_multiblock_structure(&mut self, x: i32, y: i32, z: i32) -> Option<Vec<(i32, i32, i32)>> {
        let Some(voxels_data) = self.voxels_data(Chunks::chunk_coords(x, y, z)) else {return None};
        let local = Chunks::local_coords(x, y, z);
        let Some(voxel_data) = voxels_data.get(&Chunk::voxel_index(local)) else {return None};
        let mut coords: Vec<(i32, i32, i32)> = vec![];
        match &voxel_data.additionally.as_ref() {
            VoxelAdditionalData::Drill(drill) => {
                drill.borrow().structure_coordinates().iter().for_each(|coord| {
                    coords.push(*coord);
                });
            },
            VoxelAdditionalData::AssemblingMachine(asembler) => {
                asembler.borrow().structure_coordinates().iter().for_each(|coord| {
                    coords.push(*coord);
                });
            },
            _ => (),
        };
        coords.iter().for_each(|coord| {
            self.set(coord.0, coord.1, coord.2, 0, None);
        });
        Some(coords)
    }

    pub fn get_sun(&self, x: i32, y: i32, z: i32) -> u16 {
        let local = Chunks::local_coords(x, y, z);
        self.chunk_by_global(x, y, z)
            .map_or(0, |c| c.lightmap.get_sun((local.0 as u8, local.1 as u8, local.2 as u8)))
    }


    pub fn get_light(&self, x: i32, y: i32, z: i32) -> u16 {
        let local = Chunks::local_coords(x, y, z);
        self.chunk_by_global(x, y, z)
            .map_or(0, |c| c.lightmap.get_light((local.0 as u8, local.1 as u8, local.2 as u8)))
    }
}
