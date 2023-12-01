use std::time::{Instant, Duration};
use crate::{world::{global_coords::GlobalCoords, local_coords::LocalCoords}, direction::Direction, voxels::{chunks::Chunks, block::blocks::BLOCKS}, recipes::{item::{PossibleItem, Item}, storage::Storage}};

#[derive(Debug)]
pub struct Manipulator {
    start_time: Option<Instant>,
    return_time: Option<Instant>,
    item_id: Option<u32>,
    direction: [i8; 3],
}


impl Manipulator {
    const SPEED: Duration = Duration::from_millis(300);

    pub fn new(direction: &Direction) -> Self {Self {
        start_time: None,
        return_time: None,
        item_id: None,
        direction: direction.simplify_to_one_greatest(true, false, true),
    }}

    pub fn update(&mut self, coords: GlobalCoords, chunks: *mut Chunks) {
        let return_time = self.return_time.map_or(true, |rt| rt.elapsed() >= (Self::SPEED/2));
        if self.item_id.is_none() && self.start_time.is_none() && return_time {
            let src_coords = GlobalCoords(coords.0 - self.direction[0] as i32, coords.1, coords.2 - self.direction[2] as i32);
            let src = unsafe {
                chunks.as_mut().expect("Chunks don't exist").mut_chunk(src_coords)
            };
            if let Some(src_chunk) = src {
                let Some(src_data) = src_chunk.mut_voxel_data(src_coords.into()) else {return};
                let Some(storage) = src_data.additionally.storage() else {return};
                if let Some(item) = storage.lock().unwrap().take_first_existing(1) {
                    self.item_id = Some(item.0.id());
                    self.start_time = Some(Instant::now());
                    self.return_time = None;
                };
            }
        }
        
        let start_time = self.start_time.map_or(false, |rt| rt.elapsed() >= (Self::SPEED/2));
        if self.item_id.is_some() && start_time {
            let dst_coords = GlobalCoords(coords.0 + self.direction[0] as i32, coords.1, coords.2 + self.direction[2] as i32);
            let dst = unsafe {
                chunks.as_mut().expect("Chunks don't exist").mut_chunk(dst_coords)
            };
            if let Some(dst_chunk) = dst {
                let Some(dst_data) = dst_chunk.mut_voxel_data(dst_coords.into()) else {return};
                let Some(storage) = dst_data.additionally.storage() else {return};
                let result = storage.lock().unwrap().add(&Item::new(self.item_id.unwrap(), 1), false).is_none();
                if result {
                    self.item_id = None;
                    self.start_time = None;
                    self.return_time = Some(Instant::now());
                }
            }
        }
    }


    pub fn animation_progress(&self) -> f32 {
        if let Some(start_time) = self.start_time {
            (start_time.elapsed().as_secs_f32() / Self::SPEED.as_secs_f32()).min(0.5)
        } else if let Some(return_time) = self.return_time {
            (return_time.elapsed().as_secs_f32() / Self::SPEED.as_secs_f32() + 0.5).min(1.0)
        } else {
            0.0
        }
    }


    pub fn rotation_index(&self) -> u32 {
        if self.direction[0] < 0 {return 2};
        if self.direction[2] > 0 {return 3};
        if self.direction[2] < 0 {return 1};
        0
    }
}