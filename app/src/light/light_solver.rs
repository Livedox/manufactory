use std::{cell::UnsafeCell, collections::VecDeque, i32, mem::MaybeUninit, ptr::null, sync::Arc};

use chrono::offset;
use itertools::Itertools;

use crate::{content::Content, coords::{chunk_coord::ChunkCoord, global_coord::GlobalCoord, local_coord::LocalCoord}, voxels::{chunk::{self, Chunk}, chunks::{Chunks, WORLD_BLOCK_HEIGHT}}};

use super::light::Light;

pub struct ChunkBuffer<const N: usize> {
    coords: [ChunkCoord; N],
    storage: [MaybeUninit<Arc<Chunk>>; N],
    ind: usize,
}

impl<const N: usize> ChunkBuffer<N> {
    const FAKE_COORD: ChunkCoord = ChunkCoord::new(i32::MAX, i32::MAX);
    const UNINIT: MaybeUninit<Arc<Chunk>> = MaybeUninit::uninit();
    pub const fn new() -> Self {
        Self {
            coords: [Self::FAKE_COORD; N],
            storage: [Self::UNINIT; N],
            ind: N-1
        }
    }

    pub fn with_chunks(coords: [Option<ChunkCoord>; N], chunks: [Option<Arc<Chunk>>; N]) -> Self {
        let mut s = Self::new();
        for (i, (cc, chunk)) in coords.into_iter().zip_eq(chunks.into_iter()).enumerate() {
            if let Some(chunk) = chunk {
                if let Some(cc) = cc {
                    if cc == Self::FAKE_COORD {panic!("special meaning")}
                    s.coords[i] = cc;
                    s.storage[i].write(chunk);
                }
            }
        }
        s
    }

    #[inline(always)]
    pub fn find(&self, cc: ChunkCoord) -> Option<&Arc<Chunk>> {
        self.coords.iter().position(|coord| *coord == cc)
            .map(|ind| unsafe {self.storage.get_unchecked(ind).assume_init_ref()})
    }


    #[inline(never)]
    pub fn push(&mut self, cc: ChunkCoord, chunk: Arc<Chunk>) -> &Arc<Chunk> {
        self.ind += 1;
        if self.ind > N-1 {self.ind = 1};
        unsafe {
            self.coords.swap(0, self.ind);
            self.storage.swap(0, self.ind);

            let coord = self.coords.get_unchecked_mut(0);
            let chunk_s = self.storage.get_unchecked_mut(0);
            if *coord != Self::FAKE_COORD {
                chunk_s.assume_init_drop();
            }
            *coord = cc;
            chunk_s.write(chunk)
        }
    }
}

#[derive(Debug)]
pub struct LightEntry {
    pub coords: GlobalCoord,
    pub light: Light,
}

impl Clone for LightEntry {
    #[inline]
    fn clone(&self) -> Self {
        Self { coords: self.coords, light: self.light.clone() }
    }
}

impl LightEntry {
    #[inline]
    pub const fn new(coords: GlobalCoord, light: Light) -> LightEntry {
        LightEntry { coords, light }
    }
}

const NEIGHBOURS: [GlobalCoord; 6] = [
    GlobalCoord::new(1,  0, 0),
    GlobalCoord::new(-2, 0, 0),
    GlobalCoord::new(1, 1, 0),
    GlobalCoord::new(0, -2, 0),
    GlobalCoord::new(0, 1, 1),
    GlobalCoord::new(0, 0, -2),
];

#[derive(Debug)]
pub struct LightSolver {
    add_queue: VecDeque::<LightEntry>,
    remove_queue: VecDeque::<LightEntry>,
}

impl LightSolver {
    pub fn new() -> Self {
        Self {
            add_queue: VecDeque::new(),
            remove_queue: VecDeque::new(),
        }
    }
    pub fn with_capacity(add_queue_cap: usize, remove_queue_cap: usize) -> LightSolver {
        LightSolver {
            add_queue: VecDeque::with_capacity(add_queue_cap),
            remove_queue: VecDeque::with_capacity(remove_queue_cap)
        }
    }

    #[inline]
    pub fn add_max_sun_to_solve(&mut self, gc: GlobalCoord) {
        self.add_queue.push_back(LightEntry::new(gc, Light::new(0, 0, 0, Light::MAX)));
    }

    pub fn add_with_emission_and_chunk(&mut self, chunk: &Arc<Chunk>, lc: LocalCoord, light: Light) {
        if light.all_le_one() {return};
        let entry = LightEntry::new(chunk.coord.to_global(lc), light.clone());
        chunk.modify(true);
        self.add_queue.push_back(entry);
    }

    pub fn add_with_chunk(&mut self, chunk: &Arc<Chunk>, lc: LocalCoord) {
        let Some(light) = chunk.light_map().get(lc) else {return};
        self.add_with_emission_and_chunk(chunk, lc, light.clone());
    }

    pub fn add_with_emission(&mut self, chunks: &Chunks, gc: GlobalCoord, light: Light) {
        if light.all_le_one() {return};
        if gc.y < 0 || gc.y >= WORLD_BLOCK_HEIGHT as i32 {return};
        let Some(chunk) = chunks.chunk(gc.into()) else {return};

        let entry = LightEntry::new(gc, light.clone());
        let local = LocalCoord::from(gc);
        chunk.light_map()[local].set_light(light.clone());
        chunk.modify(true);
        self.add_queue.push_back(entry);
    }

