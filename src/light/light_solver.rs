use std::{collections::VecDeque, sync::{Arc, Mutex, mpsc::{Sender, Receiver}, atomic::{AtomicUsize, Ordering}}, cell::UnsafeCell, ptr::null};

use crate::{voxels::{chunks::Chunks, block::{light_permeability::LightPermeability, blocks::BLOCKS}, chunk::Chunk}, world::{global_coords::GlobalCoords, local_coords::LocalCoords, chunk_coords::ChunkCoords}};

const COORDS: [(i32, i32, i32); 6] = [
    (1,  0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, 1),
    (0, 0, -1),
];

/// It's very unsafe, but very fast.
/// May issue STATUS_HEAP_CORRUPTION during relocation.
#[derive(Debug)]
pub struct LightQueue(UnsafeCell<VecDeque<LightEntry>>);

impl LightQueue {
    #[inline(always)]
    /// Safety
    /// If in any way the amount of light exceeds the capacity, we are fucked.
    pub fn new(capacity: usize) -> LightQueue {
        Self(UnsafeCell::new(VecDeque::<LightEntry>::with_capacity(capacity)))
    }

    #[inline(always)]
    pub fn push(&self, light: LightEntry) {
        unsafe {&mut *self.0.get()}.push_back(light);
    }

    #[inline(always)]
    pub fn pop(&self) -> Option<LightEntry> {
        unsafe {&mut *self.0.get()}.pop_front()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LightEntry {
    pub coords: GlobalCoords,
    pub light: u8,
}

impl LightEntry {
    #[inline]
    pub fn new(coords: GlobalCoords, light: u8) -> LightEntry {
        LightEntry { coords, light }
    }
}

pub struct ChunkBuffer(usize, *const Chunk);
impl ChunkBuffer {
    #[inline]
    pub fn new() -> Self {Self::default()}

    #[inline]
    pub unsafe fn get_or_init(&mut self, chunks: &Chunks, coords: GlobalCoords) -> Option<&Chunk> {
        let coords: ChunkCoords = coords.into();
        let index = coords.nindex(chunks.width, chunks.depth, chunks.ox(), chunks.oz());
        if self.0 == index {
            Some(unsafe {&*self.1})
        } else {
            if !chunks.is_in_area(coords) {return None};
            let Some(chunk) = (unsafe {chunks.get_unchecked_chunk(index)}) else {return None};
            self.0 = index;
            self.1 = chunk;
            Some(unsafe {&*self.1})
        }
    }
}

impl Default for ChunkBuffer {
    #[inline]
    fn default() -> Self {Self(usize::MAX, null())}
}

const NEIGHBOURS: [GlobalCoords; 6] = [
    GlobalCoords(1,  0, 0),
    GlobalCoords(-2, 0, 0),
    GlobalCoords(1, 1, 0),
    GlobalCoords(0, -2, 0),
    GlobalCoords(0, 1, 1),
    GlobalCoords(0, 0, -2),
];

#[derive(Debug)]
pub struct LightSolver {
    add_queue: LightQueue,
    remove_queue: LightQueue,
    channel: usize,
}

unsafe impl Send for LightSolver {}
unsafe impl Sync for LightSolver {}

impl LightSolver {
    /// Safety
    /// If in any way the amount of light exceeds the capacity, we are fucked.
    pub fn new(channel: usize, add_queue_cap: usize, remove_queue_cap: usize) -> LightSolver {
        LightSolver {
            add_queue: LightQueue::new(add_queue_cap),
            remove_queue: LightQueue::new(remove_queue_cap),
            channel
        }
    }


    pub fn add_with_emission(&self, chunks: &Chunks, x: i32, y: i32, z: i32, emission: u8) {
        if emission <= 1 {return};
        
        let global = GlobalCoords(x, y, z);
        let Some(chunk) = chunks.chunk_ptr(global).map(|c| unsafe {&*c}) else {return};

        let entry = LightEntry::new(global, emission);
        let local = LocalCoords::from(global).into();
        chunk.lightmap.get(local).set(emission, self.channel);
        chunk.modify(true);
        
        self.add_queue.push(entry);
    }

    pub fn add(&self, chunks: &Chunks, x: i32, y: i32, z: i32) {
        let emission = chunks.light((x,y,z).into(), self.channel) as u8;
        self.add_with_emission(chunks, x, y, z, emission);
    }


    pub fn remove(&self, chunks: &Chunks, x: i32, y: i32, z: i32) {
        let global = GlobalCoords(x, y, z);
        let Some(chunk) = chunks.chunk_ptr(global).map(|c| unsafe {&*c}) else {return};

        let index = LocalCoords::from(global).index();

        let light = unsafe {chunk.lightmap.0.get_unchecked(index).get_unchecked_channel(self.channel)};
        unsafe {chunk.lightmap.0.get_unchecked(index).set_unchecked_channel(0, self.channel)};

        self.remove_queue.push(LightEntry::new(global, light));
    }


    pub fn solve(&self, chunks: &Chunks) {
        self.solve_remove_queue(chunks);
        self.solve_add_queue(chunks);
    }

    fn solve_remove_queue(&self, chunks: &Chunks) {
        let mut chunk_buffer = ChunkBuffer::new();
        while let Some(mut entry) = self.remove_queue.pop() {
            let entry_prev_light = entry.light;
            for offsets in NEIGHBOURS.iter() {
                entry.coords += offsets;

                let Some(chunk) = (unsafe {chunk_buffer.get_or_init(chunks, entry.coords)}) else {continue};
                let index = LocalCoords::from(entry.coords).index();

                entry.light = unsafe {chunk.lightmap.0.get_unchecked(index)
                    .get_unchecked_channel(self.channel)};

                if entry.light != 0 && entry_prev_light != 0 && entry.light == entry_prev_light - 1 {
                    self.remove_queue.push(entry);
                    unsafe {chunk.lightmap.0.get_unchecked(index)
                        .set_unchecked_channel(0, self.channel)};
                    chunk.modify(true);
                } else if entry.light >= entry_prev_light {
                    self.add_queue.push(entry);
                }
            }
        }
    }

    fn solve_add_queue(&self, chunks: &Chunks) {
        let mut chunk_buffer = ChunkBuffer::new();
        while let Some(mut entry) = self.add_queue.pop() {
            if entry.light <= 1 { continue; }
            let prev_light = entry.light;
            entry.light -= 1;
            for offsets in NEIGHBOURS.iter() {
                entry.coords += offsets;

                let Some(chunk) = (unsafe {chunk_buffer.get_or_init(chunks, entry.coords)}) else {continue};
                let index = LocalCoords::from(entry.coords).index();

                let light = unsafe {chunk.lightmap.0.get_unchecked(index)
                    .get_unchecked_channel(self.channel)};
                let id = unsafe {chunk.voxels.get_unchecked(index).id()};

                if BLOCKS()[id as usize].is_light_passing() && (light+2) <= prev_light {
                    self.add_queue.push(entry);
                    unsafe {chunk.lightmap.0.get_unchecked(index)
                        .set_unchecked_channel(entry.light, self.channel)};
                    chunk.modify(true);
                }
            }
        }
    }
}