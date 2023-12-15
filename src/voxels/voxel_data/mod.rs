use std::sync::{Arc, Mutex, Weak};
use crate::{direction::Direction, recipes::storage::Storage, world::global_coords::GlobalCoords, gui::draw::Draw, bytes::{BytesCoder, AsFromBytes}};
use self::{voxel_box::VoxelBox, furnace::Furnace, drill::Drill, cowboy::Cowboy, assembling_machine::AssemblingMachine, transport_belt::TransportBelt, manipulator::Manipulator, multiblock::MultiBlock};

use super::chunks::Chunks;
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
    MultiBlockPart(GlobalCoords),
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
            Self::Empty | Self::VoxelBox(_) | Self::Cowboy(_) | Self::MultiBlockPart(_) => (),
        }
    }


    pub fn animation_progress(&self) -> Option<f32> {
        match self {
            Self::Manipulator(o)=> Some(o.lock().unwrap().animation_progress()),
            Self::Cowboy(o) => Some(o.lock().unwrap().animation_progress()),
            Self::Empty | Self::VoxelBox(_) | Self::Furnace(_) |
            Self::Drill(_) | Self::AssemblingMachine(_) | Self::TransportBelt(_) |
            Self::MultiBlockPart(_) => None,
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


    pub fn structure_coordinates(&self) -> Option<Vec<GlobalCoords>> {
        match self {
            VoxelAdditionalData::Drill(d) => Some(Vec::from(d.lock().unwrap().structure_coordinates())),
            VoxelAdditionalData::AssemblingMachine(d) => Some(Vec::from(d.lock().unwrap().structure_coordinates())),
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

impl VoxelAdditionalData {
    fn encode_bytes(&self) -> Box<[u8]> {
        match self {
            Self::Empty => Box::new([]),
            Self::MultiBlockPart(b) => {b.as_bytes().into()},
            Self::VoxelBox(b) => {b.lock().unwrap().encode_bytes()},
            Self::Furnace(b) => {b.lock().unwrap().encode_bytes()},
            Self::Manipulator(b) => {b.lock().unwrap().encode_bytes()},
            Self::Cowboy(b) => {b.lock().unwrap().encode_bytes()},
            Self::TransportBelt(b) => {b.lock().unwrap().encode_bytes()},
            Self::Drill(b) => {b.lock().unwrap().encode_bytes()},
            Self::AssemblingMachine(b) => {b.lock().unwrap().encode_bytes()},
        }
    }

    fn decode_bytes(bytes: &[u8], id: u32) -> Self {
        match id {
            1 => {Self::MultiBlockPart(GlobalCoords::from_bytes(bytes))},
            9 => {Self::Manipulator(Box::new(Mutex::new(Manipulator::decode_bytes(bytes))))},
            12 => {Self::Cowboy(Box::new(Mutex::new(Cowboy::decode_bytes(bytes))))},
            13 => {Self::VoxelBox(Arc::new(Mutex::new(VoxelBox::decode_bytes(bytes))))},
            14 => {Self::Furnace(Arc::new(Mutex::new(Furnace::decode_bytes(bytes))))},
            17 => {Self::TransportBelt(Arc::new(Mutex::new(TransportBelt::decode_bytes(bytes))))},

            16 => {Self::AssemblingMachine(Arc::new(Mutex::new(AssemblingMachine::decode_bytes(bytes))))},
            15 => {Self::Drill(Arc::new(Mutex::new(Drill::decode_bytes(bytes))))},
            _ => unimplemented!(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Header {
    id: u32,
    global_coords: GlobalCoords,
}
impl AsFromBytes for Header {}

impl BytesCoder for VoxelData {
    fn encode_bytes(&self) -> Box<[u8]> {
        let mut bytes = vec![];
        bytes.extend(Header {id: self.id, global_coords: self.global_coords}.as_bytes());
        bytes.extend(self.additionally.encode_bytes().as_ref());
        bytes.into()
    }

    fn decode_bytes(bytes: &[u8]) -> Self {
        let header = Header::from_bytes(&bytes[0..Header::size()]);
        Self {
            id: header.id,
            global_coords: header.global_coords,
            additionally: Arc::new(VoxelAdditionalData::decode_bytes(&bytes[Header::size()..], header.id)),
        }
    }
}