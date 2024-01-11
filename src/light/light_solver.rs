use std::{collections::VecDeque, sync::{Arc, Mutex, mpsc::{Sender, Receiver}, atomic::{AtomicUsize, Ordering}}, cell::UnsafeCell};

use crate::{voxels::{chunks::Chunks, block::{light_permeability::LightPermeability, blocks::BLOCKS}, chunk::Chunk}, world::{global_coords::GlobalCoords, local_coords::LocalCoords, chunk_coords::ChunkCoords}};
use crossbeam_deque::{Worker};
use crossbeam_queue::{SegQueue, ArrayQueue};
use std::sync::mpsc::channel;
pub static MAX: AtomicUsize = AtomicUsize::new(0);

const COORDS: [(i32, i32, i32); 6] = [
    (1,  0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, 1),
    (0, 0, -1),
];


#[derive(Debug)]
struct LightEntry {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub light: u8,
}


impl LightEntry {
    #[inline]
    pub fn new(x: i32, y: i32, z: i32, light: u8) -> LightEntry {
        LightEntry { x, y, z, light }
    }
}

#[derive(Debug)]
pub struct LightSolver {
    add_queue: ArrayQueue<LightEntry>,
    remove_queue: ArrayQueue<LightEntry>,
    channel: usize,
}

unsafe impl Send for LightSolver {}
unsafe impl Sync for LightSolver {}

impl LightSolver {
    pub fn new(channel: usize, add_queue_cap: usize, remove_queue_cap: usize) -> LightSolver {
        LightSolver {
            add_queue: ArrayQueue::new(262_144),
            remove_queue: ArrayQueue::new(131_072),
            channel
        }
    }


    pub fn add_with_emission(&self, chunks: &Chunks, x: i32, y: i32, z: i32, emission: u8) {
        if emission <= 1 { return; }
        let Some(chunk) = chunks.chunk_ptr(GlobalCoords(x, y, z)) else {return};
        let chunk = unsafe {&*chunk};
        let entry = LightEntry::new(x, y, z, emission);

        chunk.lightmap.set(LocalCoords::from(GlobalCoords(x, y, z)).into(), emission, self.channel);
        chunk.modify(true);
        
        let _ = self.add_queue.push(entry);
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

            let _ = self.remove_queue.push(LightEntry::new(x, y, z, light));
        }
    }


    pub fn solve(&self, chunks: &Chunks) {
        self.solve_remove_queue(chunks);
        self.solve_add_queue(chunks);
    }


    #[inline(never)]
    fn solve_remove_queue(&self, chunks: &Chunks) {
        while let Some(entry) = self.remove_queue.pop() {
            for (nx, ny, nz) in COORDS.into_iter() {
                let x = entry.x + nx;
                let y = entry.y + ny;
                let z = entry.z + nz;
                let global = GlobalCoords(x, y, z);
                let Some(chunk) = chunks.chunk_ptr(global) else {continue};
                let chunk = unsafe {&*chunk};
                let index = LocalCoords::from(global).index();
                let light = unsafe {chunk.lightmap.map.get_unchecked(index).get_unchecked_channel(self.channel)};
                let nentry = LightEntry::new(x, y, z, light); 
                if light != 0 && entry.light != 0 && light == entry.light-1 {
                    let _ = self.remove_queue.push(nentry);
                    unsafe {chunk.lightmap.map.get_unchecked(index).set_unchecked_channel(0, self.channel)};
                    chunk.modify(true);
                } else if light >= entry.light {
                    let _ = self.add_queue.push(nentry);
                }
            }
        }
    }

    #[inline(never)]
    fn solve_add_queue(&self, chunks: &Chunks) {
        while let Some(entry) = self.add_queue.pop() {
            if entry.light <= 1 { continue; }
            for (nx, ny, nz) in COORDS.into_iter() {
                let x = entry.x + nx;
                let y = entry.y + ny;
                let z = entry.z + nz;
                let global = GlobalCoords(x, y, z);
                let Some(chunk) = chunks.chunk_ptr(global) else {continue};
                let chunk = unsafe {&*chunk};
                let index = LocalCoords::from(global).index();
                let light = unsafe {chunk.lightmap.map.get_unchecked(index).get_unchecked_channel(self.channel)};
                let id = unsafe {chunk.voxels.get_unchecked(index).id()};
                if BLOCKS()[id as usize].is_light_passing() && (light+2) <= entry.light {
                    let _ = self.add_queue.push(LightEntry::new(x, y, z, entry.light-1));
                    unsafe {chunk.lightmap.map.get_unchecked(index).set_unchecked_channel(entry.light-1, self.channel)};
                    chunk.modify(true);
                }
            }
        }
    }
}