use std::sync::{Arc, Mutex, Weak};

use serde::{Deserialize, Serialize};

use crate::{bytes::{AsFromBytes, BytesCoder}, direction::{self, Direction}, engine::texture::TextureAtlas, gui::{draw::Draw, my_widgets::inventory_slot::inventory_slot}, live_voxel_default_deserialize, player::inventory::PlayerInventory, player_unlockable, recipes::{item::{Item, PossibleItem}, recipe::ActiveRecipe, recipes::RECIPES, storage::Storage}, voxels::chunks::Chunks, world::global_coords::GlobalCoords};

use super::{LiveVoxel, LiveVoxelDesiarialize, LiveVoxelNew, LiveVoxelNewMultiblock, NewMultiBlockLiveVoxel, PlayerUnlockable};

#[derive(Debug, Serialize, Deserialize)]
pub struct MultiBlock {
    pub coords: Vec<GlobalCoords>,
}

impl LiveVoxelNewMultiblock for MultiBlock {
    fn new_multiblock(_: &Direction, structure_coordinates: Vec<GlobalCoords>) -> Box<dyn LiveVoxel> {
        Box::new(Self {coords: structure_coordinates})
    }
}

impl LiveVoxel for MultiBlock {
    fn structure_coordinates(&self) -> Option<Vec<GlobalCoords>> {
        Some(self.coords.clone())
    }

    fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}
live_voxel_default_deserialize!(MultiBlock);