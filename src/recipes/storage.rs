use crate::bytes::{BytesCoder, AsFromBytes};

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
            let remainder = possible_item.try_add_item(&added_item)?;
            added_item = remainder;
        }
        if smart {return self.add(&added_item, false)}
        Some(added_item)
    }

    fn add_by_index(&mut self, item: &Item, index: usize) -> Option<Item> {
        self.mut_storage()[index].try_add_item(item)
    }

    fn remove_from_start(&mut self) -> Option<(Item, usize)> {
        let position = self.storage().iter().position(|item| item.0.is_some())?;
        self.mut_storage()[position].clear().map(|item| (item, position))
    }

    fn remove_from_end(&mut self) -> Option<(Item, usize)> {
        let position = self.storage().iter().rev().position(|item| item.0.is_some())?;
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
            let remainder = possible_item.try_sub_item(&sub_item)?;
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ItemHeader {
    index: u32,
    id: u32,
    count: u32,
}
impl ItemHeader {
    #[inline]
    fn new(index: u32, id: u32, count: u32) -> Self {Self {
        index,
        id,
        count,
    }}}
impl AsFromBytes for ItemHeader {}


impl<const N: usize> BytesCoder for [PossibleItem; N] {
    fn encode_bytes(&self) -> Box<[u8]> {
        let mut bytes = Vec::new();
        self.iter().enumerate().for_each(|(index, item)| {
            let Some(item) = item.0 else {return};
            bytes.extend(ItemHeader::new(index as u32, item.id(), item.count).as_bytes());
        });
        bytes.into()
    }

    fn decode_bytes(bytes: &[u8]) -> Self {
        let mut storage: [PossibleItem; N] = [PossibleItem::new_none(); N];
        bytes.chunks(12).for_each(|header_bytes| {
            let header = ItemHeader::from_bytes(header_bytes);
            storage[header.index as usize] = PossibleItem::new(header.id, header.count);
        });
        storage
    }
}