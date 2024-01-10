use std::{collections::VecDeque, sync::{Arc, Mutex, mpsc::{Sender, Receiver}}, cell::UnsafeCell};

use crate::{voxels::{chunks::Chunks, block::{light_permeability::LightPermeability, blocks::BLOCKS}}, world::{global_coords::GlobalCoords, local_coords::LocalCoords}};
use crossbeam_deque::{Worker};
use crossbeam_queue::SegQueue;
use std::sync::mpsc::channel;

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

#[derive(Debug)]
pub struct LightSolver {
    add_queue: SegQueue<LightEntry>,
    remove_queue: SegQueue<LightEntry>,
    channel: u8,
}


impl LightSolver {
    pub fn new(channel: u8) -> LightSolver {
        LightSolver {
            add_queue: SegQueue::new(),
            remove_queue: SegQueue::new(),
            channel
        }
    }


    pub fn add_with_emission(&self, chunks: &Chunks, x: i32, y: i32, z: i32, emission: u8) {
        if emission <= 1 { return; }

        if let Some(chunk) = chunks.chunk(GlobalCoords(x, y, z)) {
            let entry = LightEntry::new(x, y, z, emission);

            chunk.lightmap.set(LocalCoords::from(GlobalCoords(x, y, z)).into(), emission as u16, self.channel);
            chunk.modify(true);
            
            self.add_queue.push(entry);
        }
    }

    pub fn add(&self, chunks: &Chunks, x: i32, y: i32, z: i32) {
        let emission = chunks.light((x,y,z).into(), self.channel) as u8;
        self.add_with_emission(chunks, x, y, z, emission);
    }


    pub fn remove(&self, chunks: &Chunks, x: i32, y: i32, z: i32) {
        if let Some(chunk) = chunks.chunk(GlobalCoords(x, y, z)) {
            let local = LocalCoords::from(GlobalCoords(x, y, z)).into();
            
            let light = chunk.lightmap.get(local, self.channel) as u8;
            chunk.lightmap.set(local, 0, self.channel);

            self.remove_queue.push(LightEntry::new(x, y, z, light));
        }
    }


    pub fn solve(&self, chunks: &Chunks) {
        self.solve_remove_queue(chunks);
        self.solve_add_queue(chunks);
    }


    #[inline(never)]
    fn solve_remove_queue(&self, chunks: &Chunks) {
        loop {
            let Some(entry) = self.remove_queue.pop() else {break};
            for i in 0..6 {
                let x: i32 = entry.x + COORDS[i*3];
                let y: i32 = entry.y + COORDS[i*3+1];
                let z: i32 = entry.z + COORDS[i*3+2];
                let global = GlobalCoords(x, y, z);
                let light = chunks.light(global, self.channel) as u8;
                let chunk = chunks.chunk(global);
                let Some(chunk) = chunk else {continue};
                let nentry = LightEntry::new(x, y, z, light); 
                if light != 0 && entry.light != 0 && light == entry.light-1 {
                    self.remove_queue.push(nentry);
                    chunk.lightmap.set(LocalCoords::from(global).into(), 0, self.channel);
                    chunk.modify(true);
                } else if light >= entry.light {
                    self.add_queue.push(nentry);
                }
            }
        }
    }

    #[inline(never)]
    fn solve_add_queue(&self, chunks: &Chunks) {
        loop {
            let Some(entry) = self.add_queue.pop() else {break};
            if entry.light <= 1 { continue; }
            let entry_id = chunks
                .voxel_global((entry.x, entry.y, entry.z).into())
                .map_or(0, |v| v.id);
            for (i, side) in PERMEABILITYS.iter().enumerate() {
                let x = entry.x + COORDS[i*3];
                let y = entry.y + COORDS[i*3+1];
                let z = entry.z + COORDS[i*3+2];
                let global = GlobalCoords(x, y, z);
                let Some(chunk) = chunks.chunk(global) else {continue};
                let local = LocalCoords::from(global).into();
                let light = chunk.get_light(local).get(self.channel);
                let id = chunk.voxel(local).id;
                if Self::check_light_passing(entry_id, id, side) && (light+2) <= entry.light as u16 {
                    self.add_queue.push(LightEntry::new(x, y, z, entry.light-1));
                    chunk.lightmap.set(local.into(), (entry.light-1).into(), self.channel);
                    chunk.modify(true);
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