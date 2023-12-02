use std::{cell::RefCell, rc::{Rc}, time::{Duration, Instant}, sync::{Arc, Mutex, Weak}};
use crate::{direction::Direction, voxels::chunk::Chunk, recipes::{recipe::ActiveRecipe, storage::Storage, item::{PossibleItem, Item}, recipes::RECIPES}, world::{global_coords::GlobalCoords, local_coords::LocalCoords}, gui::draw::Draw, bytes::{DynByteInterpretation, ConstByteInterpretation}};
use self::{voxel_box::VoxelBox, furnace::Furnace, drill::Drill, cowboy::Cowboy, assembling_machine::AssemblingMachine, transport_belt::TransportBelt, manipulator::Manipulator};

use super::{chunks::Chunks, block::blocks::BLOCKS};
use crate::bytes::NumFromBytes;
pub mod voxel_box;
pub mod furnace;
pub mod multiblock;
pub mod drill;
pub mod cowboy;
pub mod manipulator;
pub mod assembling_machine;
pub mod transport_belt;

pub trait DrawStorage: Draw + Storage {}


#[derive(Debug)]
pub struct VoxelData {
    pub id: u32,
    pub global_coords: GlobalCoords,

    pub additionally: Arc<VoxelAdditionalData>,
}

impl VoxelData {
    pub fn update(&self, chunks: *mut Chunks) {
        if self.id == 1 {return};
        self.additionally.update(self.global_coords, chunks);
    }

    pub fn rotation_index(&self) -> Option<u32> {
        self.additionally.rotation_index()
    }

    pub fn player_unlockable(&self) -> Option<Weak<Mutex<dyn DrawStorage>>> {
        self.additionally.player_unlockable()
    }
}


#[derive(Debug)]
pub enum VoxelAdditionalData {
    Empty,
    Manipulator(Box<Mutex<Manipulator>>),
    Cowboy(Box<Mutex<Cowboy>>),
    VoxelBox(Arc<Mutex<VoxelBox>>),
    Furnace(Arc<Mutex<Furnace>>),
    Drill(Arc<Mutex<Drill>>),
    AssemblingMachine(Arc<Mutex<AssemblingMachine>>),
    TransportBelt(Arc<Mutex<TransportBelt>>),
}


impl VoxelAdditionalData {
    pub fn new_multiblock(id: u32, direction: &Direction, structure_coordinates: Vec<GlobalCoords>) -> Self {
        match id {
            15 => Self::Drill(Arc::new(Mutex::new(Drill::new(structure_coordinates, direction)))),
            16 => Self::AssemblingMachine(Arc::new(Mutex::new(AssemblingMachine::new(structure_coordinates)))),
            _ => Self::Empty,
        }
    }

    pub fn new(id: u32, direction: &Direction) -> Self {
        match id {
            9 => Self::Manipulator(Box::new(Mutex::new(Manipulator::new(direction)))),
            12 => Self::Cowboy(Box::new(Mutex::new(Cowboy::new()))),
            13 => Self::VoxelBox(Arc::new(Mutex::new(VoxelBox::new()))),
            14 => Self::Furnace(Arc::new(Mutex::new(Furnace::new()))),
            17 => Self::TransportBelt(Arc::new(Mutex::new(TransportBelt::new(direction)))),
            _ => Self::Empty,
        }
    }


    pub fn transport_belt(&self) -> Option<Arc<Mutex<TransportBelt>>> {
        match self {
            VoxelAdditionalData::TransportBelt(b) => Some(b.clone()),
            _ => None,
        }
    }


    pub fn storage(&self) -> Option<Arc<Mutex<dyn Storage>>> {
        Some(match self {
            VoxelAdditionalData::VoxelBox(b) => b.clone(),
            VoxelAdditionalData::Furnace(f) => f.clone(),
            VoxelAdditionalData::AssemblingMachine(a) => a.clone(),
            VoxelAdditionalData::TransportBelt(c) => c.clone(),
            _ => return None,
        })
    }


