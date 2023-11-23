use std::{cell::RefCell, rc::{Rc}, time::{Duration, Instant}, sync::{Arc, Mutex, Weak}};

use crate::{direction::Direction, voxels::chunk::Chunk, recipes::{recipe::ActiveRecipe, storage::Storage, item::{PossibleItem, Item}, recipes::RECIPES}, world::{global_coords::GlobalCoords, local_coords::LocalCoords}};

use super::{chunks::Chunks, assembling_machine::AssemblingMachine, transport_belt::TransportBelt, block::blocks::BLOCKS};

#[derive(Debug)]
pub enum PlayerUnlockableStorage {
    VoxelBox(Weak<Mutex<VoxelBox>>),
    Furnace(Weak<Mutex<Furnace>>),
    AssemblingMachine(Weak<Mutex<AssemblingMachine>>),
}


impl PlayerUnlockableStorage {
    pub fn to_storage(&self) -> Weak<Mutex<dyn Storage>> {
        match self {
            PlayerUnlockableStorage::VoxelBox(s) => s.clone(),
            PlayerUnlockableStorage::Furnace(s) => s.clone(),
            PlayerUnlockableStorage::AssemblingMachine(s) => s.clone(),
        }
    }
}


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

    pub fn player_unlockable_storage(&self) -> Option<PlayerUnlockableStorage> {
        self.additionally.player_unlockable_storage()
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


    pub fn player_unlockable_storage(&self) -> Option<PlayerUnlockableStorage> {
        match self {
            Self::VoxelBox(o)=> {Some(PlayerUnlockableStorage::VoxelBox(Arc::downgrade(o)))},
            Self::Furnace(o) => {Some(PlayerUnlockableStorage::Furnace(Arc::downgrade(o)))},
            Self::AssemblingMachine(o) =>  {Some(PlayerUnlockableStorage::AssemblingMachine(Arc::downgrade(o)))}
            _ => None,
        }
    } 
}

#[derive(Debug)]
pub struct Manipulator {
    start_time: Option<Instant>,
    return_time: Option<Instant>,
    item_id: Option<u32>,
    direction: [i8; 3],
}


impl Manipulator {
    const SPEED: Duration = Duration::from_millis(300);

    pub fn new(direction: &Direction) -> Self {Self {
        start_time: None,
        return_time: None,
        item_id: None,
        direction: direction.simplify_to_one_greatest(true, false, true),
    }}

    pub fn update(&mut self, coords: GlobalCoords, chunks: *mut Chunks) {
        let return_time = self.return_time.map_or(true, |rt| rt.elapsed() >= (Self::SPEED/2));
        if self.item_id.is_none() && self.start_time.is_none() && return_time {
            let src_coords = GlobalCoords(coords.0 - self.direction[0] as i32, coords.1, coords.2 - self.direction[2] as i32);
            let src = unsafe {
                chunks.as_mut().expect("Chunks don't exist").mut_chunk(src_coords)
            };
            if let Some(src_chunk) = src {
                let Some(src_data) = src_chunk.mut_voxel_data(src_coords.into()) else {return};
                let Some(storage) = src_data.additionally.storage() else {return};
                if let Some(item) = storage.lock().unwrap().take_first_existing(1) {
                    self.item_id = Some(item.0.id());
                    self.start_time = Some(Instant::now());
                    self.return_time = None;
                };
            }
        }
        
        let start_time = self.start_time.map_or(false, |rt| rt.elapsed() >= (Self::SPEED/2));
        if self.item_id.is_some() && start_time {
            let dst_coords = GlobalCoords(coords.0 + self.direction[0] as i32, coords.1, coords.2 + self.direction[2] as i32);
            let dst = unsafe {
                chunks.as_mut().expect("Chunks don't exist").mut_chunk(dst_coords)
            };
            if let Some(dst_chunk) = dst {
                let Some(dst_data) = dst_chunk.mut_voxel_data(dst_coords.into()) else {return};
                let Some(storage) = dst_data.additionally.storage() else {return};
                let result = storage.lock().unwrap().add(&Item::new(self.item_id.unwrap(), 1), false).is_none();
                if result {
                    self.item_id = None;
                    self.start_time = None;
                    self.return_time = Some(Instant::now());
                }
            }
        }
    }


