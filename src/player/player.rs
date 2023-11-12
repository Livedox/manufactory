use std::{rc::{Rc, Weak}, cell::RefCell};

use crate::{recipes::{storage::Storage, items::ITEMS, item_interaction::ItemInteraction}, voxels::voxel_data::PlayerUnlockableStorage, world::{World, global_xyz::GlobalXYZ}, direction::Direction};

use super::inventory::PlayerInventory;

#[derive(Debug)]
pub struct Player {
    pub active_slot: usize,
    pub open_storage: Option<PlayerUnlockableStorage>,
    inventory: Rc<RefCell<PlayerInventory>>,
}


impl Player {
    pub fn new() -> Self {
        Self {
            open_storage: None,
            inventory: Rc::new(RefCell::new(PlayerInventory::new())),
            active_slot: 0,
        }
    }


    pub fn inventory(&mut self) -> Rc<RefCell<PlayerInventory>> {
        self.inventory.clone()
    }


    pub fn on_right_click(&mut self, world: &mut World, xyz: &GlobalXYZ, dir: &Direction) {
        let Some(item_id) = self.inventory
            .borrow_mut()
            .storage()[self.active_slot].0
            .map(|item| item.id()) else {return};
        ITEMS()[item_id as usize].on_right_click(world, self, xyz, dir);
    }
}