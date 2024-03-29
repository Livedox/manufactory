use std::{sync::{Arc, atomic::{Ordering, AtomicBool}}, thread::{self, JoinHandle}, time::{Instant, Duration}};

use crate::{world::{World}};

pub fn spawn(world: Arc<World>, exit: Arc<AtomicBool>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            if exit.load(Ordering::Relaxed) {break};
            let now = Instant::now();
            for chunk in unsafe {&*(world.chunks.chunks.get())}.iter() {
                let Some(chunk) = chunk else {continue};

                if !chunk.live_voxels.0.read().unwrap().is_empty() {
                    chunk.save(true);
                }

                for vd in chunk.live_voxels.0.read().unwrap().values() {
                    vd.update(&world.chunks);
                }
            }
            thread::sleep(Duration::from_millis(100u64.saturating_sub(now.elapsed().as_millis() as u64)));
        }
    })
}