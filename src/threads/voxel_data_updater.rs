use std::{sync::{Arc, Mutex, mpsc::{Sender, channel, Receiver, SendError, TryRecvError}}, thread::{self, JoinHandle}, collections::HashMap, time::{Instant, Duration}};

use crate::{meshes::Meshes, voxels::chunks::Chunks, models::animated_model::AnimatedModel, graphic::render::{RenderResult, render}, world::{World, SyncUnsafeWorldCell}};

pub fn spawn(world: Arc<SyncUnsafeWorldCell>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::new(1000000, 0));
            let world = world.get_mut();
            let now = Instant::now();
            if world.chunks.is_translate {continue};
            let ptr = &mut world.chunks as *mut Chunks;
            for chunk in world.chunks.chunks.iter() {
                let Some(chunk) = chunk else {continue};

                for vd in chunk.voxels_data.values() {
                    vd.update(ptr)
                }
            }

            thread::sleep(Duration::from_millis(100u64.checked_sub(now.elapsed().as_millis() as u64).unwrap_or(0)));
        }
    })
}