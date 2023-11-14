use crate::{recipes::{item::{PossibleItem, Item}, storage::Storage, recipe::{Recipe, ActiveRecipe}, recipes::RECIPES}, world::global_coords::GlobalCoords};

use super::voxel_data::MultiBlock;

const INGREDIENT_LENGTH: usize = 3;
const RESULT_LENGTH: usize = 1;
const TOTAL_LENGTH: usize = INGREDIENT_LENGTH+RESULT_LENGTH;


#[derive(Debug)]
pub struct AssemblingMachine {
    storage: [PossibleItem; TOTAL_LENGTH],

    structure_coordinates: Vec<GlobalCoords>,

    selected_recipe: Option<&'static Recipe>,
    active_recipe: Option<ActiveRecipe>,
}


impl AssemblingMachine {
    pub fn new(structure_coordinates: Vec<GlobalCoords>) -> Self {
        Self {
            storage: [PossibleItem::new_none(); TOTAL_LENGTH],
            structure_coordinates,
            selected_recipe: None,
            active_recipe: None,
        }
    }

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
        if !(active_recipe.is_finished() && self.storage()[3].is_possible_add(&active_recipe.recipe.result)) {return};
        
        let add_item = active_recipe.recipe.result;
        self.mut_storage()[3].try_add_item(&add_item);
        self.active_recipe = None;
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
            .sum::<u32>() > item.count
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


impl MultiBlock for AssemblingMachine {
    fn structure_coordinates(&self) -> &[GlobalCoords] {
        &self.structure_coordinates
    }

    fn mut_structure_coordinates(&mut self) -> &mut [GlobalCoords] {
        &mut self.structure_coordinates
    }
}