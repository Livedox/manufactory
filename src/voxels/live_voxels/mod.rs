use std::{collections::HashMap, hash::Hash, sync::{Arc, Mutex, Weak}};

use crate::{direction::Direction, gui::draw::Draw, recipes::storage::Storage, world::global_coords::GlobalCoords};
use std::fmt::Debug;
use self::furnace::Furnace;

use super::{chunks::Chunks, voxel_data::transport_belt::TransportBelt};
pub mod furnace;

pub trait PlayerUnlockable: Draw {
    fn get_storage(&self) -> Option<&dyn Storage> {None}
    fn get_mut_storage(&mut self) -> Option<&mut dyn Storage> {None}
}

#[derive(Debug)]
pub struct LiveVoxelContainer {
    pub id: u32,
    pub coords: GlobalCoords,
    pub live_voxel: Box<dyn LiveVoxel>
}

impl LiveVoxelContainer {
    pub fn to_bytes(&self) -> Vec<u8> {
        vec![]
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        todo!();
    }
}

pub type NewLiveVoxel = &'static (dyn Fn(&Direction) -> Box<dyn LiveVoxel> + Send + Sync);
pub type DesiarializeLiveVoxel = &'static (dyn Fn(&[u8]) -> Box<dyn LiveVoxel> + Send + Sync);

pub struct LiveVoxelRegistrator {
    pub new: HashMap<String, NewLiveVoxel>,
    pub deserialize: HashMap<String, DesiarializeLiveVoxel>,
}

impl Debug for LiveVoxelRegistrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LiveVoxelRegistrator")
    }
}

pub fn register() -> LiveVoxelRegistrator {
    let mut new = HashMap::<String, NewLiveVoxel>::new();
    let mut deserialize = HashMap::<String, DesiarializeLiveVoxel>::new();

    new.insert(String::from("furnace"), &<Arc<Mutex<Furnace>>>::new_livevoxel);
    deserialize.insert(String::from("furnace"), &<Arc<Mutex<Furnace>>>::deserialize);

    LiveVoxelRegistrator {
        new: new,
        deserialize,
    }
}

pub trait LiveVoxel: Debug {
    fn src_livevoxel(&self) -> Option<GlobalCoords> {None}
    fn player_unlockable(&self) -> Option<Weak<Mutex<dyn PlayerUnlockable>>> {None}
    fn structure_coordinates(&self) -> Option<Vec<GlobalCoords>> {None}
    fn rotation_index(&self) -> Option<u32> {None}
    fn storage(&self) -> Option<Arc<Mutex<dyn Storage>>> {None}
    fn update(&self, coords: GlobalCoords, chunks: &Chunks) {}
    fn transport_belt(&self) -> Option<Arc<Mutex<TransportBelt>>> {None}
    fn animation_progress(&self) -> f32 {0.0}

    fn serialize(&self) -> Vec<u8>;
}

pub trait LiveVoxelNew {
    fn new_livevoxel(direction: &Direction) -> Box<dyn LiveVoxel>;
}

pub trait LiveVoxelNewMultiblock {
    fn new_multiblock(direction: &Direction, structure_coordinates: Vec<GlobalCoords>) -> Box<dyn LiveVoxel>;
}

pub trait LiveVoxelDesiarialize {
    fn deserialize(bytes: &[u8]) -> Box<dyn LiveVoxel>;
}