use crate::{player::player::Player, voxels::{chunks::Chunks, block::blocks::BLOCKS}, direction::Direction, world::{World, global_xyz::GlobalXYZ}, recipes::{storage::Storage, item::Item}};

pub trait ItemInteraction {
    fn id(&self) -> u32;
    fn block_id(&self) -> Option<u32>;
    fn stack_size(&self) -> u32;

    fn on_right_click(&self, world: &mut World, player: &mut Player, xyz: &GlobalXYZ, dir: &Direction) {
        if let Some(block_id) = self.block_id() {
            if BLOCKS()[block_id as usize].on_block_set(world, player, xyz, dir) {
                player.inventory().borrow_mut().remove_by_index(&Item::new(self.id(), 1), player.active_slot);
            };
        }
    }
}