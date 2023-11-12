use super::item_interaction::ItemInteraction;

pub struct ItemType {
    pub id: u32,
    pub stack_size: u32,
    pub block_id: Option<u32>,
}

impl ItemType {
    pub fn new(id: u32, stack_size: u32, block_id: Option<u32>) -> Self {Self {
        id,
        stack_size,
        block_id
    }}
}

impl ItemInteraction for ItemType {
    fn id(&self) -> u32 {self.id}
    fn block_id(&self) -> Option<u32> {self.block_id}
    fn stack_size(&self) -> u32 {self.stack_size}
}