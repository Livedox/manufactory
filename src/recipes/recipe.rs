use std::{time::{Instant, Duration}, collections::HashMap};

use bitflags::bitflags;

use super::{storage::Storage, item::Item};


bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct RecipeCrafter: u8 {
        const PLAYER = 0b1;
        const ASSEMBLER = 0b10;
        const FURNACE = 0b100;

        const PA = Self::PLAYER.bits() | Self::ASSEMBLER.bits();
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct RecipeCategory: u8 {
        const ITEM = 0b1;
        const BLOCK = 0b10;
    }
}


#[derive(Debug)]
pub struct ActiveRecipe {
    start_time: Instant,
    pub recipe: Recipe,
}

impl ActiveRecipe {
    pub fn new(start_time: Instant, recipe: Recipe) -> Self {
        Self { start_time, recipe }
    }

    pub fn cancel(&self, storage: &mut dyn Storage) -> bool {
        if storage.is_spaces_exist(&self.recipe.ingredients[..]) {
            storage.add_items(&self.recipe.ingredients[..]);
            return true;
        }
        false
    }

    pub fn progress(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32() / self.recipe.duration.as_secs_f32() % 1.0
    }

    pub fn is_finished(&self) -> bool {
        self.start_time.elapsed() > self.recipe.duration
    }

    pub fn update(&self, storage: &mut dyn Storage) -> bool {
        if self.is_finished() {
            if storage.is_space_exist(&self.recipe.result) {
                storage.add(&self.recipe.result, false);
                return true;
            }
        }
        false
    }
}


#[derive(Debug, Clone)]
pub struct Recipe {
    pub index: usize,
    pub id: u32,
    pub duration: Duration,
    pub crafter: RecipeCrafter,
    pub category: RecipeCategory,
    pub ingredients: Vec<Item>,
    pub result: Item,
}


impl Recipe {
    pub fn start(&self, storage: &mut dyn Storage) -> Option<ActiveRecipe> {
        if storage.is_items_exist(&self.ingredients[..]) {
            storage.remove_items(&self.ingredients[..]);
            return Some(ActiveRecipe { start_time: Instant::now(), recipe: self.clone() });
        }
        None
    }
}

#[derive(Debug)]
pub struct CraftStation<'a> {
    all: Vec<&'a Recipe>,
    ingredient_recipe: HashMap<u32, Vec<&'a Recipe>>,
    result_recipe: HashMap<u32, Vec<&'a Recipe>>,
}


impl<'a> CraftStation<'a> {
    pub fn new(recipes: &'a [Recipe], crafter: RecipeCrafter) -> CraftStation {
        let mut all = vec![];
        let mut ingredient_recipe = HashMap::<u32, Vec<&Recipe>>::new();
        let mut result_recipe = HashMap::<u32, Vec<&Recipe>>::new();
        recipes.iter().for_each(|recipe| {
            if !recipe.crafter.intersects(crafter) {return};

            all.push(recipe);

            recipe.ingredients.iter().for_each(|item| {
                ingredient_recipe
                    .entry(item.id())
                    .and_modify(|v| v.push(&recipe))
                    .or_insert(vec![&recipe]);
            });

            result_recipe
                .entry(recipe.id)
                .and_modify(|v| v.push(&recipe))
                .or_insert(vec![&recipe]);
        });

        Self { all, ingredient_recipe, result_recipe }
    }

    pub fn all(&self) -> &[&'a Recipe] {
        &self.all[..]
    }

    pub fn first_by_ingredient(&self, ingredient_id: u32) -> Option<&Recipe> {
        self.get_by_ingredient(ingredient_id).and_then(|v| Some(v[0]))
    }
    pub fn first_by_result(&self, result_id: u32) -> Option<&Recipe> {
        self.get_by_result(result_id).and_then(|v| Some(v[0]))
    }
    pub fn get_by_ingredient(&self, ingredient_id: u32) -> Option<&[&Recipe]> {
        self.ingredient_recipe.get(&ingredient_id).and_then(|v| Some(v.as_slice()))
    }
    pub fn get_by_result(&self, result_id: u32) -> Option<&[&Recipe]> {
        self.result_recipe.get(&result_id).and_then(|v| Some(v.as_slice()))
    }
}


#[derive(Debug)]
pub struct Recipes {
    pub all: &'static [Recipe],
    pub furnace: CraftStation<'static>,
    pub assembler: CraftStation<'static>,
    pub player: CraftStation<'static>,
}

impl Recipes {
    pub fn new(all: &'static [Recipe]) -> Self {
        Self {
            all,
            furnace: CraftStation::new(all, RecipeCrafter::FURNACE),
            player: CraftStation::new(all, RecipeCrafter::PLAYER),
            assembler: CraftStation::new(all, RecipeCrafter::ASSEMBLER)
        }
    }
}
