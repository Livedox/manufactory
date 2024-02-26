use std::{collections::HashMap, sync::OnceLock};

use crate::{direction::Direction, player::player::Player, recipes::{item::Item, storage::Storage}, world::{global_coords::GlobalCoords, World}};

use super::block_test::BlockBase;

pub fn player_add_item(base: &BlockBase, _world: &World, player: &mut Player, _xyz: &GlobalCoords, _dir: &Direction) -> bool {
    player.inventory().lock().unwrap().add(&Item::new(base.item_id.expect("Not have item_id"), 1), true);
    return true;
}

pub fn on_break(_base: &BlockBase, world: &World, _player: &mut Player, xyz: &GlobalCoords, _dir: &Direction) -> bool {
    world.break_voxel(xyz);
    return true;
}

pub fn on_set(base: &BlockBase, world: &World, _player: &mut Player, xyz: &GlobalCoords, dir: &Direction) -> bool {
    if world.voxel(xyz).map(|v| v.id == 0).unwrap_or(true) {
        world.chunks.set_block(*xyz, base.id, Some(dir));
        world.light.on_block_set(&world.chunks, xyz.0, xyz.1, xyz.2, base.id);
        return true;
    }
    false
}

pub fn on_multiblock_break(_base: &BlockBase, world: &World, _player: &mut Player, xyz: &GlobalCoords, _dir: &Direction) -> bool {
    if let Some(xyz) = world.chunks.remove_multiblock_structure(*xyz) {
        xyz.iter().for_each(|c| {
            world.light.on_block_break(&world.chunks, c.0, c.1, c.2);
        });
    };
    return true;
}

pub fn on_multiblock_set(base: &BlockBase, world: &World, _player: &mut Player, xyz: &GlobalCoords, dir: &Direction) -> bool {
    // FIX THIS SHIT
    let mut width = base.width as i32;
    let mut depth = base.depth as i32;
    if base.id == 5 {
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

pub type Function = &'static (dyn Fn(&BlockBase, &World, &mut Player, &GlobalCoords, &Direction) -> bool + Send + Sync);

static FUNCTIONS_CONTAINER: OnceLock<HashMap<String, Function>> = OnceLock::new();

#[allow(non_snake_case)]
pub fn FUNCTIONS() -> &'static HashMap<String, Function> {
    FUNCTIONS_CONTAINER.get_or_init(|| {
        let mut fns = HashMap::<String, Function>::new();

        fns.insert(String::from("on_set"), &on_set);
        fns.insert(String::from("on_break"), &on_break);
        fns.insert(String::from("on_multiblock_break"), &on_multiblock_break);
        fns.insert(String::from("on_multiblock_set"), &on_multiblock_set);
        fns.insert(String::from("player_add_item"), &on_multiblock_set);
        fns
    })
}