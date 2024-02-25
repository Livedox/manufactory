use std::{collections::HashMap, hash::Hash, sync::{Arc, Mutex, Weak}};

use serde::{ser::SerializeStruct, Deserialize, Serialize};

use crate::{bytes::AsFromBytes, content::Content, direction::Direction, gui::draw::Draw, recipes::storage::Storage, world::global_coords::GlobalCoords};
use std::fmt::Debug;
use self::{drill::Drill, furnace::Furnace, voxel_box::VoxelBox};

use super::{chunks::Chunks, voxel_data::transport_belt::TransportBelt};
pub mod furnace;
pub mod voxel_box;
pub mod unit;
pub mod drill;

pub trait PlayerUnlockable: Draw {
    fn get_storage(&self) -> Option<&dyn Storage> {None}
    fn get_mut_storage(&mut self) -> Option<&mut dyn Storage> {None}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MultiBlock {
    Slave(GlobalCoords),
    Master(Vec<GlobalCoords>)
}

#[derive(Debug)]
pub struct LiveVoxelContainer {
    pub id: u32,
    pub coord: GlobalCoords,
    pub multiblock: Option<MultiBlock>,
    pub live_voxel: Box<dyn LiveVoxelBehavior>
}

impl LiveVoxelContainer {
    #[inline]
    pub fn new(id: u32, coord: GlobalCoords, live_voxel: Box<dyn LiveVoxelBehavior>) -> Self {
        Self { id, coord, live_voxel, multiblock: None }
    }

    #[inline]
    pub fn new_arc(id: u32, coord: GlobalCoords, live_voxel: Box<dyn LiveVoxelBehavior>) -> Arc<Self> {
        Arc::new(Self::new(id, coord, live_voxel))
    }

    #[inline]
    pub fn new_arc_master(id: u32, coord: GlobalCoords, all: Vec<GlobalCoords>, live_voxel: Box<dyn LiveVoxelBehavior>) -> Arc<Self> {
        Arc::new(Self {
            id,
            coord,
            multiblock: Some(MultiBlock::Master(all)),
            live_voxel,
        })
    }

    #[inline]
    pub fn new_arc_slave(coord: GlobalCoords, master: GlobalCoords) -> Arc<Self> {
        Arc::new(Self {
            id: 1,
            coord,
            multiblock: Some(MultiBlock::Slave(master)),
            live_voxel: Box::new(()),
        })
    }

    pub fn coord(&self) -> GlobalCoords {
        self.coord
    }

    #[inline]
    pub fn update(&self, chunks: &Chunks) {
        let multiblock = self.multiblock_coords().unwrap_or(&[]);
        self.live_voxel.update(chunks, self.coord(), multiblock);
    }
    
    #[inline]
    pub fn master_coord(&self) -> Option<GlobalCoords> {
        match &self.multiblock {
            Some(MultiBlock::Slave(c)) => Some(*c),
            _ => None
        }
    }
    #[inline] 
    pub fn multiblock_coords(&self) -> Option<&[GlobalCoords]> {
        match &self.multiblock {
            Some(MultiBlock::Master(v)) => Some(&v),
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
        let multiblock = bincode::serialize(&self.multiblock).unwrap();
        bytes.extend((multiblock.len() as u32).to_le_bytes());
        bytes.extend(multiblock);
        bytes.extend(self.live_voxel.serialize());
        bytes
    }

    pub fn from_bytes(content: &Content, bytes: &[u8]) -> Self {
        let id = u32::from_le_bytes([0, 1, 2, 3].map(|i| *bytes.get(i).unwrap()));
        let coord_end = 8 + u32::from_le_bytes([4, 5, 6, 7].map(|i| *bytes.get(i).unwrap())) as usize;
        let coord = bincode::deserialize(&bytes[8..coord_end]).unwrap();
        let multiblock_end = 4 + coord_end + u32::from_le_bytes([0, 1, 2, 3]
            .map(|i| *bytes.get(i+coord_end).unwrap())) as usize;
        let multiblock = bincode::deserialize(&bytes[coord_end+4..multiblock_end]).unwrap();

        let live_voxel: Box<dyn LiveVoxelBehavior> = if let Some(name) = content.blocks[id as usize].live_voxel() {
            content.live_voxel.deserialize.get(name)
                .map_or(Box::new(()), |desiarialize| desiarialize(&bytes[multiblock_end..]))
        } else {
            Box::new(())
        };

        Self { id, coord, live_voxel, multiblock }
    }
}

pub type NewLiveVoxel = &'static (dyn Fn(&Direction) -> Box<dyn LiveVoxelBehavior> + Send + Sync);
pub type DesiarializeLiveVoxel = &'static (dyn Fn(&[u8]) -> Box<dyn LiveVoxelBehavior> + Send + Sync);

pub struct LiveVoxelRegistrator {
    pub new: HashMap<String, NewLiveVoxel>,
    pub deserialize: HashMap<String, DesiarializeLiveVoxel>
}

impl Debug for LiveVoxelRegistrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LiveVoxelRegistrator")
    }
}

pub fn register() -> LiveVoxelRegistrator {
    let mut new = HashMap::<String, NewLiveVoxel>::new();
    let mut deserialize = HashMap::<String, DesiarializeLiveVoxel>::new();

    new.insert(String::from("furnace"), &<Arc<Mutex<Furnace>>>::create);
    deserialize.insert(String::from("furnace"), &<Arc<Mutex<Furnace>> as LiveVoxelCreation>::deserialize);

    new.insert(String::from("voxel_box"), &<Arc<Mutex<VoxelBox>>>::create);
    deserialize.insert(String::from("voxel_box"), &<Arc<Mutex<VoxelBox>> as LiveVoxelCreation>::deserialize);

    deserialize.insert(String::from("drill"), &<Mutex<Drill> as LiveVoxelCreation>::deserialize);
    new.insert(String::from("drill"), &<Mutex<Drill>>::create);

    LiveVoxelRegistrator { 
        new,
        deserialize,
    }
}

pub trait LiveVoxelBehavior: Debug {
    fn player_unlockable(&self) -> Option<Weak<Mutex<dyn PlayerUnlockable>>> {None}
    fn rotation_index(&self) -> Option<u32> {None}
    fn storage(&self) -> Option<Arc<Mutex<dyn Storage>>> {None}
    fn update(&self, chunks: &Chunks, coord: GlobalCoords, multiblock: &[GlobalCoords]) {}
    fn transport_belt(&self) -> Option<Arc<Mutex<TransportBelt>>> {None}
    fn animation_progress(&self) -> f32 {0.0}

    fn serialize(&self) -> Vec<u8>;
}


pub trait LiveVoxelCreation {
    fn create(direction: &Direction) -> Box<dyn LiveVoxelBehavior>;
    fn deserialize(bytes: &[u8]) -> Box<dyn LiveVoxelBehavior>;
}