    pub fn animation_progress(&self) -> f32 {
        if let Some(start_time) = self.start_time {
            (start_time.elapsed().as_secs_f32() / Self::SPEED.as_secs_f32()).min(0.5)
        } else if let Some(return_time) = self.return_time {
            (return_time.elapsed().as_secs_f32() / Self::SPEED.as_secs_f32() + 0.5).min(1.0)
        } else {
            0.0
        }
    }


    pub fn rotation_index(&self) -> u32 {
        if self.direction[0] < 0 {return 2};
        if self.direction[2] > 0 {return 3};
        if self.direction[2] < 0 {return 1};
        0
    }
}


#[derive(Debug)]
pub struct Cowboy {
    time: Instant,
}


impl Cowboy {
    pub fn new() -> Self {Self { time: Instant::now() }}

    pub fn animation_progress(&self) -> f32 {
        self.time.elapsed().as_secs_f32() % 1.0
    }
}

impl Default for Cowboy {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug, Clone)]
pub struct VoxelBox {
    storage: [PossibleItem; 30]
}

impl VoxelBox {
    pub fn new() -> Self {
        Self { storage: [PossibleItem::new_none(); 30] }
    }
}

impl Storage for VoxelBox {
    fn storage(&self) -> &[PossibleItem] {
        &self.storage
    }

    fn mut_storage(&mut self) -> &mut [PossibleItem] {
        &mut self.storage
    }
}

impl Default for VoxelBox {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug)]
pub struct Furnace {
    storage: [PossibleItem; 2],
    active_recipe: Option<ActiveRecipe>,
}


impl Furnace {
    pub fn new() -> Self {
        Self {
            storage: [PossibleItem::new_none(); 2],
            active_recipe: None
        }
    }

    pub fn update(&mut self) {
        let active_recipe_take = self.active_recipe.take();
        if let Some(active_recipe) = &active_recipe_take {
            let storage = self.mut_storage();
            if active_recipe.is_finished() && storage[1].is_possible_add(&active_recipe.recipe.result) {
                storage[1].try_add_item(&active_recipe.recipe.result);
                self.active_recipe = None;
            } else {
                self.active_recipe = active_recipe_take;
            }
        } else {
            let Some(item) = &self.storage[0].0 else {return};
            let Some(recipe) = RECIPES().furnace.first_by_ingredient(item.id()).cloned() else {return};
            self.active_recipe = self.start_recipe(&recipe);
        }
    }
}



impl Storage for Furnace {
    fn storage(&self) -> &[PossibleItem] {
        &self.storage
    }

    fn mut_storage(&mut self) -> &mut [PossibleItem] {
        &mut self.storage
    }

    fn take_first_existing(&mut self, max_count: u32) -> Option<(Item, usize)> {
        self.mut_storage()[1].try_take(max_count).map(|i| (i, 1))
    }

    fn add(&mut self, item: &Item, _: bool) -> Option<Item> {
        if RECIPES().furnace.get_by_ingredient(item.id()).is_some() {
            return self.mut_storage()[0].try_add_item(item);
        }
        Some(*item)
    }

    fn is_item_exist(&self, item: &Item) -> bool {
        self.storage[0].contains(item.id()) >= item.count
    }
}

impl Default for Furnace {
    fn default() -> Self {
        Self::new()
    }
}


pub trait MultiBlock {
    fn structure_coordinates(&self) -> &[GlobalCoords];
    fn mut_structure_coordinates(&mut self) -> &mut [GlobalCoords];
}


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