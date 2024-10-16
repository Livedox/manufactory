use std::{thread::{self, JoinHandle}, sync::{Arc, atomic::{Ordering, AtomicBool}}, time::{Duration}};



use crate::{light::light_solvers::LightSolvers, save_load::WorldRegions, unsafe_mutex::UnsafeMutex, world::World};


pub fn spawn(
    world: Arc<World>,
    world_regions: Arc<WorldRegions>,
    exit: Arc<AtomicBool>,
) -> JoinHandle<()> {
    let thread = std::thread::Builder::new().name("world_loader".to_owned()).stack_size(32 * 1024 * 1024);
    thread.spawn(move || {
        let mut solvers = vec![
            LightSolvers::new(&world.chunks.content),
            LightSolvers::new(&world.chunks.content),
            LightSolvers::new(&world.chunks.content),
            LightSolvers::new(&world.chunks.content),
        ];
        loop {
            if exit.load(Ordering::Relaxed) {break};
            let coords = world.chunks.find_some_unloaded();
            if !coords.is_empty() {
                world.load_chunks(&world_regions, &mut solvers, coords);
            } else {
                thread::sleep(Duration::from_millis(200));
            }
        }
    }).unwrap()
}