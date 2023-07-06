use std::{collections::VecDeque, sync::Arc, cell::RefCell};

use crate::voxels::{chunks::Chunks, chunk::{CHUNK_SIZE}, block::{LightPermeability, BLOCKS}};


const PERMEABILITYS: [LightPermeability; 6] = [
    LightPermeability::RIGHT,
    LightPermeability::LEFT,
    LightPermeability::UP,
    LightPermeability::DOWN,
    LightPermeability::FRONT,
    LightPermeability::BACK
];


const COORDS: [i32; 18] = [
    1,  0, 0,
    -1, 0, 0,
    0, 1, 0,
    0, -1, 0,
    0, 0, 1,
    0, 0, -1,
];


#[derive(Debug)]
struct LightEntry {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub light: u8,
}


impl LightEntry {
    pub fn new(x: i32, y: i32, z: i32, light: u8) -> LightEntry {
        LightEntry { x, y, z, light }
    }
}


pub struct LightSolver {
    add_queue: VecDeque<LightEntry>,
    remove_queue: VecDeque<LightEntry>,
    chunks: Arc<RefCell<Chunks>>,
    channel: u8,
}


impl LightSolver {
    pub fn new(chunks: Arc<RefCell<Chunks>>, channel: u8) -> LightSolver {
        LightSolver {
            add_queue: VecDeque::new(),
            remove_queue: VecDeque::new(),
            chunks,
            channel
        }
    }


    pub fn add_with_emission(&mut self, x: i32, y: i32, z: i32, emission: u8) {
        if emission <= 1 { return; }

        let mut chunks = self.chunks.borrow_mut();
        if let Some(chunk) = chunks.mut_chunk_by_global(x, y, z) {
            let entry = LightEntry::new(x, y, z, emission);

            chunk.lightmap.set(Chunks::u8_local_coords(x, y, z), emission as u16, self.channel);
            chunk.modified = true;
            
            self.add_queue.push_back(entry);
        }
    }

    pub fn add(&mut self, x: i32, y: i32, z: i32) {
        let emission = self.chunks.borrow().light(x,y,z, self.channel) as u8;
        self.add_with_emission(x, y, z, emission);
    }


    pub fn remove(&mut self, x: i32, y: i32, z: i32) {
        let mut chunks = self.chunks.borrow_mut();
        if let Some(chunk) = chunks.mut_chunk_by_global(x, y, z) {
            let local = Chunks::u8_local_coords(x, y, z);
            
            let light = chunk.lightmap.get(local, self.channel) as u8;
            chunk.lightmap.set(local, 0, self.channel);

            self.remove_queue.push_back(LightEntry::new(x, y, z, light));
        }
    }


    pub fn solve(&mut self) {
        while !self.remove_queue.is_empty() {
            let entry = self.remove_queue.pop_front();

            if let Some(entry) = entry {
                for i in 0..6 {
                    let x: i32 = entry.x + COORDS[i*3];
                    let y: i32 = entry.y + COORDS[i*3+1];
                    let z: i32 = entry.z + COORDS[i*3+2];
                    let mut chunks = self.chunks.borrow_mut();
                    let light = chunks.light(x, y, z, self.channel) as u8;
                    let chunk = chunks.mut_chunk_by_global(x, y, z);
                    
                    if let Some(chunk) = chunk {
                        let nentry = LightEntry::new(x, y, z, light); 
                        if light != 0 && entry.light != 0 && light == entry.light-1 {
                            self.remove_queue.push_back(nentry);
                            chunk.lightmap.set(Chunks::u8_local_coords(x, y, z), 0, self.channel);
                            chunk.modified = true;
                        } else if light >= entry.light {
                            self.add_queue.push_back(nentry);
                        }
                    }
                }
            }
        }


        while !self.add_queue.is_empty() {
            let entry = self.add_queue.pop_front();

            if let Some(entry) = entry {
                if entry.light <= 1 { continue; }
                let mut chunks = self.chunks.borrow_mut();
                let entry_voxel_id = if let Some(voxel) = chunks.voxel_global(entry.x, entry.y, entry.z) {voxel.id} else {0};
                for (i, side) in PERMEABILITYS.iter().enumerate() {
                    let x = entry.x + COORDS[i*3];
                    let y = entry.y + COORDS[i*3+1];
                    let z = entry.z + COORDS[i*3+2];
                    
                    let light = chunks.light(x, y, z, self.channel);
                    let voxel = chunks.voxel_global(x, y, z);

                    let id = if let Some(voxel) = voxel {voxel.id} else {0};

                    let chunk = chunks.mut_chunk_by_global(x, y, z);

                    if let Some(chunk) = chunk {
                        let is = BLOCKS[id as usize].light_permeability.check_permeability(&BLOCKS[entry_voxel_id as usize].light_permeability, side)
                        || ((BLOCKS[entry_voxel_id as usize].emission[0] > 0
                        || BLOCKS[entry_voxel_id as usize].emission[1] > 0
                        || BLOCKS[entry_voxel_id as usize].emission[2] > 0) && BLOCKS[id as usize].light_permeability.check_permeability(&LightPermeability::ALL, side));
                        if is && (light+2) < entry.light as u16 {
                            self.add_queue.push_back(LightEntry::new(x, y, z, entry.light-1));
                            chunk.lightmap.set(Chunks::u8_local_coords(x, y, z), (entry.light-1).into(), self.channel);
                            chunk.modified = true;
                        }
                    }
                }
            }
        }
    }
}
