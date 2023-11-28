use std::{sync::{Mutex, Arc}, time::{Duration, Instant}, thread::{self, JoinHandle}};

use crate::{world::{World, chunk_coords::ChunkCoords, global_coords::GlobalCoords, SyncUnsafeWorldCell}, graphic::render::{RenderResult, render}, unsafe_mutex::UnsafeMutex};

pub fn spawn(
    world: Arc<UnsafeMutex<World>>,
    player_coords: Arc<Mutex<(f32, f32, f32)>>,
    render_result: Arc<Mutex<Option<RenderResult>>>
) -> JoinHandle<()> {
    let mut results = Vec::<RenderResult>::new();
    thread::spawn(move || {loop {
        let mut world = world.lock_unsafe(false).unwrap();
        let now = Instant::now();
        let pc = player_coords.lock().unwrap().clone();
        let pc: ChunkCoords = GlobalCoords::from(pc).into();
        
        let chunk_position = world.chunks.find_pos_stable_xyz(
            &|c| c.map_or(false, |c| c.modified()));
        
        if let Some(chunk_index) = chunk_position.map(|cp| cp.chunk_index(&world.chunks)) {
            world.chunks.chunks[chunk_index].as_mut().unwrap().modify(false);
            // Why with sleep it work better ????????????
            if let Some(indx) = results.iter().position(|a| {a.chunk_index == chunk_index}) {
                results.remove(indx);
            }
            
            let pos_time = Instant::now();
            if let Some(r) = render(chunk_index, &world) {
                results.push(r);
            }
            println!("Find pos time: {:?}", pos_time.elapsed().as_secs_f32());
            
            results.sort_by(|a, b| {
                let a = world.chunks.chunks[a.chunk_index].as_ref()
                    .map(|c| c.xyz)
                    .map_or(i32::MAX, |a| (a.0-pc.0).abs() + (a.1-pc.1).abs() +(a.2-pc.2).abs());
                let b = world.chunks.chunks[b.chunk_index].as_ref()
                    .map(|c| c.xyz)
                    .map_or(i32::MAX, |b| (b.0-pc.0).abs() + (b.1-pc.1).abs() +(b.2-pc.2).abs());

                a.cmp(&b).reverse()
            });
        }

        if !results.is_empty() {
            let is_same_chunk = render_result.lock().unwrap().as_ref().map_or(false, |active| {
                results.last().map_or(false, |new| active.chunk_index == new.chunk_index)
            });
            if is_same_chunk || render_result.lock().unwrap().is_none() {
                *render_result.lock().unwrap() = results.pop();
            }
            println!("Time to render 1 chunk: {:?}", now.elapsed().as_secs_f32());
        } else {
            drop(world);
            thread::sleep(Duration::from_millis(16));
        }
    }})
}