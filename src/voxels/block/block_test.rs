use crate::{direction::Direction, player::player::Player, recipes::{item::Item, storage::Storage}, world::{coords::Coords, global_coords::GlobalCoords, World}};

use super::block_type::BlockType;

pub fn block_player_add_item(base: &BlockBase, world: &World, player: &mut Player, xyz: &GlobalCoords) {
    player.inventory().lock().unwrap().add(&Item::new(base.item_id, 1), true);
}

pub fn block_world_break(base: &BlockBase, world: &World, player: &mut Player, xyz: &GlobalCoords) {
    world.break_voxel(xyz);
}

pub fn block_world_set(base: &BlockBase, world: &World, player: &mut Player, xyz: &GlobalCoords, dir: &Direction) -> bool {
    if world.voxel(xyz).map(|v| v.id == 0).unwrap_or(true) {
        world.set_voxel(xyz, base.id, dir);
        return true;
    }
    false
}

pub fn block_multiblock_break(base: &BlockBase, world: &World, player: &mut Player, xyz: &GlobalCoords) {
    if let Some(xyz) = world.chunks.remove_multiblock_structure(*xyz) {
        xyz.iter().for_each(|c| {
            world.light.on_block_break(&world.chunks, c.0, c.1, c.2);
        });
    };
}

pub fn block_multiblock_set(base: &BlockBase, world: &World, player: &mut Player, xyz: &GlobalCoords, dir: &Direction) -> bool {
    // FIX THIS SHIT
    let mut width = base.width as i32;
    let mut depth = base.depth as i32;
    if base.id == 15 {
        let d = dir.simplify_to_one_greatest(true, false, true);
        if d[2] < 0 {width = -(base.width as i32)};
        if d[2] < 0 {depth = -(base.depth as i32)};
        if d[0] > 0 {depth = -(base.depth as i32)};
        if d[0] < 0 {width = -(base.width as i32)};
    }
    
    let coords = world.chunks
        .add_multiblock_structure(xyz, width, base.height as i32, depth, base.id, dir);
    if let Some(coords) = coords {
        coords.iter().for_each(|c| {
            world.light.on_block_set(&world.chunks, c.0, c.1, c.2, base.id);
        });
        return true;
    }
    false
}

pub struct BlockBase {
    pub item_id: u32,
    pub id: u32,
    pub emission: [u8; 3],
    pub block_type: BlockType,
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    
    pub is_light_passing: bool,
    pub is_additional_data: bool,
    pub is_glass: bool,
}

pub struct Block {
    pub base: BlockBase,

    pub on_block_break: Box<[&'static dyn Fn(&BlockBase, &World, &mut Player, &GlobalCoords)]>,
    pub on_block_set: Box<[&'static dyn Fn(&BlockBase, &World, &mut Player, &GlobalCoords, &Direction) -> bool]>
}

impl Block {
    pub fn on_block_break(&self, world: &World, player: &mut Player, xyz: &GlobalCoords) {
        self.on_block_break.iter().for_each(|f| {
            f(&self.base, world, player, xyz);
        });
    }

    pub fn on_block_set(&self, world: &World, player: &mut Player, xyz: &GlobalCoords, dir: &Direction) -> bool {
        self.on_block_set.iter().for_each(|f| {
            f(&self.base, world, player, xyz, dir);
        });
        return true;
    }

    pub fn id(&self) -> u32 {self.base.id}
    pub fn emission(&self) -> &[u8; 3] {&self.base.emission}
    pub fn is_light_passing(&self) -> bool {self.base.is_light_passing}
    pub fn block_type(&self) -> &BlockType {&self.base.block_type}
    pub fn is_additional_data(&self) -> bool {self.base.is_additional_data}
    pub fn is_glass(&self) -> bool {self.base.is_glass}
    
    pub fn width(&self) -> usize {1}
    pub fn height(&self) -> usize {1}
    pub fn depth(&self) -> usize {1}
    pub fn min_point(&self) -> &Coords {&Coords(0.0, 0.0, 0.0)}
    pub fn max_point(&self) -> &Coords {&Coords(1.0, 1.0, 1.0)}
    pub fn is_multiblock(&self) -> bool {false}
    pub fn is_voxel_size(&self) -> bool {false}

    pub fn ore(&self) -> Option<Item> {None}
}

unsafe impl Send for Block {}
unsafe impl Sync for Block {}