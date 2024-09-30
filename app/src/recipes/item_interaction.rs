use crate::{content::Content, coords::global_coord::GlobalCoord, direction::Direction, player::player::Player, recipes::{item::Item, storage::Storage}, world::World};

pub trait ItemInteraction {
    fn id(&self) -> u32;
    fn block_id(&self) -> Option<u32>;
    fn stack_size(&self) -> u32;

    fn on_right_click(&self, world: &World, player: &mut Player, xyz: &GlobalCoord, dir: &Direction, content: &Content) {
        if let Some(block_id) = self.block_id() {
            if content.blocks[block_id as usize].on_block_set(world, player, xyz, dir) {
                player.inventory().lock().unwrap().remove_by_index(&Item::new(self.id(), 1), player.active_slot);
            };
        }
    }
}