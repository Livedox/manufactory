use std::sync::{Arc, Mutex, Weak};

use serde::{Deserialize, Serialize};

use crate::{bytes::{AsFromBytes, BytesCoder}, direction::{self, Direction}, engine::texture::TextureAtlas, gui::{draw::Draw, my_widgets::inventory_slot::inventory_slot}, live_voxel_default_deserialize, player::inventory::PlayerInventory, player_unlockable, recipes::{item::{Item, PossibleItem}, recipe::ActiveRecipe, recipes::RECIPES, storage::Storage}, voxels::chunks::Chunks, world::global_coords::GlobalCoords};

use super::{LiveVoxelBehavior, PlayerUnlockable, LiveVoxelCreation};

#[derive(Debug, Serialize, Deserialize)]
pub struct Furnace {
    storage: [PossibleItem; 2],
    active_recipe: Option<ActiveRecipe>,
}

impl LiveVoxelCreation for Arc<Mutex<Furnace>> {
    fn create(_: &Direction) -> Box<dyn LiveVoxelBehavior> {
        Box::new(Arc::new(Mutex::new(Furnace::default())))
    }

    live_voxel_default_deserialize!(Arc<Mutex<Furnace>>);
}

impl LiveVoxelBehavior for Arc<Mutex<Furnace>> {
    player_unlockable!();

    fn storage(&self) -> Option<Arc<Mutex<dyn Storage>>> {
        Some(self.clone())
    }

    fn update(&self, _: &Chunks, _: GlobalCoords, _: &[GlobalCoords]) {
        let mut furnace = self.lock().unwrap();
        let active_recipe_take = furnace.active_recipe.take();
        if let Some(active_recipe) = &active_recipe_take {
            let storage = furnace.mut_storage();
            if active_recipe.is_finished() && storage[1].is_possible_add(&active_recipe.recipe.result) {
                storage[1].try_add_item(&active_recipe.recipe.result);
                furnace.active_recipe = None;
            } else {
                furnace.active_recipe = active_recipe_take;
            }
        } else {
            let Some(item) = &furnace.storage[0].0 else {return};
            let Some(recipe) = RECIPES().furnace.first_by_ingredient(item.id()).cloned() else {return};
            furnace.active_recipe = furnace.start_recipe(&recipe);
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

impl Storage for Furnace {
    fn storage(&self) -> &[PossibleItem] {
        &self.storage
    }

    fn mut_storage(&mut self) -> &mut [PossibleItem] {
        &mut self.storage
    }

    fn take_first_existing(&mut self, max_count: u32) -> Option<(Item, usize)> {
        self.mut_storage()[1].try_take(max_count).map(|i| (i, 1))
    }

    fn add(&mut self, item: &Item, _: bool) -> Option<Item> {
        if RECIPES().furnace.get_by_ingredient(item.id()).is_some() {
            return self.mut_storage()[0].try_add_item(item);
        }
        Some(*item)
    }

    fn is_item_exist(&self, item: &Item) -> bool {
        self.storage[0].contains(item.id()) >= item.count
    }
}

impl Default for Furnace {
    fn default() -> Self {
        Self {
            storage: [PossibleItem::new_none(); 2],
            active_recipe: None
        }
    }
}

impl Draw for Furnace {
    fn draw(&mut self, ui: &mut egui::Ui, atlas: Arc<TextureAtlas>, inventory: Arc<Mutex<PlayerInventory>>) {
        let mut task: Option<usize> = None;
        ui.horizontal(|ui| {
            for (index, item) in self.storage().iter().enumerate() {
                if ui.add(inventory_slot(&atlas, item)).drag_started() {
                    task = Some(index);
                }
            }
        });

        if let Some(task) = task {
            let Some(item) = self.mut_storage()[task].0.take() else {return};
            let remainder = inventory.lock().unwrap().add(&item, true);
            if let Some(r) = remainder {self.set(&r, task)}
        }
    }
}

impl PlayerUnlockable for Furnace {
    fn get_storage(&self) -> Option<&dyn Storage> {
        Some(self)
    }

    fn get_mut_storage(&mut self) -> Option<&mut dyn Storage> {
        Some(self)
    }
}