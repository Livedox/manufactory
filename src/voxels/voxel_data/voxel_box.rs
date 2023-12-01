use std::sync::{Arc, Mutex};

use crate::{recipes::{item::PossibleItem, storage::Storage}, gui::{draw::Draw, my_widgets::inventory_slot::inventory_slot}, player::inventory::PlayerInventory, engine::texture::TextureAtlas};

use super::DrawStorage;

#[derive(Debug, Clone)]
pub struct VoxelBox {
    storage: [PossibleItem; 30]
}

impl VoxelBox {
    pub fn new() -> Self {
        Self { storage: [PossibleItem::new_none(); 30] }
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
        Self::new()
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
                        if ui.add(inventory_slot(&atlas, &self.storage()[i*10 + j])).clicked() {
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

impl DrawStorage for VoxelBox {}