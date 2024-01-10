use std::{sync::{Mutex, Arc, mpsc::Sender, atomic::{Ordering, AtomicBool}, RwLock}, time::{Duration, Instant}, thread::{JoinHandle}};

use crate::{world::{World, chunk_coords::ChunkCoords, global_coords::GlobalCoords}, graphic::render::{RenderResult, render}, unsafe_mutex::UnsafeMutex, WORLD_EXIT};

pub fn spawn(
    world: Arc<World>,
    sender: Sender<RenderResult>,
    exit: Arc<AtomicBool>
) -> JoinHandle<()> {
    let thread = std::thread::Builder::new().name("renderer".to_owned()).stack_size(32 * 1024 * 1024);
    thread.spawn(move || {loop {
        if exit.load(Ordering::Relaxed) {break};

        let ox = world.chunks.ox();
        let oz = world.chunks.oz();
        let width = world.chunks.width;
        let depth = world.chunks.depth;
        
        if let Some(chunk) = world.chunks.find_unrendered() {
            chunk.modify(false);
            if let Some(result) = render(chunk.xyz.nindex(width, depth, ox, oz), &world.chunks) {
                let _ = sender.send(result);
            }
        } else {
            std::thread::sleep(Duration::from_millis(16));
        }
    }}).unwrap()
}