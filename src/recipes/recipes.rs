use std::{time::Duration, cell::OnceCell, sync::OnceLock};

use crate::recipes::item::Item;
use crate::recipes::recipe::{Recipe, RecipeCrafter, RecipeCategory};

use super::recipe::Recipes;

static ALL_RECIPES_CONTAINER: OnceLock<Vec<Recipe>> = OnceLock::new();
pub fn all_recipe() -> &'static [Recipe] {
    ALL_RECIPES_CONTAINER.get_or_init(|| vec![
        Recipe {
            index: 0,
            id: 0,
            duration: Duration::from_secs_f32(0.3),
            crafter: RecipeCrafter::PA,
            category: RecipeCategory::ITEM,
            ingredients: vec![Item::new(1, 2)],
            result: Item::new(2, 1)
        },
        Recipe {
            index: 1,
            id: 1,
            duration: Duration::new(2, 0),
            crafter: RecipeCrafter::PA,
            category: RecipeCategory::ITEM,
            ingredients: vec![Item::new(1, 8)],
            result: Item::new(4, 1)
        },
        Recipe {
            index: 2,
            id: 2,
            duration: Duration::new(1, 0),
            crafter: RecipeCrafter::FURNACE,
            category: RecipeCategory::ITEM,
            ingredients: vec![Item::new(0, 1)],
            result: Item::new(1, 1)
        },
        Recipe {
            index: 3,
            id: 3,
            duration: Duration::new(1, 0),
            crafter: RecipeCrafter::PA,
            category: RecipeCategory::ITEM,
            ingredients: vec![Item::new(3, 2), Item::new(2, 1)],
            result: Item::new(5, 2)
        },
        Recipe {
            index: 4,
            id: 4,
            duration: Duration::new(5, 0),
            crafter: RecipeCrafter::PA,
            category: RecipeCategory::ITEM,
            ingredients: vec![Item::new(2, 10)],
            result: Item::new(6, 1)
        },
        Recipe {
            index: 5,
            id: 5,
            duration: Duration::new(2, 0),
            crafter: RecipeCrafter::PA,
            category: RecipeCategory::ITEM,
            ingredients: vec![Item::new(3, 15)],
            result: Item::new(7, 1)
        },
        Recipe {
            index: 6,
            id: 6,
            duration: Duration::new(1, 0),
            crafter: RecipeCrafter::PA,
            category: RecipeCategory::ITEM,
            ingredients: vec![Item::new(2, 10)],
            result: Item::new(8, 1)
        },
        Recipe {
            index: 7,
            id: 7,
            duration: Duration::new(1, 0),
            crafter: RecipeCrafter::PA,
            category: RecipeCategory::ITEM,
            ingredients: vec![Item::new(2, 5), Item::new(3, 5)],
            result: Item::new(9, 1)
        },
        Recipe {
            index: 8,
            id: 8,
            duration: Duration::new(1, 0),
            crafter: RecipeCrafter::PA,
            category: RecipeCategory::ITEM,
            ingredients: vec![Item::new(2, 1)],
            result: Item::new(10, 1)
        },
        Recipe {
            index: 9,
            id: 9,
            duration: Duration::new(1, 0),
            crafter: RecipeCrafter::PA,
            category: RecipeCategory::ITEM,
            ingredients: vec![Item::new(2, 1)],
            result: Item::new(11, 1)
        },
        Recipe {
            index: 10,
            id: 10,
            duration: Duration::new(3, 0),
            crafter: RecipeCrafter::PA,
            category: RecipeCategory::ITEM,
            ingredients: vec![Item::new(1, 4), Item::new(2, 2)],
            result: Item::new(12, 1)
        },
        Recipe {
            index: 11,
            id: 11,
            duration: Duration::new(1, 0),
            crafter: RecipeCrafter::PA,
            category: RecipeCategory::ITEM,
            ingredients: vec![Item::new(2, 1)],
            result: Item::new(13, 1)
        },
    ])
}

static RECIPES_CONTAINER: OnceLock<Recipes> = OnceLock::new();
#[allow(non_snake_case)]
pub fn RECIPES() -> &'static Recipes {
    RECIPES_CONTAINER.get_or_init(|| Recipes::new(all_recipe()))
}