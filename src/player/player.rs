use std::{rc::Rc, cell::RefCell, sync::{Mutex, Arc, Weak}};

use crate::{recipes::{storage::Storage, items::ITEMS, item_interaction::ItemInteraction}, world::{World, global_coords::GlobalCoords}, direction::Direction, voxels::voxel_data::DrawStorage};

use super::inventory::PlayerInventory;

#[derive(Debug)]
pub struct Player {
    pub active_slot: usize,
    pub open_storage: Option<Weak<Mutex<dyn DrawStorage>>>,
    inventory: Arc<Mutex<PlayerInventory>>,
}


impl Player {
    pub fn new() -> Self {
        Self {
            open_storage: None,
            inventory: Arc::new(Mutex::new(PlayerInventory::new())),
            active_slot: 0,
        }
    }


    pub fn inventory(&mut self) -> Arc<Mutex<PlayerInventory>> {
        self.inventory.clone()
    }


    pub fn on_right_click(&mut self, world: &mut World, xyz: &GlobalCoords, dir: &Direction) {
        let Some(item_id) = self.inventory
            .lock().unwrap()
            .storage()[self.active_slot].0
            .map(|item| item.id()) else {return};
        ITEMS()[item_id as usize].on_right_click(world, self, xyz, dir);
    }
}