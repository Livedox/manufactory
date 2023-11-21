use std::{sync::{mpsc::{Sender, Receiver, channel, TryRecvError, SendError}, Arc, Mutex}, thread, time::{Instant, Duration}};

use itertools::iproduct;

use crate::{world::{World, chunk_coords::ChunkCoords, global_coords::GlobalCoords}, voxels::{chunks::WORLD_HEIGHT, chunk::CHUNK_SIZE}, my_time::Timer};

pub struct WorldLoaderTest {
    send_to_loader: Sender<(i32, i32)>,
    receive_from_loader: Receiver<World>,
}

impl WorldLoaderTest {
    pub fn new(world: *mut World, player_coords: Arc<Mutex<(f32, f32, f32)>>) -> Self {
        let (send_to_loader, receiver) = channel::<(i32, i32)>();
        let (sender, receive_from_loader) = channel::<World>();

        Self::spawn_world_loader_thread(world, player_coords);

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

    fn spawn_world_loader_thread(
        world: *mut World,
        player_coords: Arc<Mutex<(f32, f32, f32)>>,
    ) {
        let world = unsafe {
            world.as_mut().unwrap()
        };
        thread::spawn(move || {
            loop {
                let p_coords = player_coords.lock().unwrap().clone();
                let p_coords: ChunkCoords = GlobalCoords::from(p_coords).into();
                let cxz: Option<(i32, i32)> = world.chunks
                    .find_nearest_position_xz(p_coords, &|c| c.is_none())
                    .map(|pos| (pos.0, pos.2));
                
                if let Some((ox, oz)) = cxz {
                    let mut new_chunks = World::new(1, WORLD_HEIGHT as i32, 1, ox, 0, oz);
                    loop { if !new_chunks.chunks.load_visible() {break;} };
                    new_chunks.build_sky_light();

                    let chunks = new_chunks.chunks;
                    let cxz = (chunks.chunks[0].as_ref().unwrap().xyz.0, chunks.chunks[0].as_ref().unwrap().xyz.2);
                    for chunk in chunks.chunks.into_iter() {
                        let Some(chunk) = chunk else {continue};
                        let index = chunk.xyz.index(world.chunks.depth, world.chunks.width);
                        world.chunks.chunks[index] = Some(chunk);
                    };

                    let min_x = cxz.0*CHUNK_SIZE as i32-1;
                    let max_x = (cxz.0+1)*CHUNK_SIZE as i32+1;

                    let min_z = cxz.1*CHUNK_SIZE as i32-1;
                    let max_z = (cxz.1+1)*CHUNK_SIZE as i32+1;
                    for (gy, gz, gx) in iproduct!(0i32..((WORLD_HEIGHT*CHUNK_SIZE) as i32), min_z..max_z, min_x..max_x) {
                        if gx == min_x || gx == max_x || gz == max_z || gz == min_z {
                            world.light.add_rgbs(&mut world.chunks, gx, gy, gz); 
                        }
                    }
                    world.light.solve_rgbs(&mut world.chunks);
                }
            }
        });
    }
}


fn find_nearest_chunk(world: &World, player_coords: (i32, i32, i32)) -> Option<(i32, i32)> {
    let depth = world.chunks.depth;
    let width = world.chunks.width;
    let px = player_coords.0;
    let pz = player_coords.2;
    for i in 0..(depth.max(width)) {
        let min = if px > i {-i} else {0};
        let max = if i+px < width {i} else {width - px - 1};
        for cx in min..=max {
            let cz_arr: Box<[i32]> = if pz < i {
                Box::new([i])
            } else if i + pz > depth {
                Box::new([-i])
            } else {
                Box::new([-i, i])
            };
            for cz in cz_arr.iter() {
                if world.chunks.chunk(ChunkCoords(cx+px, 0, cz+pz)).is_none()
                    && !(cx+px < 0 || cz+pz < 0)
                {
                    return Some((cx+px, cz+pz));
                }
            }
        }
        let min = if pz > i {-i + 1} else {0};
        let max = if i+pz < depth - 1 {i - 1} else {depth - pz - 1};
        for cz in min..=max {
            let cx_arr: Box<[i32]> = if px < i {
                Box::new([i])
            } else if i + px > width {
                Box::new([-i])
            } else {
                Box::new([-i, i])
            };
            for cx in cx_arr.iter() {
                if world.chunks.chunk(ChunkCoords(cx+px, 0, cz+pz)).is_none()
                    && !(cx+px < 0 || cz+pz < 0)
                {
                    return Some((cx+px, cz+pz));
                }
            }
        }
    }
    None
}