use serde::{Deserialize, Serialize};

use crate::direction::{Direction};
use crate::recipes::item::PossibleItem;
use crate::voxels::new_chunks::{Chunks, GlobalCoord};
use crate::{live_voxel_default_deserialize};

use std::sync::{Arc, Mutex};

use crate::{recipes::{item::{Item}, storage::Storage}};


use super::{LiveVoxelBehavior, LiveVoxelCreation};

impl LiveVoxelCreation for Arc<Mutex<TransportBelt>> {
    fn create(direction: &Direction) -> Box<dyn LiveVoxelBehavior> {
        Box::new(Arc::new(Mutex::new(TransportBelt::new(direction))))
    }

    live_voxel_default_deserialize!(Arc<Mutex<TransportBelt>>);
}

impl LiveVoxelBehavior for Arc<Mutex<TransportBelt>> {
    fn transport_belt(&self) -> Option<Arc<Mutex<TransportBelt>>> {
        Some(self.clone())
    }

    fn storage(&self) -> Option<Arc<Mutex<dyn Storage>>> {
        Some(self.clone())
    }

    fn update(&self, chunks: &Chunks, coord: GlobalCoord, _: &[GlobalCoord]) {
        self.lock().unwrap().update(coord, chunks);
    }

    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

use itertools::Itertools;
// TODO: PLEASE UPDATE THIS SHIT

#[derive(Debug, PartialEq, Eq)]
pub enum TransportBeltSide {
    Left,
    Right
}


#[derive(Debug, Serialize, Deserialize)]
pub struct TransportBelt {
    item_progress: [f32; 6],
    direction: [i8; 3],
    storage: [PossibleItem; 6],
}


impl TransportBelt {
    pub fn new(direction: &Direction) -> Self {
        Self {
            storage: [PossibleItem::new_none(); 6],
            item_progress: [0.0; 6],
            direction: direction.simplify_to_one_greatest(true, false, true),
        }
    }

    pub fn rotation_index(&self) -> u32 {
        if self.direction[0] < 0 {return 3};
        if self.direction[2] > 0 {return 0};
        if self.direction[2] < 0 {return 1};
        2
    }

    pub fn update(&mut self, coords: GlobalCoord, chunks: &Chunks) {
        if self.storage[0].0.is_some() {self.item_progress[0] += 0.1;}
        if self.storage[3].0.is_some() {self.item_progress[3] += 0.1;}

        let mut checking_progress = self.item_progress[0] - 0.33;
        self.item_progress[1..3].iter_mut().enumerate().for_each(|(index, progress)| {
            if self.storage[index+1].0.is_some() && checking_progress > *progress {
                *progress += 0.1;
            } 
            checking_progress = *progress - 0.33;
        });

        let mut checking_progress = self.item_progress[3] - 0.33;
        self.item_progress[4..6].iter_mut().enumerate().for_each(|(index, progress)| {
            if self.storage[index+4].0.is_some() && checking_progress > *progress {
                *progress += 0.1;
            }
            checking_progress = *progress - 0.33;
        });

        let dst_coords = GlobalCoord::new(coords.x+self.direction[0] as i32, coords.y, coords.z+self.direction[2] as i32);
        let Some(dst) = chunks.master_live_voxel(dst_coords).and_then(|lv| lv.transport_belt()) else {return};
        
        if self.item_progress[0] > 1.0
         && dst.lock().unwrap().put(&self.storage[0].0.unwrap(), TransportBeltSide::Left).is_none() {
            self.item_progress[0] = self.item_progress[1];
            self.item_progress[1] = self.item_progress[2];
            self.item_progress[2] = 0.0;

            self.storage[0] = self.storage[1];
            self.storage[1] = self.storage[2];
            self.storage[2] = PossibleItem::new_none(); 
        }

        if self.item_progress[3] > 1.0 
         && dst.lock().unwrap().put(&self.storage[3].0.unwrap(), TransportBeltSide::Right).is_none() {
            self.item_progress[3] = self.item_progress[4];
            self.item_progress[4] = self.item_progress[5];
            self.item_progress[5] = 0.0;

            self.storage[3] = self.storage[4];
            self.storage[4] = self.storage[5];
            self.storage[5] = PossibleItem::new_none();
        }
    }

    pub fn put(&mut self, item: &Item, side: TransportBeltSide) -> Option<Item> {
        if side == TransportBeltSide::Left {
            for possible_item in self.storage[0..3].iter_mut(){
                if possible_item.0.is_none() {
                    return possible_item.try_add_item(item);
                }
            }
        } else {
            for possible_item in self.storage[3..6].iter_mut(){
                if possible_item.0.is_none() {
                    return possible_item.try_add_item(item);
                }
            }
        }
        Some(Item::from(item))
    }
}


impl Storage for TransportBelt {
    fn storage(&self) -> & [PossibleItem] {
        &self.storage
    }

    fn mut_storage(&mut self) -> &mut [PossibleItem] {
        &mut self.storage
    }

    fn take_first_existing(&mut self, max_count: u32) -> Option<(Item, usize)> {
        for (index, (_, possible_item)) in self.item_progress
          .iter()
          .zip(self.storage.iter_mut())
          .enumerate()
          .sorted_by(|(_, (a, _)), (_, (b, _))| (*a - 0.5).abs().total_cmp(&(*b - 0.5).abs()))
        {
            let Some(item) = possible_item.try_take(max_count) else {continue};
            self.item_progress[index] = 0.0;
            return Some((item, index))
        }
        None
    }


    fn add(&mut self, item: &Item, _: bool) -> Option<Item> {
        let mut returned_item = Item::from(item);
        let mut added_item = Item::new(item.id(), std::cmp::min(1, item.count));
        returned_item.sub_count(1);

        for possible_item in self.mut_storage().iter_mut() {
            if possible_item.0.is_none() {
                let remainder = possible_item.try_add_item(&added_item);
                let Some(remainder) = remainder else {
                    if returned_item.count > 0 {
                        return Some(returned_item);
                    } else {
                        return None;
                    };
                };
                added_item = remainder;
            } 
        }

        added_item.try_add(&returned_item);
        Some(added_item)
    }
}