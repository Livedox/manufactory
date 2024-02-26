use std::{thread::{self, JoinHandle}, sync::{Arc, atomic::{Ordering, AtomicBool}}, time::{Duration}};



use crate::{world::{World}, unsafe_mutex::UnsafeMutex, save_load::{WorldRegions}};


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
                world.load_column_of_chunks(&mut regions, cx, cz);
            } else {
                thread::sleep(Duration::from_millis(200));
            }
        }
    })
}