use std::{collections::HashMap, hash::Hash, sync::{Arc, Mutex, Weak}};

use serde::{ser::SerializeStruct, Serialize};

use crate::{bytes::AsFromBytes, content::Content, direction::Direction, gui::draw::Draw, recipes::storage::Storage, world::global_coords::GlobalCoords};
use std::fmt::Debug;
use self::{furnace::Furnace, voxel_box::VoxelBox};

use super::{chunks::Chunks, voxel_data::transport_belt::TransportBelt};
pub mod furnace;
pub mod voxel_box;
pub mod unit;

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
        let mut bytes = Vec::new();
        bytes.extend(&self.id.to_le_bytes());
        bytes.extend(bincode::serialize(&self.coords).unwrap());
        bytes.extend(self.live_voxel.serialize());
        bytes
    }

    pub fn from_bytes(content: &Content, bytes: &[u8]) -> Self {
        let id = u32::from_le_bytes([0, 1, 2, 3].map(|i| *bytes.get(i).unwrap()));
        let coords_end = 4+std::mem::size_of::<GlobalCoords>();
        let coords = bincode::deserialize(&bytes[4..coords_end]).unwrap();
        let live_voxel: Box<dyn LiveVoxel> = if let Some(name) = content.blocks[id as usize].live_voxel() {
            content.live_voxel.deserialize.get(name)
                .map_or(Box::new(()), |desiarialize| desiarialize(&bytes[coords_end..]))
        } else {
            Box::new(())
        };

        Self { id, coords, live_voxel }
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

    new.insert(String::from("voxel_box"), &<Arc<Mutex<VoxelBox>>>::new_livevoxel);
    deserialize.insert(String::from("voxel_box"), &<Arc<Mutex<VoxelBox>>>::deserialize);

    LiveVoxelRegistrator { 
        new,
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