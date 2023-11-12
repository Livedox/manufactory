use std::{time::{Instant, Duration}, collections::HashMap};

const STACK_SIZE: u32 = 100;

#[derive(Debug, Clone, Copy)]
pub struct PossibleItem(pub Option<Item>);

impl PossibleItem {
    pub fn new_none() -> Self {Self(None)}
    pub fn new(id: u32, count: u32) -> Self {Self(Some(Item::new(id, count)))}

    pub fn try_add(&mut self, possible_item: &PossibleItem) -> Option<Item> {
        possible_item.0.as_ref().and_then(|item| self.try_add_item(&item))
    }


    pub fn try_add_item(&mut self, item: &Item) -> Option<Item> {
        if let Some(item_src) = &mut self.0 {
            item_src.try_add(item)
        } else {
            self.0 = Some(Item::new(item.id, std::cmp::min(item.count, STACK_SIZE)));
            if item.count > STACK_SIZE {return Some(Item::new(item.id, item.count - STACK_SIZE))};
            None
        }
    }

    pub fn try_sub(&mut self, possible_item: &PossibleItem) -> Option<Item> {
        possible_item.0.as_ref().and_then(|item| self.try_sub_item(&item))
    }

    pub fn try_sub_item(&mut self, item: &Item) -> Option<Item> {
        let result = self.0.as_mut().map_or(Some(Item::from(item)), |i| i.try_sub(&item));
        self.clear_if_empty();
        result
    }


    pub fn try_take(&mut self, max_count: u32) -> Option<Item> {
        let result = self.0.as_mut().and_then(|item| Some(item.take(max_count)));
        self.clear_if_empty();
        result
    }


    pub fn available_space(&self, item_id: u32) -> u32 {
        self.0.as_ref()
            .and_then(|item| Some(item.available_space(item_id)))
            .unwrap_or(STACK_SIZE)
    }

    pub fn free_space(&self, item_id: u32) -> u32 {
        self.0.as_ref().map_or(STACK_SIZE, |_| 0)
    }

    pub fn residual_space(&self, item_id: u32) -> u32 {
        self.0.as_ref()
            .and_then(|item| Some(item.available_space(item_id)))
            .unwrap_or(0)
    }

    pub fn contains(&self, item_id: u32) -> u32 {
        self.0.as_ref().map_or(0, |item| item.contains(item_id))
    }

    pub fn is_possible_add(&self, item: &Item) -> bool {
        self.available_space(item.id()) >= item.count
    }


    pub fn clear(&mut self) -> Option<Item> {
        let mut result: Option<Item> = None;
        std::mem::swap(&mut result, &mut self.0);
        result
    }


    pub fn clear_if_empty(&mut self) {
        let count = if let Some(item) = &self.0 {item.count} else {return};
        if count == 0 {self.0 = None}
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Item {
    id: u32,
    pub count: u32,
}

impl Item {
    pub fn new(id: u32, count: u32) -> Self {Self { id, count }}

    pub fn from(item: &Item) -> Self {
        Self { id: item.id, count: item.count }
    }

    pub fn id(&self) -> u32 {self.id}
    pub fn try_add(&mut self, item: &Self) -> Option<Item> {
        if self.id != item.id {return Some(Item::from(item))};
        let sum = self.count + item.count;
        self.count = std::cmp::min(sum, STACK_SIZE);
        if sum > self.count {return Some(Item::new(self.id, sum - self.count))}
        None
    }


    pub fn try_sub(&mut self, item: &Self) -> Option<Item> {
        if self.id != item.id {return Some(Item::from(item))};
        let sub = self.count as i32 - item.count as i32;
        self.count = std::cmp::max(sub, 0i32) as u32;
        if sub < 0 {return Some(Item::new(self.id, sub.abs() as u32))}
        None
    }


    pub fn available_space(&self, item_id: u32) -> u32 {
        if self.id != item_id {return 0};
        if self.count < STACK_SIZE {return STACK_SIZE - self.count};
        0
    }

    pub fn contains(&self, item_id: u32) -> u32 {
        if item_id == self.id {return self.count}
        0
    }

    pub fn take(&mut self, max_count: u32) -> Item {
        self.try_sub(&Item::new(self.id, max_count))
            .and_then(|i| Some(Item::new(self.id, max_count - i.count)))
            .unwrap_or(Item::new(self.id, max_count))
    }

    pub fn add_count(&mut self, count: u32) {
        self.count += count;
    }

    pub fn sub_count(&mut self, count: u32) {
        if self.count >= count {self.count -= count}
    }

    pub fn stack_size(&self) -> u32 {
        STACK_SIZE
    }
}