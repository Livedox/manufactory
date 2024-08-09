use std::{thread::{self, JoinHandle}, sync::{Arc, atomic::{Ordering, AtomicBool}}, time::{Duration}};



use crate::{world::{World}, unsafe_mutex::UnsafeMutex, save_load::{WorldRegions}};


pub fn spawn(
    world: Arc<World>,
    world_regions: Arc<WorldRegions>,
    exit: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            if exit.load(Ordering::Relaxed) {break};
            if let Some((cx, cz)) = world.chunks.find_unloaded() {
                world.load_column_of_chunks(&world_regions, cx, cz);
            } else {
                thread::sleep(Duration::from_millis(200));
            }
        }
    })
}