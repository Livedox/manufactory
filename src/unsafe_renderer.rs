use std::{sync::{Arc, Mutex, mpsc::{Sender, channel, Receiver, SendError, TryRecvError}}, thread, collections::HashMap};

use crate::{meshes::Meshes, voxels::chunks::Chunks, models::animated_model::AnimatedModel, graphic::render::{RenderResult, render}, world::World};

pub struct UnsafeRenderer {
    send_to_renderer: Sender<usize>,
    receive_from_renderer: Receiver<RenderResult>,
}

impl UnsafeRenderer {
    pub fn new(world: *const World) -> Self {
        let (send_to_renderer, receiver) = channel::<usize>();
        let (sender, receive_from_renderer) = channel::<RenderResult>();
        
        Self::spawn_renderer_thread(world, receiver, sender);

        Self {
            send_to_renderer,
            receive_from_renderer,
        }
    }

    pub fn send(&self, index: usize) -> Result<(), SendError<usize>> {
        self.send_to_renderer.send(index)
    }

    pub fn try_recv(&self) -> Result<RenderResult, TryRecvError> {
        self.receive_from_renderer.try_recv()
    }

    fn spawn_renderer_thread(world: *const World, receiver: Receiver<usize>, sender: Sender<RenderResult>) {
        let world = unsafe {
            world.as_ref().unwrap()
        };
        thread::spawn(move || {loop {
            let index = receiver.recv().unwrap();
            let _ = sender.send(render(index, world));
        }});
    }
}