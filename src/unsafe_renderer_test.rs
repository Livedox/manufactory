use std::{sync::{Arc, Mutex, mpsc::{Sender, channel, Receiver, SendError, TryRecvError}}, thread, collections::HashMap};

use crate::{meshes::Meshes, voxels::chunks::Chunks, models::animated_model::AnimatedModel, graphic::render::{RenderResult, render}, world::World, TestStruct};

pub struct UnsafeRendererTest {
    send_to_renderer: Sender<Vec<usize>>,
    receive_from_renderer: Receiver<RenderResult>,
}

impl UnsafeRendererTest {
    pub fn new(world: *mut World, indices: Arc<Mutex<Vec<usize>>>) -> Self {
        let (send_to_renderer, receiver) = channel::<Vec<usize>>();
        let (sender, receive_from_renderer) = channel::<RenderResult>();
        
        // Self::spawn_renderer_thread(world, indices, receiver, sender);
        Self::spawn_renderer_thread(world, indices, sender);

        Self {
            send_to_renderer,
            receive_from_renderer,
        }
    }

    pub fn send(&self, index: Vec<usize>) -> Result<(), SendError<Vec<usize>>> {
        self.send_to_renderer.send(index)
    }

    pub fn try_recv(&self) -> Result<RenderResult, TryRecvError> {
        self.receive_from_renderer.try_recv()
    }

    fn spawn_renderer_thread(world: *mut World, indices_test: Arc<Mutex<Vec<usize>>>, sender: Sender<RenderResult>) {
        let world = unsafe {
            world.as_mut().unwrap()
        };
        thread::spawn(move || {loop {
            // for i in receiver.try_iter() {
            //     indices = i;
            // }
            // let chunk_index = world.chunks.chunks
            //         .iter()
            //         .position(|chunk| chunk.as_ref().map_or(false, |c| c.modified));
            let lock = indices_test.lock().unwrap();
            let chunk_index = lock.iter().find(|i| {
                world.chunks.chunks
                    .get(**i)
                    .map_or(false, |chunk| chunk.as_ref().map_or(false, |c| c.modified))
            }).copied();
            drop(lock);
            if let Some(chunk_index) = chunk_index {
                world.chunks.chunks[chunk_index].as_mut().unwrap().modified = false;
                let _ = sender.send(render(chunk_index, world));
            }
        }});
    }


    // fn spawn_renderer_thread_test(world: *mut World, indices_test: *const TestStruct, sender: Sender<RenderResult>) {
    //     let indices = unsafe {
    //         indices_test.as_ref().unwrap()
    //     };
        
    //     let world = unsafe {
    //         world.as_mut().unwrap()
    //     };
    //     thread::spawn(move || {loop {
    //         // for i in receiver.try_iter() {
    //         //     indices = i;
    //         // }
    //         // let chunk_index = world.chunks.chunks
    //         //         .iter()
    //         //         .position(|chunk| chunk.as_ref().map_or(false, |c| c.modified));
    //         println!("{:?}", indices.indices);
    //         let chunk_index = indices.indices.iter().find(|i| {
    //             world.chunks.chunks
    //                 .get(**i)
    //                 .map_or(false, |chunk| chunk.as_ref().map_or(false, |c| c.modified))
    //         });
    //         if let Some(chunk_index) = chunk_index {
    //             world.chunks.chunks[*chunk_index].as_mut().unwrap().modified = false;
    //             let _ = sender.send(render(*chunk_index, world));
    //         }
    //     }});
    // }
}