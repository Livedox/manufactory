use std::{thread::{self, JoinHandle}, sync::{Arc, Mutex, Condvar}, time::{Duration, Instant}, cell::{UnsafeCell}};

use itertools::iproduct;
use wgpu::Instance;

use crate::{world::{World, chunk_coords::ChunkCoords, global_coords::GlobalCoords}, voxels::{chunks::WORLD_HEIGHT, chunk::CHUNK_SIZE}, unsafe_mutex::UnsafeMutex, save_load::WorldRegions};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SaveState {
    Unsaved,
    Saved,
    WorldExit,
}

pub fn spawn(
    world: Arc<UnsafeMutex<World>>,
    world_regions: Arc<UnsafeMutex<WorldRegions>>,
    save_condvar: Arc<(Mutex<SaveState>, Condvar)>
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let (lock, cvar) = &*save_condvar;
            let (mut save_state, _) = cvar.wait_timeout(lock.lock().unwrap(), Duration::new(60, 0)).unwrap();

            
            let mut world = world.lock_unsafe(false).unwrap();
            let mut world_regions = world_regions.lock_unsafe(false).unwrap();

            let mut chunks_awaiting_deletion = world.chunks.chunks_awaiting_deletion.lock().unwrap();
            chunks_awaiting_deletion.iter().for_each(|chunk| {
                world_regions.save_chunk(chunk);
            });
            chunks_awaiting_deletion.clear();
            drop(chunks_awaiting_deletion);

            world.chunks.chunks.iter_mut().for_each(|chunk| {
                let Some(chunk) = chunk else {return};
                if !chunk.unsaved {return};
                world_regions.save_chunk(chunk);
                chunk.unsaved = false;
            });
            world_regions.save_all_regions();


            if *save_state == SaveState::WorldExit {break};
            *save_state = SaveState::Saved;
        }
    })
}