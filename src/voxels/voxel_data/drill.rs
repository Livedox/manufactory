use std::time::{Instant, Duration};

use crate::bytes::{BytesCoder, AsFromBytes, cast_vec_from_bytes, cast_bytes_from_slice};
use crate::{world::global_coords::GlobalCoords, direction::Direction, voxels::{chunks::Chunks}, recipes::{item::PossibleItem, storage::Storage}};

use super::multiblock::MultiBlock;

#[derive(Debug)]
pub struct Drill {
    dir: [i8; 3],
    storage: [PossibleItem; 1],
    structure_coordinates: Vec<GlobalCoords>,
    start: Instant,
}


impl Drill {
    const DURATION: Duration = Duration::new(4, 0);

    pub fn new(structure_coordinates: Vec<GlobalCoords>, dir: &Direction) -> Self {Self {
        storage: [PossibleItem::new_none()],
        structure_coordinates,
        start: Instant::now(),
        dir: dir.simplify_to_one_greatest(true, false, true)
    }}

    pub fn update(&mut self, chunks: &Chunks) {
        let xyz = self.structure_coordinates[0];
        let global = GlobalCoords(xyz.0 - self.dir[0] as i32, xyz.1, xyz.2-self.dir[2] as i32);
        if let Some(storage) = chunks.voxel_data(global).and_then(|vd| vd.additionally.storage()) {
            if let Some(item) = self.storage[0].0.take() {
                if let Some(r_item) = storage.lock().unwrap().add(&item, false) {
                    self.storage[0].try_add_item(&r_item);
                }
            }
        }

        if self.start.elapsed() < Self::DURATION {return}
        self.start = Instant::now();
        
        
        self.structure_coordinates.iter().for_each(|coord| {
            let ore_coords = GlobalCoords(coord.0, coord.1-1, coord.2);
            let voxel = chunks.voxel_global(ore_coords);
            let Some(voxel) = voxel else {return};
            if let Some(item) = chunks.content.blocks[voxel.id as usize].ore() {
                self.storage[0].try_add_item(&item);
            }
        });
    }

    pub fn rotation_index(&self) -> u32 {
        if self.dir[2] > 0 {return 0};
        if self.dir[0] < 0 {return 3};
        if self.dir[2] < 0 {return 2};
        1
    }
}



impl Storage for Drill {
    fn storage(&self) -> &[PossibleItem] {
        &self.storage
    }

    fn mut_storage(&mut self) -> &mut [PossibleItem] {
        &mut self.storage
    }
}


impl MultiBlock for Drill {
    fn structure_coordinates(&self) -> &[GlobalCoords] {
        &self.structure_coordinates
    }

    fn mut_structure_coordinates(&mut self) -> &mut [GlobalCoords] {
        &mut self.structure_coordinates
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Header {
    storage_len: u32,
    structure_len: u32,
    direction: [i8; 3],
}
impl AsFromBytes for Header {}

impl BytesCoder for Drill {
    fn decode_bytes(bytes: &[u8]) -> Self {
        let header = Header::from_bytes(&bytes[0..Header::size()]);

        let storage_size = Header::size() + header.storage_len as usize;
        let storage = <[PossibleItem; 1]>::decode_bytes(&bytes[Header::size()..storage_size]);
        let structure_size = storage_size+header.structure_len as usize;
        let structure = cast_vec_from_bytes(&bytes[storage_size..structure_size]);

        Self {
            dir: header.direction,
            storage,
            structure_coordinates: structure,
            start: Instant::now(),
        }
    }
    fn encode_bytes(&self) -> Box<[u8]> {
        let mut bytes = Vec::new();

        let storage = self.storage.encode_bytes();
        let structure = cast_bytes_from_slice(&self.structure_coordinates);
        bytes.extend(Header {
            direction: self.dir,
            storage_len: storage.len() as u32,
            structure_len: structure.len() as u32,
        }.as_bytes());
        bytes.extend(storage.as_ref());
        bytes.extend(structure);
        bytes.into()
    }
}