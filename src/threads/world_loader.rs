use std::{thread::{self, JoinHandle}, sync::{Arc, Mutex}, time::{Duration, Instant}};

use itertools::iproduct;

use crate::{world::{World, chunk_coords::ChunkCoords, global_coords::GlobalCoords}, voxels::{chunks::WORLD_HEIGHT, chunk::CHUNK_SIZE}, unsafe_mutex::UnsafeMutex, save_load::WorldRegions, WORLD_EXIT};


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
                let mut new_chunks = World::new(1, WORLD_HEIGHT as i32, 1, ox, 0, oz);
                new_chunks.chunks.load_all(world_regions.clone());
                new_chunks.build_sky_light();
                let chunks = new_chunks.chunks;
                let cxz = (chunks.chunks[0].as_ref().unwrap().xyz.0, chunks.chunks[0].as_ref().unwrap().xyz.2);

                for chunk in chunks.chunks.into_iter() {
                    let Some(chunk) = chunk else {continue};
                    let xyz = chunk.xyz;
                    let index = ChunkCoords(ox, xyz.1, oz).chunk_index(&world.chunks);
                    // let index = chunk.xyz.chunk_index(&world.chunks);
                    if let Some(c) = world.chunks.chunks.get_mut(index) {
                        *c = Some(chunk);
                    }
                };

                let min_x = cxz.0*CHUNK_SIZE as i32-1;
                let max_x = (cxz.0+1)*CHUNK_SIZE as i32+1;

                let min_z = cxz.1*CHUNK_SIZE as i32-1;
                let max_z = (cxz.1+1)*CHUNK_SIZE as i32+1;
                for (gy, gz, gx) in iproduct!(0i32..((WORLD_HEIGHT*CHUNK_SIZE) as i32), min_z..=max_z, min_x..=max_x) {
                    if gx == min_x || gx == max_x || gz == max_z || gz == min_z {
                        world.add_rgbs(gx, gy, gz);
                    }
                }
                world.solve_rgbs();
            } else {
                drop(world);
                thread::sleep(Duration::from_millis(200));
            }
        }
    })
}