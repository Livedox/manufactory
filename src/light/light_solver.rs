use std::collections::VecDeque;

use crate::voxels::{chunks::Chunks, block::{light_permeability::LightPermeability, blocks::BLOCKS}};


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
    channel: u8,
}


impl LightSolver {
    pub fn new(channel: u8) -> LightSolver {
        LightSolver {
            add_queue: VecDeque::new(),
            remove_queue: VecDeque::new(),
            channel
        }
    }


    pub fn add_with_emission(&mut self, chunks: &mut Chunks, x: i32, y: i32, z: i32, emission: u8) {
        if emission <= 1 { return; }

        if let Some(chunk) = chunks.mut_chunk_by_global(x, y, z) {
            let entry = LightEntry::new(x, y, z, emission);

            chunk.lightmap.set(Chunks::u8_local_coords(x, y, z), emission as u16, self.channel);
            chunk.modified = true;
            
            self.add_queue.push_back(entry);
        }
    }

    pub fn add(&mut self, chunks: &mut Chunks, x: i32, y: i32, z: i32) {
        let emission = chunks.light(x,y,z, self.channel) as u8;
        self.add_with_emission(chunks, x, y, z, emission);
    }


    pub fn remove(&mut self, chunks: &mut Chunks, x: i32, y: i32, z: i32) {
        if let Some(chunk) = chunks.mut_chunk_by_global(x, y, z) {
            let local = Chunks::u8_local_coords(x, y, z);
            
            let light = chunk.lightmap.get(local, self.channel) as u8;
            chunk.lightmap.set(local, 0, self.channel);

            self.remove_queue.push_back(LightEntry::new(x, y, z, light));
        }
    }


    pub fn solve(&mut self, chunks: &mut Chunks) {
        self.solve_remove_queue(chunks);
        self.solve_add_queue(chunks);
    }


    fn solve_remove_queue(&mut self, chunks: &mut Chunks) {
        while let Some(entry) = self.remove_queue.pop_front() {
            for i in 0..6 {
                let x: i32 = entry.x + COORDS[i*3];
                let y: i32 = entry.y + COORDS[i*3+1];
                let z: i32 = entry.z + COORDS[i*3+2];
                let light = chunks.light(x, y, z, self.channel) as u8;
                let chunk = chunks.mut_chunk_by_global(x, y, z);
                let Some(chunk) = chunk else {continue};

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


    fn solve_add_queue(&mut self, chunks: &mut Chunks) {
        while let Some(entry) = self.add_queue.pop_front() {
            if entry.light <= 1 { continue; }

            let entry_id = chunks
                .voxel_global(entry.x, entry.y, entry.z)
                .map_or(0, |v| v.id);

            for (i, side) in PERMEABILITYS.iter().enumerate() {
                let x = entry.x + COORDS[i*3];
                let y = entry.y + COORDS[i*3+1];
                let z = entry.z + COORDS[i*3+2];
                let light = chunks.light(x, y, z, self.channel);
                let id = chunks.voxel_global(x, y, z).map_or(0, |v| v.id);
                let Some(chunk) = chunks.mut_chunk_by_global(x, y, z) else {continue};

                
                if Self::check_light_passing(entry_id, id, side) && (light+2) <= entry.light as u16 {
                    self.add_queue.push_back(LightEntry::new(x, y, z, entry.light-1));
                    chunk.lightmap.set(Chunks::u8_local_coords(x, y, z), (entry.light-1).into(), self.channel);
                    chunk.modified = true;
                }
            }
        }
    }


    fn check_light_passing(entry_id: u32, id: u32, side: &LightPermeability) -> bool {
        let entry_id = entry_id as usize;
        let id = id as usize;
        BLOCKS()[id].light_permeability().check_permeability(&BLOCKS()[entry_id].light_permeability(), side)
        ||
        (
            BLOCKS()[entry_id].emission().iter().any(|e| *e > 0) &&
            BLOCKS()[id].light_permeability().check_permeability(&LightPermeability::ALL, side)
        )
    }
}
