use std::{sync::{Mutex, Arc}, time::{Duration, Instant}, thread::{self, JoinHandle}};

use crate::{world::{World, chunk_coords::ChunkCoords, global_coords::GlobalCoords}, graphic::render::{RenderResult, render}, unsafe_mutex::UnsafeMutex, WORLD_EXIT};

pub fn spawn(
    world: Arc<UnsafeMutex<World>>,
    player_coords: Arc<Mutex<(f32, f32, f32)>>,
    render_result: Arc<Mutex<Option<RenderResult>>>
) -> JoinHandle<()> {
    let mut results = Vec::<RenderResult>::new();
    thread::spawn(move || {loop {
        if unsafe { WORLD_EXIT } {break};
        let mut world = world.lock_unsafe(true).unwrap();
        let pc = player_coords.lock().unwrap().clone();
        let pc: ChunkCoords = GlobalCoords::from(pc).into();
        
        let chunk_position = world.chunks.find_pos_stable_xyz(
            &|c| c.map_or(false, |c| c.modified()));
        if let Some(cp) = chunk_position {
            let index = cp.chunk_index(&world.chunks);
            world.chunks.chunks[index].as_mut().unwrap().modify(false);
            if let Some(indx) = results.iter().position(|a| {a.xyz == cp}) {
                results.remove(indx);
            }
            
            if let Some(r) = render(cp.chunk_index(&world.chunks), &world) {
                results.push(r);
            }
            results.retain(|r| {
                let nx = r.xyz.0 - world.chunks.ox;
                let nz = r.xyz.2 - world.chunks.oz;
                nx >= 0 && nz >= 0 && nx < world.chunks.width && nz < world.chunks.depth
            });
            results.sort_by(|a, b| {
                let a = (a.xyz.0-pc.0).abs() + (a.xyz.1-pc.1).abs() + (a.xyz.2-pc.2).abs();
                let b = (b.xyz.0-pc.0).abs() + (b.xyz.1-pc.1).abs() + (b.xyz.2-pc.2).abs();

                a.cmp(&b).reverse()
            });
        }

        if !results.is_empty() {
            let is_same_chunk = render_result.lock().unwrap().as_ref().map_or(false, |active| {
                results.last().map_or(false, |new| active.xyz == new.xyz)
            });
            if is_same_chunk || render_result.lock().unwrap().is_none() {
                *render_result.lock().unwrap() = results.pop();
            }
        } else {
            drop(world);
            thread::sleep(Duration::from_millis(16));
        }
    }})
}