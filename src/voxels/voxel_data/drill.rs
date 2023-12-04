use std::time::{Instant, Duration};

use nalgebra_glm::vec1;
use crate::bytes::NumFromBytes;
use crate::{world::{global_coords::GlobalCoords, local_coords::LocalCoords}, direction::Direction, voxels::{chunks::Chunks, block::blocks::BLOCKS}, recipes::{item::PossibleItem, storage::Storage}, bytes::{DynByteInterpretation, ConstByteInterpretation}};

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

    pub fn update(&mut self, chunks: *mut Chunks) {
        let xyz = self.structure_coordinates[0];
        let global = GlobalCoords(xyz.0 - self.dir[0] as i32, xyz.1, xyz.2-self.dir[2] as i32);
        let chunks = unsafe {chunks.as_mut().expect("Chunks don't exist")};
        if let Some(storage) = chunks.mut_voxel_data(global).and_then(|vd| vd.additionally.storage()) {
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
            if let Some(item) = BLOCKS()[voxel.id as usize].ore() {
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


impl DynByteInterpretation for Drill {
    fn from_bytes(data: &[u8]) -> Self {
        let dir = [data[0] as i8, data[1] as i8, data[2] as i8];
        let s_len = u32::from_bytes(&data[3..7]) as usize;
        let s = <[PossibleItem; 1]>::from_bytes(&data[7..7+s_len]);
        let sc_len = u32::from_bytes(&data[7+s_len..7+s_len+4]);

        let mut sc = vec![];
        for d in data[7+s_len+4..].chunks(12) {
            sc.push(GlobalCoords::from_bytes(d));
        }

        Self {
            dir,
            storage: s,
            structure_coordinates: sc,
            start: Instant::now(),
        }
    }
    fn to_bytes(&self) -> Box<[u8]> {
        let mut v = Vec::new();
        let storage = self.storage.to_bytes();
        let storage_len = storage.len() as u32;
        let sc_len = (self.structure_coordinates.len() * 12) as u32;
        let mut sc = Vec::<u8>::new();
        for gc in &self.structure_coordinates {
            sc.extend(gc.to_bytes().as_ref());
        }

        v.extend([self.dir[0] as u8, self.dir[1] as u8, self.dir[2] as u8]);
        v.extend(storage_len.to_le_bytes());
        v.extend(storage.as_ref());
        v.extend(sc_len.to_le_bytes());
        v.extend(sc);
        v.into()
    }
}