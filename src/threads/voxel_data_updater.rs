use std::{sync::{Arc, Mutex, mpsc::{Sender, channel, Receiver, SendError, TryRecvError}}, thread::{self, JoinHandle}, collections::HashMap, time::{Instant, Duration}};

use crate::{meshes::Meshes, voxels::chunks::Chunks, models::animated_model::AnimatedModel, graphic::render::{RenderResult, render}, world::World};

pub fn spawn(world_ptr: *mut World) -> JoinHandle<()> {
    let world = unsafe { world_ptr.as_mut().unwrap() };

    thread::spawn(move || {
        loop {
            let now = Instant::now();
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