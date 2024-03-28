use std::sync::Mutex;
use std::time::{Instant, Duration};

use serde::{Deserialize, Serialize};


use crate::live_voxel_default_deserialize;

use crate::{coords::global_coord::GlobalCoord, direction::Direction, voxels::{chunks::Chunks}, recipes::{item::PossibleItem, storage::Storage}};
use crate::voxels::live_voxels::LiveVoxelBehavior;

use crate::voxels::live_voxels::LiveVoxelCreation;
fn new_instant() -> Instant {Instant::now()}

#[derive(Debug, Serialize, Deserialize)]
pub struct Drill {
    dir: [i8; 3],
    storage: [PossibleItem; 1],
    #[serde(skip)]
    #[serde(default = "new_instant")] 
    start: Instant,
}
impl Drill {
    pub const DURATION: Duration = Duration::new(4, 0);
}

impl LiveVoxelCreation for Mutex<Drill> {
    fn create(direction: &Direction) -> Box<dyn LiveVoxelBehavior> {
        Box::new(Mutex::new(Drill {
            storage: [PossibleItem::new_none()],
            start: Instant::now(),
            dir: direction.simplify_to_one_greatest(true, false, true)
        }))
    }

    live_voxel_default_deserialize!(Mutex<Drill>);
}

impl LiveVoxelBehavior for Mutex<Drill> {
    fn update(&self, chunks: &Chunks, xyz: GlobalCoord, multiblock: &[GlobalCoord]) {
        let mut drill = self.lock().unwrap();
        let global = GlobalCoord::new(xyz.x - drill.dir[0] as i32, xyz.y, xyz.z-drill.dir[2] as i32);
        if let Some(storage) = chunks.master_live_voxel(global).and_then(|vd| vd.storage()) {
            if let Some(item) = drill.storage[0].0.take() {
                if let Some(r_item) = storage.lock().unwrap().add(&item, false) {
                    drill.storage[0].try_add_item(&r_item);
                }
            }
        }

        if drill.start.elapsed() < Drill::DURATION {return}
        drill.start = Instant::now();
        
        let mut ores = vec![];
        multiblock.iter().for_each(|coord| {
            let ore_coords = GlobalCoord::new(coord.x, coord.y-1, coord.z);
            let voxel = chunks.voxel_global(ore_coords);
            let Some(voxel) = voxel else {return};
            if let Some(item) = chunks.content.blocks[voxel.id as usize].ore() {
                ores.push(item);
            }
        });
        ores.into_iter().for_each(|ore| {
            drill.storage[0].try_add_item(&ore);
        });
    }

    fn rotation_index(&self) -> Option<u32> {
        let drill = self.lock().unwrap();
        if drill.dir[2] > 0 {return Some(0)};
        if drill.dir[0] < 0 {return Some(3)};
        if drill.dir[2] < 0 {return Some(2)};
        Some(1)
    }

    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}



impl Storage for Drill {
    fn storage(&self) -> &[PossibleItem] {
        &self.storage
    }

    fn mut_storage(&mut self) -> &mut [PossibleItem] {
        &mut self.storage
    }
}