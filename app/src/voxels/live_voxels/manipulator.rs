use std::{sync::{Mutex}, time::{Duration, Instant}};
use serde::{Deserialize, Serialize};

use crate::{direction::Direction, live_voxel_default_deserialize, recipes::item::Item, voxels::chunks::Chunks, coords::global_coord::GlobalCoord};

use super::{LiveVoxelBehavior, LiveVoxelCreation};

#[derive(Debug, Serialize, Deserialize)]
pub struct Manipulator {
    #[serde(skip)]
    start_time: Option<Instant>,
    #[serde(skip)]
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

    pub fn update(&mut self, coords: GlobalCoord, chunks: &Chunks) {
        let return_time = self.return_time.map_or(true, |rt| rt.elapsed() >= (Self::SPEED/2));
        if self.item_id.is_none() && self.start_time.is_none() && return_time {
            let src_coords = GlobalCoord::new(coords.x - self.direction[0] as i32, coords.y, coords.z - self.direction[2] as i32);
            let Some(storage) = chunks.master_live_voxel(src_coords).and_then(|lv| lv.storage()) else {return};
            if let Some(item) = storage.lock().unwrap().take_first_existing(1) {
                self.item_id = Some(item.0.id());
                self.start_time = Some(Instant::now());
                self.return_time = None;
            };
        }
        
        let start_time = self.start_time.map_or(true, |rt| rt.elapsed() >= (Self::SPEED/2));
        if self.item_id.is_some() && start_time {
            let dst_coords = GlobalCoord::new(coords.x + self.direction[0] as i32, coords.y, coords.z + self.direction[2] as i32);
            let Some(storage) = chunks.master_live_voxel(dst_coords).and_then(|lv| lv.storage()) else {return};
            let result = storage.lock().unwrap().add(&Item::new(self.item_id.unwrap(), 1), false).is_none();
            if result {
                self.item_id = None;
                self.start_time = None;
                self.return_time = Some(Instant::now());
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

impl LiveVoxelBehavior for Mutex<Manipulator> {
    fn animation_progress(&self) -> f32 {
        self.lock().unwrap().animation_progress()
    }

    fn rotation_index(&self) -> Option<u32> {
        Some(self.lock().unwrap().rotation_index())
    }

    fn update(&self, chunks: &Chunks, coord: GlobalCoord, _: &[GlobalCoord]) {
        self.lock().unwrap().update(coord, chunks);
    }

    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

impl LiveVoxelCreation for Mutex<Manipulator> {
    fn create(direction: &Direction) -> Box<dyn LiveVoxelBehavior> {
        Box::new(Mutex::new(Manipulator::new(direction)))
    }
    
    live_voxel_default_deserialize!(Arc<Mutex<AssemblingMachine>>);
}