    pub fn update(&self, coords: GlobalCoords, chunks: *mut Chunks) {
        match self {
            Self::Manipulator(o) => o.lock().unwrap().update(coords, chunks),
            Self::Drill(d) => d.lock().unwrap().update(chunks),
            Self::Furnace(f) => f.lock().unwrap().update(),
            Self::AssemblingMachine(a) => a.lock().unwrap().update(),
            Self::TransportBelt(c) => c.lock().unwrap().update(coords, chunks),
            Self::Empty | Self::VoxelBox(_) | Self::Cowboy(_) => (),
        }
    }


    pub fn animation_progress(&self) -> Option<f32> {
        match self {
            Self::Manipulator(o)=> Some(o.lock().unwrap().animation_progress()),
            Self::Cowboy(o) => Some(o.lock().unwrap().animation_progress()),
            Self::Empty | Self::VoxelBox(_) | Self::Furnace(_) |
            Self::Drill(_) | Self::AssemblingMachine(_) | Self::TransportBelt(_) => None,
        }
    }


    pub fn rotation_index(&self) -> Option<u32> {
        match self {
            Self::Manipulator(o) => {Some(o.lock().unwrap().rotation_index())},
            Self::TransportBelt(o) => {Some(o.lock().unwrap().rotation_index())},
            Self::Drill(o) => {Some(o.lock().unwrap().rotation_index())},
            _ => None,
        }
    }


    pub fn player_unlockable(&self) -> Option<Weak<Mutex<dyn DrawStorage>>> {
        match self {
            Self::VoxelBox(o)=> {
                let o: Arc<Mutex<dyn DrawStorage>> = o.clone();
                Some(Arc::downgrade(&o))
            },
            Self::Furnace(o) => {
                let o: Arc<Mutex<dyn DrawStorage>> = o.clone();
                Some(Arc::downgrade(&o))
            },
            Self::AssemblingMachine(o) => {
                let o: Arc<Mutex<dyn DrawStorage>> = o.clone();
                Some(Arc::downgrade(&o))
            },
            _ => None,
        }
    } 
}

impl DynByteInterpretation for VoxelAdditionalData {
    fn to_bytes(&self) -> Box<[u8]> {
        match self {
            Self::Empty => Box::new([]),
            Self::VoxelBox(b) => {b.lock().unwrap().to_bytes()},
            Self::Furnace(b) => {b.lock().unwrap().to_bytes()},
            Self::Manipulator(b) => {b.lock().unwrap().to_bytes()},
            Self::Cowboy(b) => {b.lock().unwrap().to_bytes()},
            Self::TransportBelt(b) => {b.lock().unwrap().to_bytes()},
            _ => unimplemented!(),
        }
    }

    fn from_bytes(data: &[u8]) -> Self {
        let id = u32::from_bytes(&data[0..4]);
        let len = u32::from_bytes(&data[4..8]) as usize + 8;
        match id {
            9 => {Self::Manipulator(Box::new(Mutex::new(Manipulator::from_bytes(&data[8..len]))))},
            12 => {Self::Cowboy(Box::new(Mutex::new(Cowboy::from_bytes(&data[8..len]))))},
            13 => {Self::VoxelBox(Arc::new(Mutex::new(VoxelBox::from_bytes(&data[8..len]))))},
            14 => {Self::Furnace(Arc::new(Mutex::new(Furnace::from_bytes(&data[8..len]))))},
            17 => {Self::TransportBelt(Arc::new(Mutex::new(TransportBelt::from_bytes(&data[8..len]))))},
            _ => unimplemented!(),
        }
    }
}

impl DynByteInterpretation for VoxelData {
    fn to_bytes(&self) -> Box<[u8]> {
        let mut v = vec![];
        v.extend(self.global_coords.to_bytes().as_ref());
        v.extend(self.id.to_le_bytes());

        let bytes = self.additionally.to_bytes();
        v.extend((bytes.len() as u32).to_le_bytes());
        v.extend(bytes.as_ref());
        v.into()
    }

    fn from_bytes(data: &[u8]) -> Self {
        let gc = GlobalCoords::from_bytes(&data[0..12]);
        let id = u32::from_bytes(&data[12..16]);
        Self {
            id,
            global_coords: gc,
            additionally: Arc::new(VoxelAdditionalData::from_bytes(&data[12..])),
        }
    }
}