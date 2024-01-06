use std::{sync::{Arc, atomic::Ordering}, thread::{self, JoinHandle}, time::{Instant, Duration}};

use crate::{voxels::chunks::Chunks, world::World, unsafe_mutex::UnsafeMutex, WORLD_EXIT};

pub fn spawn(world: Arc<UnsafeMutex<World>>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            if WORLD_EXIT.load(Ordering::Relaxed) {break};
            let mut world = unsafe {world.lock_unsafe()}.unwrap();
            let now = Instant::now();
            let ptr = &mut world.chunks as *mut Chunks;
            for chunk in world.chunks.chunks.iter_mut() {
                let Some(chunk) = chunk.as_mut() else {continue};

                if !chunk.voxels_data.is_empty() {
                    chunk.unsaved = true;
                }

                for vd in chunk.voxels_data.values() {
                    vd.update(ptr)
                }
            }
            drop(world);
            thread::sleep(Duration::from_millis(100u64.saturating_sub(now.elapsed().as_millis() as u64)));
        }
    })
}