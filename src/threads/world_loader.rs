use std::{thread::{self, JoinHandle}, sync::{Arc, Mutex, atomic::{Ordering, AtomicBool}, RwLock}, time::{Duration, Instant}};

use itertools::iproduct;

use crate::{world::{World, chunk_coords::ChunkCoords, global_coords::GlobalCoords}, voxels::{chunks::WORLD_HEIGHT, chunk::{CHUNK_SIZE, Chunk}}, unsafe_mutex::UnsafeMutex, save_load::{WorldRegions, EncodedChunk}, WORLD_EXIT, bytes::BytesCoder};


pub fn spawn(
    world: Arc<World>,
    world_regions: Arc<UnsafeMutex<WorldRegions>>,
    exit: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            if exit.load(Ordering::Relaxed) {break};
            if let Some((cx, cz)) = world.chunks.find_unloaded() {
                let mut regions = unsafe {world_regions.lock_unsafe()}.unwrap();
                println!("2");
                world.load_column_of_chunks(&mut regions, cx, cz);
                println!("Load");
            } else {
                thread::sleep(Duration::from_millis(200));
            }
        }
    })
}