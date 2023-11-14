use std::{collections::HashMap, rc::Rc, sync::{mpsc::{Receiver, Sender, self}, Arc}};

use itertools::iproduct;

use crate::{vertices::block_vertex::BlockVertex, models::animated_model::AnimatedModel, direction::Direction, world::{global_coords::GlobalCoords, local_coords::LocalCoords, chunk_coords::ChunkCoords}};

use super::{chunk::{Chunk, CHUNK_SIZE, CHUNK_BIT_SHIFT}, voxel::Voxel, voxel_data::{VoxelAdditionalData, VoxelData, MultiBlock}};

pub const WORLD_HEIGHT: usize = 256 / CHUNK_SIZE; // In chunks

#[derive(Debug)]
pub struct Chunks {
    pub chunks: Vec<Option<Chunk>>,
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

        Chunks { chunks, volume, width, height, depth, ox, oy, oz }
    }

    /// This function is SHIT
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


    // pub fn build_meshes(&mut self, render: &mut VoxelRenderer, animated_models: &HashMap<String, AnimatedModel>) -> Option<usize> {
    //     let mut near_x: i32 = 0;
    //     let mut near_y: i32 = 0;
    //     let mut near_z: i32 = 0;
    //     let mut min_distance = i32::MAX;
    //     for y in 0..self.height {
    //         for z in 0..self.depth {
    //             for x in 0..self.width {
    //                 let chunk = &self.chunks[((y * self.depth + z) * self.width + x) as usize];
    //                 if chunk.is_none() { continue; }
    //                 let mesh = &self.meshes[((y * self.depth + z) * self.width + x) as usize];
    //                 if mesh.is_some() && !chunk.as_ref().unwrap().modified { continue; }
                       
    //                 let lx = x - self.width/2;
    //                 let ly = y - self.height/2;
    //                 let lz = z - self.depth/2;
    //                 let distance = lx * lx + ly * ly + lz * lz;
    //                 if distance < min_distance {
    //                     min_distance = distance;
    //                     near_x = x;
    //                     near_y = y;
    //                     near_z = z;
    //                 }
    //             }
    //         }
    //     }
    //     let index = ((near_y * self.depth + near_z) * self.width + near_x) as usize;
    //     let chunk = &mut self.chunks[index];
    //     if chunk.is_none() { return None; }

    //     let mesh = &mut self.meshes[index];
    //     if mesh.is_none() || chunk.as_ref().unwrap().modified {
    //         if chunk.as_ref().unwrap().is_empty() || mesh.is_some() {
    //             self.meshes[index] = None;
    //         }
    //         chunk.as_mut().unwrap().modified = false;
    //     } else {
    //         return None;
    //     }
        
    //     let voxels = render.render_test(index, self);
    //     self.meshes[index] = Some(voxels);
	// 	Some(index)
    // }


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

    pub fn voxel(&self, chunk_coords: ChunkCoords, local_coords: LocalCoords) -> Option<&Voxel> {
        self.chunk(chunk_coords).map(|c| c.voxel(local_coords))
    }

    pub fn voxel_global(&self, coords: GlobalCoords) -> Option<&Voxel> {
        self.voxel(coords.into(), coords.into())
    }

    pub fn is_air_global(&self, coords: GlobalCoords) -> bool {
        let Some(voxel) = self.voxel(coords.into(), coords.into()) else {return false};
        voxel.id == 0
    }


    pub fn set(&mut self, global: GlobalCoords, id: u32, direction: Option<&Direction>) -> Option<u32> {
        let coords: ChunkCoords = global.into();
        let Some(chunk) = self.mut_chunk(coords) else {return None};

        let local: LocalCoords = global.into();
    
        let x_offset = (local.0 == (CHUNK_SIZE-1) as u8) as i32 - (local.0 == 0) as i32;
        let y_offset = (local.1 == (CHUNK_SIZE-1) as u8) as i32 - (local.1 == 0) as i32;
        let z_offset = (local.2 == (CHUNK_SIZE-1) as u8) as i32 - (local.2 == 0) as i32;
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


    pub fn light(&self, coords: GlobalCoords, channel: u8) -> u16 {
        let chunk = self.chunk(coords);

        if !self.is_in_area(coords.into()) || chunk.is_none() { return 0; }

        chunk.unwrap().lightmap.get(LocalCoords::from(coords).into(), channel)
    }


    pub fn is_in_area(&self, chunk_coords: ChunkCoords) -> bool {
        chunk_coords.0 >= 0 && chunk_coords.0 < self.width &&
        chunk_coords.1 >= 0 && chunk_coords.1 < self.height &&
        chunk_coords.2 >= 0 && chunk_coords.2 < self.depth
    }

    pub fn chunk<T: Into<ChunkCoords>>(&self, coords: T) -> Option<&Chunk> {
        let coords: ChunkCoords = coords.into();
        if !self.is_in_area(coords) { return None; }
        let index = coords.index(self.depth, self.width);
        let chunk = self.chunks.get(index);
        if let Some(chunk) = chunk { return chunk.as_ref() }
        None
    }

    pub fn mut_chunk<T: Into<ChunkCoords>>(&mut self, coords: T) -> Option<&mut Chunk> {
        let coords: ChunkCoords = coords.into();
        if !self.is_in_area(coords) { return None; }
        let index = coords.index(self.depth, self.width);
        let chunk = self.chunks.get_mut(index);
        if let Some(chunk) = chunk { return chunk.as_mut() }
        None
    }

    pub fn voxels_data<T: Into<ChunkCoords>>(&self, coords: T) -> Option<&HashMap<usize, VoxelData>> {
        self.chunk(coords).map(|c| c.voxels_data())
    }

    pub fn mut_voxels_data<T: Into<ChunkCoords>>(&mut self, coords: T) -> Option<&mut HashMap<usize, VoxelData>> {
        self.mut_chunk(coords).map(|c| c.mut_voxels_data())
    }


    pub fn add_multiblock_structure(&mut self, xyz: &GlobalCoords, width: i32, height: i32, depth: i32, id: u32, dir: &Direction) -> Option<Vec<GlobalCoords>> {
        let mut coords: Vec<GlobalCoords> = vec![];
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
        coords.push(*xyz);
        for (nx, nz, ny) in iproduct!(width_range, depth_range, height_range) {
            if nx == xyz.0 && ny == xyz.1 && nz == xyz.2 {continue};
            if !self.is_air_global((nx, ny, nz).into()) {return None};
            coords.push((nx, ny, nz).into());
        }

        
        let voxel_additional_data = Arc::new(VoxelAdditionalData::new_multiblock(id, dir, coords.clone()));
        coords.iter().enumerate().for_each(|(index, coord)| {
            let id = if index == 0 {id} else {1};
            self.set(*coord, id, None);
            let voxels_data = self.mut_voxels_data(*coord).unwrap();
            voxels_data.insert(LocalCoords::from(*coord).index(), VoxelData {
                id,
                global_coords: *coord,
                additionally: voxel_additional_data.clone()
            });
        });
        Some(coords)
    }


    pub fn remove_multiblock_structure(&mut self, global: GlobalCoords) -> Option<Vec<GlobalCoords>> {
        let Some(voxels_data) = self.voxels_data(global) else {return None};
        let Some(voxel_data) = voxels_data.get(&LocalCoords::from(global).index()) else {return None};
        let mut coords: Vec<GlobalCoords> = vec![];
        match &voxel_data.additionally.as_ref() {
            VoxelAdditionalData::Drill(drill) => {
                drill.lock().unwrap().structure_coordinates().iter().for_each(|coord| {
                    coords.push(*coord);
                });
            },
            VoxelAdditionalData::AssemblingMachine(asembler) => {
                asembler.lock().unwrap().structure_coordinates().iter().for_each(|coord| {
                    coords.push(*coord);
                });
            },
            _ => (),
        };
        coords.iter().for_each(|coord| {
            self.set(*coord, 0, None);
        });
        Some(coords)
    }

    pub fn get_sun(&self, coords: GlobalCoords) -> u16 {
        let local: LocalCoords = coords.into();
        self.chunk(coords)
            .map_or(0, |c| c.lightmap.get_sun(local.into()))
    }


    pub fn get_light(&self, coords: GlobalCoords) -> u16 {
        let local: LocalCoords = coords.into();
        self.chunk(coords)
            .map_or(0, |c| c.lightmap.get_light(local.into()))
    }
}
