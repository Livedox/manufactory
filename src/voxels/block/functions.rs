use std::{collections::HashMap, sync::OnceLock};

use crate::{player::player::Player, recipes::{item::Item, storage::Storage}, world::{global_coords::GlobalCoords, World}};

use super::block_test::BlockBase;

pub fn block_player_add_item(base: &BlockBase, world: &World, player: &mut Player, xyz: &GlobalCoords) {
    player.inventory().lock().unwrap().add(&Item::new(base.item_id, 1), true);
}

pub fn block_world_break(base: &BlockBase, world: &World, player: &mut Player, xyz: &GlobalCoords) {
    world.break_voxel(xyz);
}

pub type Function = &'static (dyn Fn(&BlockBase, &World, &mut Player, &GlobalCoords) + Send + Sync);

static FUNCTIONS_CONTAINER: OnceLock<HashMap<String, Function>> = OnceLock::new();

#[allow(non_snake_case)]
pub fn FUNCTIONS() -> &'static HashMap<String, Function> {
    FUNCTIONS_CONTAINER.get_or_init(|| {
        let mut fns = HashMap::<String, Function>::new();

        fns.insert(String::from("1"), &block_player_add_item);
        fns.insert(String::from("2"), &block_world_break);
        fns
    })
}