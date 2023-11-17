use std::{sync::{Arc, Mutex, mpsc::{Sender, channel, Receiver, SendError, TryRecvError}}, thread, collections::HashMap, time::{Instant, Duration}};

use crate::{meshes::Meshes, voxels::chunks::Chunks, models::animated_model::AnimatedModel, graphic::render::{RenderResult, render}, world::World};

pub fn spawn_unsafe_voxel_data_updater(chunks_ptr: *mut Chunks) {
    let chunks = unsafe {
        chunks_ptr.as_mut().unwrap()
    };

    thread::spawn(move || {
        loop {
            let now = Instant::now();
            let ptr = chunks as *mut Chunks;
            for chunk in chunks.chunks.iter() {
                let Some(chunk) = chunk else {continue};

                for vd in chunk.voxels_data.values() {
                    vd.update(ptr)
                }
            }

            thread::sleep(Duration::from_millis(100u64.checked_sub(now.elapsed().as_millis() as u64).unwrap_or(0)));
        }
    });
}