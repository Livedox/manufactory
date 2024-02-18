use crate::{player::player::Player, direction::Direction, world::{World, global_coords::GlobalCoords, coords::Coords}, recipes::item::Item};

use super::{block_type::BlockType};

pub trait BlockInteraction {
    fn id(&self) -> u32;
    fn emission(&self) -> &[u8; 3];
    fn is_light_passing(&self) -> bool;
    fn block_type(&self) -> &BlockType;
    fn is_additional_data(&self) -> bool;
    
    fn is_glass(&self) -> bool {false}
    fn width(&self) -> usize {1}
    fn height(&self) -> usize {1}
    fn depth(&self) -> usize {1}
    fn min_point(&self) -> &Coords {&Coords(0.0, 0.0, 0.0)}
    fn max_point(&self) -> &Coords {&Coords(1.0, 1.0, 1.0)}
    fn is_multiblock(&self) -> bool {false}
    fn is_voxel_size(&self) -> bool {false}

    fn ore(&self) -> Option<Item> {None}


    fn on_block_break(&self, world: &World, _: &mut Player, xyz: &GlobalCoords) {
        world.break_voxel(xyz);
    }
    fn on_block_set(&self, world: &World, _: &mut Player, xyz: &GlobalCoords, dir: &Direction) -> bool {
        if world.voxel(xyz).map(|v| v.id == 0).unwrap_or(true) {
            // world.set_voxel(xyz, self.id(), dir);
            return true;
        }
        false
    }
}

pub trait BlockItem {
    fn item_id(&self) -> u32;
}