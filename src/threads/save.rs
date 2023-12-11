use std::{thread::{self, JoinHandle}, sync::{Arc, Mutex}, time::{Duration, Instant}, cell::{UnsafeCell}};

use itertools::iproduct;
use wgpu::Instance;

use crate::{world::{World, chunk_coords::ChunkCoords, global_coords::GlobalCoords}, voxels::{chunks::WORLD_HEIGHT, chunk::CHUNK_SIZE}, unsafe_mutex::UnsafeMutex, save_load::WorldRegions};


pub fn spawn(
    world: Arc<UnsafeMutex<World>>,
    world_regions: Arc<UnsafeMutex<WorldRegions>>
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
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
            drop(world);
            drop(world_regions);
            thread::sleep(Duration::from_secs(4));
        }
    })
}