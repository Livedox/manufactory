use std::{thread::{self, JoinHandle}, sync::{Arc, Mutex}, time::{Duration, Instant}};

use itertools::iproduct;

use crate::{world::{World, chunk_coords::ChunkCoords, global_coords::GlobalCoords}, voxels::{chunks::WORLD_HEIGHT, chunk::{CHUNK_SIZE, Chunk}}, unsafe_mutex::UnsafeMutex, save_load::{WorldRegions, EncodedChunk}, WORLD_EXIT, bytes::BytesCoder};


pub fn spawn(
    world: Arc<UnsafeMutex<World>>,
    world_regions: Arc<UnsafeMutex<WorldRegions>>
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            if unsafe { WORLD_EXIT } {break};
            let mut world = unsafe {world.lock_unsafe()}.unwrap();
            let cxz: Option<(i32, i32)> = world.chunks.find_unloaded();
            if let Some((ox, oz)) = cxz {
                for cy in (0..WORLD_HEIGHT as i32).rev() {
                    let chunk = match unsafe {world_regions.lock_unsafe()}.unwrap().chunk((ox, cy, oz).into()) {
                        EncodedChunk::None => Chunk::new(ox, cy, oz),
                        EncodedChunk::Some(b) => Chunk::decode_bytes(b),
                    };
                    let index = chunk.xyz.chunk_index(&world.chunks);
                    world.chunks.chunks[index] = Some(Box::new(chunk));
                    world.build_chunk(ox, cy, oz);
                }
                world.solve_rgbs();
            } else {
                drop(world);
                thread::sleep(Duration::from_millis(200));
            }
        }
    })
}