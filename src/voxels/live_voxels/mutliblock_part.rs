use std::sync::{Arc, Mutex, Weak};

use serde::{Deserialize, Serialize};

use crate::{bytes::{AsFromBytes, BytesCoder}, direction::{self, Direction}, engine::texture::TextureAtlas, gui::{draw::Draw, my_widgets::inventory_slot::inventory_slot}, live_voxel_default_deserialize, player::inventory::PlayerInventory, player_unlockable, recipes::{item::{Item, PossibleItem}, recipe::ActiveRecipe, recipes::RECIPES, storage::Storage}, voxels::chunks::Chunks, world::global_coords::GlobalCoords};

use super::{LiveVoxel, LiveVoxelDesiarialize, LiveVoxelNew, PlayerUnlockable};

#[derive(Debug, Serialize, Deserialize)]
pub struct MultiBlockPart {
    pub parent_coord: GlobalCoords,
}

impl MultiBlockPart {
    pub fn new(parent_coord: GlobalCoords) -> Self {
        Self {parent_coord}
    }
}

impl LiveVoxel for MultiBlockPart {
    fn parent_coord(&self) -> Option<GlobalCoords> {
        Some(self.parent_coord)
    }

    fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}
live_voxel_default_deserialize!(MultiBlockPart);
