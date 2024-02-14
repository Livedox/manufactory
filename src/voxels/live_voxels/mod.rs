use std::{collections::HashMap, hash::Hash, sync::{Arc, Mutex, Weak}};

use serde::{ser::SerializeStruct, Deserialize, Serialize};

use crate::{bytes::AsFromBytes, content::Content, direction::Direction, gui::draw::Draw, recipes::storage::Storage, world::global_coords::GlobalCoords};
use std::fmt::Debug;
use self::{drill::Drill, furnace::Furnace, multiblock::MultiBlock, mutliblock_part::MultiBlockPart, voxel_box::VoxelBox};

use super::{chunks::Chunks, voxel_data::transport_belt::TransportBelt};
pub mod furnace;
pub mod voxel_box;
pub mod unit;
pub mod mutliblock_part;
pub mod drill;
pub mod multiblock;

pub trait PlayerUnlockable: Draw {
    fn get_storage(&self) -> Option<&dyn Storage> {None}
    fn get_mut_storage(&mut self) -> Option<&mut dyn Storage> {None}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiveVoxelCoord {
    Standard(GlobalCoords),
    Slave([GlobalCoords; 2]),
    Master(Vec<GlobalCoords>)
}

impl From<GlobalCoords> for LiveVoxelCoord {
    fn from(value: GlobalCoords) -> Self {
        Self::Standard(value)
    }
}

#[derive(Debug)]
pub struct LiveVoxelContainer {
    pub id: u32,
    pub coord: LiveVoxelCoord,
    pub live_voxel: Box<dyn LiveVoxel>
}

impl LiveVoxelContainer {
    #[inline]
    pub fn new(id: u32, coord: LiveVoxelCoord, live_voxel: Box<dyn LiveVoxel>) -> Self {
        Self { id, coord, live_voxel }
    }

    #[inline]
    pub fn new_arc(id: u32, coord: LiveVoxelCoord, live_voxel: Box<dyn LiveVoxel>) -> Arc<Self> {
        Arc::new(Self::new(id, coord, live_voxel))
    }

    #[inline]
    pub fn new_arc_multiblock_part(coord: LiveVoxelCoord, parent: GlobalCoords) -> Arc<Self> {
        Arc::new(Self::new(1, coord, Box::new(MultiBlockPart::new(parent))))
    }

    pub fn coord(&self) -> GlobalCoords {
        match &self.coord {
            LiveVoxelCoord::Standard(g) => *g,
            LiveVoxelCoord::Slave(v) => v[0],
            LiveVoxelCoord::Master(v) => v[0]
        }
    }

    #[inline]
    pub fn update(&self, chunks: &Chunks) {
        self.live_voxel.update(self.coord(), chunks);
    }
    
    #[inline]
    pub fn parent_coord(&self) -> Option<GlobalCoords> {
        match &self.coord {
            LiveVoxelCoord::Slave(v) => Some(v[1]),
            _ => None
        }
    }
    #[inline] 
    pub fn structure_coordinates(&self) -> Option<Vec<GlobalCoords>> {
        match &self.coord {
            LiveVoxelCoord::Master(v) => Some(v.clone()),
            _ => None
        }
    }
    #[inline] 
    pub fn player_unlockable(&self) -> Option<Weak<Mutex<dyn PlayerUnlockable>>> {
        self.live_voxel.player_unlockable()
    }
    #[inline] 
    pub fn rotation_index(&self) -> Option<u32> {
        self.live_voxel.rotation_index()
    }
    #[inline] 
    pub fn storage(&self) -> Option<Arc<Mutex<dyn Storage>>> {
        self.live_voxel.storage()
    }
    #[inline] 
    pub fn transport_belt(&self) -> Option<Arc<Mutex<TransportBelt>>> {
        self.live_voxel.transport_belt()
    }
    #[inline] 
    pub fn animation_progress(&self) -> f32 {
        self.live_voxel.animation_progress()
    }
}

impl LiveVoxelContainer {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.id.to_le_bytes());
        let coord = bincode::serialize(&self.coord).unwrap();
        bytes.extend((coord.len() as u32).to_le_bytes());
        bytes.extend(coord);
        bytes.extend(self.live_voxel.serialize());
        bytes
    }

    pub fn from_bytes(content: &Content, bytes: &[u8]) -> Self {
        let id = u32::from_le_bytes([0, 1, 2, 3].map(|i| *bytes.get(i).unwrap()));
        let coord_end = 8 + u32::from_le_bytes([4, 5, 6, 7].map(|i| *bytes.get(i).unwrap())) as usize;
        let coord = bincode::deserialize(&bytes[8..coord_end]).unwrap();
        let live_voxel: Box<dyn LiveVoxel> = if let Some(name) = content.blocks[id as usize].live_voxel() {
            content.live_voxel.deserialize.get(name)
                .map_or(Box::new(()), |desiarialize| desiarialize(&bytes[coord_end..]))
        } else {
            Box::new(())
        };

        Self { id, coord, live_voxel }
    }
}

pub type NewLiveVoxel = &'static (dyn Fn(&Direction) -> Box<dyn LiveVoxel> + Send + Sync);
pub type NewMultiBlockLiveVoxel = &'static (dyn Fn(&Direction, Vec<GlobalCoords>) -> Box<dyn LiveVoxel> + Send + Sync);
pub type DesiarializeLiveVoxel = &'static (dyn Fn(&[u8]) -> Box<dyn LiveVoxel> + Send + Sync);

pub struct LiveVoxelRegistrator {
    pub new: HashMap<String, NewLiveVoxel>,
    pub deserialize: HashMap<String, DesiarializeLiveVoxel>,
    pub new_multiblock: HashMap<String, NewMultiBlockLiveVoxel>
}

impl Debug for LiveVoxelRegistrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LiveVoxelRegistrator")
    }
}

pub fn register() -> LiveVoxelRegistrator {
    let mut new = HashMap::<String, NewLiveVoxel>::new();
    let mut deserialize = HashMap::<String, DesiarializeLiveVoxel>::new();
    let mut new_multiblock = HashMap::<String, NewMultiBlockLiveVoxel>::new();

    new.insert(String::from("furnace"), &<Arc<Mutex<Furnace>>>::new_livevoxel);
    deserialize.insert(String::from("furnace"), &<Arc<Mutex<Furnace>> as LiveVoxelDesiarialize>::deserialize);

    new.insert(String::from("voxel_box"), &<Arc<Mutex<VoxelBox>>>::new_livevoxel);
    deserialize.insert(String::from("voxel_box"), &<Arc<Mutex<VoxelBox>> as LiveVoxelDesiarialize>::deserialize);

    deserialize.insert(String::from("multiblock_part"), &<MultiBlockPart as LiveVoxelDesiarialize>::deserialize);

    deserialize.insert(String::from("drill"), &<Mutex<Drill> as LiveVoxelDesiarialize>::deserialize);
    new_multiblock.insert(String::from("drill"), &<Mutex<Drill>>::new_multiblock);

    deserialize.insert(String::from("multiblock"), &<MultiBlock as LiveVoxelDesiarialize>::deserialize);
    new_multiblock.insert(String::from("multiblock"), &<MultiBlock>::new_multiblock);

    LiveVoxelRegistrator { 
        new,
        deserialize,
        new_multiblock,
    }
}

pub trait LiveVoxel: Debug {
    fn parent_coord(&self) -> Option<GlobalCoords> {None}
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