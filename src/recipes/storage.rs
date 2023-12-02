use crate::bytes::{DynByteInterpretation, any_as_u8_slice};
use crate::bytes::NumFromBytes;
use super::{item::{PossibleItem, Item}, recipe::{ActiveRecipe, Recipe}};
use std::{fmt::Debug, time::Instant};

pub trait Storage {
    fn storage(&self) -> & [PossibleItem];
    fn mut_storage(&mut self) -> &mut [PossibleItem];

    fn add_items(&mut self, items: &[Item]) {
        items.iter().for_each(|item| { self.add(item, false); });
    }

    fn add(&mut self, item: &Item, smart: bool) -> Option<Item> {
        let mut added_item = Item::from(item);
        for possible_item in self.mut_storage().iter_mut() {
            if smart && possible_item.0.is_none() {continue}
            let remainder = possible_item.try_add_item(&added_item);
            let Some(remainder) = remainder else {return None};
            added_item = remainder;
        }
        if smart {return self.add(&added_item, false)}
        Some(added_item)
    }

    fn add_by_index(&mut self, item: &Item, index: usize) -> Option<Item> {
        self.mut_storage()[index].try_add_item(item)
    }

    fn remove_from_start(&mut self) -> Option<(Item, usize)> {
        let position = self.storage().iter().position(|item| item.0.is_some());
        let Some(position) = position else {return None};
        self.mut_storage()[position].clear().map(|item| (item, position))
    }

    fn remove_from_end(&mut self) -> Option<(Item, usize)> {
        let position = self.storage().iter().rev().position(|item| item.0.is_some());
        let Some(position) = position else {return None};
        self.mut_storage()[position].clear().map(|item| (item, position))
    }
    
    fn is_empty(&self) -> bool {self.storage().iter().all(|item| item.0.is_none())}

    fn take_first_existing(&mut self, max_count: u32) -> Option<(Item, usize)> {
        for (i, possible_item) in self.mut_storage().iter_mut().enumerate() {
            let Some(item) = possible_item.try_take(max_count) else {continue};
            return Some((item, i))
        }
        None
    }

    fn is_item_exist(&self, item: &Item) -> bool {
        self.storage()
            .iter()
            .map(|possible_item| possible_item.contains(item.id()))
            .sum::<u32>() >= item.count
    }

    fn is_items_exist(&self, items: &[Item]) -> bool {
        items.iter().all(|item| self.is_item_exist(item))
    }

    fn is_space_exist(&self, item: &Item) -> bool {
        self.storage()
            .iter()
            .map(|possible_item| possible_item.available_space(item.id()))
            .sum::<u32>() >= item.count
    }


    fn is_spaces_exist(&self, items: &[Item]) -> bool {
        let free_slots = self.storage()
            .iter()
            .map(|possible_item| {possible_item.0.is_none() as u32})
            .sum::<u32>();

        let need_slots = items.iter().map(|item| {
            let residual_space = self.storage()
                .iter()
                .map(|possible_item| {possible_item.residual_space(item.id()) as f32})
                .sum::<f32>();
            let count = item.count as f32 - residual_space;
            if count < 0.0 {0} else {(count / item.stack_size() as f32).ceil() as u32}
        }).sum::<u32>();
        
        free_slots >= need_slots
    }


    fn remove(&mut self, item: &Item) -> Option<Item> {
        let mut sub_item = Item::from(item);
        for possible_item in self.mut_storage().iter_mut() {
            let remainder = possible_item.try_sub_item(&sub_item);
            let Some(remainder) = remainder else {return None};
            sub_item = remainder;
        }
        Some(sub_item)
    }

    fn remove_items(&mut self, items: &[Item]) {
        items.iter().for_each(|item| {self.remove(item);});
    }

    fn remove_by_index(&mut self, item: &Item, index: usize) -> Option<Item> {
        self.mut_storage()[index].try_sub_item(item)
    }

    fn set(&mut self, item: &Item, index: usize) {
        self.mut_storage()[index].0 = Some(Item::from(item));
    }

    fn start_recipe(&mut self, recipe: &Recipe) -> Option<ActiveRecipe> {
        if self.is_items_exist(&recipe.ingredients[..]) {
            self.remove_items(&recipe.ingredients[..]);
            return Some(ActiveRecipe::new(Instant::now(), recipe.clone()));
        }
        None
    }

    fn cancel_recipe(&mut self, recipe: &Recipe) -> bool {
        if self.is_spaces_exist(&recipe.ingredients[..]) {
            self.add_items(&recipe.ingredients[..]);
            return true;
        }
        false
    }
}


impl Debug for dyn Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Storage: {:?}", self.storage())
    }
}


impl<const N: usize> DynByteInterpretation for [PossibleItem; N] {
    fn to_bytes(&self) -> Box<[u8]> {
        let mut v = Vec::new();
        for (i, item) in self.iter().enumerate() {
            if let Some(item) = item.0 {
                v.extend(unsafe { any_as_u8_slice(&[i as u32, item.id(), item.count]) });
            }
        }
        v.into()
    }

    fn from_bytes(data: &[u8]) -> Self {
        let mut s: [PossibleItem; N] = [PossibleItem::new_none(); N];
        for c in data.chunks(12) {
            s[u32::from_bytes(&c[0..4]) as usize] = PossibleItem(Some(Item::new(
                u32::from_bytes(&c[4..8]), u32::from_bytes(&c[8..12]))));
        }
        s
    }
}