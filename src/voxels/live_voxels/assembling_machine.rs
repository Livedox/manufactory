use serde::{Deserialize, Serialize};

use crate::direction::Direction;
use crate::recipes::item::PossibleItem;
use crate::voxels::chunks::Chunks;
use crate::{live_voxel_default_deserialize, player_unlockable, GlobalCoords};
use crate::recipes::recipe::{ActiveRecipe, Recipe};
use std::sync::{Arc, Mutex};
use std::sync::Weak;
use crate::{recipes::{item::{Item}, storage::Storage, recipes::RECIPES}, gui::{draw::Draw, my_widgets::{assembling_machine_slot::assembling_machine_slot, recipe::recipe}}, player::inventory::PlayerInventory, engine::texture::TextureAtlas};
use crate::gui::my_widgets::container::container;

use super::{LiveVoxelBehavior, LiveVoxelCreation, PlayerUnlockable};

const INGREDIENT_LENGTH: usize = 3;
const RESULT_LENGTH: usize = 1;
const TOTAL_LENGTH: usize = INGREDIENT_LENGTH+RESULT_LENGTH;

impl LiveVoxelCreation for Arc<Mutex<AssemblingMachine>> {
    fn create(_: &Direction) -> Box<dyn LiveVoxelBehavior> {
        Box::new(Arc::new(Mutex::new(AssemblingMachine::default())))
    }

    live_voxel_default_deserialize!(Arc<Mutex<AssemblingMachine>>);
}

impl LiveVoxelBehavior for Arc<Mutex<AssemblingMachine>> {
    player_unlockable!();

    fn storage(&self) -> Option<Arc<Mutex<dyn Storage>>> {
        Some(self.clone())
    }

    fn update(&self, _: &Chunks, _: GlobalCoords, _: &[GlobalCoords]) {
        self.lock().unwrap().update();
    }

    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssemblingMachine {
    storage: [PossibleItem; TOTAL_LENGTH],

    #[serde(skip)]
    selected_recipe: Option<&'static Recipe>,
    active_recipe: Option<ActiveRecipe>,
}

impl AssemblingMachine {
    pub fn selected_recipe(&self) -> Option<&'static Recipe> {
        self.selected_recipe
    }

    pub fn select_recipe(&mut self, index: usize) -> ([PossibleItem; TOTAL_LENGTH], Vec<Item>) {
        self.selected_recipe = Some(&RECIPES().all[index]);
        let mut result = [PossibleItem::new_none(); TOTAL_LENGTH];
        std::mem::swap(&mut result, &mut self.storage);
        let ingredients = self.active_recipe.take().map_or(vec![], |ac| ac.recipe.ingredients);
        (result, ingredients)
    }

    pub fn update(&mut self) {
        if self.active_recipe.is_none() && self.selected_recipe.is_some() {
            self.active_recipe = self.start_recipe(self.selected_recipe.unwrap());
        }

        let Some(active_recipe) = &self.active_recipe else {return};
        if !active_recipe.is_finished() || !self.storage()[3].is_possible_add(&active_recipe.recipe.result) {return};
        
        let add_item = active_recipe.recipe.result;
        self.mut_storage()[3].try_add_item(&add_item);
        self.active_recipe = None;
    }
}


impl Default for AssemblingMachine {
    fn default() -> Self {
        Self {
            storage: [PossibleItem::new_none(); TOTAL_LENGTH],
            selected_recipe: None,
            active_recipe: None,
        }
    }
}


impl Storage for AssemblingMachine {
    fn storage(&self) -> &[PossibleItem] {
        &self.storage
    }

    fn mut_storage(&mut self) -> &mut [PossibleItem] {
        &mut self.storage
    }

    fn is_item_exist(&self, item: &Item) -> bool {
        self.storage()[0..INGREDIENT_LENGTH]
            .iter()
            .map(|possible_item| possible_item.contains(item.id()))
            .sum::<u32>() >= item.count
    }

    fn remove(&mut self, item: &Item) -> Option<Item> {
        let mut sub_item = Item::from(item);
        for possible_item in self.mut_storage()[0..INGREDIENT_LENGTH].iter_mut() {
            let remainder = possible_item.try_sub_item(&sub_item);
            let Some(remainder) = remainder else {return None};
            sub_item = remainder;
        }
        Some(sub_item)
    }

    fn add(&mut self, item: &Item, _: bool) -> Option<Item> {
        let mut added_item = Item::from(item);
        let Some(recipe) = self.selected_recipe else {return Some(added_item)};
        for (index, possible_item) in self.mut_storage()[0..INGREDIENT_LENGTH].iter_mut().enumerate() {
            if recipe.ingredients.get(index).map(|i| i.id()) == Some(item.id()) {
                let remainder = possible_item.try_add_item(&added_item);
                let Some(remainder) = remainder else {return None};
                added_item = remainder;
            }
        }
        Some(added_item)
    }

    fn take_first_existing(&mut self, max_count: u32) -> Option<(Item, usize)> {
        for (i, possible_item) in self.mut_storage()[INGREDIENT_LENGTH..TOTAL_LENGTH].iter_mut().enumerate() {
            let Some(item) = possible_item.try_take(max_count) else {continue};
            return Some((item, i))
        }
        None
    }
}


impl Draw for AssemblingMachine {
    fn draw(&mut self, ui: &mut egui::Ui, atlas: Arc<TextureAtlas>, inventory: Arc<Mutex<PlayerInventory>>) {
        let mut task: Option<usize> = None;
        let selected_recipe = self.selected_recipe();
        if let Some(selected_recipe) = selected_recipe {
            ui.horizontal(|ui| {
                for (i, item) in self.storage().iter().enumerate() {
                    if ui.add(assembling_machine_slot(&atlas, item, i, selected_recipe, i==3)).drag_started() {
                        task = Some(i);
                    };
                }
            });
        }
        ui.vertical(|ui| {
            ui.add(container(|ui| {
                let style = egui::Style {
                    spacing: egui::style::Spacing { item_spacing: egui::vec2(8.0, 8.0), ..Default::default() },
                    ..Default::default()
                };
                ui.set_style(style);
                ui.horizontal(|ui| {
                    for i in RECIPES().assembler.all() {
                        if ui.add(recipe(&atlas, i)).drag_started() {
                            let result = self.select_recipe(i.index);
                            for item in result.0 {
                                let Some(item) = item.0 else {continue};
                                inventory.lock().unwrap().add(&item, true);
                            }
                            for item in result.1 {
                                inventory.lock().unwrap().add(&item, true);
                            }
                        };
                    }
                });
            }, None));
        });

        if let Some(task) = task {
            let Some(item) = self.mut_storage()[task].0.take() else {return};
            let remainder = inventory.lock().unwrap().add(&item, true);
            if let Some(r) = remainder {self.set(&r, task)}
        }
    }
}

impl PlayerUnlockable for AssemblingMachine {
    fn get_storage(&self) -> Option<&dyn Storage> {
        Some(self)
    }

    fn get_mut_storage(&mut self) -> Option<&mut dyn Storage> {
        Some(self)
    }
}