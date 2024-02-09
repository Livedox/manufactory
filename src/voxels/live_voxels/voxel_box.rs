use std::sync::{Arc, Mutex, Weak};

use serde::{Deserialize, Serialize};

use crate::{bytes::BytesCoder, direction::Direction, engine::texture::TextureAtlas, gui::{draw::Draw, my_widgets::inventory_slot::inventory_slot}, player::inventory::PlayerInventory, player_unlockable, recipes::{item::PossibleItem, storage::Storage}};

use super::{LiveVoxel, LiveVoxelDesiarialize, LiveVoxelNew, PlayerUnlockable};

impl LiveVoxelNew for Arc<Mutex<VoxelBox>> {
    fn new_livevoxel(_: &Direction) -> Box<dyn LiveVoxel> {
        Box::new(Arc::new(Mutex::new(VoxelBox::default())))
    }
}

impl LiveVoxelDesiarialize for Arc<Mutex<VoxelBox>> {
    fn deserialize(bytes: &[u8]) -> Box<dyn LiveVoxel> {
        Box::new(bincode::deserialize::<Self>(bytes).unwrap())
    }
}

impl LiveVoxel for Arc<Mutex<VoxelBox>> {
    player_unlockable!();
    fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

impl PlayerUnlockable for VoxelBox {
    fn get_storage(&self) -> Option<&dyn Storage> {
        Some(self)
    }

    fn get_mut_storage(&mut self) -> Option<&mut dyn Storage> {
        Some(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelBox {
    storage: [PossibleItem; 30]
}

impl VoxelBox {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Storage for VoxelBox {
    fn storage(&self) -> &[PossibleItem] {
        &self.storage
    }

    fn mut_storage(&mut self) -> &mut [PossibleItem] {
        &mut self.storage
    }
}

impl Default for VoxelBox {
    fn default() -> Self {
        Self { storage: [PossibleItem::new_none(); 30] }
    }
}

impl Draw for VoxelBox {
    fn draw(&mut self, ui: &mut egui::Ui, atlas: Arc<TextureAtlas>,inventory: Arc<Mutex<PlayerInventory>>) {
        let mut task: Option<usize> = None;
        ui.horizontal(|ui| {ui.vertical(|ui| {
            let len = self.storage().len();
            let count = (len as f32 / 10.0).ceil() as usize;
            for i in 0..count {
                ui.horizontal(|ui| {
                    for j in 0..(std::cmp::min(10, len - i*10)) {
                        if ui.add(inventory_slot(&atlas, &self.storage()[i*10 + j])).drag_started() {
                            task = Some(i*10 + j);
                        };
                    }
                });
            }
        })});

        if let Some(task) = task {
            let Some(item) = self.mut_storage()[task].0.take() else {return};
            let remainder = inventory.lock().unwrap().add(&item, true);
            if let Some(r) = remainder {self.set(&r, task)}
        }
    }
}