use std::{thread::{self, JoinHandle}, sync::{Arc, Mutex}, time::Duration, cell::{UnsafeCell}};

use itertools::iproduct;

use crate::{world::{World, chunk_coords::ChunkCoords, global_coords::GlobalCoords, SyncUnsafeWorldCell}, voxels::{chunks::WORLD_HEIGHT, chunk::CHUNK_SIZE}};


pub fn spawn(
    world: Arc<SyncUnsafeWorldCell>,
    player_coords: Arc<Mutex<(f32, f32, f32)>>
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let world = world.get_mut();
            let p_coords = player_coords.lock().unwrap().clone();
            let p_coords: ChunkCoords = GlobalCoords::from(p_coords).into();
            let cxz: Option<(i32, i32)> = world.chunks
                .find_pos_stable_xz(&|c| c.is_none())
                .map(|pos| (pos.0, pos.2));
            if world.chunks.is_translate {continue};
            if let Some((ox, oz)) = cxz {
                let mut new_chunks = World::new(1, WORLD_HEIGHT as i32, 1, ox, 0, oz);
                loop { if !new_chunks.chunks.load_visible() {break;} };
                new_chunks.build_sky_light();
                let chunks = new_chunks.chunks;
                let cxz = (chunks.chunks[0].as_ref().unwrap().xyz.0, chunks.chunks[0].as_ref().unwrap().xyz.2);

                for chunk in chunks.chunks.into_iter() {
                    let Some(chunk) = chunk else {continue};
                    let xyz = chunk.xyz;
                    let index = ChunkCoords(ox, xyz.1, oz).chunk_index(&world.chunks);
                    // let index = chunk.xyz.chunk_index(&world.chunks);
                    println!("{} {} {} {} {}", ox, oz, index, world.chunks.ox, world.chunks.oz);
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
                        world.light.add_rgbs(&mut world.chunks, gx, gy, gz); 
                    }
                }
                world.light.solve_rgbs(&mut world.chunks);
            } else {
                thread::sleep(Duration::from_millis(200));
            }
        }
    })
}