use crate::{recipes::{item::Item, storage::Storage}, world::{World, global_coords::GlobalCoords}, player::player::Player};

use super::{interaction::{BlockInteraction, BlockItem}, block_type::BlockType, light_permeability::LightPermeability};

pub struct BlockOre {
    pub item_id: u32,
    pub id: u32,
    pub emission: [u8; 3],
    pub is_light_passing: bool,
    pub block_type: BlockType,
    pub is_additional_data: bool,
}

impl BlockInteraction for BlockOre {
    fn id(&self) -> u32 {self.id}
    fn emission(&self) -> &[u8; 3] {&self.emission}
    fn is_light_passing(&self) -> bool {self.is_light_passing}
    fn block_type(&self) -> &BlockType {&self.block_type}
    fn is_additional_data(&self) -> bool {self.is_additional_data}

    fn on_block_break(&self, world: &World, player: &mut Player, xyz: &GlobalCoords) {
        world.break_voxel(xyz);
        player.inventory().lock().unwrap().add(&Item::new(self.item_id(), 1), true);
    }

    fn ore(&self) -> Option<Item> {
        Some(Item::new(self.item_id(), 1))
    }
}

impl BlockItem for BlockOre {
    fn item_id(&self) -> u32 {self.item_id}
}