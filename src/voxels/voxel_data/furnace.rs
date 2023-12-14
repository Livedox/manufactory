use std::sync::{Arc, Mutex};

use crate::{recipes::{recipes::RECIPES, item::{PossibleItem, Item}, recipe::ActiveRecipe, storage::Storage}, gui::{draw::Draw, my_widgets::inventory_slot::inventory_slot}, player::inventory::PlayerInventory, engine::texture::TextureAtlas, bytes::{BytesCoder, AsFromBytes}};

use super::DrawStorage;

#[derive(Debug)]
pub struct Furnace {
    storage: [PossibleItem; 2],
    active_recipe: Option<ActiveRecipe>,
}


impl Furnace {
    pub fn new() -> Self {
        Self {
            storage: [PossibleItem::new_none(); 2],
            active_recipe: None
        }
    }

    pub fn update(&mut self) {
        let active_recipe_take = self.active_recipe.take();
        if let Some(active_recipe) = &active_recipe_take {
            let storage = self.mut_storage();
            if active_recipe.is_finished() && storage[1].is_possible_add(&active_recipe.recipe.result) {
                storage[1].try_add_item(&active_recipe.recipe.result);
                self.active_recipe = None;
            } else {
                self.active_recipe = active_recipe_take;
            }
        } else {
            let Some(item) = &self.storage[0].0 else {return};
            let Some(recipe) = RECIPES().furnace.first_by_ingredient(item.id()).cloned() else {return};
            self.active_recipe = self.start_recipe(&recipe);
        }
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
        Self::new()
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

impl DrawStorage for Furnace {}

impl BytesCoder for Furnace {
    fn decode_bytes(data: &[u8]) -> Self {
        let recipe_index = u32::from_bytes(&data[0..u32::size()]);
        let mut furnace = Self {
            active_recipe: None,
            storage: <[PossibleItem; 2]>::decode_bytes(&data[u32::size()..]),
        };
        if recipe_index != u32::MAX {
            let ar = RECIPES().all[recipe_index as usize].start_absolute();
            furnace.active_recipe = Some(ar);
        }
        furnace
    }
    fn encode_bytes(&self) -> Box<[u8]> {
        let mut bytes = Vec::new();
        let ri = self.active_recipe.as_ref().map(|ar| ar.recipe.index as u32).unwrap_or(u32::MAX);
        bytes.extend(ri.as_bytes());
        bytes.extend(self.storage.encode_bytes().as_ref());
        bytes.into()
    }
}