use crate::{player::player::Player, voxels::{chunks::Chunks}, direction::Direction, world::{World, global_xyz::GlobalXYZ, xyz::XYZ}, recipes::{storage::Storage, item::Item}};

use super::{block_type::BlockType, light_permeability::LightPermeability};

pub trait BlockInteraction {
    fn id(&self) -> u32;
    fn emission(&self) -> &[u8; 3];
    fn light_permeability(&self) -> LightPermeability;
    fn block_type(&self) -> &BlockType;
    fn is_additional_data(&self) -> bool;

    fn width(&self) -> usize {1}
    fn height(&self) -> usize {1}
    fn depth(&self) -> usize {1}
    fn min_point(&self) -> &XYZ {&XYZ(0.0, 0.0, 0.0)}
    fn max_point(&self) -> &XYZ {&XYZ(1.0, 1.0, 1.0)}
    fn is_multiblock(&self) -> bool {false}
    fn is_voxel_size(&self) -> bool {false}

    fn ore(&self) -> Option<Item> {None}


    fn on_block_break(&self, world: &mut World, player: &mut Player, xyz: &GlobalXYZ) {
        world.break_voxel(xyz);
    }
    fn on_block_set(&self, world: &mut World, player: &mut Player, xyz: &GlobalXYZ, dir: &Direction) -> bool {
        if world.voxel(xyz).map(|v| v.id == 0).unwrap_or(true) {
            world.set_voxel(xyz, self.id(), dir);
            return true;
        }
        false
    }
}

pub trait BlockItem {
    fn item_id(&self) -> u32;
}