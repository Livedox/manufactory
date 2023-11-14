use std::{sync::mpsc::{Sender, Receiver, channel, TryRecvError, SendError}, thread, time::{Instant, Duration}};

use crate::{world::World, voxels::chunks::WORLD_HEIGHT, my_time::Timer};

pub struct WorldLoader {
    send_to_loader: Sender<(i32, i32)>,
    receive_from_loader: Receiver<World>,
}

impl WorldLoader {
    pub fn new() -> Self {
        let (send_to_loader, receiver) = channel::<(i32, i32)>();
        let (sender, receive_from_loader) = channel::<World>();

        Self::spawn_world_loader_thread(receiver, sender);

        Self {
            send_to_loader,
            receive_from_loader,
        }
    }

    pub fn send(&self, cxz: (i32, i32)) -> Result<(), SendError<(i32, i32)>> {
        self.send_to_loader.send(cxz)
    }

    pub fn try_recv(&self) -> Result<World, TryRecvError> {
        self.receive_from_loader.try_recv()
    }

    fn spawn_world_loader_thread(receiver: Receiver<(i32, i32)>, sender: Sender<World>) {
        thread::spawn(move || {
            loop {
                let (ox, oz) = receiver.recv().unwrap();
                let mut world = World::new(1, WORLD_HEIGHT as i32, 1, ox, 0, oz);
                loop { if !world.chunks.load_visible() {break;} };
                world.build_sky_light();
    
                let _ = sender.send(world);
            }
        });
    }
}