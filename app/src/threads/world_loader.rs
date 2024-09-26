use std::{thread::{self, JoinHandle}, sync::{Arc, atomic::{Ordering, AtomicBool}}, time::{Duration}};



use crate::{world::{World}, unsafe_mutex::UnsafeMutex, save_load::{WorldRegions}};


pub fn spawn(
    world: Arc<World>,
    world_regions: Arc<WorldRegions>,
    exit: Arc<AtomicBool>,
) -> JoinHandle<()> {
    let thread = std::thread::Builder::new().name("world_loader".to_owned()).stack_size(32 * 1024 * 1024);
    thread.spawn(move || {
        loop {
            if exit.load(Ordering::Relaxed) {break};
            println!("r1");
            if let Some((cx, cz)) = world.chunks.find_unloaded() {
                println!("r2");
                world.load_column_of_chunks(&world_regions, cx, cz);
                println!("r3");
            } else {
                thread::sleep(Duration::from_millis(200));
            }
        }
    }).unwrap()
}