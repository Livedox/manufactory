use crate::{player::player::Player, voxels::block::blocks::BLOCKS, direction::Direction, world::{World, global_coords::GlobalCoords}, recipes::{storage::Storage, item::Item}};

pub trait ItemInteraction {
    fn id(&self) -> u32;
    fn block_id(&self) -> Option<u32>;
    fn stack_size(&self) -> u32;

    fn on_right_click(&self, world: &World, player: &mut Player, xyz: &GlobalCoords, dir: &Direction) {
        if let Some(block_id) = self.block_id() {
            if BLOCKS()[block_id as usize].on_block_set(world, player, xyz, dir) {
                player.inventory().lock().unwrap().remove_by_index(&Item::new(self.id(), 1), player.active_slot);
            };
        }
    }
}