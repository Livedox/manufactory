use std::ops::Range;

use crate::{recipes::{recipe::{Recipe, ActiveRecipe, RecipeCrafter}, item::PossibleItem, storage::Storage, recipes::RECIPES}, bytes::{BytesCoder, cast_bytes_from_slice, AsFromBytes, cast_vec_from_bytes}};


#[derive(Debug)]
pub struct ActiveRecipes(pub Vec<ActiveRecipe>);


#[derive(Debug)]
pub struct PlayerInventory {
    storage: [PossibleItem; 50],
    active_recipes: ActiveRecipes
}


impl PlayerInventory {
    pub fn new() -> Self {
        Self { storage: [PossibleItem::new_none(); 50], active_recipes: ActiveRecipes(vec![]) }
    }

    pub fn active_recipe(&self) -> &Vec<ActiveRecipe> {
        &self.active_recipes.0
    }

    pub fn start_recipe(&mut self, recipe: &Recipe) -> bool {
        if !recipe.crafter.intersects(RecipeCrafter::PLAYER) {return false};
        let Some(active_recipe) = recipe.start(self) else {return false};
        self.active_recipes.0.push(active_recipe);
        true
    }

    pub fn update_recipe(&mut self) {
        let self_ptr = self as *mut Self;
        self.active_recipes.0.retain(|ar| !ar.update(unsafe {self_ptr.as_mut().unwrap()}));
    }


    pub fn cancel_active_recipe(&mut self, index: usize) -> bool {
        let Some(recipe) = self.active_recipes.0.get(index).map(|ar| ar.recipe.clone()) else {return false};
        if self.cancel_recipe(&recipe) {
            self.active_recipes.0.remove(index);
            return true;
        }
        false
    }


    pub fn place_in_hotbar(&mut self, index: usize) -> bool {
        let len = self.storage.iter().take(10).len();
        self.place_in_range(index, 0..len)
    }

    pub fn place_in_inventory(&mut self, index: usize) -> bool {
        let len = self.storage.iter().skip(10).len();
        self.place_in_range(index, 10..len)
    }

    fn place_in_range(&mut self, index: usize, range: Range<usize>) -> bool {
        let pi = self.storage[index].0.take();
        if let Some(item) = &pi {
            for sitem_index in range {
                if self.storage[sitem_index].is_possible_add(item) {
                    self.storage[sitem_index].try_add_item(item);
                    return true;
                }
            }
        }
        self.storage[index].0 = pi;
        false
    }
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self { storage: [PossibleItem::new_none(); 50], active_recipes: ActiveRecipes(vec![]) }
    }
}


impl Storage for PlayerInventory {
    fn storage(&self) -> & [PossibleItem] {
        &self.storage
    }

    fn mut_storage(&mut self) -> &mut [PossibleItem] {
        &mut self.storage
    }
}


impl BytesCoder for PlayerInventory {
    fn encode_bytes(&self) -> Box<[u8]> {
        let mut bytes = Vec::new();
        let recipies: Vec<u32> = self.active_recipes.0.iter().map(|ar| ar.recipe.id).collect();
        let recipies_bytes = cast_bytes_from_slice(&recipies);
        let recipies_len = recipies_bytes.len();

        let storage = self.storage.encode_bytes();
        let storage_len = storage.len();

        bytes.extend((recipies_len as u32).as_bytes());
        bytes.extend((storage_len as u32).as_bytes());
        bytes.extend(recipies_bytes);
        bytes.extend(storage.as_ref());
        bytes.into()
    }

    fn decode_bytes(bytes: &[u8]) -> Self {
        let recipe_end = u32::from_bytes(&bytes[0..4]) as usize + 8;
        let storage_end = recipe_end + u32::from_bytes(&bytes[4..8]) as usize;

        let recipies_id = cast_vec_from_bytes::<u32>(&bytes[8..recipe_end]);
        let storage = <[PossibleItem; 50]>::decode_bytes(&bytes[recipe_end..storage_end]);
        let active_recipes = ActiveRecipes(recipies_id.iter()
            .map(|id| RECIPES().all[*id as usize].start_absolute())
            .collect::<Vec<ActiveRecipe>>());

        Self { storage, active_recipes }
    }
}