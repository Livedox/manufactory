use std::{thread::{self, JoinHandle}, sync::{Arc, Mutex}, time::{Duration, Instant}, cell::{UnsafeCell}};

use itertools::iproduct;
use wgpu::Instance;

use crate::{world::{World, chunk_coords::ChunkCoords, global_coords::GlobalCoords}, voxels::{chunks::WORLD_HEIGHT, chunk::CHUNK_SIZE}, unsafe_mutex::UnsafeMutex, save_load::WorldRegions};


pub fn spawn(
    world: Arc<UnsafeMutex<World>>,
    world_regions: Arc<UnsafeMutex<WorldRegions>>,
    player_coords: Arc<Mutex<(f32, f32, f32)>>
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let mut world = world.lock_unsafe(false).unwrap();
            let p_coords = player_coords.lock().unwrap().clone();
            let p_coords: ChunkCoords = GlobalCoords::from(p_coords).into();
            let cxz: Option<(i32, i32)> = world.chunks
                .find_pos_stable_xz(&|c| c.is_none())
                .map(|pos| (pos.0, pos.2));
            if let Some((ox, oz)) = cxz {
                let mut new_chunks = World::new(1, WORLD_HEIGHT as i32, 1, ox, 0, oz);
                new_chunks.chunks.load_all(world_regions.clone());
                new_chunks.build_sky_light();
                let chunks = new_chunks.chunks;
                let cxz = (chunks.chunks[0].as_ref().unwrap().xyz.0, chunks.chunks[0].as_ref().unwrap().xyz.2);

                for chunk in chunks.chunks.into_iter() {
                    let Some(chunk) = chunk.map(|c| c) else {continue};
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