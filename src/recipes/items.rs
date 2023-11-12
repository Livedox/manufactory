use std::{time::Duration, cell::OnceCell, sync::OnceLock};

use super::item_type::ItemType;

static ITEMS_CONTAINER: OnceLock<Vec<ItemType>> = OnceLock::new();
#[allow(non_snake_case)]
pub fn ITEMS() -> &'static [ItemType] {
    ITEMS_CONTAINER.get_or_init(|| {
        vec![
            ItemType::new(0, 100, Some(5)),
            ItemType::new(1, 100, None),
            ItemType::new(2, 100, None),
            ItemType::new(3, 100, Some(7)),

            ItemType::new(4, 50, Some(15)),
            ItemType::new(5, 50, Some(17)),
            ItemType::new(6, 50, Some(16)),
            ItemType::new(7, 50, Some(14)),

            ItemType::new(8, 100, Some(13)),
            ItemType::new(9, 100, Some(4)),
            ItemType::new(10, 50, Some(10)),
            ItemType::new(11, 50, Some(11)),

            ItemType::new(12, 50, Some(9)),
            ItemType::new(13, 50, Some(12)),
        ]
    })
}