    pub fn add(&mut self, chunks: &Chunks, gc: GlobalCoord) {
        let light = chunks.get_light(gc);
        self.add_with_emission(chunks, gc, light);
    }


    pub fn remove_s(&mut self, chunks: &Chunks, gc: GlobalCoord) {
        if gc.y < 0 || gc.y >= WORLD_BLOCK_HEIGHT as i32 {return};
        let Some(chunk) = chunks.chunk(gc.into()) else {return};

        let index = LocalCoord::from(gc).index();
        let light = unsafe {chunk.lightmap.0.get_unchecked(index).clone()};
        unsafe {chunk.lightmap.0.get_unchecked(index).set_sun(0);};

        self.remove_queue.push_back(LightEntry::new(gc, light));
    }


    pub fn remove_rgb(&mut self, chunks: &Chunks, gc: GlobalCoord) {
        if gc.y < 0 || gc.y >= WORLD_BLOCK_HEIGHT as i32 {return};
        let Some(chunk) = chunks.chunk(gc.into()) else {return};

        let index = LocalCoord::from(gc).index();
        let light = unsafe {chunk.lightmap.0.get_unchecked(index).clone()};
        unsafe {chunk.lightmap.0.get_unchecked(index).set_rgb(0, 0, 0)};
        light.set_sun(0);
        self.remove_queue.push_back(LightEntry::new(gc, light));
    }

    pub fn remove_all(&mut self, chunks: &Chunks, gc: GlobalCoord) {
        if gc.y < 0 || gc.y >= WORLD_BLOCK_HEIGHT as i32 {return};
        let Some(chunk) = chunks.chunk(gc.into()) else {return};

        let index = LocalCoord::from(gc).index();
        let light = unsafe {chunk.lightmap.0.get_unchecked(index).clone()};
        unsafe {chunk.lightmap.0.get_unchecked(index).set_light(Light::default())};

        self.remove_queue.push_back(LightEntry::new(gc, light));
    }


    pub fn solve(&mut self, chunks: &Chunks, content: &Content) {
        self.solve_remove_queue(chunks);
        self.solve_add_queue(chunks, content);
    }

    fn solve_remove_queue(&mut self, chunks: &Chunks) {
        while let Some(mut entry) = self.remove_queue.pop_front() {
            let pvsub = entry.light.clone().saturated_sub_one();

            for offsets in NEIGHBOURS.iter() {
                entry.coords += offsets;
                if entry.coords.y < 0 || entry.coords.y >= WORLD_BLOCK_HEIGHT as i32 {continue};

                let Some(chunk) = chunks.chunk(entry.coords.into()) else {continue};
                let index = LocalCoord::from(entry.coords).index();

                entry.light = chunk.lightmap.0[index].clone();

                let new = entry.light.zero_if_equal_elements(pvsub.clone());

                if entry.light.to_number() != 0 && entry.light != new {
                    self.remove_queue.push_back(entry.clone());
                    chunk.lightmap.0[index].set_light(new);
                    chunk.modify(true);
                } else if entry.light.has_greater_element(pvsub.clone()) {
                    self.add_queue.push_back(entry.clone());
                }
            }
        }
    }

    #[inline(never)]
    fn solve_add_queue(&mut self, chunks: &Chunks, content: &Content) {
        // Optimize this function in the region of 50%-70%,
        // because the light is mostly in one chunk,
        // and access to the hash table is long.
        let mut buffer = ChunkBuffer::<3>::new();
        // let mut chunk_buffer: (ChunkCoord, MaybeUninit<Arc<Chunk>>) =
        //     (ChunkCoord::new(i32::MAX, i32::MAX), MaybeUninit::uninit());

        while let Some(mut entry) = self.add_queue.pop_front() {
            if entry.light.all_le_one() {continue};
            entry.light = entry.light.saturated_sub_one();

            for offset in NEIGHBOURS.iter() {
                entry.coords += offset;
                if entry.coords.y < 0 || entry.coords.y >= WORLD_BLOCK_HEIGHT as i32 {continue};

                let cc: ChunkCoord = entry.coords.into();
                let chunk = if let Some(chunk) = buffer.find(cc) {
                    chunk
                } else {
                    let Some(chunk) = chunks.chunk(cc) else {continue};
                    buffer.push(cc, chunk)
                };
                // let chunk = if cc == chunk_buffer.0 {
                //     unsafe {chunk_buffer.1.assume_init_ref()}
                // } else {
                //     let Some(chunk) = chunks.chunk(cc) else {continue};
                //     chunk_buffer.0 = cc;
                //     chunk_buffer.1.write(chunk);
                //     unsafe {chunk_buffer.1.assume_init_ref()}
                // };

                let index = LocalCoord::from(entry.coords).index();
                let light = unsafe {chunk.light_map().0.get_unchecked(index)};
                let id = unsafe {chunk.voxels().0.get_unchecked(index).id()} as usize;
                let max = entry.light.max_element_wise(light.clone());

                if content.blocks[id].is_light_passing() && max != *light {
                    unsafe {chunk.lightmap.0.get_unchecked(index).set_light(max)};
                    chunk.modify(true);
                    self.add_queue.push_back(entry.clone());
                }
            }
        }
    }
}