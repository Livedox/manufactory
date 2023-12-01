use std::time::{Instant, Duration};

use crate::{world::{global_coords::GlobalCoords, local_coords::LocalCoords}, direction::Direction, voxels::{chunks::Chunks, block::blocks::BLOCKS}, recipes::{item::PossibleItem, storage::Storage}};

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
        let voxels_data = unsafe {chunks.as_mut().expect("Chunks don't exist").mut_voxels_data(global)};
        if let Some(storage) = voxels_data
            .and_then(|vd| vd.get_mut(&LocalCoords::from(global).index()))
            .and_then(|d| d.additionally.storage()) {
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
            let voxel = unsafe {chunks.as_mut().expect("Chunks don't exist").voxel_global(ore_coords)};
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