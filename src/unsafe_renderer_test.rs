use std::{sync::{Arc, Mutex, mpsc::{Sender, channel, Receiver, SendError, TryRecvError}}, thread, collections::HashMap, time::Duration};

use itertools::iproduct;

use crate::{meshes::Meshes, voxels::chunks::Chunks, models::animated_model::AnimatedModel, graphic::render::{RenderResult, render}, world::{World, chunk_coords::ChunkCoords, global_coords::GlobalCoords}};

pub struct UnsafeRendererTest {
    send_to_renderer: Sender<Vec<usize>>,
    receive_from_renderer: Receiver<RenderResult>,
}

impl UnsafeRendererTest {
    pub fn new_test(
        world: *mut World,
        player_coords: Arc<Mutex<(f32, f32, f32)>>,
        render_result: Arc<Mutex<Option<RenderResult>>>
    ) -> Self {
        let (send_to_renderer, receiver) = channel::<Vec<usize>>();
        let (sender, receive_from_renderer) = channel::<RenderResult>();
        
        // Self::spawn_renderer_thread(world, indices, receiver, sender);
        Self::spawn_renderer_thread_test2(world, player_coords, render_result);

        Self {
            send_to_renderer,
            receive_from_renderer,
        }
    }

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
                    .map_or(false, |chunk| chunk.as_ref().map_or(false, |c| c.modified()))
            }).copied();
            drop(lock);
            if let Some(chunk_index) = chunk_index {
                world.chunks.chunks[chunk_index].as_mut().unwrap().modify(false);
                let _ = sender.send(render(chunk_index, world));
            }
        }});
    }


    fn spawn_renderer_thread_test2(
        world: *mut World,
        player_coords: Arc<Mutex<(f32, f32, f32)>>,
        render_result: Arc<Mutex<Option<RenderResult>>>
    ) {
        let mut container_results = Vec::<RenderResult>::new();
        let world = unsafe {
            world.as_mut().unwrap()
        };
        thread::spawn(move || {loop {
            let p_coords = player_coords.lock().unwrap().clone();
            let p_coords: ChunkCoords = GlobalCoords::from(p_coords).into();
            let chunk_position = world.chunks.find_nearest_position_xyz(
                p_coords, &|c| c.map_or(false, |c| c.modified()));

            if let Some(chunk_index) = chunk_position.map(|cp| cp.index(world.chunks.depth, world.chunks.width)) {
                world.chunks.chunks[chunk_index].as_mut().unwrap().modify(false);
                // Why with sleep it work better ????????????
                thread::sleep(Duration::from_millis(2));
                if let Some(indx) = container_results.iter().position(|a| {a.chunk_index == chunk_index}) {
                    container_results.remove(indx);
                }
                
                container_results.push(render(chunk_index, world));
            }

            let mut render_result_lock = render_result.lock().unwrap();
            let is_same_chunk = container_results.last()
                .map_or(false, |r| render_result_lock.as_ref().map_or(false, |rr| {
                    r.chunk_index == rr.chunk_index
                }));
            if is_same_chunk || render_result_lock.is_none() {
                *render_result_lock = container_results.pop();
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


fn find_nearest_chunk(world: &World, player_coords: (i32, i32, i32)) -> Option<(i32, i32, i32)> {
    let width = world.chunks.width;
    let height = world.chunks.height;
    let depth = world.chunks.depth;
    let px = player_coords.0;
    let py = player_coords.1;
    let pz = player_coords.2;
    for i in 0..(depth.max(width).max(height)) {
        let min_x = if px > i {-i} else {0};
        let max_x = if i+px < width {i} else {width - px - 1};
        let min_y = if py > i {-i} else {0};
        let max_y = if i+py < width {i} else {height - py - 1};
        let cz_arr: Box<[i32]> = check_size(i, pz, depth);
        for (cx, cy, cz) in iproduct!(min_x..=max_x, min_y..=max_y, cz_arr.iter()) {
            if world.chunks
                .chunk(ChunkCoords(cx+px, cy+py, cz+pz))
                .map_or(false, |c| c.modified())
            {
                return Some((cx+px, cy+py, cz+pz));
            }
        }

        let cx_arr: Box<[i32]> = check_size(i, px, width);
        let min_z = if pz > i {-i + 1} else {0};
        let max_z = if i+pz < depth - 1 {i - 1} else {depth - pz - 1};
        for (cy, cz, cx) in iproduct!(min_y..=max_y, min_z..=max_z, cx_arr.iter()) {
            if world.chunks
                .chunk(ChunkCoords(cx+px, cy+py, cz+pz))
                .map_or(false, |c| c.modified())
            {
                return Some((cx+px, cy+py, cz+pz));
            }
        }

        let cy_arr: Box<[i32]> = check_size(i, py, height);
        let min_x = if px > i {-i + 1} else {0};
        let max_x = if i+px < width {i - 1} else {width - px - 1};
        for (cx, cz, cy) in iproduct!(min_x..=max_x, min_z..=max_z, cy_arr.iter()) {
            if world.chunks
                .chunk(ChunkCoords(cx+px, cy+py, cz+py))
                .map_or(false, |c| c.modified())
            {
                return Some((cx+px, cy+py, cz+py));
            }
        }
    }
    None
}

fn check_size(i: i32, p: i32, size: i32) -> Box<[i32]> {
    if p < i {
        Box::new([i])
    } else if i + p > size {
        Box::new([-i])
    } else {
        Box::new([-i, i])
    }
}